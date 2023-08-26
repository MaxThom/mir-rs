use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use lapin::types::ShortString;
use log::{debug, error, info, trace, warn};
use serde_json::{json, Value};
use surrealdb::{engine::remote::ws::Client, Surreal};
use x::device_twin::{DeviceTwin, MetaProperties, NewDevice, Properties, Record, TargetProperties};
use y::clients::amqp::Amqp;

use crate::{twin_service::*, RMQ_TWIN_EXCHANGE_NAME};

pub struct ApiState {
    pub amqp: Amqp,
    pub db: Surreal<Client>,
}

const DEVICE_ID_KEY: &str = "device_id";

pub async fn get_records(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let twins: &Vec<Record> = &state.db.select("device_twin").await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(twins);

    Ok(Json(json!({ "result": twins })))
}

pub async fn get_device_twins(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str())
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    dbg!(twins);

    Ok(Json(json!({ "result": twins })))
}

pub async fn get_device_twins_properties(
    State(state): State<Arc<ApiState>>,
    Path(target): Path<TargetProperties>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str())
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    dbg!(twins);

    match target {
        TargetProperties::Meta => {
            let twins_meta: &Vec<Option<MetaProperties>> = &twins
                .iter()
                .map(|twin| twin.meta_properties.clone())
                .collect();
            Ok(Json(json!({ "result": twins_meta })))
        }
        TargetProperties::Tag => {
            let twins_tag: &Vec<Option<Properties>> = &twins
                .iter()
                .map(|twin| twin.tag_properties.clone())
                .collect();
            Ok(Json(json!({ "result": twins_tag })))
        }
        TargetProperties::Desired => {
            let twins_desired: &Vec<Option<Properties>> = &twins
                .iter()
                .map(|twin| twin.desired_properties.clone())
                .collect();
            Ok(Json(json!({ "result": twins_desired })))
        }
        TargetProperties::Reported => {
            let twins_reported: &Vec<Option<Properties>> = &twins
                .iter()
                .map(|twin| twin.reported_properties.clone())
                .collect();
            Ok(Json(json!({ "result": twins_reported })))
        }
        TargetProperties::All => Ok(Json(json!({ "result": twins }))),
    }
}

pub async fn update_device_twins_properties(
    State(state): State<Arc<ApiState>>,
    Path(target): Path<TargetProperties>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<Properties>,
) -> Result<Json<Value>, StatusCode> {
    debug!("update_device_twin");
    // Api info
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    dbg!(&device_id);
    dbg!(&payload);

    // Update db
    let updated_twin_result = update_device_twins_properties_in_db(
        state.db.clone(),
        device_id.as_str(),
        &target,
        &payload,
    )
    .await
    .map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    });

    let updated_twin = if let Err(_) = updated_twin_result {
        //return Ok(Json(json!({ "result": 200 })));
        // TODO: proper return when surrealdb is fixed
        None
    } else {
        updated_twin_result.unwrap()
    };

    // Send msg to device with update properties if its desired
    // Create reusable channel
    if target.clone() == TargetProperties::Desired {
        debug!("sending desired properties to device {device_id}");
        let str_payload = serde_json::to_string(&payload).unwrap();
        match state
            .amqp
            .send_message(&str_payload, "", device_id.as_str())
            .await
        {
            Ok(x) => {
                info!("{x}")
            }
            Err(e) => {
                error!("{:?}", e);
            } // TODO: Add error type to telemetry sent
        };
    }

    dbg!(&updated_twin);
    Ok(Json(json!({ "result": updated_twin.unwrap() })))
}

pub async fn create_device_twins(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<NewDevice>,
) -> Result<Json<Value>, StatusCode> {
    debug!("create_device_twin");
    dbg!(&payload);

    let created = create_device_twins_in_db(state.db.clone(), payload)
        .await
        .map_err(|error| {
            error!("Error: {}", error.to_string());
            StatusCode::INTERNAL_SERVER_ERROR;
        });

    if let Err(error) = created {
        return Ok(Json(json!({ "result": error })));
    }

    dbg!(&created);
    Ok(Json(json!({ "result": created })))
}

pub async fn delete_device_twins(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    debug!("delete_device_twin");
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    dbg!(&device_id);

    let twins = &delete_device_twins_in_db(state.db.clone(), device_id.as_str())
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    dbg!(&twins);
    Ok(Json(json!({ "result": &twins })))
}

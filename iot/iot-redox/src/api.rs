use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use log::{debug, error};
use serde_json::{json, Value};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use y::{
    clients::amqp::Amqp,
    models::{
        device_twin::{
            ConnectionState, MetaProperties, NewDevice, Record,
            StatusReason, Properties, TargetProperties,
        },
        DeviceTwin,
    },
};

use crate::twin_service::*;

pub struct ApiState {
    pub amqp: Amqp,
    pub db: Surreal<Client>,
}

const DEVICE_ID_KEY: &str = "device_id";
const ETAG_KEY: &str = "etag";

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
        },
        TargetProperties::Tag => {
            let twins_tag: &Vec<Option<Properties>> = &twins
            .iter()
            .map(|twin| twin.tag_properties.clone())
            .collect();
            Ok(Json(json!({ "result": twins_tag })))
        },
        TargetProperties::Desired => {
            let twins_desired: &Vec<Option<Properties>> = &twins
            .iter()
            .map(|twin| twin.desired_properties.clone())
            .collect();
            Ok(Json(json!({ "result": twins_desired })))
        },
        TargetProperties::Reported => {
            let twins_reported: &Vec<Option<Properties>> = &twins
            .iter()
            .map(|twin| twin.reported_properties.clone())
            .collect();
            Ok(Json(json!({ "result": twins_reported })))
        },
        TargetProperties::All => {
            Ok(Json(json!({ "result": twins })))
        }
    }

}

pub async fn update_device_twins_properties(
    State(state): State<Arc<ApiState>>,
    Path(target): Path<TargetProperties>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<Properties>,
) -> Result<Json<Value>, StatusCode> {
    debug!("update_device_twin");
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    let mut etag = "".to_string();
    if params.contains_key(ETAG_KEY) {
        etag = params[ETAG_KEY].clone();
    }
    dbg!(&device_id);
    dbg!(&payload);

    // TODO: Find etag from device_id

    let twins = &update_device_twins_properties_in_db(state.db.clone(), etag.as_str(), target, payload)
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    dbg!(&twins);
    Ok(Json(json!({ "result": twins })))
}

pub async fn create_device_twins(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<NewDevice>,
) -> Result<Json<Value>, StatusCode> {
    debug!("create_device_twin");
    dbg!(&payload);

    // TODO: Find etag from device_id

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
    Query(params): Query<HashMap<String, String>>
) -> Result<Json<Value>, StatusCode> {
    debug!("delete_device_twin");
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    let mut etag = "".to_string();
    if params.contains_key(ETAG_KEY) {
        etag = params[ETAG_KEY].clone();
    }
    dbg!(&device_id, &etag);

    // TODO: Find etag from device_id

    let twins = &delete_device_twins_in_db(state.db.clone(), etag.as_str())
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    dbg!(&twins);
    Ok(Json(json!({ "result": &twins })))
}
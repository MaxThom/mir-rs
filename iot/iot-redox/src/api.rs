use std::{sync::Arc, collections::HashMap};

use axum::{extract::{State, Query}, Json, http::StatusCode};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use surrealdb::{engine::remote::ws::Client, Surreal, sql::Thing, Response};
use y::{clients::amqp::Amqp, models::{DeviceTwin, device_twin::{StatusReason, Status, ConnectionState, NewDevice, MetaProperties, DesiredProperties, ReportedProperties, TagProperties}}};
use log::{error, info, trace, debug, };

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}


pub struct ApiState {
    pub amqp: Amqp,
    pub db: Surreal<Client>,
}

pub async fn get_records(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let twins: &Vec<Record> = &state.db.select("device_twin").await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(twins);


    Ok(Json(json!(twins)))
}

pub async fn get_device_twins(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key("device_id") {
        device_id = params["device_id"].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(twins);
    Ok(Json(json!(twins)))
}

pub async fn update_device_twins(State(state): State<Arc<ApiState>>, Json(payload): Json<DeviceTwin>) -> Result<Json<Value>, StatusCode> {
    //let mut device_id = "".to_string();
    //if params.contains_key("device_id") {
    //    device_id = params["device_id"].clone();
    //}
    //let twins = &update_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
    //    error!("Error: {}", error);
    //    StatusCode::INTERNAL_SERVER_ERROR
    //})?;
    //dbg!(twins);
    //Ok(Json(json!(twins)))
    todo!()
}

pub async fn get_device_twins_meta(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key("device_id") {
        device_id = params["device_id"].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let twins_meta: &Vec<MetaProperties> = &twins.iter().map(|twin| {
        twin.meta_properties.clone()
    }).collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_tag(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key("device_id") {
        device_id = params["device_id"].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let twins_meta: &Vec<TagProperties> = &twins.iter().map(|twin| {
        twin.tag_properties.clone()
    }).collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_desired(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key("device_id") {
        device_id = params["device_id"].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let twins_meta: &Vec<DesiredProperties> = &twins.iter().map(|twin| {
        twin.desired_properties.clone()
    }).collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_reported(State(state): State<Arc<ApiState>>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    let mut device_id = "".to_string();
    if params.contains_key("device_id") {
        device_id = params["device_id"].clone();
    }
    let twins = &get_device_twins_from_db(state.db.clone(), device_id.as_str()).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let twins_meta: &Vec<ReportedProperties> = &twins.iter().map(|twin| {
        twin.reported_properties.clone()
    }).collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn create_device_twins(State(state): State<Arc<ApiState>>, Json(payload): Json<NewDevice>) -> Result<Json<Value>, StatusCode> {
    // TODO: check if device is unique
    let x = DeviceTwin {
        meta_properties: MetaProperties {
            device_id: payload.device_id.clone(),
            model_id: payload.model_id,
            status: payload.status,
            status_reason: StatusReason::Provisioned,
            status_update_time: Utc::now().timestamp_nanos(),
            connection_state: ConnectionState::Disconnected,
            last_activity_time: Utc::now().timestamp_nanos(),
            version: 1,
        },
        tag_properties: TagProperties::default(),
        desired_properties: DesiredProperties::default(),
        reported_properties: ReportedProperties::default(),
    };

    let created: Record = state.db
        .create("device_twin")
        .content(x)
        .await.map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    dbg!(&created);


    Ok(Json(json!({ "records": &created })))
}

async fn get_device_twins_from_db(db: Surreal<Client>, device_id: &str) -> Result<Vec<DeviceTwin>, surrealdb::Error> {
    if !device_id.is_empty() {
        // Filter on device id
        let mut results = db
            .query("SELECT * FROM device_twin WHERE meta_properties.device_id = $device_id")
            .bind(("device_id", device_id))
            .await?;
            let twin: Vec<DeviceTwin> = results.take(0)?;
            return Ok(twin);
    }

    // Return all device twins meta
    let twins: Vec<DeviceTwin> = db.select("device_twin").await?;
    return Ok(twins);
}

async fn update_device_twins_from_db(db: Surreal<Client>, device_id: &str, device_twin: DeviceTwin) -> Result<Vec<Record>, surrealdb::Error> {
   // TODO: Implement patching for each categories of properties
    let updated: Vec<Record> = db
        .update("device_twin")
        .content(device_twin)
        .await?;
    dbg!(&updated);
    Ok(updated)
}
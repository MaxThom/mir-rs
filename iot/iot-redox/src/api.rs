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

use crate::twin_service::{
    generate_threadsafe_random_string, get_device_twins_from_db, update_device_twins_properties_in_db,
};

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

    Ok(Json(json!(twins)))
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
    Ok(Json(json!(twins)))
}

pub async fn update_device_twins(
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
    Ok(Json(json!("")))
}

pub async fn get_device_twins_meta(
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
    let twins_meta: &Vec<Option<MetaProperties>> = &twins
        .iter()
        .map(|twin| twin.meta_properties.clone())
        .collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_tag(
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
    let twins_meta: &Vec<Option<Properties>> = &twins
        .iter()
        .map(|twin| twin.tag_properties.clone())
        .collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_desired(
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
    let twins_meta: &Vec<Option<Properties>> = &twins
        .iter()
        .map(|twin| twin.desired_properties.clone())
        .collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn get_device_twins_reported(
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
    let twins_meta: &Vec<Option<Properties>> = &twins
        .iter()
        .map(|twin| twin.reported_properties.clone())
        .collect();
    dbg!(twins_meta);
    Ok(Json(json!(twins_meta)))
}

pub async fn create_device_twins(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<NewDevice>,
) -> Result<Json<Value>, StatusCode> {
    let etag: String = generate_threadsafe_random_string();

    let id = Thing::from((String::from("device_twin"), etag));
    println!("id: {:?}", id);

    // TODO: check if device is unique
    let x = DeviceTwin {
        id: None,
        meta_properties: Some(MetaProperties {
            device_id: payload.device_id.clone(),
            model_id: payload.model_id,
            etag: id.id.to_string(),
            status: payload.status,
            status_reason: StatusReason::Provisioned,
            status_update_time: Utc::now().timestamp_nanos(),
            connection_state: ConnectionState::Disconnected,
            last_activity_time: Utc::now().timestamp_nanos(),
            version: 1,
        }),
        tag_properties: Some(Properties::default()),
        desired_properties: Some(Properties::default()),
        reported_properties: Some(Properties::default()),
    };

    let created: Record = state.db.create(id).content(x).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(&created);

    Ok(Json(json!({ "records": &created })))
}

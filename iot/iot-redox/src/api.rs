use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use log::{debug, error, info, trace};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Response, Surreal};
use y::{
    clients::amqp::Amqp,
    models::{
        device_twin::{
            ConnectionState, DesiredProperties, MetaProperties, NewDevice, ReportedProperties,
            Status, StatusReason, TagProperties,
        },
        DeviceTwin,
    },
};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

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
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<DeviceTwin>,
) -> Result<Json<Value>, StatusCode> {
    debug!("update_device_twin");
    let mut device_id = "".to_string();
    if params.contains_key(DEVICE_ID_KEY) {
        device_id = params[DEVICE_ID_KEY].clone();
    }
    dbg!(&device_id);
    dbg!(&payload);
    let twins =
        &update_device_twins_in_db(state.db.clone(), device_id.as_str(), payload.tag_properties)
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
    let twins_meta: &Vec<MetaProperties> = &twins
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
    let twins_meta: &Vec<TagProperties> = &twins
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
    let twins_meta: &Vec<DesiredProperties> = &twins
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
    let twins_meta: &Vec<ReportedProperties> = &twins
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
        etag: id.id.to_string(),
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

    let created: Record = state.db.create(id).content(x).await.map_err(|error| {
        error!("Error: {}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(&created);

    Ok(Json(json!({ "records": &created })))
}

async fn get_device_twins_from_db(
    db: Surreal<Client>,
    device_id: &str,
) -> Result<Vec<DeviceTwin>, surrealdb::Error> {
    if !device_id.is_empty() {
        // Filter on device id
        let mut results = db
            .query("SELECT * FROM device_twin WHERE meta_properties.device_id = $device_id")
            .bind((DEVICE_ID_KEY, device_id))
            .await?;
        let twin: Vec<DeviceTwin> = results.take(0)?;
        return Ok(twin);
    }

    // Return all device twins meta
    let twins: Vec<DeviceTwin> = db.select("device_twin").await?;
    return Ok(twins);
}

async fn update_device_twins_in_db(
    db: Surreal<Client>,
    device_id: &str,
    tags_properties: TagProperties,
) -> Result<Response, surrealdb::Error> {
    let results: Response = db
        .query("UPDATE device_twin SET tag_properties = $tag_prop WHERE meta_properties.device_id = $device_id")
        .bind((DEVICE_ID_KEY, device_id))
        .bind(("tag_prop", tags_properties))
        .await?;

    dbg!(&results);
    Ok(results)
}

fn generate_threadsafe_random_string() -> String {
    let rng = Arc::new(Mutex::new(thread_rng()));
    let chars: String = {
        let mut guarded_rng = rng.lock().unwrap();
        guarded_rng.clone().sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    };
    chars
}
use std::sync::{Arc, Mutex};

use chrono::Utc;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use surrealdb::sql::Thing;
use surrealdb::{engine::remote::ws::Client, opt::PatchOp, Surreal};
use thiserror::Error as ThisError;
use x::device_twin::DeviceTwin;
use x::device_twin::NewDeviceReq;
use x::device_twin::{ConnectionState, MetaProperties, Properties, StatusReason, TargetProperties};

const DEVICE_ID_KEY: &str = "device_id";

#[derive(ThisError, Debug)]

pub enum TwinServiceError {
    #[error("surrealdb error: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("message error: {0}")]
    Msg(String),
    #[error("message error: {0}")]
    RecordNotFound(String),
    #[error("record version mistmatch: stored {0}, requested {1}")]
    RecordNewer(usize, usize),
}

pub async fn get_device_twins_from_db(
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
    Ok(twins)
}

pub async fn get_device_twins_with_id_from_db(
    db: &Surreal<Client>,
    device_id: &str,
) -> Result<Option<DeviceTwin>, surrealdb::Error> {
    // Return all device twins meta
    let twin: Option<DeviceTwin> = db.select(("device_twin", device_id)).await?;
    Ok(twin)
}

pub async fn create_device_twins_in_db(
    db: Surreal<Client>,
    payload: NewDeviceReq,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    //let device_id: String = generate_threadsafe_random_string();

    let id = Thing::from((String::from("device_twin"), payload.device_id.clone()));
    println!("id: {:?}", id);

    let x = DeviceTwin {
        id: None,
        meta_properties: Some(MetaProperties {
            device_id: payload.device_id.clone(),
            model_id: payload.model_id,
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

    let created: Option<DeviceTwin> = db.create(id).content(x).await?;
    dbg!(&created);
    Ok(created)
}

pub async fn update_device_twins_properties_in_db(
    db: Surreal<Client>,
    device_id: &str,
    target: &TargetProperties,
    properties: &Properties,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    //let updated: Record = db.update(("device_twin", device_id)).merge(device_twin).await?;

    // A timestamp could be saved with each properties and version
    // This timestamp would be compare with the oie stored and only update if newer
    // This would help concurrent operation

    // Get version
    // If version is newer, update
    // Client add their +1 to the version
    // If version is older, return error -> please refresh twin for more recent change
    let device = get_device_twins_with_id_from_db(&db, device_id).await?;
    if let Some(device) = device {
        let current_version: usize = match target {
            TargetProperties::Desired => device.desired_properties.unwrap().version,
            TargetProperties::Reported => device.reported_properties.unwrap().version,
            TargetProperties::Tag => device.tag_properties.unwrap().version,
            TargetProperties::Meta => todo!(),
            TargetProperties::All => todo!(),
        };

        // The incoming update has to be higher, in oxi|dizer, we add +1 to the version for the developer.
        if current_version > properties.version {
            return Err(TwinServiceError::RecordNewer(
                current_version,
                properties.version,
            ));
        }
    } else {
        return Err(TwinServiceError::RecordNotFound(device_id.to_string()));
    }

    let updated: Result<Option<DeviceTwin>, surrealdb::Error> = db
        .update(("device_twin", device_id))
        .patch(PatchOp::replace(
            format!("/{}", target.as_device_twin_route()).as_str(),
            properties,
        ))
        .await;

    if let Ok(updated) = updated {
        dbg!(&updated);
        Ok(updated)
    } else {
        Err(TwinServiceError::Msg(
            "Warning updating device twin, see: https://github.com/surrealdb/surrealdb/issues/1998"
                .to_string(),
        ))
    }
}

pub async fn delete_device_twins_in_db(
    db: Surreal<Client>,
    device_id: &str,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    let deleted: Option<DeviceTwin> = db.delete(("device_twin", device_id)).await?;

    dbg!(&deleted);
    Ok(deleted)
}

pub fn generate_threadsafe_random_string() -> String {
    let rng = Arc::new(Mutex::new(thread_rng()));
    let chars: String = {
        let guarded_rng = rng.lock().unwrap();
        guarded_rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    };
    chars
}

pub async fn update_hearthbeat_in_db(
    db: Surreal<Client>,
    device_id: String,
    timestamp: i64,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    info!("Updating hearthbeat for device: {}", device_id);
    let updated: Option<DeviceTwin> = db
        .update(("device_twin", device_id))
        .patch(PatchOp::replace(
            "/meta_properties/last_activity_time",
            timestamp,
        ))
        .patch(PatchOp::replace(
            "/meta_properties/connection_state",
            ConnectionState::Connected,
        ))
        .await?;

    dbg!(&updated);
    Ok(updated)
}

use std::sync::{Arc, Mutex};

use chrono::Utc;
use log::{debug, info};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::Value;
use surrealdb::sql::Thing;
use surrealdb::{engine::remote::ws::Client, opt::PatchOp, Surreal};
use y::models::NewDevice;
use y::models::device_twin::{TargetProperties, Properties, MetaProperties, StatusReason, ConnectionState};
use y::models::{device_twin::Record, DeviceTwin};
use thiserror::Error as ThisError;

const DEVICE_ID_KEY: &str = "device_id";
const ETAG_KEY: &str = "etag";


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
    return Ok(twins);
}

pub async fn get_device_twins_with_id_from_db(
    db: &Surreal<Client>,
    etag: &str,
) -> Result<Option<DeviceTwin>, surrealdb::Error> {
    // Return all device twins meta
    let twin: Option<DeviceTwin> = db.select(("device_twin", etag)).await?;
    return Ok(twin);
}

pub async fn create_device_twins_in_db(
    db: Surreal<Client>,
    payload: NewDevice,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    let etag: String = generate_threadsafe_random_string();

    let id = Thing::from((String::from("device_twin"), etag));
    println!("id: {:?}", id);

    
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

    let created: Option<DeviceTwin> = db.create(id).content(x).await?;
    dbg!(&created);
    Ok(created)
}

pub async fn update_device_twins_properties_in_db(
    db: Surreal<Client>,
    etag: &str,
    target: TargetProperties,
    properties: Properties,
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    //let updated: Record = db.update(("device_twin", etag)).merge(device_twin).await?;

    
    
    // A timestamp could be saved with each properties and version
    // This timestamp would be compare with the oie stored and only update if newer
    // This would help concurrent operation

    // Get version
    // If version is newer, update
    // Client add their +1 to the version
    // If version is older, return error -> please refresh twin for more recent change
    let device = get_device_twins_with_id_from_db(&db, etag).await?;
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
            return Err(TwinServiceError::RecordNewer(current_version, properties.version));
        }
    } else {
        return Err(TwinServiceError::RecordNotFound(etag.to_string()));
    }

    let updated: Result<Option<DeviceTwin>, surrealdb::Error>  = db
        .update(("device_twin", etag))
        .patch(PatchOp::replace(
            format!("/{}", target.as_device_twin_route()).as_str(),
            properties,
        ))
        .await;

    if let Ok(updated) = updated {
        dbg!(&updated);
        Ok(updated)
    } else {
        return Err(TwinServiceError::Msg("Warning updating device twin, see: https://github.com/surrealdb/surrealdb/issues/1998".to_string()));
    }

    
}

pub async fn delete_device_twins_in_db(
    db: Surreal<Client>,
    etag: &str
) -> Result<Option<DeviceTwin>, TwinServiceError> {
    let deleted: Option<DeviceTwin> = db
        .delete(("device_twin", etag))
        .await?;

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

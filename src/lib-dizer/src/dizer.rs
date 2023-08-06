use crate::error::DizerError;
use chrono::Utc;
use log::{debug, error, info};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use x::telemetry::{DeviceHeartbeat, DeviceTelemetry, Telemetry};
use y::clients::amqp::Amqp;

const RMQ_STREAM_EXCHANGE_NAME: &str = "iot-stream";
const RMQ_STREAM_ROUTING_KEY: &str = "dizer.telemetry.v1";

const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_TWIN_HEARTHBEAT_ROUTING_KEY: &str = "dizer.hearthbeat.v1";
//const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-twin-desired";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";

const HEARTHBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct Dizer {
    pub config: Config,
    pub(crate) amqp: Amqp,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub device_id: String,
    pub log_level: String,
    pub mir_addr: String,
    pub thread_count: usize,
}

impl Dizer {
    pub async fn join_fleet(&mut self) -> Result<(), DizerError> {
        // Create amqp connection pool
        self.amqp = Amqp::new(self.config.mir_addr.clone(), self.config.thread_count);
        let test = self
            .amqp
            .get_connection()
            .await
            .map_err(|_| DizerError::CantConnectToMir)?;
        debug!("{:?}", test.status());

        setup_heartbeat_task(self.clone());

        info!(
            "{} (Class Dizer) has joined the fleet 🚀.",
            self.config.device_id
        );

        Ok(())
    }

    pub async fn leave_fleet(&mut self) -> Result<(), DizerError> {
        self.amqp.close();
        info!(
            "{} (Class Dizer) has left the fleet 🚀.",
            self.config.device_id
        );
        Ok(())
    }

    pub async fn send_telemetry(&self, telemetry: Telemetry) -> Result<&str, DizerError> {
        // Wrap
        let payload = DeviceTelemetry {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
            telemetry,
        };

        // Serialize & Send
        let str_payload = serde_json::to_string(&payload).unwrap();
        debug!("{:?}", str_payload);
        match self
            .amqp
            .send_message(
                &str_payload,
                RMQ_STREAM_EXCHANGE_NAME,
                RMQ_STREAM_ROUTING_KEY,
            )
            .await
        {
            Ok(x) => Ok(x),
            Err(_) => Err(DizerError::TelemetrySent), // TODO: Add error type to telemetry sent
        }
    }

    async fn send_hearthbeat(&self) -> Result<&str, DizerError> {
        let payload = DeviceHeartbeat {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
        };

        // Serialize & Send
        let str_payload = serde_json::to_string(&payload).unwrap();
        debug!("{:?}", str_payload);
        match self
            .amqp
            .send_message(
                &str_payload,
                RMQ_TWIN_EXCHANGE_NAME,
                RMQ_TWIN_HEARTHBEAT_ROUTING_KEY,
            )
            .await
        {
            Ok(x) => Ok(x),
            Err(_) => Err(DizerError::HeathbeatSent), // TODO: Add error type to telemetry sent
        }
    }
}

fn setup_heartbeat_task(dizer: Dizer) {
    tokio::spawn(async move {
        let mut interval = time::interval(HEARTHBEAT_INTERVAL);

        loop {
            interval.tick().await;
            debug!("HEARTHBEAT");
            if let Err(x) = dizer.send_hearthbeat().await {
                error!("error sending heartbeat: {}", x);
            }
        }
    });
}

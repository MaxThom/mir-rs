use crate::error::DizerError;
use chrono::Utc;
use log::{debug, info};
use serde::Deserialize;
use x::telemetry::{DeviceTelemetry, Telemetry};
use y::clients::amqp::Amqp;

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
            .send_message(&str_payload, "iot-stream", "dizer.telemetry.v1")
            .await
        {
            Ok(x) => Ok(x),
            Err(_) => Err(DizerError::TelemetrySent), // TODO: Add error type to telemetry sent
        }
    }
}

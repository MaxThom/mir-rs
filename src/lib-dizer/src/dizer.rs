use crate::error::DizerError;
use chrono::Utc;
use lapin::{
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
};
use log::{debug, error, info};
use serde::Deserialize;
use std::{f32::consts::E, fmt::Error, time::Duration};
use tokio::time;
use x::{
    device_twin::{DeviceTwin, Properties},
    telemetry::{DeviceHeartbeat, DeviceTelemetry, Telemetry},
};
use y::clients::amqp::{
    Amqp, AmqpError, AmqpRpcClient, ChannelSettings, ConsumerSettings, QueueSettings,
};

const RMQ_STREAM_EXCHANGE_NAME: &str = "iot-stream";
const RMQ_STREAM_ROUTING_KEY: &str = "dizer.telemetry.v1";

const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_TWIN_HEARTHBEAT_ROUTING_KEY: &str = "dizer.hearthbeat.v1";
const RMQ_TWIN_DESIRED_PROP_ROUTING_KEY: &str = "dizer.update.v1";
//const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-twin-desired";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";

const HEARTHBEAT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct Dizer {
    pub config: Config,
    pub(crate) amqp: Amqp,
    pub desired_prop_queue: Option<DesiredPropertiesQueue>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub device_id: String,
    pub log_level: String,
    pub mir_addr: String,
    pub thread_count: usize,
}

#[derive(Debug, Clone)]
pub struct DesiredPropertiesQueue {
    is_initialized: bool,
    rpc_client: AmqpRpcClient,
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

        self.desired_prop_queue = Some(DesiredPropertiesQueue {
            is_initialized: false,
            rpc_client: create_rpc_client(self.clone()).await,
        });

        if let Err(x) = self.request_desired_properties().await {
            error!("error requesting desired properties: {}", x)
        }

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

    pub async fn request_desired_properties(&self) -> Result<(), AmqpError> {
        //TODO: .is_initialized
        match self
            .desired_prop_queue
            .as_ref()
            .unwrap()
            .rpc_client
            .call(
                "requesting desired properties",
                RMQ_TWIN_EXCHANGE_NAME,
                RMQ_TWIN_HEARTHBEAT_ROUTING_KEY,
            ) // TODO: Proper message
            .await
        {
            Ok(_) => Ok(()),
            Err(x) => Err(x),
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

async fn create_rpc_client(dizer: Dizer) -> AmqpRpcClient {
    debug!("Creating RPC client");
    let rpc_client = dizer
        .amqp
        .create_rpc_client_queue(
            ChannelSettings::default(),
            QueueSettings {
                name: dizer.config.device_id.as_str(),
                options: QueueDeclareOptions {
                    exclusive: true,
                    ..Default::default()
                },
                arguments: FieldTable::default(),
            },
            ConsumerSettings {
                consumer_tag: dizer.config.device_id.as_str(),
                options: BasicConsumeOptions {
                    no_ack: true,
                    ..Default::default()
                },
                arguments: FieldTable::default(),
            },
        )
        .await;
    rpc_client
}

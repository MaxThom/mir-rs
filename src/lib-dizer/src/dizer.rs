use crate::error::DizerError;
use chrono::Utc;
use futures::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::{FieldTable, ShortString},
};
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::{
    fmt::{self, Error},
    sync::Arc,
    time::Duration,
};
use std::{option::Option, sync::Mutex};
use tokio::time;
use x::{
    device_twin::{DeviceTwin, Properties},
    telemetry::{DeviceDesiredRequest, DeviceHeartbeatRequest, DeviceTelemetryRequest, Telemetry},
};
use y::{
    clients::amqp::{
        Amqp, AmqpError, AmqpRpcClient, ChannelSettings, ConsumerSettings, QueueSettings,
    },
    utils::serialization::SerializationKind,
};

const RMQ_STREAM_EXCHANGE_NAME: &str = "iot-stream";
const RMQ_STREAM_ROUTING_KEY: &str = "dizer.telemetry.v1";

const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_TWIN_HEARTHBEAT_ROUTING_KEY: &str = "dizer.hearthbeat.v1";
const RMQ_TWIN_DESIRED_PROP_ROUTING_KEY: &str = "dizer.desired.v1";
//const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-twin-desired";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";

const HEARTHBEAT_INTERVAL: Duration = Duration::from_secs(60);

pub struct Dizer {
    pub config: Config,
    pub(crate) amqp: Amqp,
    pub receive_message_queue: Option<DesiredPropertiesQueue>,
    // TODO: could offer Fn instead of FnMut as well
    pub desired_prop_callback:
        Arc<Mutex<Option<Box<dyn FnMut(Option<Properties>, Option<ShortString>) + Send + Sync>>>>,
}

impl Clone for Dizer {
    fn clone(&self) -> Self {
        let mut cloned = Dizer {
            config: self.config.clone(),
            amqp: self.amqp.clone(),
            receive_message_queue: self.receive_message_queue.clone(),
            desired_prop_callback: Arc::new(Mutex::new(None)),
        };
        cloned
            .desired_prop_callback
            .clone_from(&self.desired_prop_callback);
        cloned
    }
}

impl fmt::Debug for Dizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cb = self.desired_prop_callback.lock().unwrap();
        let desired_cb = if let Some(_) = *cb { "Some" } else { "None" };

        f.debug_struct("Dizer")
            .field("config", &self.config)
            .field("amqp", &self.amqp)
            .field("desired_prop_queue", &self.receive_message_queue)
            .finish()
    }
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
        let connect = self
            .amqp
            .get_connection()
            .await
            .map_err(|_| DizerError::CantConnectToMir)?;
        debug!("{:?}", connect.status());

        // Mata + heathbeat
        setup_heartbeat_task(self.clone());

        // Setup receiving queue for mir -> device communication
        setup_consume_message_received(self.clone(), self.desired_prop_callback.clone());
        //self.receive_message_queue = Some(DesiredPropertiesQueue {
        //    is_initialized: false,
        //    rpc_client: create_rpc_client(self.clone()).await,
        //});
        //setup_received_message_listen(
        //    self.receive_message_queue
        //        .as_ref()
        //        .unwrap()
        //        .rpc_client
        //        .clone(),
        //    self.desired_prop_callback.clone(),
        //);
        //if let Err(x) = self.send_desired_request().await {
        //    error!("error requesting desired properties: {}", x)
        //}

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
        let payload = DeviceTelemetryRequest {
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

    pub async fn send_desired_request(&self) -> Result<(), AmqpError> {
        //TODO: .is_initialized

        let payload = DeviceDesiredRequest {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
        };
        let str_payload = serde_json::to_string(&payload).unwrap();
        info!("call - {str_payload}");
        match self
            .receive_message_queue
            .as_ref()
            .unwrap()
            .rpc_client
            .call(
                &str_payload,
                RMQ_TWIN_EXCHANGE_NAME,
                RMQ_TWIN_DESIRED_PROP_ROUTING_KEY,
            ) // TODO: Proper message
            .await
        {
            Ok(_) => Ok(()),
            Err(x) => Err(x),
        }
    }

    async fn send_hearthbeat_request(&self) -> Result<&str, DizerError> {
        let payload = DeviceHeartbeatRequest {
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

fn setup_consume_message_received(
    dizer: Dizer,
    desired_prop_callback: Arc<
        Mutex<Option<Box<dyn FnMut(Option<Properties>, Option<ShortString>) + Send + Sync>>>,
    >,
) {
    tokio::spawn(async move {
        info!("listening to desired properties queue");
        // TODO: add loop over listen for error restart
        dizer
            .amqp
            .consume_queue(
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
                        ..Default::default()
                    },
                    arguments: FieldTable::default(),
                },
                SerializationKind::Json,
                move |payload, opt| {
                    let mut data = desired_prop_callback.lock().unwrap();
                    if let Some(x) = &mut *data {
                        x(payload, opt);
                    };
                    Ok::<(), Error>(())
                },
            )
            .await;
        info!("stop listening to desired properties queue");
    });
}

fn setup_received_message_listen(
    mut c: AmqpRpcClient,
    desired_prop_callback: Arc<Mutex<Option<Box<dyn FnMut(Option<Properties>) + Send + Sync>>>>,
) {
    tokio::spawn(async move {
        info!("listening to desired properties queue");
        // TODO: add loop over listen for error restart
        c.listen(SerializationKind::Json, move |payload| {
            let mut data = desired_prop_callback.lock().unwrap();
            if let Some(x) = &mut *data {
                x(payload);
            };
            Ok::<(), Error>(())
        })
        .await;
        info!("stop listening to desired properties queue");
    });
}

fn setup_heartbeat_task(dizer: Dizer) {
    tokio::spawn(async move {
        let mut interval = time::interval(HEARTHBEAT_INTERVAL);

        loop {
            interval.tick().await;
            debug!("HEARTHBEAT");
            if let Err(x) = dizer.send_hearthbeat_request().await {
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
                    ..Default::default()
                },
                arguments: FieldTable::default(),
            },
        )
        .await;
    rpc_client
}

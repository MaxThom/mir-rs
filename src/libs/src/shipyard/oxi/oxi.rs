//use crate::error::OxiError;
use crate::shipyard::oxi::error::OxiError;
use chrono::Utc;
use lapin::{
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::{FieldTable, ShortString},
};

use crate::models::{
    device_twin::Properties,
    telemetry::{
        DeviceDesiredRequest, DeviceHeartbeatRequest, DeviceReportedRequest,
        DeviceTelemetryRequest, Telemetry,
    },
};
use crate::{
    clients::amqp::{Amqp, AmqpError, ConsumerSettings, QueueSettings},
    utils::serialization::SerializationKind,
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Error},
    sync::Arc,
    time::Duration,
};
use std::{option::Option, sync::Mutex};
use tokio::time;

const RMQ_STREAM_EXCHANGE_NAME: &str = "iot-stream";
const RMQ_STREAM_ROUTING_KEY: &str = "oxi.telemetry.v1";

const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_TWIN_HEARTHBEAT_ROUTING_KEY: &str = "oxi.hearthbeat.v1";
const RMQ_TWIN_DESIRED_PROP_ROUTING_KEY: &str = "oxi.desired.v1";
const RMQ_TWIN_REPORTED_PROP_ROUTING_KEY: &str = "oxi.reported.v1";
//const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-twin-desired";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";

const HEARTHBEAT_INTERVAL: Duration = Duration::from_secs(60);

pub struct Oxi {
    pub config: Config,
    pub amqp: Amqp,
    // TODO: could offer Fn instead of FnMut as well
    pub desired_prop_callback:
        Arc<Mutex<Vec<Box<dyn FnMut(Option<Properties>, Option<ShortString>) + Send + Sync>>>>,
}

impl Clone for Oxi {
    fn clone(&self) -> Self {
        let mut cloned = Oxi {
            config: self.config.clone(),
            amqp: self.amqp.clone(),
            desired_prop_callback: Arc::new(Mutex::new(Vec::new())),
        };
        cloned
            .desired_prop_callback
            .clone_from(&self.desired_prop_callback);
        cloned
    }
}

impl fmt::Debug for Oxi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cb = self.desired_prop_callback.lock().unwrap();
        let msg_cb = cb.len();
        //let msg_cb = if let Some(_) = *cb { "Some" } else { "None" };

        f.debug_struct("Oxi")
            .field("config", &self.config)
            .field("amqp", &self.amqp)
            .field("message_cb", &msg_cb)
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

impl Oxi {
    pub async fn join_fleet(&mut self) -> Result<(), OxiError> {
        // Create amqp connection pool
        let connect = self
            .amqp
            .get_connection()
            .await
            .map_err(|_| OxiError::CantConnectToMir)?;
        debug!("{:?}", connect.status());

        // Mata + heathbeat
        setup_heartbeat_task(self.clone());

        // Setup receiving queue for mir -> device communication
        setup_consume_message_received(self.clone(), self.desired_prop_callback.clone());

        // Request initial desired properties from mir
        info!("sending desired properties initial request");
        if let Err(x) = self.send_desired_properties_request().await {
            error!("error requesting desired properties: {}", x)
        }

        info!("{} has joined the fleet 🚀.", self.config.device_id);

        Ok(())
    }

    pub async fn leave_fleet(&mut self) -> Result<(), OxiError> {
        self.amqp.close();
        info!("{} has left the fleet 🚀.", self.config.device_id);
        Ok(())
    }

    pub async fn send_telemetry(&self, telemetry: Telemetry) -> Result<&str, OxiError> {
        // Wrap
        let payload = DeviceTelemetryRequest {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
            telemetry,
        };

        // Serialize & Send
        self.send_data_as_type(RMQ_STREAM_ROUTING_KEY, payload)
            .await
    }

    // TODO: Offer json serialization, msgpack, others
    pub async fn send_data_as_type<T>(&self, routing_key: &str, data: T) -> Result<&str, OxiError>
    where
        T: Serialize,
    {
        // Serialize & Send
        let str_data = serde_json::to_string(&data).unwrap();
        self.send_data(routing_key, &str_data).await
    }

    // TODO Add [u8] fn instead of str
    pub async fn send_data(&self, routing_key: &str, data: &str) -> Result<&str, OxiError> {
        // Serialize & Send
        debug!("{:?}", data);
        match self
            .amqp
            .send_message(&data, RMQ_STREAM_EXCHANGE_NAME, routing_key)
            .await
        {
            Ok(x) => Ok(x),
            Err(_) => Err(OxiError::TelemetrySent), // TODO: Add error type to telemetry sent
        }
    }

    pub async fn send_desired_properties_request(&self) -> Result<(), AmqpError> {
        //TODO: .is_initialized
        let channel = self.amqp.get_channel().await?;
        let payload = DeviceDesiredRequest {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
        };
        let str_payload = serde_json::to_string(&payload).unwrap();
        match Amqp::send_message_with_reply(
            &channel,
            str_payload.as_str(),
            RMQ_TWIN_EXCHANGE_NAME,
            RMQ_TWIN_DESIRED_PROP_ROUTING_KEY,
            self.config.device_id.as_str(),
            String::from(""),
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(x) => Err(x),
        }
    }

    async fn send_hearthbeat_request(&self) -> Result<&str, OxiError> {
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
            Err(_) => Err(OxiError::HeathbeatSent), // TODO: Add error type to telemetry sent
        }
    }

    pub async fn send_reported_properties_request(
        &self,
        properties: Properties,
    ) -> Result<&str, OxiError> {
        let payload = DeviceReportedRequest {
            device_id: self.config.device_id.clone(),
            timestamp: Utc::now().timestamp_nanos(),
            reported_properties: properties,
        };

        // Serialize & Send
        let str_payload = serde_json::to_string(&payload).unwrap();
        debug!("{:?}", str_payload);
        match self
            .amqp
            .send_message(
                &str_payload,
                RMQ_TWIN_EXCHANGE_NAME,
                RMQ_TWIN_REPORTED_PROP_ROUTING_KEY,
            )
            .await
        {
            Ok(x) => Ok(x),
            Err(_) => Err(OxiError::ReportedSent),
        }
    }

    pub fn add_desired_properties_handler(
        &mut self,
        callback: impl FnMut(Option<Properties>, Option<ShortString>) + Send + Sync + 'static,
    ) {
        //let cb = Some(Box::new(callback));
        //self.desired_prop_callback = Arc::new(Mutex::new(Some(Box::new(callback))));
        self.desired_prop_callback
            .lock()
            .unwrap()
            .push(Box::new(callback));
    }
}

fn setup_consume_message_received(
    oxi: Oxi,
    desired_prop_callback: Arc<
        Mutex<Vec<Box<dyn FnMut(Option<Properties>, Option<ShortString>) + Send + Sync>>>,
    >,
) {
    tokio::spawn(async move {
        info!("started consuming desired properties");
        // TODO: add loop over listen for error restart
        oxi.amqp
            .consume_queue(
                QueueSettings {
                    name: oxi.config.device_id.as_str(),
                    options: QueueDeclareOptions {
                        exclusive: true,
                        ..Default::default()
                    },
                    arguments: FieldTable::default(),
                },
                ConsumerSettings {
                    consumer_tag: oxi.config.device_id.as_str(),
                    options: BasicConsumeOptions {
                        ..Default::default()
                    },
                    arguments: FieldTable::default(),
                },
                SerializationKind::Json,
                move |payload: Option<Properties>, opt: Option<ShortString>| {
                    info!("received desired properties message");
                    let mut data = desired_prop_callback.lock().unwrap();
                    for cb in &mut *data {
                        cb(payload.clone(), opt.clone());
                    }
                    Ok::<(), Error>(())
                },
            )
            .await;
        info!("stopped consuming desired properties");
    });
}

fn setup_heartbeat_task(oxi: Oxi) {
    info!("started heartbeat");
    tokio::spawn(async move {
        let mut interval = time::interval(HEARTHBEAT_INTERVAL);

        loop {
            interval.tick().await;
            debug!("hearthbeat");
            if let Err(x) = oxi.send_hearthbeat_request().await {
                error!("error sending heartbeat: {}", x);
            }
        }
    });
}

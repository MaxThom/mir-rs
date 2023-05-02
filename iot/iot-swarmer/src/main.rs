
use chrono::Utc;
use device::{Device};
use lapin::options::{ExchangeDeclareOptions};
use lapin::types::FieldTable;
use log::{debug, error, info, trace};
use serde::{Deserialize};
use tokio::time::{sleep, Duration};
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;

use y::clients::amqp::{Amqp};
use y::models::DevicePayload;
use y::utills::logger::setup_logger;
use y::utills::config::{setup_config, FileFormat};


use device::LiveDevice;

mod device;

#[derive(ThisError, Debug)]
enum Error {
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub devices: Vec<Device>,
    pub log_level: String,
    pub amqp_addr: String,
    pub amqp_conn_count: usize,
}

// https://blog.logrocket.com/configuration-management-in-rust-web-services/
// https://tokio.rs/tokio/topics/shutdown

const APP_NAME: &str = "swarmer";

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings: Settings = setup_config(APP_NAME, FileFormat::YAML).unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

    let amqp = Amqp::new(settings.amqp_addr.clone(), settings.amqp_conn_count);
    match amqp.declare_exchange(
        "iot",
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await {
        Ok(()) => info!("topic exchange <iot> declared"),
        Err(error) => error!("can't create topic exchange <iot> {}", error)
    };

    for device in settings.devices {
        for i in 0..device.count {
            let y = device.clone();
            let cloned_token = token.clone();
            let cloned_amqp = amqp.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        debug!("The token was shutdown")
                    }
                    _ = start_device(cloned_amqp, i, y) => {
                        debug!("device shuting down...");
                    }
                }
            });
        }
    }

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutting down...");
            token.cancel();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
    info!("Shutdown complete.");
}

async fn start_device(amqp: Amqp, index: u32, template: Device)  {
    // Create virtual device
    let mut device = LiveDevice::from_template(&template, index).unwrap();

    // Loop
    loop {
        // Generate
        let mut payload = DevicePayload::default();
        payload.device_id = device.name.clone();
        payload.timestamp = Utc::now().to_string();
        for sensor in &mut device.sensors {
            let x = sensor.telemetry.next_datapoint();
            payload.payload.insert(sensor.name.clone(), x);
        }
        info!("{:?}", payload);

        // Serialize & Send
        let str_payload = serde_json::to_string(&payload).unwrap();
        match amqp.send_message(&str_payload, "iot", "swarm.telemetry.v1").await
        {
            Ok(_) => trace!("message sent"),
            Err(error) => error!("can't send message {}", error)
        };
        sleep(Duration::from_secs(template.send_interval_second.into())).await;
    }
}

use config::{Config, ConfigError, Environment, File};
use deadpool_lapin::{PoolError};
use device::{Device};
use fern::colors::{Color, ColoredLevelConfig};
use lapin::options::{ExchangeDeclareOptions};
use lapin::types::FieldTable;
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use std::time::{SystemTime};
use std::{collections::HashMap};
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;
use chrono::Utc;
use y::clients::amqp::{Amqp};


use device::LiveDevice;

mod device;

//type RMQResult<T> = Result<T, PoolError>;

//type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

#[derive(ThisError, Debug)]
enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub devices: Vec<Device>,
    pub log_level: String,
    pub amqp_addr: String,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct DevicePayload {
    pub device_id: String,
    pub timestamp: String,
    pub payload: HashMap<String, f32>,
}

const CONFIG_FILE_PATH_DEFAULT: &str = "./config/swarmer.yaml";
const CONFIG_FILE_PATH_LOCAL: &str = "./config/local_swarmer.yaml";
// This makes it so "SWARMER_DEVICES__0__NAME overrides devices[0].name
const CONFIG_ENV_PREFIX: &str = "SWARMER";
const CONFIG_ENV_SEPARATOR: &str = "__";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH_DEFAULT))
            .add_source(File::with_name(CONFIG_FILE_PATH_LOCAL))
            .add_source(Environment::with_prefix(CONFIG_ENV_PREFIX).separator(CONFIG_ENV_SEPARATOR))
            .build()
            .unwrap();
        s.try_deserialize::<Self>()
    }
}

// https://blog.logrocket.com/configuration-management-in-rust-web-services/
// https://tokio.rs/tokio/topics/shutdown

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings = Settings::new().unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

    let amqp = Amqp::new(settings.amqp_addr.clone(), 10);
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

fn setup_logger(log_level: String) -> Result<(), fern::InitError> {
    let level = match log_level.to_lowercase().trim() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
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
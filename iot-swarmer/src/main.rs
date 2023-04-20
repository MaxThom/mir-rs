use brotli::CompressorWriter;
use config::{Config, ConfigError, Environment, File};
use deadpool_lapin::{Manager, Object, Pool, PoolError};
use device::{Device};
use fern::colors::{Color, ColoredLevelConfig};
use lapin::options::BasicPublishOptions;
use lapin::{BasicProperties, ConnectionProperties};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use std::time::{SystemTime};
use std::{collections::HashMap, io::Write};
use thiserror::Error as ThisError;
use tokio_amqp::*;
use tokio_util::sync::CancellationToken;
use chrono::Utc;

use crate::device::{
    LiveDevice
};

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

// TODO: Object recycling
// TODO: AMQP connection pooling

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings = Settings::new().unwrap();
    setup_logger(settings.log_level.clone()).unwrap();

    let manager = Manager::new(
        settings.amqp_addr.clone(),
        ConnectionProperties::default().with_tokio(),
    );
    let pool: Pool = Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can create pool");
    info!("{:?}", settings);

    for device in settings.devices {
        for i in 0..device.count {
            let y = device.clone();
            let cloned_token = token.clone();
            let cloned_pool = pool.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        debug!("The token was shutdown")
                    }
                    _ = start_device(cloned_pool, i, y) => {
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

async fn start_device(pool: Pool, index: u32, template: Device)  {
    // Create virtual device
    let mut device = LiveDevice::from_template(&template, index).unwrap();

    // Loop
    loop {
        let mut payload = DevicePayload::default();
        payload.device_id = device.name.clone();
        payload.timestamp = Utc::now().to_string();
        for sensor in &mut device.sensors {
            let x = sensor.telemetry.next_datapoint();
            payload.payload.insert(sensor.name.clone(), x);
        }
        info!("{:?}", payload);
        let str_payload = serde_json::to_string(&payload).unwrap();
        _ = send_message(&str_payload, pool.clone()).await;
        sleep(Duration::from_secs(template.send_interval_second.into())).await;
    }
}

async fn send_message(payload: &str, pool: Pool) -> Result<&str, Error> {
    debug!("send_message({})", payload);
    // Create message and compress using Brotli 10
    //let payload = "Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!".as_bytes();
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
        compressor.write_all(payload.as_bytes()).unwrap();
    }

    // Get connection
    let rmq_con = match get_rmq_con(pool).await.map_err(|e| {
        eprintln!("can't connect to rmq, {}", e);
        e
    }) {
        Ok(x) => x,
        Err(error) => return Err(error),
    };

    // Create channel
    let channel = match rmq_con.create_channel().await.map_err(|e| {
        eprintln!("can't create channel, {}", e);
        e
    }) {
        Ok(x) => x,
        Err(error) => return Err(Error::RMQError(error)),
    };

    // Set encoding type
    let headers = BasicProperties::default().with_content_encoding("br".into());
    match channel
        .basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            &compressed_data,
            headers,
        )
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            e
        })?
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            e
        }) {
        Ok(x) => x,
        Err(error) => return Err(Error::RMQError(error)),
    };
    Ok("OK")
}

async fn get_rmq_con(pool: Pool) -> Result<Object, Error> {
    let connection = pool.get().await?;
    Ok(connection)
}


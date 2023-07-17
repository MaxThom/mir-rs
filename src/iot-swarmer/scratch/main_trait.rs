//
//use brotli::CompressorWriter;
//use deadpool_lapin::{Manager, Pool, PoolError, Object};
//use device::RealTelemetryGenerator;
//use lapin::options::BasicPublishOptions;
//use lapin::{ConnectionProperties, BasicProperties};
//use serde::Deserialize;
//use std::{collections::HashMap, io::Write};
//use thiserror::Error as ThisError;
//use tokio_amqp::*;
//use config::{Config, ConfigError, Environment, File};
//use log::{debug, error, info, trace, warn};
//use std::time::SystemTime;
//use fern::colors::{Color, ColoredLevelConfig};
//use tokio_util::sync::CancellationToken;
//
//use crate::device::{Swarm, LiveDevice, LiveSensor, PyramidTelemetryGenerator, LinearTelemetryGenerator, WaveTelemetryGenerator};
//
//mod device;
//
//type RMQResult<T> = Result<T, PoolError>;
//
//type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;
//
//#[derive(ThisError, Debug)]
//enum Error {
//    #[error("rmq error: {0}")]
//    RMQError(#[from] lapin::Error),
//    #[error("rmq pool error: {0}")]
//    RMQPoolError(#[from] PoolError),
//    #[error("unknown telemetry generator")]
//    UnknownTelemetryGeneratorError,
//}
//
//#[derive(Debug, Deserialize, Clone)]
//pub struct Sensor {
//    RMQError(#[from] lapin::Error),
//    #[error("rmq pool error: {0}")]
//    RMQPoolError(#[from] PoolError),
//    #[error("unknown telemetry generator")]
//    UnknownTelemetryGeneratorError,
//}
//
//#[derive(Debug, Deserialize, Clone)]
//pub struct Sensor {
//    pub name: String,
//    pub hysteresis: f32,
//    pub pattern_name: String,
//    pub pattern_args: HashMap<String, String>,
//}
//
//#[derive(Debug, Deserialize, Clone)]
//pub struct Device {
//    pub name: String,
//    pub count: u32,
//    pub send_interval_second: u32,
//    pub sensors: Vec<Sensor>,
//}
//
//#[derive(Debug, Deserialize, Clone)]
//pub struct Settings {
//    pub devices: Vec<Device>,
//    pub log_level: String,
//    pub amqp_addr: String,
//}
//
//const CONFIG_FILE_PATH_DEFAULT: &str = "./config/swarmer.yaml";
//const CONFIG_FILE_PATH_LOCAL: &str = "./config/local_swarmer.yaml";
//// This makes it so "SWARMER_DEVICES__0__NAME overrides devices[0].name
//const CONFIG_ENV_PREFIX: &str = "SWARMER";
//const CONFIG_ENV_SEPARATOR: &str = "__";
//
//impl Settings {
//    pub fn new() -> Result<Self, ConfigError> {
//        let s = Config::builder()
//            .add_source(File::with_name(CONFIG_FILE_PATH_DEFAULT))
//            .add_source(File::with_name(CONFIG_FILE_PATH_LOCAL))
//            .add_source(Environment::with_prefix(CONFIG_ENV_PREFIX).separator(CONFIG_ENV_SEPARATOR))
//            .build().unwrap();
//        s.try_deserialize::<Self>()
//    }
//}
//
//// https://blog.logrocket.com/configuration-management-in-rust-web-services/
//// https://tokio.rs/tokio/topics/shutdown
//
//// TODO: Object recycling
//// TODO: AMQP connection pooling
//
//#[tokio::main]
//async fn main() {
//    let token = CancellationToken::new();
//
//    let settings = Settings::new().unwrap();
//    setup_logger(settings.log_level.clone()).unwrap();
//
//    let manager = Manager::new(settings.amqp_addr.clone(), ConnectionProperties::default().with_tokio());
//    let pool: Pool = Pool::builder(manager)
//        .max_size(10)
//        .build()
//        .expect("can create pool");
//    info!("{:?}", settings);
//
//    let mut swarm = Swarm::new().unwrap();
//    for device in settings.devices {
//        for i in 0..device.count {
//            let mut live_device = LiveDevice::new(format!("{}-{}", device.name, i)).unwrap();
//            for sensor in device.sensors.iter() {
//                live_device.add_sensor(LiveSensor{
//                    name: sensor.name.clone(),
//                    hysteresis: sensor.hysteresis,
//                    telemetry: get_telemetry_generator_factory(sensor.pattern_name.as_str(), sensor.pattern_args.clone()).unwrap(),
//                });
//            }
//            swarm.add_device(live_device);
//        }
//    }
//
//    for dev in swarm.devices.iter() {
//        info!("Device: {}", dev.name);
//        for sensor in dev.sensors.iter() {
//            info!("Sensor: {}", sensor.name);
//            for dp in sensor.telemetry.cloned().iter() {
//                info!("Data point: {}", dp);
//            }
//        }
//    }
//
//    //for device in settings.devices {
//    //    // Create the sensors pattern
//    //    // Create the device
//    //    for i in 0..device.count {
//    //        // Create the device and copy pattern
//    //        let cloned_token = token.clone();
//    //        let cloned_pool = pool.clone();
//    //        let payload = format!("{}-{}", device.name, i);
//    //        tokio::spawn(async move {
//    //            tokio::select! {
//    //                // Step 3: Using cloned token to listen to cancellation requests
//    //                _ = cloned_token.cancelled() => {
//    //                    // The token was cancelled, task can shut down
//    //                    debug!("The token was shutdown")
//    //                }
//    //                result = send_message(payload.as_str(), cloned_pool) => {
//    //                    debug!("result: {} {}", payload, result.unwrap());
//    //                }
//    //            }
//    //        });
//    //    }
//    //}
//
//    match tokio::signal::ctrl_c().await {
//        Ok(()) => {
//            info!("Shutting down...");
//            token.cancel();
//        },
//        Err(err) => {
//            eprintln!("Unable to listen for shutdown signal: {}", err);
//        },
//    }
//    info!("Shutdown complete.");
//}
//
//fn setup_logger(log_level: String) -> Result<(), fern::InitError> {
//    let level = match log_level.to_lowercase().trim() {
//        "trace" => log::LevelFilter::Trace,
//        "debug" => log::LevelFilter::Debug,
//        "info" => log::LevelFilter::Info,
//        "warn" => log::LevelFilter::Warn,
//        "error" => log::LevelFilter::Error,
//        _ => log::LevelFilter::Info,
//    };
//
//    let colors = ColoredLevelConfig::new()
//        .info(Color::Green)
//        .debug(Color::Cyan)
//        .trace(Color::Magenta);
//
//    fern::Dispatch::new()
//        .format(move |out, message, record| {
//            out.finish(format_args!(
//                "[{} {} {}] {}",
//                humantime::format_rfc3339_seconds(SystemTime::now()),
//                colors.color(record.level()),
//                record.target(),
//                message
//            ))
//        })
//        .level(level)
//        .chain(std::io::stdout())
//        //.chain(fern::log_file("output.log")?)
//        .apply()?;
//    Ok(())
//}
//
//async fn send_message(payload : &str, pool: Pool) -> Result<&str, Error> {
//    debug!("send_message({})", payload);
//    // Create message and compress using Brotli 10
//    //let payload = "Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!".as_bytes();
//    let mut compressed_data = Vec::new();
//    {
//        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
//        compressor.write_all(payload.as_bytes()).unwrap();
//    }
//
//    // Get connection
//    let rmq_con = match get_rmq_con(pool).await.map_err(|e| {
//        eprintln!("can't connect to rmq, {}", e);
//        e
//    }) {
//        Ok(x) => x,
//        Err(error) => return Err(error)
//    };
//
//    // Create channel
//    let channel = match rmq_con.create_channel().await.map_err(|e| {
//        eprintln!("can't create channel, {}", e);
//        e
//    }) {
//        Ok(x) => x,
//        Err(error) => return Err(Error::RMQError(error))
//    };
//
//    // Set encoding type
//    let headers = BasicProperties::default().with_content_encoding("br".into());
//    match channel
//        .basic_publish(
//            "",
//            "hello",
//            BasicPublishOptions::default(),
//            &compressed_data,
//            headers,
//        )
//        .await
//        .map_err(|e| {
//            eprintln!("can't publish: {}", e);
//            e
//        })?
//        .await
//        .map_err(|e| {
//            eprintln!("can't publish: {}", e);
//            e
//        }) {
//        Ok(x) => x,
//        Err(error) => return Err(Error::RMQError(error))
//        };
//    Ok("OK")
//}
//
//async fn get_rmq_con(pool: Pool) -> Result<Object, Error> {
//    let connection = pool.get().await?;
//    Ok(connection)
//}
//
//fn get_telemetry_generator_factory(generator: &str, args: HashMap<String, String>) -> Result<Box<dyn Iterator<Item = f32>>, Error> {
//    return match generator.to_lowercase().trim() {
//        "linear" => {
//            let constant = args["constant"].parse::<f32>().unwrap();
//            Ok(Box::new(LinearTelemetryGenerator::new(constant).unwrap()))
//        },
//        "pyramid" => {
//            let rate = args["rate"].parse::<f32>().unwrap();
//            let min = args["min"].parse::<f32>().unwrap();
//            let max = args["max"].parse::<f32>().unwrap();
//            Ok(Box::new(PyramidTelemetryGenerator::new(rate, min, max).unwrap()))
//        },
//        "wave" => {
//            let rate = args["rate"].parse::<f32>().unwrap();
//            let min = args["min"].parse::<f32>().unwrap();
//            let max = args["max"].parse::<f32>().unwrap();
//            Ok(Box::new(WaveTelemetryGenerator::new(rate, min, max).unwrap()))
//        },
//        "real" => {
//            let rate = args["rate"].parse::<f32>().unwrap();
//            let min = args["min"].parse::<f32>().unwrap();
//            let max = args["max"].parse::<f32>().unwrap();
//            Ok(Box::new(RealTelemetryGenerator::new(rate, min, max).unwrap()))
//        },
//        _ => Err(Error::UnknownTelemetryGeneratorError)
//    };
//}
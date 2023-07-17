use std::sync::Arc;

use axum::routing::post;
use axum::{routing::get, Router};
use lapin::ExchangeKind;
use serde::Deserialize;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
pub mod api;
pub mod twin_service;

use lapin::{options::*, types::FieldTable};
use log::{debug, error, info, trace};
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;

use y::clients::amqp::{
    Amqp, AmqpSettings, ChannelSettings, ConsumerSettings, ExchangeSettings, QueueBindSettings,
    QueueSettings,
};
use y::models::DevicePayload;
use y::utils::cli::setup_cli;
use y::utils::config::{setup_config, FileFormat};
use y::utils::logger::setup_logger;
use y::utils::serialization::SerializationKind;

#[derive(ThisError, Debug)]
enum Error {
    #[error("surrealdb error: {0}")]
    SurrealDB(#[from] surrealdb::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadCound {
    pub meta_queue: usize,
    pub reported_queue: usize,
    pub web_srv_queues: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SurrealDb {
    pub user: String,
    pub password: String,
    pub addr: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub log_level: String,
    pub amqp_addr: String,
    pub surrealdb: SurrealDb,
    pub thread_count: ThreadCound,
    pub web_srv_port: usize,
}

const APP_NAME: &str = "redox";
const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_DEVICE_EXCHANGE_NAME: &str = "iot-devices";
const RMQ_TWIN_META_QUEUE_NAME: &str = "iot-q-twin-meta";
const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";
const RMQ_PREFETCH_COUNT: u16 = 10;

use std::path::PathBuf;

// https://www.cloudamqp.com/blog/part1-rabbitmq-best-practice.html
// docker run --rm --pull always -p 80:8000 -v ./surrealdb:/opt/surrealdb/ surrealdb/surrealdb:latest start --log trace --user root --pass root file:/opt/surrealdb/iot.db
// curl -X POST -u "root:root" -H "NS: iot" -H "DB: iot" -H "Accept: application/json" -d "SELECT * FROM device_twin" localhost:80/sql
// curl -X POST -u "root:root" -H "NS: iot" -H "DB: iot" -H "Accept: application/json" -d "SELECT * FROM type::table(device_twin) WHERE meta_properties.device_id = pig5" localhost:80/sql

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Init cli
    let matches = setup_cli();

    // Init token, logger & config
    let token = CancellationToken::new();
    let settings: Settings = setup_config(
        APP_NAME,
        FileFormat::YAML,
        matches.get_one::<PathBuf>(y::utils::cli::CONFIG_KEY),
    )
    .unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

    // Create amqp connection pool
    let amqp: Amqp = Amqp::new(
        settings.amqp_addr.clone(),
        settings.thread_count.meta_queue
            + settings.thread_count.reported_queue
            + settings.thread_count.web_srv_queues
            + 1,
    );

    // Create surrealdb connection. Surreal create handles multiple connections using channel. See .with_capacity(0)
    let db = Surreal::new::<Ws>(settings.surrealdb.addr)
        .with_capacity(0)
        .await?;
    db.signin(Root {
        username: &settings.surrealdb.user,
        password: &settings.surrealdb.password,
    })
    .await?;
    db.use_ns("iot").use_db("iot").await?;
    info!("connected to SurrealDb");

    // Exchange for device comms
    let test = amqp.get_connection().await.unwrap();
    println!("{:?}", test.status());

    let ch = amqp.get_channel().await.unwrap();
    match amqp
        .declare_exchange_with_channel(
            &ch,
            RMQ_DEVICE_EXCHANGE_NAME,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(()) => info!("topic exchange <iot-devices> declared"),
        Err(error) => error!("can't create topic exchange <iot-devices> {}", error),
    };
    println!("ASDASD");

    // Task for Meta queue
    for i in 0..settings.thread_count.meta_queue {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        //let mut sender = SenderBuilder::new(host_port.0.clone(), host_port.1.clone()).connect().unwrap();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue_meta(i, cloned_amqp) => {
                    debug!("device shuting down...");
                }
            }
        });
    }

    // Task for Reported queue
    for i in 0..settings.thread_count.reported_queue {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        //let mut sender = SenderBuilder::new(host_port.0.clone(), host_port.1.clone()).connect().unwrap();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue_reported(i, cloned_amqp) => {
                    debug!("device shuting down...");
                }
            }
        });
    }

    // Web Server
    let shared_state = Arc::new(api::ApiState {
        amqp: amqp.clone(),
        db: db.clone(),
    });
    let srv = Router::new()
        .route("/ready", get(ready))
        .route("/alive", get(alive))
        .route(
            "/devicetwins",
            get(api::get_device_twins)
                .post(api::create_device_twins)
                //.put(api::update_device_twins)
                .delete(api::delete_device_twins),
        )
        .route(
            "/devicetwins/:target",
            get(api::get_device_twins_properties).put(api::update_device_twins_properties),
        )
        .route("/devicetwins/records", get(api::get_records))
        .with_state(shared_state);
    let cloned_token = token.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = cloned_token.cancelled() => {
                debug!("The token was shutdown")
            }
            _ = async move {
                info!("serving Axum on 0.0.0.0:{} 🚀", settings.web_srv_port);
                axum::Server::bind(&format!("0.0.0.0:{}", settings.web_srv_port).parse().unwrap())
                    .serve(srv.into_make_service())
                    .await.unwrap();
            } => {
                debug!("device shuting down...");
            }
        }
    });

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
    trace!("Exiting...");
    Ok(())
}

async fn alive() -> String {
    format!("{}", true)
}

async fn ready() -> String {
    format!("{}", true)
}

async fn start_consuming_topic_queue_meta(index: usize, amqp: Amqp) {
    let settings = AmqpSettings {
        channel: ChannelSettings {
            prefetch_count: RMQ_PREFETCH_COUNT,
            options: BasicQosOptions::default(),
        },
        exchange: ExchangeSettings {
            name: RMQ_TWIN_EXCHANGE_NAME,
            kind: ExchangeKind::Topic,
            options: ExchangeDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue: QueueSettings {
            name: RMQ_TWIN_META_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings {
            routing_key: "#.twin_meta.v1",
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings {
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.consume_topic_queue(index, settings, SerializationKind::Json, move |payload| {
        push_to_puthost("sender", payload)
    })
    .await;
    debug!("{}: Shutting down...", index);
}

async fn start_consuming_topic_queue_reported(index: usize, amqp: Amqp) {
    let settings = AmqpSettings {
        channel: ChannelSettings {
            prefetch_count: RMQ_PREFETCH_COUNT,
            options: BasicQosOptions::default(),
        },
        exchange: ExchangeSettings {
            name: RMQ_TWIN_EXCHANGE_NAME,
            kind: ExchangeKind::Topic,
            options: ExchangeDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue: QueueSettings {
            name: RMQ_TWIN_REPORTED_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings {
            routing_key: "#.twin_reported.v1",
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings {
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.consume_topic_queue(index, settings, SerializationKind::Json, move |payload| {
        push_to_puthost("sender", payload)
    })
    .await;
    debug!("{}: Shutting down...", index);
}

fn push_to_puthost(sender: &str, payload: DevicePayload) -> Result<(), Error> {
    debug!("{}: {:?}", sender, payload);
    Ok(())
}

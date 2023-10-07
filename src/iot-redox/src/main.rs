use std::sync::Arc;

use axum::http::StatusCode;
use axum::{routing::get, Router};
use lapin::types::ShortString;
use lapin::ExchangeKind;
use serde::Deserialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
pub mod api;
pub mod twin_service;

use lapin::{options::*, types::FieldTable};
use log::{debug, error, info, trace};
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;

use x::device_twin::TargetProperties;
use x::telemetry::{DeviceDesiredRequest, DeviceHeartbeatRequest, DeviceReportedRequest};
use y::clients::amqp::{
    Amqp, AmqpSettings, ChannelSettings, ConsumerSettings, ExchangeSettings, QueueBindSettings,
    QueueSettings,
};
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
    pub desired_queue: usize,
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
const RMQ_TWIN_HEARTHBEAT_QUEUE_NAME: &str = "iot-q-hearthbeat";
const RMQ_TWIN_HEATHBEAT_ROUTING_KEY: &str = "#.hearthbeat.v1";
const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-reported";
const RMQ_TWIN_REPORTED_ROUTING_KEY: &str = "#.reported.v1";
const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-desired";
const RMQ_TWIN_DESIRED_ROUTING_KEY: &str = "#.desired.v1";

const RMQ_PREFETCH_COUNT: u16 = 10;

use std::path::PathBuf;

use crate::twin_service::*;

// https://www.cloudamqp.com/blog/part1-rabbitmq-best-practice.html
// docker run --rm --pull always -p 80:8000 -v ./surrealdb:/opt/surrealdb/ surrealdb/surrealdb:latest start --log trace --user root --pass root file:/opt/surrealdb/iot.db
// curl -X POST -u "root:root" -H "NS: iot" -H "DB: iot" -H "Accept: application/json" -d "SELECT * FROM device_twin" localhost:80/sql
// curl -X POST -u "root:root" -H "NS: iot" -H "DB: iot" -H "Accept: application/json" -d "SELECT * FROM type::table(device_twin) WHERE meta_properties.device_id = pig5" localhost:80/sql
//

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
    let amqp = Amqp::new(
        settings.amqp_addr.clone(),
        settings.thread_count.meta_queue
            + settings.thread_count.reported_queue
            + settings.thread_count.web_srv_queues
            + 3,
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

    // Task for Meta queue
    for i in 0..settings.thread_count.meta_queue {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        let cloned_db = db.clone();
        
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue_meta(i, cloned_amqp, cloned_db) => {
                    debug!("device shuting down...");
                }
            }
        });
    }

    // Task for Reported queue
    for i in 0..settings.thread_count.reported_queue {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        let cloned_db = db.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue_reported(i, cloned_amqp, cloned_db) => {
                    debug!("device shuting down...");
                }
            }
        });
    }

    // Task for Desired queue
    for i in 0..settings.thread_count.desired_queue {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        let cloned_db = db.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue_desired(i, cloned_amqp, cloned_db) => {
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

async fn start_consuming_topic_queue_meta(index: usize, amqp: Amqp, db: Surreal<Client>) {
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
            name: RMQ_TWIN_HEARTHBEAT_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings {
            routing_key: RMQ_TWIN_HEATHBEAT_ROUTING_KEY,
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings {
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };
    debug!("{}: Starting...", index);
    amqp.consume_topic_queue(
        index,
        settings,
        SerializationKind::Json,
        move |payload, _| receive_hearthbeat_request(db.clone(), payload),
    )
    .await;
    debug!("{}: Shutting down...", index);
}

async fn start_consuming_topic_queue_reported(index: usize, amqp: Amqp, db: Surreal<Client>) {
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
            routing_key: RMQ_TWIN_REPORTED_ROUTING_KEY,
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings {
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.clone()
        .consume_topic_queue(
            index,
            settings,
            SerializationKind::Json,
            move |payload, _| receive_reported_request(db.clone(), payload),
        )
        .await;
    debug!("{}: Shutting down...", index);
}

async fn start_consuming_topic_queue_desired(index: usize, amqp: Amqp, db: Surreal<Client>) {
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
            name: RMQ_TWIN_DESIRED_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings {
            routing_key: RMQ_TWIN_DESIRED_ROUTING_KEY,
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings {
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.clone()
        .consume_topic_queue(
            index,
            settings,
            SerializationKind::Json,
            move |payload, reply_to| {
                receive_desired_request(db.clone(), amqp.clone(), payload, reply_to)
            },
        )
        .await;
    debug!("{}: Shutting down...", index);
}

fn receive_hearthbeat_request(
    db: Surreal<Client>,
    payload: DeviceHeartbeatRequest,
) -> Result<(), Error> {
    let device_id = payload.device_id.clone();
    let ts = payload.timestamp.clone();
    tokio::spawn(async move {
        // TODO: retry logic
        let _ = update_hearthbeat_in_db(db, device_id, ts).await;
        //match resp {
        //    Ok(x) => debug!(
        //        "Updated hearthbeat for device '{}': {:?}",
        //        payload.device_id, x
        //    ),
        //    Err(e) => error!(
        //        "Error updating hearthbeat for device '{}': {}",
        //        payload.device_id, e
        //    ),
        //}
    });

    Ok(())
}

// TODO: Maybe async?
// what to send back?
// - error? and the device receives it and retry?
// - auto retry? here, both solution?
fn receive_desired_request(
    db: Surreal<Client>,
    amqp: Amqp,
    payload: DeviceDesiredRequest,
    reply_to: Option<ShortString>,
) -> Result<(), Error> {
    let device_id = payload.device_id.clone();
    let _ts = payload.timestamp.clone();
    tokio::spawn(async move {
        // TODO: retry logic
        // TODO: should you panic in async? or print out the error?
        let resp = get_device_twins_with_id_from_db(&db, device_id.as_str()).await;

        let opt_twin = if let Ok(resp) = resp {
            resp
        } else {
            error!("Error getting device twin from db: {}", resp.unwrap_err());
            return;
        };

        let twin = if let Some(opt_twin) = opt_twin {
            opt_twin
        } else {
            error!("Device '{device_id}' not found");
            return;
        };

        dbg!(&twin.desired_properties);

        let reply_queue = if let Some(reply_to) = reply_to {
            dbg!(&reply_to);
            reply_to
        } else {
            error!("No reply_to specified");
            return;
        };

        // Serialize & Send
        let str_twin = serde_json::to_string(&twin.desired_properties).unwrap();
        match amqp
            .send_message(&str_twin, "", ShortString::to_string(&reply_queue).as_str())
            .await
        {
            Ok(x) => {
                info!("{x}")
            }
            Err(e) => {
                error!("{:?}", e);
            } // TODO: Add error type to telemetry sent
        };
    });

    Ok(())
}

fn receive_reported_request(
    db: Surreal<Client>,
    payload: DeviceReportedRequest,
) -> Result<(), Error> {
    let device_id = payload.device_id.clone();
    let _ts = payload.timestamp.clone();
    tokio::spawn(async move {
        // TODO: retry logic

        // Update db
        let updated_twin_result = update_device_twins_properties_in_db(
            db.clone(),
            device_id.as_str(),
            &TargetProperties::Reported,
            &payload.reported_properties,
        )
        .await
        .map_err(|error| {
            error!("Error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        });

        let _ = if let Err(_) = updated_twin_result {
            //return Ok(Json(json!({ "result": 200 })));
            // TODO: proper return when surrealdb is fixed
            None
        } else {
            updated_twin_result.unwrap()
        };
    });

    Ok(())
}

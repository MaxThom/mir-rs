use lapin::ExchangeKind;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Thing, Field};
use surrealdb::Surreal;
use y::utills::network;

#[derive(Debug, Serialize)]
struct Name<'a> {
    first: &'a str,
    last: &'a str,
}

#[derive(Debug, Serialize)]
struct Person<'a> {
    title: &'a str,
    name: Name<'a>,
    marketing: bool,
}

#[derive(Debug, Serialize)]
struct Responsibility {
    marketing: bool,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}


use log::{error, info, trace, debug, };
use lapin::{options::*, types::FieldTable};
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;


use y::clients::amqp::{Amqp, AmqpSettings, ChannelSettings, ExchangeSettings, QueueSettings, ConsumerSettings, QueueBindSettings, AmqpError};
use y::models::DevicePayload;
use y::utills::logger::setup_logger;
use y::utills::config::{setup_config, FileFormat};
use y::utills::serialization::SerializationKind;

#[derive(ThisError, Debug)]
enum Error {
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadCound {
    pub meta_queue: usize,
    pub reported_queue: usize,
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
}

const APP_NAME: &str = "redox";
const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
const RMQ_DEVICE_EXCHANGE_NAME: &str = "iot-devices";
const RMQ_TWIN_META_QUEUE_NAME: &str = "iot-q-twin-meta";
const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";
const RMQ_PREFETCH_COUNT: u16 = 10;

// https://www.cloudamqp.com/blog/part1-rabbitmq-best-practice.html

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings: Settings = setup_config(APP_NAME, FileFormat::YAML).unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);


    let amqp: Amqp = Amqp::new(settings.amqp_addr.clone(), settings.thread_count.meta_queue + settings.thread_count.reported_queue);
    let host_port = network::parse_host_port(&settings.surrealdb.addr.as_str()).unwrap();

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


async fn start_consuming_topic_queue_meta(index: usize, amqp: Amqp) {
    let settings = AmqpSettings{
        channel: ChannelSettings{
            prefetch_count: RMQ_PREFETCH_COUNT,
            options: BasicQosOptions::default(),
        },
        exchange: ExchangeSettings{
            name: RMQ_TWIN_EXCHANGE_NAME,
            kind: ExchangeKind::Topic,
            options: ExchangeDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue: QueueSettings{
            name: RMQ_TWIN_META_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings{
            routing_key: "#.twin_meta.v1",
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings{
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.consume_topic_queue(index, settings, SerializationKind::Json, deserialize_message, move |payload| {
        push_to_puthost("sender", payload)
    }).await;
    debug!("{}: Shutting down...", index);
}

async fn start_consuming_topic_queue_reported(index: usize, amqp: Amqp) {
    let settings = AmqpSettings{
        channel: ChannelSettings{
            prefetch_count: RMQ_PREFETCH_COUNT,
            options: BasicQosOptions::default(),
        },
        exchange: ExchangeSettings{
            name: RMQ_TWIN_EXCHANGE_NAME,
            kind: ExchangeKind::Topic,
            options: ExchangeDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue: QueueSettings{
            name: RMQ_TWIN_REPORTED_QUEUE_NAME,
            options: QueueDeclareOptions::default(),
            arguments: FieldTable::default(),
        },
        queue_bind: QueueBindSettings{
            routing_key: "#.twin_reported.v1",
            options: QueueBindOptions::default(),
            arguments: FieldTable::default(),
        },
        consumer: ConsumerSettings{
            consumer_tag: "",
            options: BasicConsumeOptions::default(),
            arguments: FieldTable::default(),
        },
    };

    amqp.consume_topic_queue(index, settings, SerializationKind::Json, deserialize_message, move |payload| {
        push_to_puthost("sender", payload)
    }).await;
    debug!("{}: Shutting down...", index);
}

fn deserialize_message(payload: Vec<u8>) -> Result<DevicePayload, AmqpError> {
    //let device_payload: DevicePayload = serde_json::from_str(&uncompressed_message).unwrap();
    Ok(serde_json::from_slice(&payload).unwrap())
}

fn push_to_puthost(sender: &str, payload: DevicePayload) -> Result<(), Error> {
    debug!("{}: {:?}", sender, payload);
    Ok(())
}

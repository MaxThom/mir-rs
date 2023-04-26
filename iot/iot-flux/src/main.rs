
use futures::StreamExt;
use log::{error, info, };
use lapin::types::ShortString;
use lapin::{options::*, types::FieldTable};
use serde::Deserialize;
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;


use y::clients::amqp::{Amqp, AmqpError};
use y::models::DevicePayload;
use y::utills::logger::setup_logger;
use y::utills::config::{setup_config, FileFormat};

#[derive(ThisError, Debug)]
enum Error {
}


#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub log_level: String,
    pub amqp_addr: String,
}

const APP_NAME: &str = "flux";
const RMQ_EXCHANGE_NAME: &str = "iot";
const RMQ_QUEUE_NAME: &str = "iot-q-metrics";


#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings: Settings = setup_config(APP_NAME, FileFormat::YAML).unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);


    let amqp = Amqp::new(settings.amqp_addr.clone(), 1);

    match amqp.declare_exchange(
        RMQ_EXCHANGE_NAME,
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await {
        Ok(()) => info!("topic exchange <{}> declared", RMQ_EXCHANGE_NAME),
        Err(error) => error!("can't create topic exchange <{}> {}", RMQ_EXCHANGE_NAME, error)
    };
    let queue = match amqp.declare_queue(
        RMQ_QUEUE_NAME,
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(queue) => {
            info!("metrics queue <{}> declared", queue.name());
            queue
    },
        Err(error) => {
            error!("can't create metrics queue <{}> {}", RMQ_QUEUE_NAME, error);
            panic!("{}", error)
    }
    };

    match amqp.bind_queue(
        queue.name().as_str(),
        RMQ_EXCHANGE_NAME,
        "#.telemetry.v1",
        QueueBindOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(()) => info!("topic exchange <{}> and metric queue <{}> binded", RMQ_EXCHANGE_NAME, queue.name()),
        Err(error) => {
            error!("can't create binding <{}> <{}> {}", RMQ_EXCHANGE_NAME, RMQ_QUEUE_NAME, error);
            panic!("{}", error)}
    };

    let mut consumer = match amqp.create_consumer(
        RMQ_QUEUE_NAME,
        "",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(consumer) => {
            info!("consumer <{}> declared", consumer.tag());
            info!("consumer <{}> to queue <{}> binded", consumer.tag(), queue.name());
            consumer
        },
        Err(error) => {
            error!("can't bind consumer and queue <{}> {}", queue.name(), error);
            panic!("{}", error)
        }
    };

    let conn = match amqp.get_connection().await {
        Ok(channel) => channel,
        Err(error) => {
            error!("can't get connection {}", error);
            panic!("{}", error)
        }
    };

    let channel = match conn.create_channel().await.map_err(|e| {
        eprintln!("can't create channel, {}", e);
        e
    }) {
        Ok(x) => x,
        Err(error) => panic!("{}", error),
    };

    info!("consumer <{}> is liscening", consumer.tag());
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let payload: Vec<u8> = delivery.data.clone();
            let uncompressed_message = match delivery.properties.content_encoding().clone().unwrap_or_else(|| ShortString::from("")).as_str() {
                "br" => {
                    amqp.decompress_message(payload)
                }
                _ => {
                    Ok(String::from_utf8(payload).unwrap())
                }
            }.unwrap();

            let device_payload: DevicePayload = serde_json::from_str(&uncompressed_message).unwrap();
            info!("{:?}", device_payload);
            match channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await {
                Ok(()) => info!("acknowledged message <{}>", delivery.delivery_tag),
                Err(error) => error!("can't acknowledge message <{}> {}", delivery.delivery_tag, error)
            };
        };
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

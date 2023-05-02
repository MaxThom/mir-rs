
use futures::StreamExt;
use log::{error, info, trace, debug, };
use lapin::types::ShortString;
use lapin::{options::*, types::FieldTable};
use serde::Deserialize;
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;


use y::clients::amqp::{Amqp};
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
    pub thread_count: usize,
}

const APP_NAME: &str = "flux";
const RMQ_EXCHANGE_NAME: &str = "iot";
const RMQ_QUEUE_NAME: &str = "iot-q-metrics";
const RMQ_PREFETCH_COUNT: u16 = 10;

// https://www.cloudamqp.com/blog/part1-rabbitmq-best-practice.html
// https://github.com/infosechoudini/influxdb-rs
// https://github.com/influxdata/influxdb_iox

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings: Settings = setup_config(APP_NAME, FileFormat::YAML).unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);


    let amqp: Amqp = Amqp::new(settings.amqp_addr.clone(), settings.thread_count);

    for i in 0..settings.thread_count {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue(i, cloned_amqp) => {
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


async fn start_consuming_topic_queue(index: usize, amqp: Amqp) {
    // Get channel and declare topic, queue, binding and consumer
    let channel = &amqp.get_channel().await.unwrap();
    channel.basic_qos(RMQ_PREFETCH_COUNT, BasicQosOptions::default()).await.unwrap();
    match amqp.declare_exchange_with_channel(
        channel,
        RMQ_EXCHANGE_NAME,
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await {
        Ok(()) => info!("{}: topic exchange <{}> declared", index, RMQ_EXCHANGE_NAME),
        Err(error) => error!("{}: can't create topic exchange <{}> {}", index, RMQ_EXCHANGE_NAME, error)
    };
    let queue = match amqp.declare_queue_with_channel(
        channel,
        RMQ_QUEUE_NAME,
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(queue) => {
            info!("{}: metrics queue <{}> declared", index, queue.name());
            queue
    },
        Err(error) => {
            error!("{}: can't create metrics queue <{}> {}", index, RMQ_QUEUE_NAME, error);
            panic!("{}", error)
    }
    };

    match amqp.bind_queue_with_channel(
        channel,
        queue.name().as_str(),
        RMQ_EXCHANGE_NAME,
        "#.telemetry.v1",
        QueueBindOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(()) => info!("{}: topic exchange <{}> and metric queue <{}> binded", index, RMQ_EXCHANGE_NAME, queue.name()),
        Err(error) => {
            error!("{}: can't create binding <{}> <{}> {}", index, RMQ_EXCHANGE_NAME, RMQ_QUEUE_NAME, error);
            panic!("{}", error)}
    };

    let mut consumer = match amqp.create_consumer_with_channel(
        channel,
        RMQ_QUEUE_NAME,
        "",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await {
        Ok(consumer) => {
            info!("{}: consumer <{}> declared", index, consumer.tag());
            info!("{}: consumer <{}> to queue <{}> binded", index, consumer.tag(), queue.name());
            consumer
        },
        Err(error) => {
            error!("{}: can't bind consumer and queue <{}> {}", index, queue.name(), error);
            panic!("{}", error)
        }
    };

    // Consumer liscening to topic queue exchange
    info!("{}: consumer <{}> is liscening", index, consumer.tag());
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
            debug!("{}: {:?}", index, device_payload);
            match channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await {
                Ok(()) => trace!("{}: acknowledged message <{}>", index, delivery.delivery_tag),
                Err(error) => error!("{}: can't acknowledge message <{}> {}", index, delivery.delivery_tag, error)
            };
        };
    }
}
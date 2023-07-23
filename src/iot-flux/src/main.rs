use futures::StreamExt;
use lapin::types::ShortString;
use lapin::{options::*, types::FieldTable};
use log::{debug, error, info, trace};
use questdb::ingress::{Buffer, Sender, SenderBuilder, TimestampNanos};
use serde::Deserialize;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;
use y::utils::setup_cli;

use x::telemetry::DeviceTelemetry;
use y::clients::amqp::Amqp;
use y::utils::config::{setup_config, FileFormat};
use y::utils::logger::setup_logger;
use y::utils::network;

#[derive(ThisError, Debug)]
enum Error {
    #[error("rmq pool error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("put host error: {0}")]
    PutHostError(#[from] questdb::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub log_level: String,
    pub amqp_addr: String,
    pub questdb_addr: String,
    pub thread_count: usize,
}

const APP_NAME: &str = "flux";
const RMQ_EXCHANGE_NAME: &str = "iot-stream";
const RMQ_QUEUE_NAME: &str = "iot-q-telemetry";
const RMQ_PREFETCH_COUNT: u16 = 10;

#[tokio::main]
async fn main() {
    let matches = setup_cli();

    let token = CancellationToken::new();

    let settings: Settings = setup_config(
        APP_NAME,
        FileFormat::YAML,
        matches.get_one::<PathBuf>(y::utils::cli::CONFIG_KEY),
    )
    .unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

    let amqp: Amqp = Amqp::new(settings.amqp_addr.clone(), settings.thread_count);
    let host_port = network::parse_host_port(settings.questdb_addr.as_str()).unwrap();

    for i in 0..settings.thread_count {
        let cloned_token = token.clone();
        let cloned_amqp = amqp.clone();
        let mut sender = SenderBuilder::new(host_port.0.clone(), host_port.1.clone())
            .connect()
            .unwrap();
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    debug!("The token was shutdown")
                }
                _ = start_consuming_topic_queue(i, cloned_amqp,  move |payload| {
                    push_to_puthost(&mut sender, payload)
                }) => {
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

async fn start_consuming_topic_queue(
    index: usize,
    amqp: Amqp,
    mut callback: impl FnMut(DeviceTelemetry) -> Result<(), Error>,
) {
    // TODO: Could implement better TCP Connection and Ch
    // Get channel and declare topic, queue, binding and consumer
    let channel = &amqp.get_channel().await.unwrap();
    channel
        .basic_qos(RMQ_PREFETCH_COUNT, BasicQosOptions::default())
        .await
        .unwrap();
    match amqp
        .declare_exchange_with_channel(
            channel,
            RMQ_EXCHANGE_NAME,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(()) => info!("{}: topic exchange <{}> declared", index, RMQ_EXCHANGE_NAME),
        Err(error) => error!(
            "{}: can't create topic exchange <{}> {}",
            index, RMQ_EXCHANGE_NAME, error
        ),
    };
    let queue = match amqp
        .declare_queue_with_channel(
            channel,
            RMQ_QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(queue) => {
            info!("{}: metrics queue <{}> declared", index, queue.name());
            queue
        }
        Err(error) => {
            error!(
                "{}: can't create metrics queue <{}> {}",
                index, RMQ_QUEUE_NAME, error
            );
            panic!("{}", error)
        }
    };

    match amqp
        .bind_queue_with_channel(
            channel,
            queue.name().as_str(),
            RMQ_EXCHANGE_NAME,
            "#.telemetry.v1",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(()) => info!(
            "{}: topic exchange <{}> and metric queue <{}> binded",
            index,
            RMQ_EXCHANGE_NAME,
            queue.name()
        ),
        Err(error) => {
            error!(
                "{}: can't create binding <{}> <{}> {}",
                index, RMQ_EXCHANGE_NAME, RMQ_QUEUE_NAME, error
            );
            panic!("{}", error)
        }
    };

    let mut consumer = match amqp
        .create_consumer_with_channel(
            channel,
            RMQ_QUEUE_NAME,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        Ok(consumer) => {
            info!("{}: consumer <{}> declared", index, consumer.tag());
            info!(
                "{}: consumer <{}> to queue <{}> binded",
                index,
                consumer.tag(),
                queue.name()
            );
            consumer
        }
        Err(error) => {
            error!(
                "{}: can't bind consumer and queue <{}> {}",
                index,
                queue.name(),
                error
            );
            panic!("{}", error)
        }
    };

    // Consumer liscening to topic queue exchange
    info!("{}: consumer <{}> is liscening", index, consumer.tag());
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let payload: Vec<u8> = delivery.data.clone();
            let uncompressed_message = match delivery
                .properties
                .content_encoding()
                .clone()
                .unwrap_or_else(|| ShortString::from(""))
                .as_str()
            {
                "br" => amqp.decompress_message_as_str(payload),
                _ => Ok(String::from_utf8(payload).unwrap()),
            }
            .unwrap();

            let device_payload: DeviceTelemetry =
                serde_json::from_str(&uncompressed_message).unwrap();
            debug!("{}: {:?}", index, device_payload);
            callback(device_payload).unwrap();
            match channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
            {
                Ok(()) => trace!(
                    "{}: acknowledged message <{}>",
                    index,
                    delivery.delivery_tag
                ),
                Err(error) => error!(
                    "{}: can't acknowledge message <{}> {}",
                    index, delivery.delivery_tag, error
                ),
            };
        };
    }
    debug!("{}: Shutting down...", index);
}

fn push_to_puthost(sender: &mut Sender, payload: DeviceTelemetry) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let timestamp = payload.timestamp;
    let device_id = payload.device_id;
    for sensor in payload.telemetry.floats {
        let sensor_id = sensor.0;
        let value = sensor.1;
        buffer
            .table("Datapoint")?
            .column_str("device_id", device_id.to_string())?
            .column_i64("sensor_id", sensor_id)?
            .column_f64("value", value)?
            .at(TimestampNanos::new(timestamp).unwrap())?;
    }
    sender.flush(&mut buffer)?;

    Ok(())
}

use config::{ConfigError, Config, File, Environment};
use log::{debug, error, info, trace};
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
}

const APP_NAME: &str = "flux";

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();

    let settings: Settings = setup_config(APP_NAME, FileFormat::YAML).unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

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


//
//use lapin::{
//    options::*,
//    types::{FieldTable, ShortString},
//    BasicProperties, Connection, ConnectionProperties, Consumer,
//    ConsumerOptions, Delivery, Result,
//};
//
//async fn receive_events_from_topic() -> Result<()> {
//    // Connect to RabbitMQ server
//    let conn = Connection::connect(
//        "amqp://guest:guest@localhost:5672/",
//        ConnectionProperties::default(),
//    )
//    .await?;
//
//    // Create a channel
//    let channel = conn.create_channel().await?;
//
//    // Declare the topic exchange
//    channel
//        .exchange_declare(
//            "my_topic_exchange", // exchange name
//            "topic",             // exchange type
//            ExchangeDeclareOptions::default(),
//            FieldTable::default(),
//        )
//        .await?;
//
//    // Declare a queue to consume messages from
//    let queue = channel
//        .queue_declare(
//            "",                        // auto-generate a unique queue name
//            QueueDeclareOptions::default(),
//            FieldTable::default(),
//        )
//        .await?
//        .name;
//
//    // Bind the queue to the topic exchange
//    channel
//        .queue_bind(
//            &queue,                // queue name
//            "my_topic_exchange",   // exchange name
//            "my.topic.#",          // routing pattern (use # to receive all messages)
//            QueueBindOptions::default(),
//            FieldTable::default(),
//        )
//        .await?;
//
//    // Start consuming messages from the queue
//    let mut consumer = channel
//        .basic_consume(
//            &queue,                 // queue name
//            "",                     // consumer tag (use empty string to auto-generate tag)
//            BasicConsumeOptions::default(),
//            FieldTable::default(),
//        )
//        .await?;
//
//    println!("Waiting for messages...");
//
//    while let Some(delivery) = consumer.next().await {
//        match delivery {
//            Ok((channel, delivery)) => {
//                let payload = String::from_utf8_lossy(&delivery.data).to_string();
//                println!("Received message: {}", payload);
//
//                channel
//                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
//                    .await?;
//            }
//            Err(e) => {
//                eprintln!("Error receiving message: {:?}", e);
//                break;
//            }
//        }
//    }
//
//    Ok(())
//}
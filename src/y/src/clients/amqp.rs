use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
    string::FromUtf8Error,
};

use brotli::{CompressorWriter, Decompressor};
use deadpool_lapin::{Manager, Object, Pool, PoolError};
use futures::StreamExt;
use lapin::types::FieldTable;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        BasicQosOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::{ShortString, ShortUInt},
    BasicProperties, Channel, ConnectionProperties, Consumer, ExchangeKind, Queue,
};
use log::{debug, error, info, trace};
use serde::Deserialize;
use surrealdb::sql::Uuid;
use thiserror::Error as ThisError;
use tokio_amqp::*;

use crate::utils::serialization::{self, SerializationKind};

#[derive(ThisError, Debug)]
pub enum AmqpError {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
    #[error("compress error: {0}")]
    CompressError(#[from] std::io::Error),
    #[error("decompress error: {0}")]
    DecompressError(#[from] FromUtf8Error),
}

//trace!("-> compressed {:?}, uncompressed {:?}", compressed_data.len(), payload.len());

#[derive(Debug, Clone)]
pub struct Amqp {
    pub pool: Pool,
}

#[derive(Debug, Clone)]
pub struct AmqpRpcClient {
    pub channel: Channel,
    pub callback_queue: Queue,
    pub consumer: Consumer,
}

#[derive(Debug, Clone)]
pub struct AmqpSettings<'a> {
    pub channel: ChannelSettings,
    pub exchange: ExchangeSettings<'a>,
    pub queue: QueueSettings<'a>,
    pub queue_bind: QueueBindSettings<'a>,
    pub consumer: ConsumerSettings<'a>,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelSettings {
    pub prefetch_count: ShortUInt,
    pub options: BasicQosOptions,
}

#[derive(Debug, Clone)]
pub struct ExchangeSettings<'a> {
    pub name: &'a str,
    pub kind: ExchangeKind,
    pub options: ExchangeDeclareOptions,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone, Default)]
pub struct QueueSettings<'a> {
    pub name: &'a str,
    pub options: QueueDeclareOptions,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone)]
pub struct QueueBindSettings<'a> {
    pub routing_key: &'a str,
    pub options: QueueBindOptions,
    pub arguments: FieldTable,
}

#[derive(Debug, Clone, Default)]
pub struct ConsumerSettings<'a> {
    pub consumer_tag: &'a str,
    pub options: BasicConsumeOptions,
    pub arguments: FieldTable,
}

impl Amqp {
    #[allow(deprecated)]
    pub fn new(url: String, pool_max_size: usize) -> Self {
        let manager = Manager::new(url, ConnectionProperties::default().with_tokio());
        let pool: Pool = Pool::builder(manager)
            .max_size(pool_max_size)
            .build()
            .expect("can create pool");
        Amqp { pool }
    }

    pub fn close(&self) {
        self.pool.close();
    }

    pub async fn get_connection(&self) -> Result<Object, AmqpError> {
        let connection = self.pool.get().await?;
        Ok(connection)
    }

    pub async fn get_channel(&self) -> Result<Channel, AmqpError> {
        // Get connection
        let rmq_con = match self.get_connection().await.map_err(|e| {
            eprintln!("can't connect to rmq, {}", e);
            e
        }) {
            Ok(x) => x,
            Err(error) => return Err(error),
        };

        match rmq_con.create_channel().await.map_err(|e| {
            eprintln!("can't create channel, {}", e);
            e
        }) {
            Ok(x) => Ok(x),
            Err(error) => Err(AmqpError::RMQError(error)),
        }
    }

    pub fn decompress_message(msg: Vec<u8>) -> Result<Vec<u8>, AmqpError> {
        let mut uncompressed_message = Vec::new();
        let mut decompressor = Decompressor::new(&msg[..], 4096);
        decompressor.read_to_end(&mut uncompressed_message).unwrap();
        Ok(uncompressed_message)
    }

    pub fn decompress_message_as_str(msg: Vec<u8>) -> Result<String, AmqpError> {
        let mut uncompressed_message = Vec::new();
        let mut decompressor = Decompressor::new(&msg[..], 4096);
        decompressor.read_to_end(&mut uncompressed_message).unwrap();
        let x = String::from_utf8(uncompressed_message)?;
        Ok(x)
    }

    pub fn compress_message(msg: &str) -> Result<Vec<u8>, AmqpError> {
        let mut compressed_data = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
            compressor.write_all(msg.as_bytes())?;
        }
        Ok(compressed_data)
    }

    pub async fn declare_exchange(
        &self,
        exchange: &str,
        kind: ExchangeKind,
        options: ExchangeDeclareOptions,
        arguments: FieldTable,
    ) -> Result<(), AmqpError> {
        let channel = self.get_channel().await?;
        channel
            .exchange_declare(exchange, kind, options, arguments)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn declare_exchange_with_channel(
        &self,
        channel: &Channel,
        exchange: &str,
        kind: ExchangeKind,
        options: ExchangeDeclareOptions,
        arguments: FieldTable,
    ) -> Result<(), AmqpError> {
        channel
            .exchange_declare(exchange, kind, options, arguments)
            .await?;
        Ok(())
    }

    pub async fn declare_queue(
        &self,
        queue: &str,
        options: lapin::options::QueueDeclareOptions,
        arguments: FieldTable,
    ) -> Result<Queue, AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel
            .queue_declare(queue, options, arguments)
            .await
            .unwrap())
    }

    pub async fn declare_queue_with_channel(
        &self,
        channel: &Channel,
        queue: &str,
        options: lapin::options::QueueDeclareOptions,
        arguments: FieldTable,
    ) -> Result<Queue, AmqpError> {
        Ok(channel
            .queue_declare(queue, options, arguments)
            .await
            .unwrap())
    }

    pub async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
        options: QueueBindOptions,
        arguments: FieldTable,
    ) -> Result<(), AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel
            .queue_bind(queue, exchange, routing_key, options, arguments)
            .await
            .unwrap())
    }

    pub async fn bind_queue_with_channel(
        &self,
        channel: &Channel,
        queue: &str,
        exchange: &str,
        routing_key: &str,
        options: QueueBindOptions,
        arguments: FieldTable,
    ) -> Result<(), AmqpError> {
        Ok(channel
            .queue_bind(queue, exchange, routing_key, options, arguments)
            .await
            .unwrap())
    }

    pub async fn create_consumer(
        &self,
        queue: &str,
        consumer_tag: &str,
        options: BasicConsumeOptions,
        arguments: FieldTable,
    ) -> Result<Consumer, AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel
            .basic_consume(queue, consumer_tag, options, arguments)
            .await
            .unwrap())
    }

    pub async fn create_consumer_with_channel(
        &self,
        channel: &Channel,
        queue: &str,
        consumer_tag: &str,
        options: BasicConsumeOptions,
        arguments: FieldTable,
    ) -> Result<Consumer, AmqpError> {
        Ok(channel
            .basic_consume(queue, consumer_tag, options, arguments)
            .await
            .unwrap())
    }

    pub async fn send_message(
        &self,
        payload: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<&str, AmqpError> {
        // Create message and compress using Brotli 10
        let compressed_payload = Amqp::compress_message(payload)?;

        // Get channel
        let channel = self.get_channel().await?;

        // Set encoding type
        let headers = BasicProperties::default().with_content_encoding("br".into());
        match channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &compressed_payload,
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
            Err(error) => return Err(AmqpError::RMQError(error)),
        };
        Ok("OK")
    }

    pub async fn send_message_with_channel<'a>(
        channel: &'a Channel,
        payload: &'a str,
        exchange: &'a str,
        routing_key: &'a str,
    ) -> Result<&'a str, AmqpError> {
        // Create message and compress using Brotli 10
        let compressed_payload = Amqp::compress_message(payload)?;

        // Set encoding type
        let headers = BasicProperties::default().with_content_encoding("br".into());

        match channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &compressed_payload,
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
            Err(error) => return Err(AmqpError::RMQError(error)),
        };
        Ok("OK")
    }

    pub async fn send_message_with_reply<'a>(
        channel: &'a Channel,
        payload: &'a str,
        exchange: &'a str,
        routing_key: &'a str,
        reply_queue_name: &'a str,
        _reply_correlation_id: String,
    ) -> Result<String, AmqpError> {
        // Create message and compress using Brotli 10
        let compressed_payload = Amqp::compress_message(payload)?;

        // Set encoding type
        let headers = BasicProperties::default()
            .with_content_encoding("br".into())
            //.with_correlation_id(ShortString::from(reply_correlation_id))
            .with_reply_to(reply_queue_name.into());

        match channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &compressed_payload,
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
            Err(error) => return Err(AmqpError::RMQError(error)),
        };
        Ok(String::from("OK"))
    }

    pub async fn consume_topic_queue<T, E: Error>(
        &self,
        index: usize,
        settings: AmqpSettings<'_>,
        serialization: SerializationKind,
        mut on_msg_callback: impl FnMut(T, Option<ShortString>) -> Result<(), E>,
    ) where
        T: for<'a> Deserialize<'a> + std::fmt::Debug,
    {
        // Channel
        let channel = &self.get_channel().await.unwrap();
        channel
            .basic_qos(settings.channel.prefetch_count, settings.channel.options)
            .await
            .unwrap();
        match self
            .declare_exchange_with_channel(
                channel,
                settings.exchange.name,
                settings.exchange.kind,
                settings.exchange.options,
                settings.exchange.arguments,
            )
            .await
        {
            Ok(()) => info!(
                "{}: topic exchange <{}> declared",
                index, settings.exchange.name
            ),
            Err(error) => error!(
                "{}: can't create topic exchange <{}> {}",
                index, settings.exchange.name, error
            ),
        };

        // Queue
        let queue = match self
            .declare_queue_with_channel(
                channel,
                settings.queue.name,
                settings.queue.options,
                settings.queue.arguments,
            )
            .await
        {
            Ok(queue) => {
                info!("{}: queue <{}> declared", index, queue.name());
                queue
            }
            Err(error) => {
                error!(
                    "{}: can't create queue <{}> {}",
                    index, settings.queue.name, error
                );
                panic!("{}", error)
            }
        };

        // Binding
        match self
            .bind_queue_with_channel(
                channel,
                queue.name().as_str(),
                settings.exchange.name,
                settings.queue_bind.routing_key,
                settings.queue_bind.options,
                settings.queue_bind.arguments,
            )
            .await
        {
            Ok(()) => info!(
                "{}: topic exchange <{}> and queue <{}> binded",
                index,
                settings.exchange.name,
                queue.name()
            ),
            Err(error) => {
                error!(
                    "{}: can't create binding <{}> <{}> {}",
                    index, settings.exchange.name, settings.queue.name, error
                );
                panic!("{}", error)
            }
        };

        // Consumer
        let mut consumer = match self
            .create_consumer_with_channel(
                channel,
                settings.queue.name,
                settings.consumer.consumer_tag,
                settings.consumer.options,
                settings.consumer.arguments,
            )
            .await
        {
            Ok(consumer) => {
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

        // Liscen to topic queue exchange
        info!("{}: consumer <{}> is liscening", index, consumer.tag());
        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                let reply_to = if let Some(x) = delivery.properties.reply_to().as_ref() {
                    x.to_owned()
                } else {
                    ShortString::from("")
                };

                let payload: Vec<u8> = delivery.data.clone();
                let uncompressed_message = match delivery
                    .properties
                    .content_encoding()
                    .clone()
                    .unwrap_or_else(|| ShortString::from(""))
                    .as_str()
                {
                    "br" => Amqp::decompress_message(payload),
                    _ => Ok(payload),
                }
                .unwrap();

                let deserialized_payload: T =
                    serialization.from_vec(&uncompressed_message).unwrap();
                debug!("{}: {:?}", index, deserialized_payload);

                match on_msg_callback(deserialized_payload, Some(reply_to)) {
                    Ok(()) => {
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
                    }
                    Err(error) => {
                        error!(
                            "{}: can't act on message <{}> {}",
                            index, delivery.delivery_tag, error
                        );
                        match channel
                            .basic_nack(
                                delivery.delivery_tag,
                                BasicNackOptions {
                                    multiple: false,
                                    requeue: true,
                                },
                            )
                            .await
                        {
                            Ok(()) => trace!(
                                "{}: negative acknowledged message <{}>",
                                index,
                                delivery.delivery_tag
                            ),
                            Err(error) => error!(
                                "{}: can't negative acknowledge message <{}> {}",
                                index, delivery.delivery_tag, error
                            ),
                        };
                    }
                }
            };
        }
        debug!("{}: Shutting down...", index);
    }

    pub async fn create_rpc_client_queue(
        &self,
        cb_queue: QueueSettings<'_>,
        consumer: ConsumerSettings<'_>,
    ) -> AmqpRpcClient {
        let ch = &self.get_channel().await.unwrap();

        //ch.basic_qos(channel.prefetch_count, channel.options)
        //    .await
        //    .unwrap();

        let callback_queue = match self
            .declare_queue_with_channel(ch, cb_queue.name, cb_queue.options, cb_queue.arguments)
            .await
        {
            Ok(queue) => {
                info!("queue <{}> declared", queue.name());
                queue
            }
            Err(error) => {
                error!("can't create queue <{}> {}", cb_queue.name, error);
                panic!("{}", error)
            }
        };

        let consumer = match self
            .create_consumer_with_channel(
                ch,
                cb_queue.name,
                consumer.consumer_tag,
                consumer.options,
                consumer.arguments,
            )
            .await
        {
            Ok(consumer) => {
                info!(
                    "consumer <{}> to queue <{}> binded",
                    consumer.tag(),
                    cb_queue.name
                );
                consumer
            }
            Err(error) => {
                error!(
                    "can't bind consumer and queue <{}> {}",
                    cb_queue.name, error
                );
                panic!("{}", error)
            }
        };

        AmqpRpcClient {
            channel: ch.clone(),
            callback_queue,
            consumer,
        }
    }
}

impl AmqpRpcClient {
    pub async fn call(
        &self,
        payload: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<String, AmqpError> {
        let correlation_id = Uuid::new_v4().to_string();
        let x = Amqp::send_message_with_reply(
            &self.channel,
            payload,
            exchange,
            routing_key,
            self.callback_queue.name().as_str(),
            correlation_id,
        )
        .await?;

        Ok(x)
    }

    pub async fn listen<T, E: Error>(
        &mut self,
        serialization: SerializationKind,
        mut on_msg_callback: impl FnMut(T) -> Result<(), E>,
    ) where
        T: for<'a> Deserialize<'a> + std::fmt::Debug,
    {
        while let Some(delivery) = self.consumer.next().await {
            if let Ok(delivery) = delivery {
                let payload: Vec<u8> = delivery.data.clone();
                let uncompressed_message = match delivery
                    .properties
                    .content_encoding()
                    .clone()
                    .unwrap_or_else(|| ShortString::from(""))
                    .as_str()
                {
                    "br" => Amqp::decompress_message(payload),
                    _ => Ok(payload),
                }
                .unwrap();

                let deserialized_payload: T =
                    serialization.from_vec(&uncompressed_message).unwrap();

                match on_msg_callback(deserialized_payload) {
                    Ok(()) => {
                        match self
                            .channel
                            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                            .await
                        {
                            Ok(()) => {
                                trace!("acknowledged message <{}>", delivery.delivery_tag)
                            }
                            Err(error) => error!(
                                "can't acknowledge message <{}> {}",
                                delivery.delivery_tag, error
                            ),
                        };
                    }
                    Err(error) => {
                        error!("can't act on message <{}> {}", delivery.delivery_tag, error);
                        match self
                            .channel
                            .basic_nack(
                                delivery.delivery_tag,
                                BasicNackOptions {
                                    multiple: false,
                                    requeue: true,
                                },
                            )
                            .await
                        {
                            Ok(()) => {
                                trace!("negative acknowledged message <{}>", delivery.delivery_tag)
                            }
                            Err(error) => error!(
                                "can't negative acknowledge message <{}> {}",
                                delivery.delivery_tag, error
                            ),
                        };
                    }
                }
            }
        }
    }
}

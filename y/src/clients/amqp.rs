
use std::{io::{Write, Read}, string::FromUtf8Error};

use brotli::{CompressorWriter, Decompressor};
use deadpool_lapin::{Pool, Manager, PoolError, Object};
use lapin::{BasicProperties, ConnectionProperties, options::{ExchangeDeclareOptions, BasicPublishOptions, QueueBindOptions, BasicConsumeOptions}, ExchangeKind, Queue, Channel, Consumer};
use tokio_amqp::*;
use lapin::types::FieldTable;
use thiserror::Error as ThisError;

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

impl Amqp {
    #[allow(deprecated)]
    pub fn new(url: String, pool_max_size: usize) -> Self {
        let manager = Manager::new(
            url,
            ConnectionProperties::default().with_tokio(),
        );
        let pool: Pool = Pool::builder(manager)
            .max_size(pool_max_size)
            .build()
            .expect("can create pool");
        Amqp {
            pool: pool,
        }
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
            Err(error) => return Err(AmqpError::RMQError(error)),
        }
    }

    pub fn decompress_message(&self, msg: Vec<u8>) -> Result<String, AmqpError> {
        let mut uncompressed_message = Vec::new();
        let mut decompressor = Decompressor::new(&msg[..], 4096);
        decompressor.read_to_end(&mut uncompressed_message).unwrap();
        let x = String::from_utf8(uncompressed_message)?;
        Ok(x)
    }

    pub fn compress_message(&self, msg: &str) -> Result<Vec<u8>, AmqpError> {
        let mut compressed_data = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
            compressor.write_all(msg.as_bytes())?;
        }
        Ok(compressed_data)
    }

    pub async fn declare_exchange(&self, exchange: &str, kind: ExchangeKind, options: ExchangeDeclareOptions, arguments: FieldTable) -> Result<(), AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel.exchange_declare(
            exchange,
            kind,
            options,
            arguments).await.unwrap())
    }

    pub async fn declare_queue(&self, queue: &str, options: lapin::options::QueueDeclareOptions, arguments: FieldTable) -> Result<Queue, AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel.queue_declare(
            queue,
            options,
            arguments).await.unwrap())
    }

    pub async fn bind_queue(&self, queue: &str, exchange: &str, routing_key: &str, options: QueueBindOptions, arguments: FieldTable) -> Result<(), AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel.queue_bind(
            queue,
            exchange,
            routing_key,
            options,
            arguments).await.unwrap())
    }

    pub async fn create_consumer(&self, queue: &str, consumer_tag: &str, options: BasicConsumeOptions, arguments: FieldTable) -> Result<Consumer, AmqpError> {
        let channel = self.get_channel().await?;
        Ok(channel.basic_consume(
            queue,
            consumer_tag,
            options,
            arguments).await.unwrap())
    }

    pub async fn send_message(&self, payload: &str, exchange: &str, routing_key: &str) -> Result<&str, AmqpError> {
        // Create message and compress using Brotli 10
        let compressed_payload = self.compress_message(payload)?;

        // Get connection
        let rmq_con = match self.get_connection().await.map_err(|e| {
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
            Err(error) => return Err(AmqpError::RMQError(error)),
        };

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
}

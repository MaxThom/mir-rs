
use std::{error::Error, io::Write};

use brotli::CompressorWriter;
use deadpool_lapin::{Pool, Manager};
use lapin::{BasicProperties, ConnectionProperties, options::{ExchangeDeclareOptions, BasicPublishOptions}};
use tokio_amqp::*;
use lapin::types::FieldTable;

#[derive(Debug, Clone)]
pub struct Amqp {
    pub pool: Pool,
}

impl Amqp {
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

    //pub async fn send_message(&self, payload: &str) -> Result<&str, ()> {
    //    // Create message and compress using Brotli 10
    //    let mut compressed_data = Vec::new();
    //    {
    //        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
    //        compressor.write_all(payload.as_bytes()).unwrap();
    //    }
    //    //trace!("-> compressed {:?}, uncompressed {:?}", compressed_data.len(), payload.len());
    //
    //    // Get connection
    //     let connection = self.pool.get().await.unwrap();
    //    let rmq_con = match connection.map_err(|e| {
    //        eprintln!("can't connect to rmq, {}", e);
    //        e
    //    }) {
    //        Ok(x) => x,
    //        Err(error) => return Err(error),
    //    };
    //
    //    // Create channel
    //    let channel = match rmq_con.create_channel().await.map_err(|e| {
    //        eprintln!("can't create channel, {}", e);
    //        e
    //    }) {
    //        Ok(x) => x,
    //        Err(error) => error,//return Err(Error::RMQError(error)),
    //    };
    //
    //    channel.exchange_declare(
    //        "iot",
    //        lapin::ExchangeKind::Topic,
    //        ExchangeDeclareOptions::default(),
    //        FieldTable::default()).await?;
    //
    //    // Set encoding type
    //    let headers = BasicProperties::default().with_content_encoding("br".into());
    //    match channel
    //        .basic_publish(
    //            "iot",
    //            "swarm.telemetry.v1",
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
    //        Err(error) => error, //return Err(Error::RMQError(error)),
    //    };
    //    Ok("OK")
    //}
}

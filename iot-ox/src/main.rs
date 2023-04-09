use brotli::{Decompressor, CompressorWriter};
use deadpool_lapin::{Manager, Pool, PoolError};
use futures::{join, StreamExt};
use lapin::types::ShortString;
use lapin::{options::*, types::FieldTable, BasicProperties, ConnectionProperties};
use std::io::Write;
use std::{convert::Infallible, io::Read};
use std::result::Result as StdResult;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio_amqp::*;
use warp::{Filter, Rejection, Reply};

type WebResult<T> = StdResult<T, Rejection>;
type RMQResult<T> = StdResult<T, PoolError>;
type Result<T> = StdResult<T, Error>;

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

#[derive(ThisError, Debug)]
enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError),
}

impl warp::reject::Reject for Error {}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://admin:M3t5h7o9@165.22.226.13:32000/%2f".into());
    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
    let pool: Pool = deadpool::managed::Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can create pool");


    let health_route = warp::path!("health").and_then(health_handler);
    let add_msg_route = warp::path!("msg")
        .and(warp::post())
        .and(with_rmq(pool.clone()))
        .and_then(add_msg_handler);

    let routes = health_route
        .or(add_msg_route);

    println!("Started server at localhost:8000");
    let _ = join!(
        warp::serve(routes).run(([0, 0, 0, 0], 8000)),
        rmq_listen(pool.clone())
    );
    Ok(())
}

/// /// ///
/// Write to the queue
///

async fn add_msg_handler(pool: Pool) -> WebResult<impl Reply> {
    // Create message and compress using Brotli 10
    let payload = "Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!Hello world!".as_bytes();
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
        compressor.write_all(payload).unwrap();
    }

    // Get connection
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("can't connect to rmq, {}", e);
        warp::reject::custom(Error::RMQPoolError(e))
    })?;

    // Create channel
    let channel = rmq_con.create_channel().await.map_err(|e| {
        eprintln!("can't create channel, {}", e);
        warp::reject::custom(Error::RMQError(e))
    })?;

    // Set encoding type
    let headers = BasicProperties::default().with_content_encoding("br".into());
    channel
        .basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            &compressed_data,
            headers,
        )
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            warp::reject::custom(Error::RMQError(e))
        })?
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            warp::reject::custom(Error::RMQError(e))
        })?;
    Ok("OK")
}

fn with_rmq(pool: Pool) -> impl Filter<Extract = (Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

/// /// ///
/// Listen to the queue
///

async fn rmq_listen(pool: Pool) -> Result<()> {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("connecting rmq consumer...");
        match init_rmq_listen(pool.clone()).await {
            Ok(_) => println!("rmq listen returned"),
            Err(e) => eprintln!("rmq listen had an error: {}", e),
        };
    }
}

async fn init_rmq_listen(pool: Pool) -> Result<()> {
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("could not get rmq con: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    let queue = channel
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue);

    let mut consumer = channel
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("rmq consumer connected, waiting for messages");
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            println!("--- new message ---");
            //println!("received msg: {:?}", delivery);
            let payload = delivery.data.clone();
            let mut uncompressed_message = Vec::new();
            match delivery.properties.content_encoding().clone().unwrap_or_else(|| ShortString::from("")).as_str() {
                "br" => {
                    let mut decompressor = Decompressor::new(&payload[..], 4096);
                    decompressor.read_to_end(&mut uncompressed_message).unwrap();
                }
                _ => {
                    uncompressed_message = payload;
                }
            }

            println!("-> compressed {:?}, uncompressed {:?}", delivery.data.len(), uncompressed_message.len());
            println!("{}", String::from_utf8(uncompressed_message).unwrap());
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?
        }
    }
    Ok(())
}

async fn get_rmq_con(pool: Pool) -> RMQResult<Connection> {
    let connection = pool.get().await?;
    Ok(connection)
}

/// /// ///
/// Health handler
///

async fn health_handler() -> WebResult<impl Reply> {
    Ok("OK")
}
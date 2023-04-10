use brotli::{Decompressor, CompressorWriter};
use deadpool_lapin::{Manager, Pool, PoolError, Object};
use futures::{join, StreamExt};
use lapin::types::ShortString;
use lapin::{options::*, types::FieldTable, BasicProperties, ConnectionProperties};
use rocket::State;
use std::error::Error;
use std::io::Write;
use std::{convert::Infallible, io::Read};
use std::result::Result as StdResult;
use std::time::Duration;
use thiserror::Error as ThisError;
use tokio_amqp::*;

#[macro_use] extern crate rocket;

#[get("/msg/<msg>")]
async fn msg(pool: &State<Pool>, msg: &str) -> String {
    write_to_qeue(pool.inner().clone(), msg).await;
    format!("Hello, {}!", msg)
}

#[get("/msg?<msg>")]
async fn msg_qp(pool: &State<Pool>, msg: &str) -> String {
    write_to_qeue(pool.inner().clone(), msg).await;
    format!("Hello, {}!", msg)
}

#[get("/alive>")]
async fn alive() -> String {
    format!("{}", true)
}

#[get("/ready>")]
async fn ready() -> String {
    format!("{}", true)
}

#[launch]
fn rocket() -> _ {
    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "".into());
    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
    let pool: Pool = deadpool::managed::Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can create pool");

    rocket::build()
        .manage(pool)
        .mount("/", routes![alive])
        .mount("/", routes![ready])
        .mount("/", routes![msg])
        .mount("/", routes![msg_qp])
}

//#[tokio::main]
//#[rocket::main]
//async fn main() -> Result<(), rocket::Error> {
//    let addr = std::env::var("AMQP_ADDR")
//        .unwrap_or_else(|_| "".into());
//    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
//    let pool: Pool = deadpool::managed::Pool::builder(manager)
//        .max_size(10)
//        .build()
//        .expect("can create pool");
//
//    println!("Started server at localhost:8000");
//
//    let srv = rocket::build()
//        .manage(pool)
//        .mount("/", routes![alive])
//        .mount("/", routes![ready])
//        .mount("/", routes![msg])
//        .mount("/", routes![msg_qp]);
//
//    let _ = join!(
//        srv.launch(),
//        rmq_listen(pool.clone())
//    );
//
//    Ok(())
//}

/// /// ///
/// Write to the queue
///

async fn write_to_qeue(pool: Pool, payload: &str) -> Result<&str, Box<dyn Error>> {
    // Create message and compress using Brotli 10
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 10, 22);
        compressor.write_all(payload.as_bytes()).unwrap();
    }

    // Get connection
    let rmq_con = match get_rmq_con(pool).await.map_err(|e| {
        eprintln!("can't connect to rmq, {}", e);
        Err(e)
    }) {
        Ok(x) => x,
        Err(error) => return Err(error.unwrap_err())
    };

    // Create channel
    let channel = match rmq_con.create_channel().await.map_err(|e| {
        eprintln!("can't create channel, {}", e);
        Err(e)
    }) {
        Ok(x) => x,
        Err(error) => return Err(Box::new(error.unwrap_err()))
    };

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
            Err(e)
        })?
        .await
        .map_err(|e| {
            eprintln!("can't publish: {}", e);
            Err(e)
        })?;
    Ok("OK")
}

/// /// ///
/// Listen to the queue
///

async fn rmq_listen(pool: Pool) -> Result<String, Box<dyn Error>> {
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

async fn init_rmq_listen(pool: Pool) -> Result<(), Box<dyn Error>> {
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

async fn get_rmq_con(pool: Pool) -> Result<Object, Box<dyn Error>> {
    let connection = pool.get().await?;
    Ok(connection)
}
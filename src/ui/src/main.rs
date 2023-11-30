use std::path::PathBuf;

use askama::Template;
use axum::{routing::get, Router};
use libs::utils::setup_cli;
use log::info;

use libs::utils::config::{setup_config, FileFormat};
use libs::utils::logger::setup_logger;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
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
    pub web_srv_port: usize,
}

const APP_NAME: &str = "cockpit";

#[tokio::main]
async fn main() {
    let matches = setup_cli();

    let settings: Settings = setup_config(
        APP_NAME,
        FileFormat::YAML,
        matches.get_one::<PathBuf>(libs::utils::cli::CONFIG_KEY),
    )
    .unwrap();
    setup_logger(settings.log_level.clone()).unwrap();
    info!("{:?}", settings);

    let app = Router::new().route("/", get(hello));

    let addr = format!("0.0.0.0:{}", settings.web_srv_port);

    let listener = tokio::net::TcpListener::bind(addr.as_str()).await.unwrap();
    info!("server initialized at {addr}");
    axum::serve(listener, app).await.unwrap();
}

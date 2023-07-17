// This expose PostMQ directly in clients
pub use cli::setup_cli;
pub use config::setup_config;
pub use logger::setup_logger;
pub use network::parse_host_port;

// This expose PostMQ after importing rabbitmq::PostMQ; in the clients
pub mod cli;
pub mod config;
pub mod logger;
pub mod network;
pub mod serialization;

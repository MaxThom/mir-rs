// This expose PostMQ directly in clients
pub use logger::setup_logger;
pub use config::setup_config;

// This expose PostMQ after importing rabbitmq::PostMQ; in the clients
pub mod logger;
pub mod config;
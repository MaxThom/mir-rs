// This expose PostMQ directly in clients
//pub use rabbitmq::PostMQ;

// This expose PostMQ after importing rabbitmq::PostMQ; in the clients
pub mod amqp;
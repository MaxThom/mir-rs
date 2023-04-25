// This expose PostMQ directly in clients
pub use device::DevicePayload;

// This expose PostMQ after importing rabbitmq::PostMQ; in the clients
pub mod device;
// This expose PostMQ directly in clients
pub use device::DevicePayload;
pub use device_twin::DeviceTwin;
pub use device_twin::NewDevice;

// This expose PostMQ after importing rabbitmq::PostMQ; in the clients
pub mod device;
pub mod device_twin;
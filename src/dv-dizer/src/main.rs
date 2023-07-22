use dizer::DizerBuilder;
use log::info;
use thiserror::Error as ThisError;
use tokio_util::sync::CancellationToken;

#[derive(ThisError, Debug)]
enum Error {}

//const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
//const RMQ_DEVICE_EXCHANGE_NAME: &str = "iot-devices";
//const RMQ_TWIN_META_QUEUE_NAME: &str = "iot-q-twin-meta";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-twin-reported";
//const RMQ_PREFETCH_COUNT: u16 = 10;

//
// Builder pattern
// - register config file
//   - mir config file
//   - users config file
// - pass receive twin desired properties updates function handler
// - pass receive commands function handler
// Sending tlmt
// - implement send twin reported properties
// - implement send twin meta properties
// - implement send telemetry
// - implement logging using log interface
// - implement send metrics
//
#[tokio::main]
async fn main() -> Result<(), String> {
    // Init token, logger & config
    let token = CancellationToken::new();

    let dizer_builder = DizerBuilder::default()
        .with_cli()
        .with_config_file("")
        .with_device_id("012xwf===")
        .with_mir_server("")
        .with_thread_count(1)
        .with_logger("info")
        .build();
    if let Err(x) = dizer_builder {
        return Err(format!("error initializing Dizer: {}", x));
    }
    let mut dizer = dizer_builder.unwrap();

    // TODO: Add desired properties handler
    // TODO: Add send telemetry
    // TODO: Add send reported properties
    // TODO: Send heartbeat

    if let Err(x) = dizer.join_fleet().await {
        return Err(format!("error joining fleet: {}", x));
    }

    info!("Press ctrl+c to shutdown.");
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Dizer shutting down...");
            token.cancel();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
    info!("Shutdown complete.");

    Ok(())
}

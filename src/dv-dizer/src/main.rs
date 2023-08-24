use dizer::{builder::MirShipyard, dizer::Dizer};
use log::{debug, error, info, trace};
use thiserror::Error as ThisError;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use x::telemetry::Telemetry;
use y::utils::telemetry::{PyramidTelemetryGenerator, TelemetryGenerator};

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
// TODO: Add desired properties handler
// TODO: Add send reported properties
// TODO: Send heartbeat
//
#[tokio::main]
async fn main() -> Result<(), String> {
    // Init token & dizer
    let token = CancellationToken::new();
    let dizer_builder = MirShipyard::new()
        .with_cli()
        .with_config_file("")
        .with_device_id("012xwf===")
        .with_mir_server("")
        .with_thread_count(3)
        .with_logger("info")
        .build();
    if let Err(x) = dizer_builder {
        return Err(format!("error initializing Dizer: {}", x));
    }
    let mut dizer = dizer_builder.unwrap();

    // Connect to mir
    if let Err(x) = dizer.join_fleet().await {
        return Err(format!("error joining fleet: {}", x));
    }

    // Do stuff
    let cloned_token = token.clone();
    let dizer_clone = dizer.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = cloned_token.cancelled() => {
                debug!("The token was shutdown")
            }
            _ = send_random_telemetry(dizer_clone) => {
                debug!("device shuting down...");
            }
        }
    });

    // Wait for shutdown signal
    info!("Press ctrl+c to shutdown.");
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            let _ = dizer.leave_fleet().await;
            token.cancel();
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
    info!("Shutdown complete.");

    Ok(())
}

async fn send_random_telemetry(dizer: Dizer) {
    let mut gen = PyramidTelemetryGenerator::new(1.0, 0.0, 100.0).unwrap();
    loop {
        let mut tlm = Telemetry::default();
        tlm.floats.insert(0, gen.next_datapoint());

        match dizer.send_telemetry(tlm).await {
            Ok(_) => (), //  trace!("{}", msg),
            Err(error) => error!("{}", error),
        };

        sleep(Duration::from_secs(5)).await;
    }
}

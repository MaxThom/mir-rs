use clap::Args;
use y::{clients::amqp::Amqp, utils::cli::get_stdin_from_pipe};

//const RMQ_TWIN_EXCHANGE_NAME: &str = "iot-twin";
//const RMQ_TWIN_HEARTHBEAT_QUEUE_NAME: &str = "iot-q-hearthbeat";
//const RMQ_TWIN_HEATHBEAT_ROUTING_KEY: &str = "#.hearthbeat.v1";
//const RMQ_TWIN_REPORTED_QUEUE_NAME: &str = "iot-q-reported";
//const RMQ_TWIN_REPORTED_ROUTING_KEY: &str = "#.reported.v1";
//const RMQ_TWIN_DESIRED_QUEUE_NAME: &str = "iot-q-desired";
//const RMQ_TWIN_DESIRED_ROUTING_KEY: &str = "#.desired.v1";
//const RMQ_PREFETCH_COUNT: u16 = 10;

#[derive(Args)]
pub struct ListenCmd {
    /// list of devices to print. If empty, print all devices. If . read from stdin.
    device_ids: Vec<String>,

    #[arg(long)]
    hearthbeat: bool,
    #[arg(long)]
    device: bool,
    #[arg(long)]
    desired: bool,
    #[arg(long)]
    reported: bool,
}

pub async fn run_listen_cmd(cmd: &ListenCmd, target: String) -> Result<(), String> {
    let mut ids: Vec<String> = cmd.device_ids.clone();
    if cmd.device_ids.len() == 1 && cmd.device_ids[0] == "." {
        ids = serde_json::from_str(get_stdin_from_pipe().as_str()).unwrap();
    }

    // TODO: to do with dizer sdk once completed
    let _amqp = Amqp::new(target, 4);

    if cmd.hearthbeat {
        print!("Listen to hearthbeat for {:?}", ids);
    }
    if cmd.desired {
        print!("Listen to desired for {:?}", ids);
    }
    if cmd.reported {
        print!("Listen to reported for {:?}", ids);
    }
    if cmd.device {
        print!("Listen to devices queue for {:?}", ids);
    }

    Ok(())
}

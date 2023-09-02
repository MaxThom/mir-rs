use clap::{Args, Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set Mir server adress
    #[arg(
        short,
        long,
        value_name = "ADRESS",
        default_value_t = String::from("localhost:5047")
    )]
    target: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: MirCmds,
}

#[derive(Subcommand)]
enum MirCmds {
    /// list devices
    List(ListCmd),
}

#[derive(Args)]
struct ListCmd {
    #[command(subcommand)]
    command: ListCmds,
}

#[derive(Subcommand)]
enum ListCmds {
    /// list devices
    Devices(DeviceCmd),
}

#[derive(Args)]
struct DeviceCmd {
    device_ids: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        MirCmds::List(list_cmd) => match &list_cmd.command {
            ListCmds::Devices(device_cmd) => {
                list_all_devices(cli.target.clone(), device_cmd.device_ids.clone()).await;
            }
        },
    }

    Ok(())
}

async fn list_all_devices(url: String, device_ids: Vec<String>) {
    if device_ids.len() == 0 {
        println!("Listing all devices");
        let devices = match get_devices_data(url, None).await {
            Ok(d) => d,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        println!("{}", serde_json::to_string_pretty(&devices).unwrap());
        return;
    }

    println!("Listing devices {:?}", device_ids);
    for device_id in device_ids {
        let devices = match get_devices_data(url.clone(), Some(device_id.clone())).await {
            Ok(d) => d,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        println!("{}", serde_json::to_string_pretty(&devices).unwrap());
    }
}

async fn get_devices_data(url: String, device_id: Option<String>) -> Result<Value, reqwest::Error> {
    let url = format!("http://{}/devicetwins", url);
    let res = reqwest::get(url).await?.json::<Value>().await?;

    // TODO: add device id call ?device_id=maxi2

    // TODO: when docer issue is fix, we can updated to 1.0.0 to fix. https://github.com/surrealdb/surrealdb/issues/2574
    //let res = reqwest::get(url).await?.json::<Vec<DeviceTwin>>().await?;
    //let deserialized: Vec<DeviceTwin> = serde_json::from_str(&res).unwrap();
    Ok(res)
}

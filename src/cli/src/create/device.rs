use clap::Args;
use reqwest::Client;
use serde_json::{json, Value};
use x::device_twin::{NewDeviceReq, Status};
use y::utils::cli::get_stdin_from_pipe;

#[derive(Args)]
pub struct DeviceCmd {
    /// list of devices to print. If empty, print all devices. If . read from stdin.
    device_ids: Vec<String>,

    #[arg(short, long, value_enum)]
    status: Status,
    #[arg(short, long)]
    model_id: String,

    #[arg(long)]
    meta: bool,
    #[arg(long)]
    tag: bool,
    #[arg(long)]
    desired: bool,
    #[arg(long)]
    reported: bool,
}

pub async fn run_device_cmd(device_cmd: &DeviceCmd, target: String) -> Result<(), String> {
    let mut ids: Vec<String> = device_cmd.device_ids.clone();
    if device_cmd.device_ids.len() == 1 && device_cmd.device_ids[0] == "." {
        // TODO: do it
        ids = get_stdin_from_pipe()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    }

    let mut devices = json!([]);
    for device_id in ids {
        let mut device = create_device_request(
            target.clone(),
            NewDeviceReq {
                device_id,
                model_id: device_cmd.model_id.clone(),
                status: device_cmd.status.clone(),
            },
        )
        .await
        .map_err(|e| format!("Error: {:?}", e))?;

        let twin = if let Some(x) = device.as_object_mut() {
            x
        } else {
            return Err(format!("Error: {:?}", device));
        };

        // If all false, we show everything
        if device_cmd.meta || device_cmd.tag || device_cmd.desired || device_cmd.reported {
            if !device_cmd.meta {
                twin.remove("meta_properties");
            }
            if !device_cmd.tag {
                twin.remove("tag_properties");
            }
            if !device_cmd.desired {
                twin.remove("desired_properties");
            }
            if !device_cmd.reported {
                twin.remove("reported_properties");
            }
        }

        let v: Value = serde_json::value::to_value(twin).unwrap();
        devices.as_array_mut().unwrap().push(v);
    }

    print!("{}", serde_json::to_string_pretty(&devices).unwrap());

    Ok(())
}

async fn create_device_request(
    url: String,
    device_req: NewDeviceReq,
) -> Result<Value, reqwest::Error> {
    let url = format!("http://{}/devicetwins", url);

    let client = Client::new();
    let resp = client.post(url).json(&device_req).send().await?;
    let x = resp.json::<Value>().await?;
    // TODO: add device id call ?device_id=maxi2

    // TODO: when docer issue is fix, we can updated to 1.0.0 to fix. https://github.com/surrealdb/surrealdb/issues/2574
    //let res = reqwest::get(url).await?.json::<Vec<DeviceTwin>>().await?;
    //let deserialized: Vec<DeviceTwin> = serde_json::from_str(&res).unwrap();
    Ok(x)
}

use clap::Args;
use reqwest::Client;
use serde_json::{json, Value};
use x::device_twin::{Properties, TargetProperties};
use y::utils::cli::get_stdin_from_pipe;

#[derive(Args)]
pub struct DeviceCmd {
    /// list of devices to print. If empty, print all devices. If . read from stdin.
    device_ids: Vec<String>,

    #[arg(short, long, value_enum)]
    target: TargetProperties,

    #[arg(short, long)]
    properties: Option<String>,

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
    let ids: Vec<String> = device_cmd.device_ids.clone();
    let payload: Properties;
    if device_cmd.properties.is_none() {
        payload = serde_json::from_str(get_stdin_from_pipe().as_str()).unwrap();
    } else {
        payload = serde_json::from_str(&device_cmd.properties.clone().unwrap()).unwrap();
    }

    let mut devices = json!([]);
    for id in ids {
        let mut device =
            update_device_request(target.clone(), &device_cmd.target, &payload, id.as_str())
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

async fn update_device_request(
    url: String,
    target: &TargetProperties,
    properties: &Properties,
    device_id: &str,
) -> Result<Value, reqwest::Error> {
    let url = format!(
        "http://{}/devicetwins/{}?device_id={}",
        url,
        target.as_str(),
        device_id
    );

    let client = Client::new();
    let resp = client.put(url).json(&properties).send().await?;
    let x = resp.json::<Value>().await?;
    // TODO: add device id call ?device_id=maxi2

    // TODO: when docer issue is fix, we can updated to 1.0.0 to fix. https://github.com/surrealdb/surrealdb/issues/2574
    //let res = reqwest::get(url).await?.json::<Vec<DeviceTwin>>().await?;
    //let deserialized: Vec<DeviceTwin> = serde_json::from_str(&res).unwrap();
    Ok(x)
}

use clap::Args;
use libs::utils::cli::get_stdin_from_pipe;
use serde_json::{json, Value};

#[derive(Args)]
pub struct DeviceCmd {
    /// list of devices to delete. If . read from stdin.
    device_ids: Vec<String>,

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
        ids = serde_json::from_str(get_stdin_from_pipe().as_str()).unwrap();
    }

    let mut devices = json!([]);
    for id in ids {
        let mut device = delete_device_request(target.clone(), id)
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

async fn delete_device_request(url: String, device_id: String) -> Result<Value, reqwest::Error> {
    let url = format!("http://{}/devicetwins?device_id={}", url, device_id);
    let client = reqwest::Client::new();
    let resp = client.delete(url).send().await?;

    Ok(resp.json::<Value>().await?)
}

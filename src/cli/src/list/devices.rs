use clap::Args;
use serde_json::{json, Value};
use string_builder::Builder;
use y::utils::cli::get_stdin_from_pipe;

#[derive(Args)]
pub struct DeviceCmd {
    /// list of devices to print. If empty, print all devices. If . read from stdin.
    device_ids: Vec<String>,

    #[arg(short, long)]
    meta: bool,
    #[arg(short, long)]
    tag: bool,
    #[arg(short, long)]
    desired: bool,
    #[arg(short, long)]
    reported: bool,
}

pub async fn run_device_cmd(device_cmd: &DeviceCmd, target: String) -> Result<(), String> {
    let mut ids: Vec<String> = device_cmd.device_ids.clone();
    if device_cmd.device_ids.len() == 1 && device_cmd.device_ids[0] == "." {
        // TODO: find how to make it non blocking if empty
        // Could use async stdin, have two spawn that returns when the first one finish.
        // The first on is stdin liscen
        // The second is a timer that finish of .200ms (to be refined), to make sure we don't wait for ever.
        ids = get_stdin_from_pipe()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
    }
    list_all_devices(
        target,
        ids,
        device_cmd.meta,
        device_cmd.tag,
        device_cmd.desired,
        device_cmd.reported,
    )
    .await;

    Ok(())
}

async fn list_all_devices(
    url: String,
    device_ids: Vec<String>,
    meta: bool,
    tag: bool,
    desired: bool,
    reported: bool,
) {
    if device_ids.len() == 0 {
        let devices = match get_devices_data(url, None).await {
            Ok(d) => d,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        print!("{}", stringify_list_tag_prop(devices.clone()));
        return;
    }

    let mut devices = json!([]);
    for device_id in device_ids {
        let device = match get_devices_data(url.clone(), Some(device_id.clone())).await {
            Ok(d) => d,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };

        if let Some(x) = device.as_array() {
            let mut x = x.clone();
            if x.len() > 0 {
                if !meta {
                    x[0].as_object_mut().unwrap().remove("meta_properties");
                }
                if !tag {
                    x[0].as_object_mut().unwrap().remove("tag_properties");
                }
                if !desired {
                    x[0].as_object_mut().unwrap().remove("desired_properties");
                }
                if !reported {
                    x[0].as_object_mut().unwrap().remove("reported_properties");
                }
                devices.as_array_mut().unwrap().push(x[0].clone());
            }
        }

        //print!("{}", stringify_list_tag_prop(devices.clone()));
    }
    print!("{}", serde_json::to_string_pretty(&devices).unwrap());
}

fn stringify_list_tag_prop(twins: Value) -> String {
    let mut builder = Builder::default();
    twins.as_array().unwrap().iter().for_each(|twin| {
        builder.append(format!("{}\n", stringify_tag_prop(twin.clone())));
    });

    let x = builder.string().unwrap();
    x
}

fn stringify_tag_prop(twin: Value) -> String {
    let mut builder = Builder::default();
    builder.append(format!("{}", twin["id"]));
    builder.append("  ");
    builder.append(format!("{}", twin["tag_properties"]));

    let x = builder.string().unwrap();
    //serde_json::to_string_pretty(&x).unwrap();
    x
}

async fn get_devices_data(url: String, device_id: Option<String>) -> Result<Value, reqwest::Error> {
    if let Some(id) = device_id {
        let url = format!("http://{}/devicetwins?device_id={}", url, id);
        let res = reqwest::get(url).await?.json::<Value>().await?;
        return Ok(res);
    }

    let url = format!("http://{}/devicetwins", url);
    let res = reqwest::get(url).await?.json::<Value>().await?;

    // TODO: add device id call ?device_id=maxi2

    // TODO: when docer issue is fix, we can updated to 1.0.0 to fix. https://github.com/surrealdb/surrealdb/issues/2574
    //let res = reqwest::get(url).await?.json::<Vec<DeviceTwin>>().await?;
    //let deserialized: Vec<DeviceTwin> = serde_json::from_str(&res).unwrap();
    Ok(res)
}

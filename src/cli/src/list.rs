use clap::{command, Args, Subcommand};

pub mod devices;

#[derive(Args)]
pub struct ListCmd {
    #[command(subcommand)]
    pub command: ListCmds,
}

#[derive(Subcommand)]
pub enum ListCmds {
    /// list devices
    Devices(devices::DeviceCmd),
}

pub async fn run_list_cmd(list_cmd: &ListCmd, target: String) -> Result<(), String> {
    match &list_cmd.command {
        ListCmds::Devices(device_cmd) => devices::run_device_cmd(device_cmd, target).await,
    }
}

use clap::{command, Args, Subcommand};

pub mod device;

#[derive(Args)]
pub struct DeleteCmd {
    #[command(subcommand)]
    pub command: DeleteCmds,
}

#[derive(Subcommand)]
pub enum DeleteCmds {
    /// create device
    Device(device::DeviceCmd),
}

pub async fn run_delete_cmd(delete_cmd: &DeleteCmd, target: String) -> Result<(), String> {
    match &delete_cmd.command {
        DeleteCmds::Device(device_cmd) => device::run_device_cmd(device_cmd, target).await,
    }
}

use clap::{command, Args, Subcommand};

pub mod device;

#[derive(Args)]
pub struct CreateCmd {
    #[command(subcommand)]
    pub command: CreateCmds,
}

#[derive(Subcommand)]
pub enum CreateCmds {
    /// create device
    Device(device::DeviceCmd),
}

pub async fn run_create_cmd(create_cmd: &CreateCmd, target: String) -> Result<(), String> {
    match &create_cmd.command {
        CreateCmds::Device(device_cmd) => device::run_device_cmd(device_cmd, target).await,
    }
}

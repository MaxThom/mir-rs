use clap::{command, Args, Subcommand};

pub mod device;

#[derive(Args)]
pub struct UpdateCmd {
    #[command(subcommand)]
    pub command: UpdateCmds,
}

#[derive(Subcommand)]
pub enum UpdateCmds {
    /// create device
    Device(device::DeviceCmd),
}

pub async fn run_update_cmd(update_cmd: &UpdateCmd, target: String) -> Result<(), String> {
    match &update_cmd.command {
        UpdateCmds::Device(device_cmd) => device::run_device_cmd(device_cmd, target).await,
    }
}

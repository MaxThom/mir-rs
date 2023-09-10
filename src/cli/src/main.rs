use clap::{Parser, Subcommand};
use create::CreateCmd;
use delete::DeleteCmd;
use list::ListCmd;
use listen::ListenCmd;
use update::UpdateCmd;

pub mod create;
pub mod delete;
pub mod list;
pub mod listen;
pub mod update;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set Mir server adress
    #[arg(
        short,
        long,
        value_name = "ADRESS",
        default_value_t = String::from("amqp://guest:guest@localhost:5672")
    )]
    target: String,

    /// Set Redox server adress
    #[arg(
        short,
        long,
        value_name = "ADRESS",
        default_value_t = String::from("localhost:5047")
    )]
    redox_target: String,

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
    /// create device
    Create(CreateCmd),
    /// update device
    Update(UpdateCmd),
    /// delete device
    Delete(DeleteCmd),
    /// listen to mir streams
    Listen(ListenCmd),
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        MirCmds::List(cmd) => {
            return list::run_list_cmd(cmd, cli.redox_target).await;
        }
        MirCmds::Create(cmd) => {
            return create::run_create_cmd(cmd, cli.redox_target).await;
        }
        MirCmds::Update(cmd) => {
            return update::run_update_cmd(cmd, cli.redox_target).await;
        }
        MirCmds::Delete(cmd) => {
            return delete::run_delete_cmd(cmd, cli.redox_target).await;
        }
        MirCmds::Listen(cmd) => {
            return listen::run_listen_cmd(cmd, cli.target).await;
        }
    }
}

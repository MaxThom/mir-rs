use clap::{Parser, Subcommand};
use create::CreateCmd;
use list::ListCmd;

pub mod create;
pub mod list;

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
    /// create device
    Create(CreateCmd),
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        MirCmds::List(cmd) => {
            // TODO: better stdin from json
            return list::run_list_cmd(cmd, cli.target).await;
        }
        MirCmds::Create(cmd) => {
            // TODO: better stdin from json
            return create::run_create_cmd(cmd, cli.target).await;
        }
    }
}

use clap::{Parser, Subcommand};
use list::ListCmd;
use y::utils::cli::get_stdin_from_pipe;

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
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        MirCmds::List(list_cmd) => {
            return list::run_list_cmd(list_cmd, cli.target).await;
        }
    }
}

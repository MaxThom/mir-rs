use std::path::PathBuf;

use clap::{arg, command, value_parser, ArgMatches};

pub const CONFIG_KEY: &str = "config";

pub fn setup_cli() -> ArgMatches {
    command!() // requires `cargo` feature
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .get_matches()
}

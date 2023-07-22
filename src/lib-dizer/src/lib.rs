use std::{path::PathBuf, str::FromStr};

use clap::ArgMatches;
use log::info;
use serde::Deserialize;
use y::utils::{config::FileFormat, setup_cli, setup_config, setup_logger};

#[derive(Debug, Default)]
pub struct Dizer {
    pub config: Config,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub device_id: String,
    pub log_level: String,
    pub mir_addr: String,
    pub thread_count: usize,
}

#[derive(Default, Debug)]
pub struct DizerBuilder {
    config_file_path: Option<PathBuf>,
    device_id: Option<String>,
    mir_addr: Option<String>,
    thread_count: Option<usize>,
    log_level: Option<String>,
    cli: Option<ArgMatches>,
}

#[derive(Debug)]
pub enum IncompleteDizerBuild {
    NoMirServer,
    NoDeviceId,
}

const APP_NAME: &str = "dizer";

impl DizerBuilder {
    pub fn new() -> Self {
        Self {
            config_file_path: None,
            device_id: None,
            mir_addr: None,
            thread_count: None,
            log_level: None,
            cli: None,
        }
    }

    pub fn with_cli(&mut self) -> &mut Self {
        self.cli = Some(setup_cli());
        self
    }

    // add a byte[] representation of the content of a file
    // add support for json, toml?
    pub fn with_config_file(&mut self, filepath: &str) -> &mut Self {
        if filepath.is_empty() {
            return self;
        }
        let x = PathBuf::from_str(filepath)
            .unwrap_or_else(|_| panic!("Invalid config file path: {}", filepath));
        self.config_file_path = Some(x);
        self
    }

    pub fn with_logger(&mut self, log_level: &str) -> &mut Self {
        if log_level.is_empty() {
            return self;
        }
        self.log_level = Some(log_level.to_string());
        self
    }

    pub fn with_device_id(&mut self, device_id: &str) -> &mut Self {
        if device_id.is_empty() {
            return self;
        }
        self.device_id = Some(device_id.to_string());
        self
    }

    pub fn with_thread_count(&mut self, count: usize) -> &mut Self {
        if count == 0 {
            return self;
        }
        self.thread_count = Some(count);
        self
    }

    pub fn with_mir_server(&mut self, server_addr: &str) -> &mut Self {
        if server_addr.is_empty() {
            return self;
        }
        self.mir_addr = Some(server_addr.to_string());
        self
    }

    pub fn build(&mut self) -> Result<Dizer, IncompleteDizerBuild> {
        println!("{:?}", &self);
        let mut dizer = Dizer::default();

        // Default < Builder < Configfile < Cli

        // Builder
        if let Some(x) = &self.device_id {
            dizer.config.device_id = x.to_string();
        }
        if let Some(x) = &self.log_level {
            dizer.config.log_level = x.to_string();
        }
        // Builder
        if let Some(x) = &self.thread_count {
            dizer.config.thread_count = x.to_owned();
        }
        if let Some(x) = &self.mir_addr {
            dizer.config.mir_addr = x.to_string();
        }

        // Cli matches
        if let Some(x) = &self.cli {
            let y = x.get_one::<PathBuf>(y::utils::cli::CONFIG_KEY);
            if let Some(z) = y {
                self.config_file_path = Some(z.clone());
            }
        }

        // Configfile load
        if let Some(x) = &self.config_file_path {
            dizer.config = setup_config(APP_NAME, FileFormat::YAML, Some(x)).unwrap();
        }

        // Logger init
        if !dizer.config.log_level.is_empty() {
            setup_logger(dizer.config.log_level.clone())
                .unwrap_or_else(|e| panic!("Invalid logger configuration: {:?}", e));
        } else if let Some(x) = &self.log_level {
            setup_logger(x.to_string())
                .unwrap_or_else(|e| panic!("Invalid logger configuration: {:?}", e));
        }

        info!("{:?}", dizer.config);

        if dizer.config.device_id.is_empty() {
            return Err(IncompleteDizerBuild::NoDeviceId);
        }

        if dizer.config.mir_addr.is_empty() {
            return Err(IncompleteDizerBuild::NoMirServer);
        }

        Ok(dizer)
    }
}

use std::path::PathBuf;

use config_sys::{Config, ConfigError, Environment, File};
use serde::Deserialize;

const CONFIG_FOLDER_PATH_DEFAULT: &str = "./config/";
const CONFIG_FILE_LOCAL_PREFIX: &str = "local_";
// This makes it so "<APP_NAME>_DEVICES__0__NAME overrides devices[0].name
const CONFIG_ENV_SEPARATOR: &str = "__";

#[derive(Debug)]
pub enum FileFormat {
    INI,
    JSON,
    YAML,
    TOML,
    RON,
    JSON5,
}

impl FileFormat {
    pub fn as_str(&self) -> &str {
        match self {
            FileFormat::INI => "ini",
            FileFormat::JSON => "json",
            FileFormat::YAML => "yaml",
            FileFormat::TOML => "toml",
            FileFormat::RON => "ron",
            FileFormat::JSON5 => "json5",
        }
    }
}

pub fn setup_config<'a, T>(
    app_name: &str,
    file_format: FileFormat,
    extra_path: Option<&PathBuf>,
) -> Result<T, ConfigError>
where
    T: Deserialize<'a>,
{
    let mut s = Config::builder()
        .add_source(
            File::with_name(
                format!(
                    "{}{}.{}",
                    CONFIG_FOLDER_PATH_DEFAULT,
                    app_name,
                    file_format.as_str()
                )
                .as_str(),
            )
            .required(false),
        )
        .add_source(
            File::with_name(
                format!(
                    "{}{}{}.{}",
                    CONFIG_FOLDER_PATH_DEFAULT,
                    CONFIG_FILE_LOCAL_PREFIX,
                    app_name,
                    file_format.as_str()
                )
                .as_str(),
            )
            .required(false),
        )
        .add_source(
            Environment::with_prefix(app_name.to_uppercase().as_str())
                .separator(CONFIG_ENV_SEPARATOR),
        );

    if let Some(extra_path) = extra_path {
        s = s.add_source(File::with_name(
            extra_path.as_path().display().to_string().as_str(),
        ));
    }

    let c = s.build().unwrap();
    c.try_deserialize::<T>()
}

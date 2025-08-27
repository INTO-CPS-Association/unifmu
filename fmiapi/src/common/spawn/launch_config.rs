use std::{
    error::Error,
    fmt::{Debug, Display},
    fs::read_to_string,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub enum BackendLocation {
    #[default]
    Local,
    Remote,
}

#[derive(Debug, Deserialize)]
pub struct LaunchConfig {
    #[serde(default)]
    pub location: BackendLocation,
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>,
    pub macos: Option<Vec<String>>,
}

impl LaunchConfig {
    pub fn create(resource_path: &Path) -> ConfigResult<LaunchConfig> {
        let config_path = resource_path.join("launch.toml");
        println!("Reading configuration file at path '{:?}'",config_path);

        // Check if file is there and give error if not.
        if !config_path.exists() {
            return Err(ConfigError::NotFound(config_path));
        }

        let config_string = read_to_string(&config_path)
            .map_err(|error| ConfigError::Unreadable(config_path, error))?;

        let config: LaunchConfig = toml::from_str(&config_string)
            .map_err(ConfigError::Invalid)?;

        Ok(config)
    }

    pub fn get_launch_command(&self) -> ConfigResult<Vec<String>> {
        Ok(
            match std::env::consts::OS {
                "windows" => self.windows.as_ref(),
                "macos" => self.macos.as_ref(),
                "linux" => self.linux.as_ref(),
                _other_os=> None
            }.ok_or(
                ConfigError::UnsupportedOS(String::from(std::env::consts::OS))
            )?
            .to_vec()
        )
    }
}

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    UnsupportedOS(String),
    Invalid(toml::de::Error),
    Unreadable(PathBuf, std::io::Error),
    NotFound(PathBuf)
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedOS(os) => write!(
                f, "no backend launch command found for the current operating system ({})", os
            ),
            Self::Invalid(toml_error) => write!(
                f, "configuration file was opened, but the contents do not appear to be valid; {}", toml_error
            ),
            Self::Unreadable(path, io_error) => write!(
                f, "unable to read configuration file stored at path '{}'; {}", path.display(), io_error
            ),
            Self::NotFound(path) => write!(
                f, "the config file was not found at '{}'", path.display()
            )
        }
    }
}

impl Error for ConfigError {}
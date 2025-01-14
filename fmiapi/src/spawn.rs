use std::{
    fs::read_to_string,
    path::Path,
};

use crate::dispatcher::{Dispatch, Dispatcher};

use serde::Deserialize;
use tracing::{debug, error, info};

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
    pub fn create(resource_path: &Path) -> Result<LaunchConfig, ()> {
        let config_path = resource_path.join("launch.toml");
        info!("Reading configuration file at path '{:?}'",config_path);

        // Check if file is there and give error if not.
        if !config_path.exists() {
            error!("Configuration file not found at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.", config_path);
            return Err(());
        }

        let config = match read_to_string(&config_path) {
            Ok(config) => config,
            Err(_) => {
                error!(
                    "Unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
                     config_path
                );
                return Err(());
            }
        };

        let config: LaunchConfig = match toml::from_str(config.as_str()) {
            Ok(config) => config,
            Err(e) => {
                error!("configuration file was opened, but the contents does not appear to be valid: {:?}", e);
                return Err(());
            }
        };

        return Ok(config);

    }

    pub fn get_launch_command(& self) -> Result<Vec<String>, ()> {
        match std::env::consts::OS {
            "windows" => match &self.windows {
                Some(cmd) => Ok(cmd.to_vec()),
                None => return Err(()),
            },
            "macos" => match &self.macos {
                Some(cmd) => Ok(cmd.to_vec()),
                None => return Err(()),
            },
            _other => match &self.linux {
                Some(cmd) => Ok(cmd.to_vec()),
                None => return Err(()),
            },
        }
    }
}

pub fn spawn_slave(resource_path: &Path) -> Result<Dispatcher, ()> {
    let config = LaunchConfig::create(resource_path)?;

    debug!("{:?}", config);

    let dispatcher_result = match config.location {
        BackendLocation::Local => Dispatcher::local(
            resource_path,
            &config.get_launch_command()?
        ),
        BackendLocation::Remote => Dispatcher::remote()
    };

    let mut dispatcher = match dispatcher_result {
        Ok(dispatcher) => dispatcher,
        Err(_) => {
            error!("Couldn't create new dispatcher.");
            return Err(());
        }
    };

    info!("Awaiting handshake.");
    match dispatcher.await_handshake() {
        Ok(_) => {
            info!("Connection established!");
            Ok(dispatcher)
        },
        Err(e) => {
            error!{"Handshake failed with error {:#?}", e};
            Err(())
        }
    } 
}
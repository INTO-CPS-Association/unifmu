use std::{
    ffi::OsString,
    fs::read_to_string,
    path::Path,
};

use crate::fmi2_dispatcher::Fmi2CommandDispatcher;
use crate::fmi3_dispatcher::{
    Fmi3CommandDispatcher,
    DispatcherError as Fmi3DispatcherError,
};



use serde::Deserialize;
use subprocess::{Popen, PopenConfig};
use tracing::{debug, dispatcher, error, info, span, warn, Level};


#[derive(Deserialize)]
pub struct LaunchConfig {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>,
    pub macos: Option<Vec<String>>,
}

impl LaunchConfig {
    pub fn spawn(resource_path: &Path) -> Result<LaunchConfig, ()> {
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
            Err(_) => {
                error!("configuration file was opened, but the contents does not appear to be valid");
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

pub fn spawn_fmi2_slave(resource_path: &Path) -> Result<(Fmi2CommandDispatcher, Popen), ()> {

    let launch_command = LaunchConfig::spawn(resource_path)?
        .get_launch_command()?;

    let mut dispatcher = Fmi2CommandDispatcher::new("tcp://0.0.0.0:0");

    let endpoint = dispatcher.endpoint.to_owned();
    let endpoint_port = match endpoint
        .split(":")
        .last() {
            Some(port) => port,
            None => {
                error!("No port was specified for endpoint.");
                return Err(());
            }
        }
        .to_owned();

    // set environment variables
    let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

    env_vars.push((
        OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
        OsString::from(endpoint),
    ));
    env_vars.push((
        OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
        OsString::from(endpoint_port),
    ));

    // spawn process
    let popen = match Popen::create(
        &launch_command,
        PopenConfig {
            cwd: Some(resource_path.as_os_str().to_owned()),
            env: Some(env_vars),
            ..Default::default()
        },
    ) {
        Ok(popen) => popen,
        Err(_e) => {
            eprintln!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", launch_command);
            return Err(());
        }
    };

    match dispatcher.await_handshake() {
        Ok(_handshake) => Ok((dispatcher, popen)),
        Err(_e) => Err(()),
    }
}

pub fn spawn_fmi3_slave(resource_path: &Path) -> Result<Fmi3CommandDispatcher, ()> {
    // grab launch command for host os
    let launch_command = LaunchConfig::spawn(resource_path)?
    .get_launch_command()?;

    info!("Establishing command dispatcher.");
    let mut dispatcher = match Fmi3CommandDispatcher::create(
        resource_path,
        &launch_command,
        "tcp://127.0.0.1:0"
    ) {
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

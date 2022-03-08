use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::fmi2_dispatcher::Fmi2CommandDispatcher;
use crate::fmi3_dispatcher::Fmi3CommandDispatcher;

use serde::Deserialize;
use subprocess::{Popen, PopenConfig};

#[derive(Deserialize)]
pub struct LaunchConfig {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>,
    pub macos: Option<Vec<String>>,
}

pub fn spawn_fmi2_slave(resource_path: &Path) -> Result<(Fmi2CommandDispatcher, Popen), ()> {
    let config_path = resource_path.join("launch.toml");

    let config = read_to_string(&config_path).expect(&format!(
             "Unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
             config_path
         ));

    let config: LaunchConfig = toml::from_str(config.as_str())
        .expect("configuration file was opened, but the contents does not appear to be valid");

    let mut dispatcher = Fmi2CommandDispatcher::new("tcp://127.0.0.1:0");

    let endpoint = dispatcher.endpoint.to_owned();
    let endpoint_port = endpoint
        .split(":")
        .last()
        .expect("There should be a port after the colon")
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

    // grab launch command for host os
    let launch_command = match std::env::consts::OS {
        "windows" => match config.windows {
            Some(cmd) => cmd,
            None => return Err(()),
        },
        "macos" => match config.macos {
            Some(cmd) => cmd,
            None => return Err(()),
        },
        _other => match config.linux {
            Some(cmd) => cmd,
            None => return Err(()),
        },
    };

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
        Err(e) => {
            eprintln!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", launch_command);
            return Err(());
        }
    };

    match dispatcher.await_handshake() {
        Ok(handshake) => Ok((dispatcher, popen)),
        Err(e) => Err(()),
    }
}

pub fn spawn_fmi3_slave(resource_path: &Path) -> Result<(Fmi3CommandDispatcher, Popen), ()> {
    let config_path = resource_path.join("launch.toml");

    let config = read_to_string(&config_path).expect(&format!(
             "Unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
             config_path
         ));

    let config: LaunchConfig = toml::from_str(config.as_str())
        .expect("configuration file was opened, but the contents does not appear to be valid");

    let mut dispatcher = Fmi3CommandDispatcher::new("tcp://127.0.0.1:0");

    let endpoint = dispatcher.endpoint.to_owned();
    let endpoint_port = endpoint
        .split(":")
        .last()
        .expect("There should be a port after the colon")
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

    // grab launch command for host os
    let launch_command = match std::env::consts::OS {
        "windows" => match config.windows {
            Some(cmd) => cmd,
            None => return Err(()),
        },
        "macos" => match config.macos {
            Some(cmd) => cmd,
            None => return Err(()),
        },
        _other => match config.linux {
            Some(cmd) => cmd,
            None => return Err(()),
        },
    };

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
        Err(e) => {
            eprintln!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", launch_command);
            return Err(());
        }
    };

    match dispatcher.await_handshake() {
        Ok(handshake) => Ok((dispatcher, popen)),
        Err(e) => Err(()),
    }
}

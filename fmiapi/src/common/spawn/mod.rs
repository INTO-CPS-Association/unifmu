//! Contains the `spawn_slave()` function, which initiates the UniFMU backend
//! and returns a `Dispatcher` for further communication with the backend.
//! 
//! Also contains other types related to spawning.

mod launch_config;

use launch_config::{BackendLocation, ConfigError, LaunchConfig};

use super::dispatcher::{Dispatch, Dispatcher, DispatcherError};

use std::{
    error::Error,
    fmt::{Debug, Display},
    path::Path,
};

/// Initiates the UniFMU backend based on the contents of the `launch.toml`
/// config file. Returns an `Ok(Dispatcher)` on successful initiation.
/// 
/// User action may be required in the case that the backend is remote.
/// In this case the `remote_connection_notifier` will be called with the
/// port number that the Rust layer will be listening on.
pub fn spawn_slave(
    resource_path: &Path,
    remote_connection_notifier: impl Fn(&str)
) -> SpawnResult<Dispatcher> {
    let config = LaunchConfig::create(resource_path)?;

    let dispatcher_result = match config.location {
        BackendLocation::Local => Dispatcher::local(
            resource_path,
            &config.get_launch_command()?
        ),
        BackendLocation::Remote => Dispatcher::remote(remote_connection_notifier)
    };

    let mut dispatcher = match dispatcher_result {
        Ok(dispatcher) => dispatcher,
        Err(error) => {
            return Err(SpawnError::DispatcherCreation(error));
        }
    };

    println!("Awaiting handshake.");
    match dispatcher.await_handshake() {
        Ok(_) => {
            println!("Connection established!");
            Ok(dispatcher)
        },
        Err(error) => {
            Err(SpawnError::Handshake(error))
        }
    } 
}

pub type SpawnResult<T> = Result<T, SpawnError>;

#[derive(Debug)]
pub enum SpawnError {
    Handshake(DispatcherError),
    DispatcherCreation(DispatcherError),
    Config(ConfigError)
}

impl Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handshake(dp_error) => write!(
                f, "handshake failed; {}", dp_error
            ),
            Self::DispatcherCreation(dp_error) => write!(
                f, "couldn't create new dispatcher; {}", dp_error
            ),
            Self::Config(cf_error) => write!(
                f, "couldn't import config; {}", cf_error
            )
        }
    }
}

impl Error for SpawnError {}

impl From<ConfigError> for SpawnError {
    fn from(value: ConfigError) -> Self {
        Self::Config(value)
    }
}
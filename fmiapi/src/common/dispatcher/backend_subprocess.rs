//! Contains the BackendSubprocess, a struct wrapping a handle to the
//! process containing the backend.

use std::{
    error::Error,
    ffi::OsString,
    fmt::{Debug, Display},
    path::Path
};

use subprocess::{ExitStatus, Popen, PopenConfig, PopenError};
use tokio::time::{Duration, sleep};

/// Representes the subprocess containing the backend.
/// 
/// Stores the subprocess handle for concurrency reasons.
pub struct BackendSubprocess {
    polling_time: Duration,
    subprocess: Popen
}

impl BackendSubprocess {
    pub fn create(
        endpoint: String,
        launch_command: &Vec<String>,
        resource_path: &Path
    ) -> SubprocessResult<Self> {
        let endpoint_port = match endpoint
            .split(":")
            .last() {
                Some(port) => port,
                None => {
                    return Err(SubprocessError::NoPortGiven)
                }
            }
            .to_owned();
        
        let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
            OsString::from(endpoint),
        ));
        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
            OsString::from(endpoint_port),
        ));

        let subprocess = match Popen::create(
            launch_command,
            PopenConfig {
                cwd: Some(resource_path.as_os_str().to_owned()),
                env: Some(env_vars),
                ..Default::default()
            },
        ) {
            Ok(subprocess) => subprocess,
            Err(error) => {
                return Err(SubprocessError::UnExecutableCommand(
                    format!("{:?}", launch_command),
                    error
                ))
            }
        };

        Ok(
            Self{
                // TODO: This is a magic number. Is there a smarter way to define polling_time?
                polling_time: Duration::from_millis(100),
                subprocess
            }
        )
    }

    /// Continously polls the backend subprocess and returns if the subprocess
    /// returns an exit status.
    /// 
    /// Will only ever return an Err.
    pub async fn monitor_subprocess(&mut self) -> SubprocessResult<()> {
        loop {
            match self.subprocess.poll() {
                Some(exit_status) => {
                    return Err(SubprocessError::UnexpectedExit(exit_status))
                },
                None => {
                    sleep(self.polling_time).await; // Is magic number.
                }
            }
        }
    }
}

type SubprocessResult<T> = Result<T, SubprocessError>;

#[derive(Debug)]
pub enum SubprocessError {
    UnexpectedExit(ExitStatus),
    NoPortGiven,
    UnExecutableCommand(String, PopenError)
}

impl Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedExit(exit_status) => {
                let clarification = match exit_status {
                    ExitStatus::Exited(code) => format!(
                        "with exit status {}", code
                    ),
                    ExitStatus::Signaled(signal) => format!(
                        "because of signal {}", signal
                    ),
                    ExitStatus::Other(status) => format!(
                        "with unknown status {}", status
                    ),
                    ExitStatus::Undetermined => String::from(
                        "for an undeterminable reason"
                    )
                };
                write!(f, "backend exited unexpectedly {}", clarification)
            }
            Self::NoPortGiven => {
                write!(f, "the endpoint given to for the backend to connect to didn't have a portnumber")
            }
            Self::UnExecutableCommand(command, popen_error) => {
                write!(f, "unable to start the backend subprocess using the specified command '{}'; {}", command, popen_error)
            }
        }
    }
}

impl Error for SubprocessError {}
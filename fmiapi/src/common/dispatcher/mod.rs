//! Contains the `Dispatch` trait and structs implementing the trait.

mod backend_subprocess;
mod backend_socket;

#[cfg(test)]
mod tests;

use backend_subprocess::{BackendSubprocess, SubprocessError};
use backend_socket::{BackendSocket, SocketError};

use super::unifmu_handshake::{HandshakeStatus, HandshakeReply};

use std::{
    error::Error,
    fmt::{Debug, Display},
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path
};

use prost::{Message, UnknownEnumValue};
use tokio::runtime::Runtime;
use tokio::select;

fn guarded_communication<T>(
    failed: &mut bool,
    operation: impl FnOnce() -> DispatcherResult<T>,
) -> DispatcherResult<T> {
    if *failed {
        return Err(DispatcherError::ConnectionFailed);
    }
    // A panic can leave socket state partially mutated. Contain unwinding
    // before the C FMI ABI, but never reuse the failed connection.
    let result = catch_unwind(AssertUnwindSafe(operation)).unwrap_or_else(|payload| {
        let message = if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&str>() {
            message.to_string()
        } else {
            "non-string panic payload".to_owned()
        };
        Err(DispatcherError::CommunicationPanic(message))
    });
    *failed = result.is_err();
    result
}

/// Generic Dispatcher for dispatching FMI commands to arbitrary backend.
/// Can send and recieve messages and await handshake from backend.
pub enum Dispatcher {
    Local(LocalDispatcher),
    Remote(RemoteDispatcher)
}

impl Dispatcher {
    /// Creates a Dispatcher to a local UNIFMU backend.
    /// 
    /// The backend is started as a subprocess with the launch_command using
    /// the resources at resource_path as part of the Dispatchers creation.
    pub fn local(
        resource_path: &Path,
        launch_command: &Vec<String>
    ) -> DispatcherResult<Self> {
        Ok(
            Self::Local(
                LocalDispatcher::create(
                    resource_path,
                    launch_command
                )?
            )
        )
    }

    /// Creates a Dispatcher to a remote UNIFMU backend.
    /// 
    /// The backend must be initialized seperately. When the message queue
    /// socket is ready, the function passed in `remote_connection_notifier`
    /// will be called with the socket port number as the only parameter.
    pub fn remote(
        remote_connection_notifier: impl Fn(&str)
    ) -> DispatcherResult<Self> {
        Ok(
            Self::Remote(
                RemoteDispatcher::create(remote_connection_notifier)?
            )
        )
    }
}

impl Dispatch for Dispatcher {
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()> {
        match self {
            Dispatcher::Local(d) => d.send(msg),
            Dispatcher::Remote(d) => d.send(msg)
        }
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        match self {
            Dispatcher::Local(d) => d.recv::<R>(),
            Dispatcher::Remote(d) => d.recv::<R>()
        }
    }

    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        match self {
            Dispatcher::Local(d) => d.send_and_recv::<S, R>(msg),
            Dispatcher::Remote(d) => d.send_and_recv::<S, R>(msg)
        }
    }
}

/// Dispatcher for dispatching FMI commands to a locally run backend.
/// 
/// Holds the handle for the subprocess as well as the handle for the socket
/// to said subprocess.
pub struct LocalDispatcher {
    socket: BackendSocket,
    subprocess: BackendSubprocess,
    runtime: Runtime,
    failed: bool,
}

impl LocalDispatcher {
    pub fn create(
        resource_path: &Path,
        launch_command: &Vec<String>
    ) -> DispatcherResult<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let socket = runtime.block_on(
            BackendSocket::create("tcp://127.0.0.1:0")
        )?;

        let subprocess = BackendSubprocess::create(
            socket.endpoint.to_string(),
            launch_command,
            resource_path
        )?;

        Ok(
            Self {
                socket,
                subprocess,
                runtime,
                failed: false,
            }
        )
    }
}

impl Dispatch for LocalDispatcher {
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()> {
        let result = guarded_communication(&mut self.failed, || {
            self.runtime.block_on(async {
                select! {
                    Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
                    result = self.socket.send::<S>(msg) => Ok(result?),
                }
            })
        });
        self.with_exit_diagnostics(result)
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        let result = guarded_communication(&mut self.failed, || {
            self.runtime.block_on(async {
                select! {
                    Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
                    result = self.socket.recv::<R>() => Ok(result?),
                }
            })
        });
        self.with_exit_diagnostics(result)
    }

    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        let result = guarded_communication(&mut self.failed, || {
            self.runtime.block_on(async {
                select! {
                    result = self.socket.send_and_recv::<S, R>(msg) => Ok(result?),
                    Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
                }
            })
        });
        self.with_exit_diagnostics(result)
    }
}

impl LocalDispatcher {
    fn with_exit_diagnostics<T>(&mut self, result: DispatcherResult<T>) -> DispatcherResult<T> {
        match result {
            Err(error @ (DispatcherError::Socket(_) | DispatcherError::CommunicationPanic(_))) => {
                // Socket disconnects can precede OS exit notification. Allow a
                // bounded wait to recover the exit status and backend traceback.
                let exit = self.runtime.block_on(async {
                    tokio::time::timeout(
                        std::time::Duration::from_millis(200),
                        self.subprocess.monitor_subprocess(),
                    ).await
                });
                match exit {
                    Ok(Err(exit_error)) => Err(DispatcherError::BackendDisconnected {
                        communication: Box::new(error),
                        subprocess: exit_error,
                    }),
                    _ => Err(error),
                }
            }
            result => result,
        }
    }
}

/// Dispatcher for dispatching FMI commands to a remote backend.
/// 
/// Holds the socket to the remote backend.
pub struct RemoteDispatcher {
    socket: BackendSocket,
    runtime: Runtime,
    failed: bool,
}

impl RemoteDispatcher {
    pub fn create(remote_connection_notifier: impl Fn(&str)) -> DispatcherResult<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let socket = runtime.block_on(
            BackendSocket::create("tcp://0.0.0.0:0")
        )?;

        let port = &format!("{}", &socket.endpoint)[14..];

        // Communicate the portnumber that remote backend should connect to
        remote_connection_notifier(port);

        Ok(
            Self {
                socket,
                runtime,
                failed: false,
            }
        )
    }
}

impl Dispatch for RemoteDispatcher {
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()> {
        guarded_communication(&mut self.failed, || {
            Ok(self.runtime.block_on(self.socket.send::<S>(msg))?)
        })
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        guarded_communication(&mut self.failed, || {
            Ok(self.runtime.block_on(self.socket.recv::<R>())?)
        })
    }

    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        guarded_communication(&mut self.failed, || {
            Ok(self.runtime.block_on(self.socket.send_and_recv::<S, R>(msg))?)
        })
    }
}

/// Must be implemented by all Dispatchers.
/// Ensures that FMI commands can be dispatched.
/// Gives the await_handshake() function using implemented methods.
pub trait Dispatch {
    /// Await the initial handshake from the backend.
    /// 
    /// This method includes a call to Self::recv() and thus can be followed by
    /// a call to Self::send() or Self::send_and_recv() without error (if the
    /// await_handshake() method itself returns without error).
    fn await_handshake(&mut self) -> DispatcherResult<()> {
        let response = self.recv::<HandshakeReply>()?;
        match HandshakeStatus::try_from(response.status) {
            Ok(HandshakeStatus::Ok) => {
                Ok(())
            },
            Ok(_) => {
                Err(DispatcherError::DeniedHandshake)
            },
            Err(error) => {
                Err(DispatcherError::MalformedHandshake(error))
            }
        }
    }

    /// Send a message to the backend.
    /// 
    /// A successfull call to Self::recv() or Self::send_and_recv() must have
    /// been made previous to a call to Self::send(). If not, send() will fail.
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()>; 

    /// Receive a response from the backend.
    /// 
    /// Communication with the backend must initiated by a call to
    /// this function.
    /// Any subsequent calls to this function must be preceded by a call to
    /// Self::send(). Otherwise the recv() call will fail.
    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R>;

    /// Sends a message to - and receives a response from - the backend.
    /// 
    /// A successfull call to Self::recv() or Self::send_and_recv() must have
    /// been made previous to a call to Self::send_and_recv(). If not,
    /// send_and_recv() will fail.
    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R>;
}

pub type DispatcherResult<T> = Result<T, DispatcherError>;

#[derive(Debug)]
pub enum DispatcherError {
    MalformedHandshake(UnknownEnumValue),
    DeniedHandshake,
    Socket(SocketError),
    Subprocess(SubprocessError),
    RuntimeSetup(std::io::Error),
    ConnectionFailed,
    CommunicationPanic(String),
    BackendDisconnected {
        communication: Box<DispatcherError>,
        subprocess: SubprocessError,
    },
}

impl Display for DispatcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedHandshake(uev_error) => write!(
                f, "handshake was malformed; {}", uev_error
            ),
            Self::DeniedHandshake => write!(
                f, "backend reported error as part of handshake"
            ),
            Self::Socket(sckt_error) => write!(
                f, "error in message queue socket; {}", sckt_error
            ),
            Self::Subprocess(sp_error) => write!(
                f, "error in backend subprocess; {}", sp_error
            ),
            Self::RuntimeSetup(io_error) => write!(
                f, "couldn't setup concurrency runtime; {}", io_error
            ),
            Self::ConnectionFailed => write!(f, "backend connection has failed; create a new FMU instance"),
            Self::CommunicationPanic(message) => write!(f, "backend communication panicked: {message}"),
            Self::BackendDisconnected { communication, subprocess } => {
                write!(f, "{communication}; {subprocess}")
            }
        }
    }
}

impl Error for DispatcherError {}

impl From<SubprocessError> for DispatcherError {
    fn from(value: SubprocessError) -> Self {
        Self::Subprocess(value)
    }
}

impl From<SocketError> for DispatcherError {
    fn from(value: SocketError) -> Self {
        Self::Socket(value)
    }
}

impl From<std::io::Error> for DispatcherError {
    fn from(value: std::io::Error) -> Self {
        Self::RuntimeSetup(value)
    }
}
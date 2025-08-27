mod backend_subprocess;
mod backend_socket;

use backend_subprocess::{BackendSubprocess, SubprocessError};
use backend_socket::{BackendSocket, SocketError};

use super::unifmu_handshake::{HandshakeStatus, HandshakeReply};

use std::{
    error::Error,
    fmt::{Debug, Display},
    path::Path
};

use colored::Colorize;
use prost::{Message, UnknownEnumValue};
use tokio::runtime::Runtime;
use tokio::select;

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
    /// The backend must be initialized seperately.
    pub fn remote(
        remote_connction_notifier: impl Fn(&str)
    ) -> DispatcherResult<Self> {
        Ok(
            Self::Remote(
                RemoteDispatcher::create(remote_connction_notifier)?
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
                runtime
            }
        )
    }
}

impl Dispatch for LocalDispatcher {
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()> {
        self.runtime.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
                result = self.socket.send::<S>(msg) => Ok(result?),
            }
        })
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        self.runtime.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
                result = self.socket.recv::<R>() => Ok(result?),
            }
        })
    }

    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        self.runtime.block_on(async {
            select! {
                result = self.socket.send_and_recv::<S, R>(msg) => Ok(result?),
                Err(e) = self.subprocess.monitor_subprocess() => Ok(Err(e)?),
            }
        })
    }
}

/// Dispatcher for dispatching FMI commands to a remote backend.
/// 
/// Holds the socket to the remote backend.
pub struct RemoteDispatcher {
    socket: BackendSocket,
    runtime: Runtime,
}

impl RemoteDispatcher {
    pub fn create(remote_connction_notifier: impl Fn(&str)) -> DispatcherResult<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let socket = runtime.block_on(
            BackendSocket::create("tcp://0.0.0.0:0")
        )?;

        let port = &format!("{}", &socket.endpoint)[14..]
            .on_green()
            .bold();
        let decorated_communication = "Connect remote backend to dispatcher through port "
            .bright_green()
            .bold();

        // Communicate the portnumber that remote backend should connect to
        remote_connction_notifier(&format!("{decorated_communication}{port}"));

        Ok(
            Self {
                socket,
                runtime,
            }
        )
    }
}

impl Dispatch for RemoteDispatcher {
    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()> {
        Ok(self.runtime.block_on(self.socket.send::<S>(msg))?)
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        Ok(self.runtime.block_on(self.socket.recv::<R>())?)
    }

    fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        Ok(self.runtime.block_on(self.socket.send_and_recv::<S, R>(msg))?)
    }
}

/// Must be implemented by all Dispatchers.
/// Ensures that FMI commands can be dispatched.
/// Gives the await_handshake() function using implemented methods.
pub trait Dispatch {
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

    fn send<S: Message + Debug>(&mut self, msg: &S) -> DispatcherResult<()>; 

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R>;

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
    RuntimeSetup(std::io::Error)
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
            )
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
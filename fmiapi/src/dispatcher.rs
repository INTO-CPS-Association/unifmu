use std::{
    error::Error,
    ffi::OsString,
    fmt::{Debug, Display},
    path::Path
};

use crate::unifmu_handshake::{HandshakeStatus, HandshakeReply};

use bytes::Bytes;
use colored::Colorize;
use prost::{DecodeError, Message, UnknownEnumValue};
use subprocess::{ExitStatus, Popen, PopenConfig, PopenError};
use tokio::runtime::Runtime;
use tokio::time::{Duration, sleep};
use tokio::select;
use zeromq::{Endpoint, RepSocket, Socket, SocketRecv, SocketSend, ZmqError};

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
/// Represents the communication socket with the backend process.
/// 
/// Stores the actual ZeroMQ Socket for concurrency reasons.
struct BackendSocket {
    socket: RepSocket,
    pub endpoint: Endpoint
}

impl BackendSocket {
    pub async fn create(endpoint: &str) -> SocketResult<Self> {
        let mut socket = RepSocket::new();

        let endpoint = match socket.bind(endpoint).await {
            Ok(endpoint) => endpoint,
            Err(error) => {
                return Err(SocketError::ZmqBind(error));
            }
        };

        Ok(Self {socket, endpoint})
    }

    /// Sends the contents of a message through the ZeroMQ socket to
    /// the backend.
    /// 
    /// A call to BackendSocket::recv() must have been made successfully
    /// previous to a call to BackendSocket::send(). If not, send() will fail.
    /// 
    /// A call to send() will return when the message has been put on the
    /// ZeroMQ message queue, NOT when the message has actually been received
    /// by the backend. As such, there is no absolute guarantee that the
    /// message has been received when this returns. 
    async fn send<S: Message + Debug>(&mut self, msg: &S) -> SocketResult<()> {
        let bytes_send: Bytes = msg.encode_to_vec().into();
        match  self.socket.send(bytes_send.into()).await {
            Ok(_) => Ok(()),
            Err(error) => {
                Err(SocketError::ZmqSend(format!("{:?}", msg), error))
            }
        }
    }

    /// Receives a message from the backend through the ZeroMQ socket.
    /// 
    /// Communication with the backend must be initiated by a call to
    /// this function.
    /// Any subsequent calls to this function must be preceded by a call to
    /// BackendSocket::send(). Otherwise the recv() call will fail.
    /// 
    /// A call to recv() will await until a message is received through the
    /// ZeroMQ socket.
    async fn recv<R: Message + Default>(&mut self) -> SocketResult<R> {
        match self.socket.recv().await {
            Ok(buf) => {
                let buf: Bytes = match buf.get(0) {
                    Some(bytes) => bytes,
                    None => {
                        return Err(SocketError::EmptyBuffer)
                    }
                }.to_owned();
                match R::decode(buf.as_ref()) {
                    Ok(msg) => Ok(msg),
                    Err(error) => {
                        Err(SocketError::Decode(error))
                    },
                }
            },
            Err(error) => {
                Err(SocketError::ZmqReceive(error))
            }
        }
    }

    /// Shorthand for a call to BackendSocket::send() followed by a call to
    /// BackendSocket::recv().
    async fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> SocketResult<R> {
        self.send(msg).await?;
        self.recv().await
    }
}

type SocketResult<T> = Result<T, SocketError>;

#[derive(Debug)]
pub enum SocketError {
    ZmqBind(ZmqError),
    ZmqSend(String, ZmqError),
    ZmqReceive(ZmqError),
    EmptyBuffer,
    Decode(DecodeError)
}

impl Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZmqBind(error) => write!(
                f, "couldn't bind to backend socket; {}", error
            ),
            Self::ZmqSend(message, error) => write!(
                f, "failed to send message {}; {}", message, error
            ),
            Self::ZmqReceive(error) => write!(
                f, "receiving on socket failed; {}", error
            ),
            Self::EmptyBuffer => write!(
                f, "no bytes in received message"
            ),
            Self::Decode(error) => write!(
                f, "couldn't decode received message; {}", error
            )
        }
    }
}

impl Error for SocketError {}

/// Representes the subprocess containing the backend.
/// 
/// Stores the subprocess handle for concurrency reasons.
struct BackendSubprocess {
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
                polling_time: Duration::from_millis(100),
                subprocess
            }
        )
    }

    /// Continously polls the backend subprocess and returns if the subprocess
    /// returns an exit status.
    /// 
    /// Will only ever return an Err.
    async fn monitor_subprocess(&mut self) -> SubprocessResult<()> {
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
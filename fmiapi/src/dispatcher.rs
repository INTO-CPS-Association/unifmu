use std::{
    ffi::OsString,
    path::Path
};

use crate::unifmu_handshake::{HandshakeStatus, HandshakeReply,};

use bytes::Bytes;
use prost::Message;
use subprocess::{ExitStatus, Popen, PopenConfig};
use tokio::runtime::Runtime;
use tokio::time::{Duration, sleep};
use tokio::select;
use tracing::{debug, error, info, warn};
use zeromq::{Endpoint, RepSocket, Socket, SocketRecv, SocketSend};

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
        launch_command: &Vec<String>,
    ) -> DispatcherResult<Self> {
        Ok(Self::Local(LocalDispatcher::create(resource_path, launch_command)?))
    }

    /// Creates a Dispatcher to a remote UNIFMU backend.
    /// 
    /// The backend must be initialized seperately.
    pub fn remote() -> DispatcherResult<Self> {
        Ok(Self::Remote(RemoteDispatcher::create()?))
    }
}

impl Dispatch for Dispatcher {
    fn send<S: Message>(&mut self, msg: &S) -> DispatcherResult<()> {
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

    fn send_and_recv<S: Message, R: Message + Default>(
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
    runtime: Runtime,
    socket: BackendSocket,
    subprocess: BackendSubprocess,
}

impl LocalDispatcher {
    pub fn create(
        resource_path: &Path,
        launch_command: &Vec<String>,
    ) -> Result<Self, DispatcherError> {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build() {
                Ok(runtime) => runtime,
                Err(e) => {
                    error!("Couldn't setup concurrency runtime: {:#?}", e);
                    return Err(DispatcherError::ConcurrencyError);
                }
            };

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
                runtime,
                socket,
                subprocess
            }
        )
    }
}

impl Dispatch for LocalDispatcher {
    fn send<S: Message>(&mut self, msg: &S) -> DispatcherResult<()> {
        self.runtime.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
                result = self.socket.send::<S>(msg) => result,
            }
        })
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        self.runtime.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
                result = self.socket.recv::<R>() => result,
            }
        })
    }

    fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        self.runtime.block_on(async {
            select! {
                result = self.socket.send_and_recv::<S, R>(msg) => result,
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
            }
        })
    }
}

/// Dispatcher for dispatching FMI commands to a remote backend.
/// 
/// Holds the socket to the remote backend.
pub struct RemoteDispatcher {
    runtime: Runtime,
    socket: BackendSocket,
}

impl RemoteDispatcher {
    pub fn create() -> Result<Self, DispatcherError> {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build() {
                Ok(runtime) => runtime,
                Err(e) => {
                    error!("Couldn't setup concurrency runtime: {:#?}", e);
                    return Err(DispatcherError::ConcurrencyError);
                }
            };

        let socket = runtime.block_on(
            BackendSocket::create("tcp://0.0.0.0:0")
        )?;

        // Aleksander: As far as I understand, all that the backend_proxy code
        // does is telling the user what endpoint to point their remote backend
        // to. This can be done in rust methinks. This way we can avoid writing
        // a script for each language and FMI version that just reads some 
        // environement variables and prints them out.
        // We also avoid starting a subprocess and parsing a launch file.
        info!(
            "Connect remote backend to dispatcher via endpoint {}",
            socket.endpoint
        );

        Ok(
            Self {
                runtime,
                socket,
            }
        )
    }
}

impl Dispatch for RemoteDispatcher {
    fn send<S: Message>(&mut self, msg: &S) -> DispatcherResult<()> {
        self.runtime.block_on(self.socket.send::<S>(msg))
    }

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R> {
        self.runtime.block_on(self.socket.recv::<R>())
    }

    fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R> {
        self.runtime.block_on(self.socket.send_and_recv::<S, R>(msg))
    }
}

/// Must be implemented by all Dispatchers.
/// Ensures that FMI commands can be dispatched.
/// Gives the await_handshake() function using implemented methods.
pub trait Dispatch {
    fn await_handshake(&mut self) -> DispatcherResult<()> {
        match self.recv::<HandshakeReply>() {
            Ok(response) => match HandshakeStatus::try_from(response.status) {
                Ok(HandshakeStatus::Ok) => {
                    info!("Handshake successful.");
                    Ok(())
                },
                Ok(_) => {
                    error!("Backend reported error as part of handshake.");
                    Err(DispatcherError::BackendImplementationError)
                },
                Err(e) => {
                    error!("Malformed handshake response.");
                    Err(DispatcherError::EnumDecodeError(e))
                }
            },
            Err(dispatcher_error) => Err(dispatcher_error)
        }
    }

    fn send<S: Message>(&mut self, msg: &S) -> DispatcherResult<()>; 

    fn recv<R: Message + Default>(&mut self) -> DispatcherResult<R>;

    fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> DispatcherResult<R>;
}

pub type DispatcherResult<T> = Result<T, DispatcherError>;

#[derive(Debug)]
pub enum DispatcherError {
    DecodeError(prost::DecodeError),
    EnumDecodeError(prost::UnknownEnumValue),
    EncodeError,
    SocketError,
    SubprocessError,
    ConcurrencyError,
    Timeout,
    BackendImplementationError,
}

/// Represents the communication socket with the backend process.
/// 
/// Stores the actual ZeroMQ Socket for concurrency reasons.
struct BackendSocket {
    socket: RepSocket,
    pub endpoint: Endpoint
}

impl BackendSocket {
    pub async fn create(endpoint: &str) -> Result<Self, DispatcherError> {
        let mut socket = RepSocket::new();

        let endpoint = match socket.bind(endpoint).await {
            Ok(endpoint) => endpoint,
            Err(e) => {
                error!("Couldn't bind to backend socket; {:#?}", e);
                return Err(DispatcherError::SocketError);
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
    async fn send<S: Message>(&mut self, msg: &S) -> Result<(), DispatcherError> {
        let bytes_send: Bytes = msg.encode_to_vec().into();
        match  self.socket.send(bytes_send.into()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Sending message {:?} failed with error {:?}", msg, e);
                Err(DispatcherError::SocketError)
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
    async fn recv<R: Message + Default>(&mut self) -> Result<R, DispatcherError> {
        match self.socket.recv().await {
            Ok(buf) => {
                let buf: Bytes = match buf.get(0) {
                    Some(bytes) => bytes,
                    None => {
                        warn!("No bytes in socket buffer");
                        return Err(DispatcherError::SocketError);
                    }
                }.to_owned();
                match R::decode(buf.as_ref()) {
                    Ok(msg) => Ok(msg),
                    Err(e) => {
                        error!("Couldn't decode message.");
                        Err(DispatcherError::DecodeError(e))
                    },
                }
            },
            Err(e) => {
                error!("Receiving on socket failed with error '{:#?}'", e);
                Err(DispatcherError::SocketError)
            }
        }
    }

    /// Shorthand for a call to BackendSocket::send() followed by a call to
    /// BackendSocket::recv().
    async fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> Result<R, DispatcherError> {
        self.send(msg).await?;
        self.recv().await
    }
}

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
    ) -> DispatcherResult<Self> {
        let endpoint_port = match endpoint
            .split(":")
            .last() {
                Some(port) => port,
                None => {
                    error!("No port was specified for endpoint.");
                    return Err(DispatcherError::SocketError);
                }
            }
            .to_owned();
        
        info!("Collecting local environment variables.");
        let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
            OsString::from(endpoint),
        ));
        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
            OsString::from(endpoint_port),
        ));

        info!("Spawning backend process.");
        debug!("Launch command: {}", &launch_command[0]);
        debug!("Environment variables: {:#?}", env_vars);
        let subprocess = match Popen::create(
            &launch_command,
            PopenConfig {
                cwd: Some(resource_path.as_os_str().to_owned()),
                env: Some(env_vars),
                ..Default::default()
            },
        ) {
            Ok(subprocess) => subprocess,
            Err(_e) => {
                error!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell.", launch_command);
                return Err(DispatcherError::SubprocessError);
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
    async fn monitor_subprocess(&mut self) -> DispatcherResult<()> {
        loop {
            match self.subprocess.poll() {
                Some(exit_status) => {
                    match exit_status {
                        ExitStatus::Exited(code) => {
                            error!("Backend exited unexpectedly with exit status {}.", code);
                        },
                        ExitStatus::Signaled(signal) => {
                            error!("Backend exited unexpectedly because of signal {}.", signal);
                        },
                        _ => {
                            error!("Backend exited unexpectedly.");
                        }
                    }
                    return Err(DispatcherError::SubprocessError)
                },
                None => {
                    sleep(self.polling_time).await; // Is magic number.
                }
            }
        }
    }
}
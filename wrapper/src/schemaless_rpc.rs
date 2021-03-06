use std::{convert::TryInto, path::PathBuf};

use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde_bytes::{ByteBuf, Bytes};
use serde_repr::Serialize_repr;

use subprocess::Popen;
use subprocess::PopenConfig;

use crate::{rpc::Fmi2CommandRPC, Fmi2Status};

lazy_static! {
    static ref ZMQ_CONTEXT: zmq::Context = zmq::Context::new();
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ZMQConfig {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
    pub macos: Vec<String>,
    pub serialization_format: SerializationFormat,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum SerializationFormat {
    Pickle,
    Protobuf,
    Json,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct HandshakeInfo {
    pub serialization_format: SerializationFormat,
    pub command_endpoint: String,
}

// ---------------------------- Binding ---------------------------

pub trait BindToRandom {
    /// Quality of life function inspired by
    /// https://pyzmq.readthedocs.io/en/latest/api/zmq.html?highlight=bind_random#zmq.Socket.bind_to_random_port
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, String>;
}

impl BindToRandom for zmq::Socket {
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, String> {
        let connection_str = format!("tcp://{}:*", addr);
        self.bind(&connection_str).unwrap();

        let endpoint = self.get_last_endpoint().unwrap().unwrap();
        let port: &str = endpoint.split(":").collect::<Vec<&str>>()[2];
        let port: i32 = port.parse().unwrap();
        return Ok(port);
    }
}

// // -------------------- JSON Traits ------------------------------------
pub trait JsonSender<T> {
    fn send_as_json(&self, value: T) -> Result<(), zmq::Error>;
}

pub trait JsonReceiver<T> {
    fn recv_from_json(&self) -> Result<T, zmq::Error>;
}

impl<T> JsonSender<T> for zmq::Socket
where
    T: serde::ser::Serialize,
{
    fn send_as_json(&self, value: T) -> Result<(), zmq::Error> {
        let pickle = serde_pickle::to_vec(&value, true).expect("unable to convert object to json");
        self.send(&pickle, 0)
    }
}

impl<'a, T> JsonReceiver<T> for zmq::Socket
where
    T: serde::de::DeserializeOwned,
{
    fn recv_from_json(&self) -> Result<T, zmq::Error> {
        let bytes = self
            .recv_string(0)?
            .expect("message appears does not appear to use valid UTF-8 encoding");

        let value: T = serde_json::from_str(&bytes).expect("unable to deserialize json object");
        std::result::Result::Ok(value)
    }
}

#[repr(i32)]
#[derive(Serialize_repr)]
enum Fmi2SchemalessCommandId {
    // ----- common functions ------
    // GetTypesPlatform <- implemented by wrapper
    // GetVersion <- implemented by wrapper
    SetDebugLogging = 0,
    // Instantiate <- implemented by wrapper
    SetupExperiement = 1,
    FreeInstance = 2,
    EnterInitializationMode = 3,
    ExitInitializationMode = 4,
    Terminate = 5,
    Reset = 6,
    SetXXX = 7,
    GetXXX = 8,
    Serialize = 9,
    Deserialize = 10,
    GetDirectionalDerivative = 11,
    // model-exchange (not implemented)
    // ----- cosim ------
    SetRealInputDerivatives = 12,
    GetRealOutputDerivatives = 13,
    DoStep = 14,
    CancelStep = 15,
    GetXXXStatus = 16,
}
/// Perform remote procedure calls through using zmq and Python's pickle serialization format.
/// Serialization is built on the serde's library, for info see:
/// - serde: https://serde.rs/
/// - serde_pickle: https://docs.rs/serde-pickle/
/// - cpython docs: https://docs.python.org/3/library/pickle.html
pub struct ZMQSchemalessRPC {
    socket: zmq::Socket,
    process: Popen,
    serialization: SerializationFormat,
}

impl ZMQSchemalessRPC {
    pub fn new(
        config: &ZMQConfig,
        resources_dir: &PathBuf,
    ) -> Result<Box<dyn Fmi2CommandRPC>, String> {
        let handshake_socket = ZMQ_CONTEXT
            .socket(zmq::PULL)
            .expect("Unable to create handshake");
        let command_socket = ZMQ_CONTEXT.socket(zmq::REQ).unwrap();
        let handshake_port = handshake_socket.bind_to_random_port("*").unwrap();

        // to start the slave-process the os-specific launch command is read from the launch.toml file
        // in addition to the manually specified arguments, the endpoint of the wrappers handshake socket is appended to the args
        // specifically the argument appended is  "--handshake-endpoint tcp://..."
        let mut command = match std::env::consts::OS {
            "windows" => config.windows.clone(),
            "macos" => config.macos.clone(),
            _ => config.linux.clone(),
        };

        command.push("--handshake-endpoint".to_string());
        let endpoint = format!("tcp://localhost:{:?}", handshake_port);
        command.push(endpoint.to_string());

        let popen = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect(&format!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", command));

        let handshake_info: HandshakeInfo = handshake_socket
            .recv_from_json()
            .expect("Failed to read and parse handshake information sent by slave");

        command_socket
            .connect(&handshake_info.command_endpoint)
            .expect(&format!(
            "unable to establish a connection to the slave's command socket with endpoint '{:?}'",
            handshake_info.command_endpoint
        ));

        // Choose rpc implementation based on handshake from slave
        let rpc = Box::new(Self {
            socket: command_socket,
            process: popen,
            serialization: config.serialization_format,
        });

        Ok(rpc)
    }

    fn send_and_recv<T, V>(&self, value: T) -> Result<V, zmq::Error>
    where
        T: serde::ser::Serialize,
        V: DeserializeOwned,
    {
        let pickle = serde_pickle::to_vec(&value, true).expect("unable to pickle object");
        self.socket.send(&pickle, 0)?;
        let bytes = self
            .socket
            .recv_bytes(0)
            .expect("failed receiving bytes from slave");
        let res: V = serde_pickle::from_slice(&bytes).expect(
            "Received bytes from slave but was unable to convert to Rust types from pickle",
        );

        Ok(res)
    }
}

impl Fmi2CommandRPC for ZMQSchemalessRPC {
    fn fmi2DoStep(&mut self, current_time: f64, step_size: f64, no_step_prior: bool) -> Fmi2Status {
        self.send_and_recv::<_, i32>((
            Fmi2SchemalessCommandId::DoStep,
            current_time,
            step_size,
            no_step_prior,
        ))
        .unwrap()
        .try_into()
        .unwrap()
    }

    fn fmi2CancelStep(&mut self) -> Fmi2Status {
        todo!()
    }

    fn fmi2SetDebugLogging(&mut self, categories: &[&str], logging_on: bool) -> Fmi2Status {
        self.send_and_recv::<_, i32>((
            Fmi2SchemalessCommandId::SetDebugLogging,
            categories,
            logging_on,
        ))
        .unwrap()
        .try_into()
        .unwrap()
    }

    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Fmi2Status {
        self.send_and_recv::<_, i32>((
            Fmi2SchemalessCommandId::SetupExperiement,
            start_time,
            stop_time,
            tolerance,
        ))
        .unwrap()
        .try_into()
        .unwrap()
    }

    fn fmi2EnterInitializationMode(&mut self) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::EnterInitializationMode,))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2ExitInitializationMode(&mut self) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::ExitInitializationMode,))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2Terminate(&mut self) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::Terminate,))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2Reset(&mut self) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::Reset,))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2SetReal(&mut self, references: &[u32], values: &[f64]) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2SetInteger(&mut self, references: &[u32], values: &[i32]) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2SetBoolean(&mut self, references: &[u32], values: &[bool]) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2SetString(&mut self, references: &[u32], values: &[&str]) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2GetReal(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<f64>>) {
        let (status, values) = self
            .send_and_recv::<_, (i32, _)>((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap();

        (status.try_into().unwrap(), values)
    }

    fn fmi2GetInteger(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<i32>>) {
        let (status, values) = self
            .send_and_recv::<_, (i32, _)>((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap();

        (status.try_into().unwrap(), values)
    }

    fn fmi2GetBoolean(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<bool>>) {
        let (status, values) = self
            .send_and_recv::<_, (i32, _)>((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap();

        (status.try_into().unwrap(), values)
    }

    fn fmi2GetString(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<String>>) {
        let (status, values) = self
            .send_and_recv::<_, (i32, _)>((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap();

        (status.try_into().unwrap(), values)
    }

    fn serialize(&mut self) -> (Fmi2Status, Option<Vec<u8>>) {
        // wrap in ByteBuf to serialize bytes as bytes rather than sequence
        let (status, bytes) = self
            .send_and_recv::<_, (i32, Option<ByteBuf>)>((Fmi2SchemalessCommandId::Serialize,))
            .unwrap();
        (
            status.try_into().unwrap(),
            bytes.and_then(|bb| Some(bb.to_vec())),
        )
    }

    fn deserialize(&mut self, bytes: &[u8]) -> Fmi2Status {
        self.send_and_recv::<_, i32>((Fmi2SchemalessCommandId::Deserialize, Bytes::new(bytes)))
            .unwrap()
            .try_into()
            .unwrap()
    }

    fn fmi2FreeInstance(&mut self) {
        self.send_and_recv((Fmi2SchemalessCommandId::FreeInstance,))
            .unwrap()
    }
}

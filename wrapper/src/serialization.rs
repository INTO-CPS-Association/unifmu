use anyhow::Error;
use serde::de::DeserializeOwned;

use serde_bytes::Bytes;
use serde_repr::Serialize_repr;
// ----------------------------- Serialization -----------------------------------------------------

// pub trait ObjectSender<T> {
//     fn send_as_object(
//         &self,
//         value: T,
//         serialization: SerializationFormat,
//         timeout: Option<i32>,
//     ) -> Result<(), zmq::Error>;
// }

// impl<T> ObjectSender<T> for zmq::Socket
// where
//     T: serde::ser::Serialize,
// {
//     fn send_as_object(
//         &self,
//         value: T,
//         serialization: SerializationFormat,
//         timeout: Option<i32>,
//     ) -> Result<(), zmq::Error> {
//         let data = match serialization {
//             SerializationFormat::Pickle => {
//                 serde_pickle::to_vec(&value, true).expect("unable to pickle object")
//             }
//             SerializationFormat::Json => {
//                 serde_json::to_vec(&value).expect("unable to convert object to json")
//             }
//         };

//         self.send(&data, 0)
//     }
// }

// ---------------------------- Binding ---------------------------

pub trait BindToRandom {
    /// Quality of life function inspired by
    /// https://pyzmq.readthedocs.io/en/latest/api/zmq.html?highlight=bind_random#zmq.Socket.bind_to_random_port
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, Error>;
}

impl BindToRandom for zmq::Socket {
    fn bind_to_random_port(&self, addr: &str) -> Result<i32, Error> {
        let connection_str = format!("tcp://{}:*", addr);
        self.bind(&connection_str).unwrap();

        let endpoint = self.get_last_endpoint().unwrap().unwrap();
        let port: &str = endpoint.split(":").collect::<Vec<&str>>()[2];
        let port: i32 = port.parse().unwrap();
        return Ok(port);
    }
}

// // --------------------- Pickling Traits --------------------------

// pub trait PickleSender<T> {
//     fn send_as_pickle(&self, value: T) -> Result<(), zmq::Error>;
// }

// pub trait PickleReceiver<T> {
//     fn recv_from_pickle(&self) -> Result<T, zmq::Error>;
// }

// impl<T> PickleSender<T> for zmq::Socket
// where
//     T: serde::ser::Serialize,
// {
//     fn send_as_pickle(&self, value: T) -> Result<(), zmq::Error> {
//         let pickle = serde_pickle::to_vec(&value, true).expect("unable to pickle object");
//         self.send(&pickle, 0)
//     }
// }

// impl<'a, T> PickleReceiver<T> for zmq::Socket
// where
//     T: serde::de::Deserialize<'a>,
// {
//     fn recv_from_pickle(&self) -> Result<T, zmq::Error> {
//         let bytes = self.recv_bytes(0)?;

//         let value: T = serde_pickle::from_slice(&bytes).expect("unable to un-pickle object");
//         std::result::Result::Ok(value)
//     }
// }

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

pub trait Fmi2CommandRPC {
    fn fmi2DoStep(&mut self, current_time: f64, step_size: f64, no_step_prior: bool) -> i32;
    fn fmi2CancelStep(&mut self) -> i32;
    fn fmi2SetDebugLogging(&mut self, categories: Vec<&str>, logging_on: bool) -> i32;
    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> i32;
    fn fmi2EnterInitializationMode(&mut self) -> i32;
    fn fmi2ExitInitializationMode(&mut self) -> i32;
    fn fmi2Terminate(&mut self) -> i32;
    fn fmi2Reset(&mut self) -> i32;

    fn fmi2SetReal(&mut self, references: &[u32], values: &[f64]) -> i32;
    fn fmi2SetInteger(&mut self, references: &[u32], values: &[i32]) -> i32;
    fn fmi2SetBoolean(&mut self, references: &[u32], values: &[bool]) -> i32;
    fn fmi2SetString(&mut self, references: &[u32], values: &[&str]) -> i32;

    // TODO add status to return
    fn fmi2GetReal(&mut self, references: &[u32]) -> (i32, Option<Vec<f64>>);
    fn fmi2GetInteger(&mut self, references: &[u32]) -> (i32, Option<Vec<i32>>);
    fn fmi2GetBoolean(&mut self, references: &[u32]) -> (i32, Option<Vec<bool>>);
    fn fmi2GetString(&mut self, references: &[u32]) -> (i32, Option<Vec<String>>);

    fn serialize(&mut self) -> (i32, Option<Vec<u8>>);
    fn deserialize(&mut self, bytes: &[u8]) -> i32;

    fn fmi2FreeInstance(&mut self);
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
pub struct PickleRPC {
    socket: zmq::Socket,
}

impl PickleRPC {
    pub fn new(socket: zmq::Socket) -> Self {
        Self { socket }
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

impl Fmi2CommandRPC for PickleRPC {
    fn fmi2DoStep(&mut self, current_time: f64, step_size: f64, no_step_prior: bool) -> i32 {
        self.send_and_recv((
            Fmi2SchemalessCommandId::DoStep,
            current_time,
            step_size,
            no_step_prior,
        ))
        .unwrap()
    }

    fn fmi2CancelStep(&mut self) -> i32 {
        todo!()
    }

    fn fmi2SetDebugLogging(&mut self, categories: Vec<&str>, logging_on: bool) -> i32 {
        self.send_and_recv((
            Fmi2SchemalessCommandId::SetDebugLogging,
            categories,
            logging_on,
        ))
        .unwrap()
    }

    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> i32 {
        self.send_and_recv((
            Fmi2SchemalessCommandId::SetupExperiement,
            start_time,
            stop_time,
            tolerance,
        ))
        .unwrap()
    }

    fn fmi2EnterInitializationMode(&mut self) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::EnterInitializationMode,))
            .unwrap()
    }

    fn fmi2ExitInitializationMode(&mut self) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::ExitInitializationMode,))
            .unwrap()
    }

    fn fmi2Terminate(&mut self) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::Terminate,))
            .unwrap()
    }

    fn fmi2Reset(&mut self) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::Reset,))
            .unwrap()
    }

    fn fmi2SetReal(&mut self, references: &[u32], values: &[f64]) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
    }

    fn fmi2SetInteger(&mut self, references: &[u32], values: &[i32]) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
    }

    fn fmi2SetBoolean(&mut self, references: &[u32], values: &[bool]) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
    }

    fn fmi2SetString(&mut self, references: &[u32], values: &[&str]) -> i32 {
        self.send_and_recv((Fmi2SchemalessCommandId::SetXXX, references, values))
            .unwrap()
    }

    fn fmi2GetReal(&mut self, references: &[u32]) -> (i32, Option<Vec<f64>>) {
        self.send_and_recv((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap()
    }

    fn fmi2GetInteger(&mut self, references: &[u32]) -> (i32, Option<Vec<i32>>) {
        self.send_and_recv((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap()
    }

    fn fmi2GetBoolean(&mut self, references: &[u32]) -> (i32, Option<Vec<bool>>) {
        self.send_and_recv((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap()
    }

    fn fmi2GetString(&mut self, references: &[u32]) -> (i32, Option<Vec<String>>) {
        self.send_and_recv((Fmi2SchemalessCommandId::GetXXX, references))
            .unwrap()
    }

    fn serialize(&mut self) -> (i32, Option<Vec<u8>>) {
        self.send_and_recv((Fmi2SchemalessCommandId::Serialize,))
            .unwrap()
    }

    fn deserialize(&mut self, bytes: &[u8]) -> i32 {
        let bytes = Bytes::new(bytes);
        self.send_and_recv((Fmi2SchemalessCommandId::Deserialize, bytes))
            .unwrap()
    }

    fn fmi2FreeInstance(&mut self) {
        self.send_and_recv((Fmi2SchemalessCommandId::FreeInstance,))
            .unwrap()
    }
}

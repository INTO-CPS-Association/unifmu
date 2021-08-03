use prost::Message;
use serde::{Deserialize, Serialize};

mod fmi2_proto;
pub mod master;
pub mod protobuf_compatability;
pub mod slave;

// ################################# SERIALIZATION #########################################

#[derive(Clone, Copy)]
pub enum SerializationFormat {
    Pickle,
    Json,
    Protobuf,
}

// ################################# SOCKET ##############################

/// A sender that implements framing.
/// Examples include :
/// * Message queues such as zmq, for which framing is a natural part of the abstraction.
/// * TCP socket paired with a framing protocol.
pub trait FramedSocket {
    fn recv_into(&self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn send(&self, buf: &[u8]);
    fn recv_bytes(&self) -> Vec<u8>;
}

impl FramedSocket for zmq::Socket {
    fn recv_into(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = zmq::Socket::recv_into(&self, buf, 0).unwrap();
        assert!(
            buf.len() >= size,
            "number of bytes received '{:?}' by 'recv_into' exceeds the size '{:?}' of the provided buffer",size,buf.len()
        );

        Ok(size)
    }

    fn send(&self, buf: &[u8]) {
        zmq::Socket::send(self, buf, 0).unwrap()
    }

    fn recv_bytes(&self) -> Vec<u8> {
        self.recv_bytes(0).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Fmi2Command {
    Fmi2DoStep {
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    },

    Fmi2EnterInitializationMode,
    Fmi2ExitInitializationMode,
    Fmi2FreeInstance,
    Fmi2SetupExperiment {
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    },
    Fmi2SetReal {
        references: Vec<u32>,
        values: Vec<f64>,
    },
    Fmi2SetInteger {
        references: Vec<u32>,
        values: Vec<i32>,
    },
    Fmi2SetBoolean {
        references: Vec<u32>,
        values: Vec<bool>,
    },
    Fmi2SetString {
        references: Vec<u32>,
        values: Vec<String>,
    },
    Fmi2GetReal {
        references: Vec<u32>,
    },
    Fmi2GetInteger {
        references: Vec<u32>,
    },
    Fmi2GetBoolean {
        references: Vec<u32>,
    },
    Fmi2GetString {
        references: Vec<u32>,
    },

    Fmi2Reset,
    Fmi2Terminate,
    Fmi2CancelStep,

    Fmi2ExtSerializeSlave,
    Fmi2ExtDeserializeSlave {
        state: Vec<u8>,
    },
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Fmi2Return {
    Fmi2StatusReturn { status: i32 },
    Fmi2GetRealReturn { status: i32, values: Vec<f64> },
    Fmi2GetIntegerReturn { status: i32, values: Vec<i32> },
    Fmi2GetBooleanReturn { status: i32, values: Vec<bool> },
    Fmi2GetStringReturn { status: i32, values: Vec<String> },
    Fmi2ExtSerializeSlaveReturn { status: i32, state: Vec<u8> },
    Fmi2ExtHandshake,
}

pub trait FormattedSerialize {
    fn serialize_as(&self, format: &SerializationFormat) -> Vec<u8>;
}

impl Fmi2Return {
    pub fn deserialize_from(buf: &[u8], format: &SerializationFormat) -> Fmi2Return {
        match format {
            SerializationFormat::Pickle => serde_pickle::from_slice(buf).unwrap(),
            SerializationFormat::Json => serde_json::from_slice(buf).unwrap(),
            SerializationFormat::Protobuf => fmi2_proto::Fmi2Return::decode(buf).unwrap().into(),
        }
    }

    pub fn serialize_as(&self, format: &SerializationFormat) -> Vec<u8> {
        match format {
            SerializationFormat::Pickle => serde_pickle::to_vec(self, true).unwrap(),
            SerializationFormat::Json => serde_json::to_vec(self).unwrap(),
            SerializationFormat::Protobuf => {
                let ret = fmi2_proto::Fmi2Return::from(self.to_owned());
                ret.encode_to_vec()
            }
        }
    }
}

impl Fmi2Command {
    pub fn serialize_as(&self, format: &SerializationFormat) -> Vec<u8> {
        match format {
            SerializationFormat::Pickle => serde_pickle::to_vec(self, true).unwrap(),
            SerializationFormat::Json => serde_json::to_vec(self).unwrap(),
            SerializationFormat::Protobuf => {
                let command = Some(fmi2_proto::fmi2_command::Command::from(self.to_owned()));
                fmi2_proto::Fmi2Command { command }.encode_to_vec()
            }
        }
    }

    pub fn deserialize_from(buf: &[u8], format: &SerializationFormat) -> Fmi2Command {
        match format {
            SerializationFormat::Pickle => serde_pickle::from_slice(buf).unwrap(),
            SerializationFormat::Json => serde_json::from_slice(buf).unwrap(),
            SerializationFormat::Protobuf => fmi2_proto::Fmi2Command::decode(buf).unwrap().into(),
        }
    }
}

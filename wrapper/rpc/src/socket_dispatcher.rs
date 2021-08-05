use std::convert::TryFrom;

use crate::{fmi2_proto, Fmi2Command, Fmi2CommandDispatcher, Fmi2Return};
use common::Fmi2Status;
use prost::Message;
use serde::Deserialize;

// ################################# SERIALIZATION #########################################

#[derive(Clone, Copy, Deserialize)]
pub enum SerializationFormat {
    #[serde(alias = "pickle")]
    Pickle,
    #[serde(alias = "json")]
    Json,
    #[serde(alias = "protobuf")]
    Protobuf,
}

// ################################# SOCKET ##############################

/// An object that dispatches FMI2 calls over a socket to an FMU instance.
///
/// The socket must implement framing, i.e. the transmitted messages must encode the length of the message.
///
/// Two choices for these would be:
/// * A message queue such as zmq, where the framing is built into the abstraction.
/// * A TCP socket coupled with a framing protocol.
pub struct Fmi2SocketDispatcher<T: FramedSocket> {
    format: SerializationFormat,
    socket: T,
}

// type mytype = Box<dyn FnOnce() -> Box<dyn Fmi2CommandDispatcher>>;

// pub fn from_new_zmq_socket_dynamic(
//     format: SerializationFormat,
// ) -> (String, Box<dyn Fmi2CommandDispatcher>) {
//     let ctx = zmq::Context::new();
//     let socket = ctx.socket(zmq::SocketType::REP).unwrap();
//     socket.bind("tcp://*:0").unwrap();
//     let endpoint = socket.get_last_endpoint().unwrap().unwrap();

//     let func = move || {
//         socket.recv_bytes(0).unwrap();
//         let a: Box<dyn Fmi2CommandDispatcher> = Box::new(
//             Fmi2SocketDispatcher::<zmq::Socket>::from_connected_socket(format, socket),
//         );
//         a
//     };

//     (endpoint, Box::new(func))
// }

pub fn new_boxed_socket_dispatcher(
    format: SerializationFormat,
) -> (String, Box<dyn Fmi2CommandDispatcher>) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SocketType::REP).unwrap();
    socket.bind("tcp://*:0").unwrap();
    let endpoint = socket.get_last_endpoint().unwrap().unwrap();

    let dispatcher = Fmi2SocketDispatcher { format, socket };

    (endpoint, Box::new(dispatcher))
}

#[allow(non_snake_case)]
impl<T: FramedSocket> Fmi2SocketDispatcher<T> {
    pub fn new(format: SerializationFormat) -> (String, Fmi2SocketDispatcher<zmq::Socket>) {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::SocketType::REP).unwrap();
        socket.bind("tcp://*:0").unwrap();
        let endpoint = socket.get_last_endpoint().unwrap().unwrap();

        let dispatcher = Fmi2SocketDispatcher { format, socket };

        (endpoint, dispatcher)
    }

    /// Creates a client from a socket that has already been connected to a rpc server.
    pub fn from_connected_socket(
        format: SerializationFormat,
        command_socket: T,
    ) -> Fmi2SocketDispatcher<T> {
        Self {
            format: format,
            socket: command_socket,
        }
    }
}

impl<T: FramedSocket> Fmi2CommandDispatcher for Fmi2SocketDispatcher<T> {
    fn invoke_command(&mut self, command: &Fmi2Command) -> Fmi2Return {
        println!("sending command: {:?}", command);
        let buf = command.serialize_as(&self.format);
        self.socket.send(&buf);
        let res = self.socket.recv_bytes();
        Fmi2Return::deserialize_from(&res, &self.format)
    }

    fn await_handshake(&mut self) {
        let buf = self.socket.recv_bytes();
        let ret = Fmi2Return::deserialize_from(&buf, &self.format);
        match ret {
            Fmi2Return::Fmi2ExtHandshake => (),
            _ => panic!("unexpected message received"),
        }
    }
}

impl Into<Fmi2Return> for fmi2_proto::Fmi2StatusReturn {
    fn into(self) -> Fmi2Return {
        Fmi2Return::Fmi2StatusReturn {
            status: Fmi2Status::try_from(self.status).unwrap(),
        }
    }
}

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

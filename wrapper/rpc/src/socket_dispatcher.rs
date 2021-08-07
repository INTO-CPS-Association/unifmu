use std::convert::TryFrom;

use crate::{Fmi2CommandDispatcher, Fmi2CommandDispatcherError};

use crate::fmi2_proto::fmi2_command::Command as c_enum;
use crate::fmi2_proto::{
    self, Fmi2DoStep, Fmi2EnterInitializationMode, Fmi2ExitInitializationMode,
    Fmi2ExtSerializeSlaveReturn, Fmi2GetReal, Fmi2GetRealReturn, Fmi2SetupExperiment,
};
use crate::fmi2_proto::{Fmi2Command as c_obj, Fmi2ExtSerializeSlave};

use common::Fmi2Status;
use prost::Message;
use serde::Deserialize;

// ################################# SERIALIZATION #########################################

#[derive(Clone, Copy, Deserialize)]
pub enum SerializationFormat {
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

pub fn new_boxed_socket_dispatcher(
    format: SerializationFormat,
) -> (String, Box<dyn Fmi2CommandDispatcher>) {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SocketType::REP).unwrap();
    socket.bind("tcp://*:0").unwrap();
    let endpoint = socket.get_last_endpoint().unwrap().unwrap();

    let command = fmi2_proto::Fmi2Command {
        ..Default::default()
    };

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

    pub fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> Result<R, Fmi2CommandDispatcherError> {
        let bytes_send = msg.encode_to_vec();
        self.socket.send(&bytes_send);
        let bytes_recv = self.socket.recv_bytes();

        match R::decode(bytes_recv.as_ref()) {
            Ok(result) => Ok(result),
            Err(e) => Err(Fmi2CommandDispatcherError::DecodeError(e)),
        }
    }
}

impl From<fmi2_proto::Fmi2StatusReturn> for Fmi2Status {
    fn from(src: fmi2_proto::Fmi2StatusReturn) -> Self {
        match src.status() {
            fmi2_proto::Fmi2Status::Ok => Self::Fmi2OK,
            fmi2_proto::Fmi2Status::Warning => Self::Fmi2Warning,
            fmi2_proto::Fmi2Status::Discard => Self::Fmi2Discard,
            fmi2_proto::Fmi2Status::Error => Self::Fmi2Error,
            fmi2_proto::Fmi2Status::Fatal => Self::Fmi2Fatal,
            fmi2_proto::Fmi2Status::Pending => Self::Fmi2Pending,
        }
    }
}

impl<T: FramedSocket> Fmi2CommandDispatcher for Fmi2SocketDispatcher<T> {
    fn await_handshake(&mut self) -> Result<(), Fmi2CommandDispatcherError> {
        let buf = self.socket.recv_bytes();
        fmi2_proto::Fmi2ExtHandshakeReturn::decode(buf.as_ref())
            .map(|_| ())
            .map_err(|e| Fmi2CommandDispatcherError::DecodeError(e))
    }

    fn fmi2EnterInitializationMode(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2ExitInitializationMode(
                Fmi2ExitInitializationMode {},
            )),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2ExitInitializationMode(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2EnterInitializationMode(
                Fmi2EnterInitializationMode {},
            )),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2DoStep(
        &mut self,
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2DoStep(Fmi2DoStep {
                current_time,
                step_size,
                no_step_prior,
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetupExperiment(Fmi2SetupExperiment {
                start_time,
                stop_time,
                tolerance,
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2ExtSerializeSlave(
        &mut self,
    ) -> Result<(Fmi2Status, Vec<u8>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2ExtSerializeSlave(Fmi2ExtSerializeSlave {})),
        };
        self.send_and_recv::<_, Fmi2ExtSerializeSlaveReturn>(&cmd)
            .map(|res| (Fmi2Status::try_from(res.status).unwrap(), res.state))
    }

    fn fmi2ExtDeserializeSlave(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2CancelStep(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2Terminate(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2Reset(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2FreeInstance(&mut self) -> Result<(), Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2SetReal(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<common::Fmi2Status, Fmi2CommandDispatcherError> {
        todo!();
    }

    fn fmi2SetInteger(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<common::Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<common::Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<common::Fmi2Status, Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2GetReal(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetReal(Fmi2GetReal {
                references: references.to_owned(),
            })),
        };
        self.send_and_recv::<_, Fmi2GetRealReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi2Status::try_from(result.status).unwrap(), values)
            })
    }

    fn fmi2GetInteger(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<i32>>), Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2GetBoolean(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), Fmi2CommandDispatcherError> {
        todo!()
    }

    fn fmi2GetString(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), Fmi2CommandDispatcherError> {
        todo!()
    }
}

/// A sender that implements framing.
/// Examples include :
/// * Message queues such as zmq, for which framing is a natural part of the abstraction.
/// * TCP socket paired with a framing protocol.
pub trait FramedSocket {
    fn send(&self, buf: &[u8]);
    fn recv_bytes(&self) -> Vec<u8>;
}

impl FramedSocket for zmq::Socket {
    fn send(&self, buf: &[u8]) {
        zmq::Socket::send(self, buf, 0).unwrap()
    }

    fn recv_bytes(&self) -> Vec<u8> {
        self.recv_bytes(0).unwrap()
    }
}

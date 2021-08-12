use std::convert::TryFrom;

use crate::fmi2_proto::fmi2_command::Command as c_enum;
use crate::fmi2_proto::{
    self, Fmi2CancelStep, Fmi2DoStep, Fmi2EnterInitializationMode, Fmi2ExitInitializationMode,
    Fmi2ExtDeserializeSlave, Fmi2ExtSerializeSlaveReturn, Fmi2FreeInstance, Fmi2GetBoolean,
    Fmi2GetBooleanReturn, Fmi2GetDirectionalDerivatives, Fmi2GetInteger, Fmi2GetIntegerReturn,
    Fmi2GetReal, Fmi2GetRealOutputDerivatives, Fmi2GetRealReturn, Fmi2GetString,
    Fmi2GetStringReturn, Fmi2Reset, Fmi2SetBoolean, Fmi2SetDebugLogging, Fmi2SetInteger,
    Fmi2SetReal, Fmi2SetRealInputDerivatives, Fmi2SetString, Fmi2SetupExperiment, Fmi2StatusReturn,
    Fmi2Terminate,
};
use crate::fmi2_proto::{Fmi2Command as c_obj, Fmi2ExtSerializeSlave};
use crate::{Fmi2CommandDispatcher, Fmi2CommandDispatcherError};

use crate::Fmi2Status;
use bytes::Bytes;
use prost::Message;
use tokio::runtime::Runtime;
use zeromq::{RepSocket, Socket, SocketRecv, SocketSend};

// ################################# SERIALIZATION #########################################

// #[derive(Clone, Copy, Deserialize)]
// pub enum SerializationFormat {
//     #[serde(alias = "protobuf")]
//     Protobuf,
// }

// ################################# SOCKET ##############################

/// An object that dispatches FMI2 calls over a socket to an FMU instance.
///
/// The socket must implement framing, i.e. the transmitted messages must encode the length of the message.
///
/// Two choices for these would be:
/// * A message queue such as zmq, where the framing is built into the abstraction.
/// * A TCP socket coupled with a framing protocol.
pub struct Fmi2SocketDispatcher {
    socket: zeromq::RepSocket,
    rt: Runtime,
    pub endpoint: String,
}

// pub fn new_boxed_socket_dispatcher_zeromq() -> (String, Box<dyn Fmi2CommandDispatcher>) {
//     let mut socket = RepSocket::new();

//     let rt = tokio::runtime::Builder::new_current_thread()
//         .enable_io()
//         .build()
//         .unwrap();

//     let endpoint = rt.block_on(socket.bind("tcp://127.0.0.1:0")).unwrap();

//     let dispatcher = Fmi2SocketDispatcher {
//         socket,
//         rt,
//         endpoint: endpoint.to_string(),
//     };

//     (endpoint.to_string(), Box::new(dispatcher))
// }

#[allow(non_snake_case)]
impl Fmi2SocketDispatcher {
    pub fn new(endpoint: &str) -> Self {
        let mut socket = RepSocket::new();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        let endpoint = rt.block_on(socket.bind(endpoint)).unwrap();

        Self {
            socket,
            rt,
            endpoint: endpoint.to_string(),
        }
    }

    pub fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> Result<R, Fmi2CommandDispatcherError> {
        let bytes_send: Bytes = msg.encode_to_vec().into();

        self.rt
            .block_on(self.socket.send(bytes_send.into()))
            .unwrap();

        let bytes_recv = self.rt.block_on(self.socket.recv()).unwrap();

        let bytes_recv: Bytes = bytes_recv.get(0).unwrap().to_owned();

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

impl Fmi2CommandDispatcher for Fmi2SocketDispatcher {
    fn await_handshake(&mut self) -> Result<(), Fmi2CommandDispatcherError> {
        let buf = self.rt.block_on(self.socket.recv()).unwrap();

        let buf: Bytes = buf.get(0).unwrap().to_owned();

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

    fn fmi2ExtDeserializeSlave(
        &mut self,
        state: &[u8],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2ExtDeserializeSlave(Fmi2ExtDeserializeSlave {
                state: state.to_owned(),
            })),
        };

        self.send_and_recv::<_, Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2CancelStep(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2CancelStep(Fmi2CancelStep {})),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2Terminate(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2Terminate(Fmi2Terminate {})),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2Reset(&mut self) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2Reset(Fmi2Reset {})),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    fn fmi2FreeInstance(&mut self) -> Result<(), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2FreeInstance(Fmi2FreeInstance {})),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2FreeInstanceReturn>(&cmd)
            .map(|_| ())
    }

    fn fmi2SetReal(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetReal(Fmi2SetReal {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    fn fmi2SetInteger(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetInteger(Fmi2SetInteger {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    fn fmi2SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetBoolean(Fmi2SetBoolean {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    fn fmi2SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetString(Fmi2SetString {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
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
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetInteger(Fmi2GetInteger {
                references: references.to_owned(),
            })),
        };
        self.send_and_recv::<_, Fmi2GetIntegerReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi2Status::try_from(result.status).unwrap(), values)
            })
    }

    fn fmi2GetBoolean(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetBoolean(Fmi2GetBoolean {
                references: references.to_owned(),
            })),
        };
        self.send_and_recv::<_, Fmi2GetBooleanReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi2Status::try_from(result.status).unwrap(), values)
            })
    }

    fn fmi2GetString(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetString(Fmi2GetString {
                references: references.to_owned(),
            })),
        };
        self.send_and_recv::<_, Fmi2GetStringReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi2Status::try_from(result.status).unwrap(), values)
            })
    }

    fn fmi2SetDebugLogging(
        &mut self,
        categories: &[String],
        logging_on: bool,
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetDebugLogging(Fmi2SetDebugLogging {
                categories: categories.to_vec(),
                logging_on,
            })),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    fn fmi2GetRealOutputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetRealOutputDerivatives(
                Fmi2GetRealOutputDerivatives {
                    references: references.to_owned(),
                    orders: orders.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2GetRealOutputDerivativesReturn>(&cmd)
            .map(|s| {
                let status = Fmi2Status::try_from(s.status).unwrap();
                let values = match s.values.is_empty() {
                    true => Some(s.values),
                    false => None,
                };
                (status, values)
            })
    }

    fn fmi2SetRealInputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
        values: &[f64],
    ) -> Result<Fmi2Status, Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetRealInputDerivatives(
                Fmi2SetRealInputDerivatives {
                    references: references.to_owned(),
                    orders: orders.to_owned(),
                    values: values.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    fn fmi2GetDirectionalDerivative(
        &mut self,
        references_unknown: &[u32],
        references_known: &[u32],
        direction_known: &[f64],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Fmi2CommandDispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetDirectionalDerivatives(
                Fmi2GetDirectionalDerivatives {
                    references_unknown: references_unknown.to_owned(),
                    references_known: references_known.to_owned(),
                    direction_known: direction_known.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi2_proto::Fmi2GetDirectionalDerivativesReturn>(&cmd)
            .map(|s| {
                let status = Fmi2Status::try_from(s.status).unwrap();
                let values = match s.values.is_empty() {
                    true => Some(s.values),
                    false => None,
                };
                (status, values)
            })
    }
}

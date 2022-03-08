use std::convert::TryFrom;

use crate::fmi3_messages::{
    self, Fmi3DeserializeFmuState, Fmi3DoStep, Fmi3EmptyReturn, Fmi3EnterInitializationMode,
    Fmi3ExitInitializationMode, Fmi3FreeInstance, Fmi3GetBoolean, Fmi3GetBooleanReturn,
    Fmi3GetFloat32, Fmi3GetFloat32Return, Fmi3GetFloat64, Fmi3GetFloat64Return, Fmi3GetInt16,
    Fmi3GetInt16Return, Fmi3GetInt32, Fmi3GetInt32Return, Fmi3GetInt64, Fmi3GetInt64Return,
    Fmi3GetInt8, Fmi3GetInt8Return, Fmi3GetString, Fmi3GetStringReturn, Fmi3GetUInt16,
    Fmi3GetUInt16Return, Fmi3GetUInt32, Fmi3GetUInt32Return, Fmi3GetUInt64, Fmi3GetUInt64Return,
    Fmi3GetUInt8, Fmi3GetUInt8Return, Fmi3InstantiateCoSimulation, Fmi3InstantiateModelExchange,
    Fmi3Reset, Fmi3SerializeFmuState, Fmi3SerializeFmuStateReturn, Fmi3StatusReturn, Fmi3Terminate,
};

use crate::fmi3::Fmi3Status;
use crate::fmi3_messages::fmi3_command::Command as c_enum;
use crate::fmi3_messages::Fmi3Command as c_obj;

use bytes::Bytes;
use prost::Message;
use tokio::runtime::Runtime;
use zeromq::{RepSocket, Socket, SocketRecv, SocketSend};

pub struct Fmi3CommandDispatcher {
    socket: zeromq::RepSocket,
    rt: Runtime,
    pub endpoint: String,
}

#[derive(Debug)]
pub enum DispatcherError {
    DecodeError(prost::DecodeError),
    EncodeError,
    SocketError,
    Timeout,
    BackendImplementationError,
}

#[allow(non_snake_case)]
impl Fmi3CommandDispatcher {
    pub fn await_handshake(&mut self) -> Result<(), DispatcherError> {
        self.recv::<Fmi3EmptyReturn>().map(|_| ())
    }

    pub fn Fmi3SerializeFmuState(&mut self) -> Result<(Fmi3Status, Vec<u8>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SerializeFmuState(Fmi3SerializeFmuState {})),
        };
        self.send_and_recv::<_, Fmi3SerializeFmuStateReturn>(&cmd)
            .map(|res| (Fmi3Status::try_from(res.status).unwrap(), res.state))
    }

    pub fn Fmi3DeserializeFmuState(&mut self, state: &[u8]) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3DeserializeFmuState(Fmi3DeserializeFmuState {
                state: state.to_owned(),
            })),
        };

        self.send_and_recv::<_, Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    // ================= FMI3 ======================
    // https://github.com/modelica/fmi-standard/blob/master/headers/fmi3FunctionTypes.h
    pub fn fmi3InstantiateModelExchange(&mut self) -> Result<Fmi3EmptyReturn, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3InstantiateModelExchange(
                Fmi3InstantiateModelExchange {
                    instance_name: todo!(),
                    instantiation_token: todo!(),
                    resource_path: todo!(),
                    visible: todo!(),
                    logging_on: todo!(),
                },
            )),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3EmptyReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3InstantiateCoSimulation(
        &mut self,
        instance_name: String,
        instantiation_token: String,
        resource_path: String,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: Vec<u32>, // instance_enviornment: *const c_void,
                                                   // log_message: *const c_void,
                                                   // intermediate_update: *const c_void
    ) -> Result<Fmi3EmptyReturn, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3InstantiateCoSimulation(
                Fmi3InstantiateCoSimulation {
                    instance_name,
                    instantiation_token,
                    resource_path,
                    visible,
                    logging_on,
                    event_mode_used,
                    early_return_allowed,
                    required_intermediate_variables,
                },
            )),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3EmptyReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3EnterInitializationMode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3EnterInitializationMode(
                Fmi3EnterInitializationMode {
                    tolerance,
                    start_time,
                    stop_time,
                },
            )),
        };
        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3ExitInitializationMode(&mut self) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3ExitInitializationMode(
                Fmi3ExitInitializationMode {},
            )),
        };
        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3DoStep(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3DoStep(Fmi3DoStep {
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point,
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3DoStepReturn>(&cmd)
            .map(|s| s.status().into())
    }

    pub fn fmi3GetFloat32(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<f32>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetFloat32(Fmi3GetFloat32 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetFloat32Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetFloat64(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<f64>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetFloat64(Fmi3GetFloat64 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetFloat64Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetInt8(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<i8>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetInt8(Fmi3GetInt8 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetInt8Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values.iter().map(|v| *v as i8).collect()),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetUInt8(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<u8>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetUInt8(Fmi3GetUInt8 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetUInt8Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values.iter().map(|v| *v as u8).collect()),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetInt16(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<i16>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetInt16(Fmi3GetInt16 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetInt16Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values.iter().map(|v| *v as i16).collect()),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetUInt16(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<u16>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetUInt16(Fmi3GetUInt16 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetUInt16Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values.iter().map(|v| *v as u16).collect()),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetInt32(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<i32>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetInt32(Fmi3GetInt32 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetInt32Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetUInt32(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<u32>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetUInt32(Fmi3GetUInt32 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetUInt32Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetInt64(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<i64>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetInt64(Fmi3GetInt64 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetInt64Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetUInt64(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<u64>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetUInt64(Fmi3GetUInt64 { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetUInt64Return>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetBoolean(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<bool>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetBoolean(Fmi3GetBoolean { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetBooleanReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetString(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<String>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetString(Fmi3GetString { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetStringReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3GetBinary(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<Vec<u8>>>), DispatcherError> {
        todo!()
    }

    pub fn fmi3Terminate(&mut self) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3Terminate(Fmi3Terminate {})),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3Reset(&mut self) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3Reset(Fmi3Reset {})),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi3FreeInstance(&mut self) -> Result<(), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3FreeInstance(Fmi3FreeInstance {})),
        };

        self.send(&cmd)
    }

    // socket
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
    ) -> Result<R, DispatcherError> {
        let bytes_send: Bytes = msg.encode_to_vec().into();

        match self.send(msg) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        self.recv()
    }

    pub fn send<S: Message>(&mut self, msg: &S) -> Result<(), DispatcherError> {
        let bytes_send: Bytes = msg.encode_to_vec().into();

        match self.rt.block_on(self.socket.send(bytes_send.into())) {
            Ok(_) => Ok(()),
            Err(e) => Err(DispatcherError::SocketError),
        }
    }

    pub fn recv<R: Message + Default>(&mut self) -> Result<R, DispatcherError> {
        let buf = self.rt.block_on(self.socket.recv()).unwrap();

        let buf: Bytes = buf.get(0).unwrap().to_owned();

        match R::decode(buf.as_ref()) {
            Ok(msg) => Ok(msg),
            Err(e) => Err(DispatcherError::DecodeError(e)),
        }
    }
}

impl From<fmi3_messages::Fmi3StatusReturn> for Fmi3Status {
    fn from(src: fmi3_messages::Fmi3StatusReturn) -> Self {
        match src.status() {
            fmi3_messages::Fmi3Status::Fmi3Ok => Self::OK,
            fmi3_messages::Fmi3Status::Fmi3Warning => Self::Warning,
            fmi3_messages::Fmi3Status::Fmi3Discard => Self::Discard,
            fmi3_messages::Fmi3Status::Fmi3Error => Self::Error,
            fmi3_messages::Fmi3Status::Fmi3Fatal => Self::Fatal,
        }
    }
}

impl From<fmi3_messages::Fmi3Status> for Fmi3Status {
    fn from(s: fmi3_messages::Fmi3Status) -> Self {
        match s {
            fmi3_messages::Fmi3Status::Fmi3Ok => Self::OK,
            fmi3_messages::Fmi3Status::Fmi3Warning => Self::Warning,
            fmi3_messages::Fmi3Status::Fmi3Discard => Self::Discard,
            fmi3_messages::Fmi3Status::Fmi3Error => Self::Error,
            fmi3_messages::Fmi3Status::Fmi3Fatal => Self::Fatal,
        }
    }
}

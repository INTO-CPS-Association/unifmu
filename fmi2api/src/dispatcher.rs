use std::convert::TryFrom;

use crate::fmi_proto::fmi2_command::Command as c_enum;
use crate::fmi_proto::{
    self, Fmi2CancelStep, Fmi2DoStep, Fmi2EnterInitializationMode, Fmi2ExitInitializationMode,
    Fmi2FreeInstance, Fmi2GetBoolean, Fmi2GetBooleanReturn, Fmi2GetDirectionalDerivatives,
    Fmi2GetInteger, Fmi2GetIntegerReturn, Fmi2GetReal, Fmi2GetRealOutputDerivatives,
    Fmi2GetRealReturn, Fmi2GetString, Fmi2GetStringReturn, Fmi2Reset, Fmi2SetBoolean,
    Fmi2SetDebugLogging, Fmi2SetInteger, Fmi2SetReal, Fmi2SetRealInputDerivatives, Fmi2SetString,
    Fmi2SetupExperiment, Fmi2StatusReturn, Fmi2Terminate, UnifmuDeserialize,
    UnifmuFmi2SerializeReturn, UnifmuHandshakeReturn,
};

use crate::fmi2::{Fmi2Status, Fmi2Type};
use crate::fmi3::Fmi3Status;
use crate::fmi_proto::Fmi2Command as c_obj;
use crate::fmi_proto::UnifmuSerialize;
use bytes::Bytes;
use prost::Message;
use tokio::runtime::Runtime;
use zeromq::{RepSocket, Socket, SocketRecv, SocketSend};

pub struct CommandDispatcher {
    socket: zeromq::RepSocket,
    rt: Runtime,
    pub endpoint: String,
}

pub enum DispatcherError {
    DecodeError(prost::DecodeError),
    EncodeError,
    SocketError,
    Timeout,
    BackendImplementationError,
}

impl CommandDispatcher {
    // ================= Common (FMI2+FMI3) ====================

    pub fn await_handshake(&mut self) -> Result<(), DispatcherError> {
        todo!()
    }

    pub fn UnifmuSerialize(&mut self) -> Result<(Fmi2Status, Vec<u8>), DispatcherError> {
        todo!()
    }

    pub fn UnifmuDeserialize(&mut self, state: &[u8]) -> Result<Fmi2Status, DispatcherError> {
        todo!()
    }

    // ================= FMI3 ======================
    // https://github.com/modelica/fmi-standard/blob/master/headers/fmi3FunctionTypes.h
    pub fn fmi3InstanteModelExchange(&mut self) {
        todo!()
    }

    pub fn fmi3InstantiateCoSimulation(
        &mut self,
        instance_name: &str,
        instantiation_token: &str,
        resources_path: &str,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: &[i32], // instance_enviornment: *const c_void,
                                                 // log_message: *const c_void,
                                                 // intermediate_update: *const c_void
    ) {
        todo!()
    }

    pub fn fmi3DoStep(&mut self) -> Result<Fmi3Status, DispatcherError> {
        todo!()
    }

    // ================= FMI2 ======================
    // https://github.com/modelica/fmi-standard/blob/support/v2.0.x/headers/fmi2FunctionTypes.h
    pub fn fmi2Instantiate(
        &mut self,
        instance_name: &str,
        fmu_type: Fmi2Type,
        fmu_guid: &str,
        fmu_resources_location: &str,
        // functions: *const c_void,
        visible: bool,
        logging_on: bool,
    ) -> DispatcherError {
        todo!()
    }

    pub fn fmi2EnterInitializationMode(&mut self) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2ExitInitializationMode(
                Fmi2ExitInitializationMode {},
            )),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2ExitInitializationMode(&mut self) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2EnterInitializationMode(
                Fmi2EnterInitializationMode {},
            )),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2DoStep(
        &mut self,
        current_time: f64,
        step_size: f64,
        no_step_prior: bool,
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2DoStep(Fmi2DoStep {
                current_time,
                step_size,
                no_step_prior,
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetupExperiment(Fmi2SetupExperiment {
                start_time,
                stop_time,
                tolerance,
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn unifmuSerialize(&mut self) -> Result<(Fmi2Status, Vec<u8>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::UnifmuSerialize(UnifmuSerialize {})),
        };
        self.send_and_recv::<_, UnifmuFmi2SerializeReturn>(&cmd)
            .map(|res| (Fmi2Status::try_from(res.status).unwrap(), res.state))
    }

    pub fn fmi2ExtDeserializeSlave(&mut self, state: &[u8]) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::UnifmuDeserialize(UnifmuDeserialize {
                state: state.to_owned(),
            })),
        };

        self.send_and_recv::<_, Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2CancelStep(&mut self) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2CancelStep(Fmi2CancelStep {})),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2Terminate(&mut self) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2Terminate(Fmi2Terminate {})),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2Reset(&mut self) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2Reset(Fmi2Reset {})),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| s.into())
    }

    pub fn fmi2FreeInstance(&mut self) -> Result<(), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2FreeInstance(Fmi2FreeInstance {})),
        };

        self.send(&cmd)
    }

    pub fn fmi2SetReal(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetReal(Fmi2SetReal {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2SetInteger(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetInteger(Fmi2SetInteger {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetBoolean(Fmi2SetBoolean {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetString(Fmi2SetString {
                references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2GetReal(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
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

    pub fn fmi2GetInteger(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<i32>>), DispatcherError> {
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

    pub fn fmi2GetBoolean(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), DispatcherError> {
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

    pub fn fmi2GetString(
        &mut self,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), DispatcherError> {
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

    pub fn fmi2SetDebugLogging(
        &mut self,
        categories: &[String],
        logging_on: bool,
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetDebugLogging(Fmi2SetDebugLogging {
                categories: categories.to_vec(),
                logging_on,
            })),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2GetRealOutputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetRealOutputDerivatives(
                Fmi2GetRealOutputDerivatives {
                    references: references.to_owned(),
                    orders: orders.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2GetRealOutputDerivativesReturn>(&cmd)
            .map(|s| {
                let status = Fmi2Status::try_from(s.status).unwrap();
                let values = match s.values.is_empty() {
                    true => Some(s.values),
                    false => None,
                };
                (status, values)
            })
    }

    pub fn fmi2SetRealInputDerivatives(
        &mut self,
        references: &[u32],
        orders: &[i32],
        values: &[f64],
    ) -> Result<Fmi2Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2SetRealInputDerivatives(
                Fmi2SetRealInputDerivatives {
                    references: references.to_owned(),
                    orders: orders.to_owned(),
                    values: values.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2StatusReturn>(&cmd)
            .map(|s| Fmi2Status::try_from(s.status).unwrap())
    }

    pub fn fmi2GetDirectionalDerivative(
        &mut self,
        references_unknown: &[u32],
        references_known: &[u32],
        direction_known: &[f64],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi2GetDirectionalDerivatives(
                Fmi2GetDirectionalDerivatives {
                    references_unknown: references_unknown.to_owned(),
                    references_known: references_known.to_owned(),
                    direction_known: direction_known.to_owned(),
                },
            )),
        };

        self.send_and_recv::<_, fmi_proto::Fmi2GetDirectionalDerivativesReturn>(&cmd)
            .map(|s| {
                let status = Fmi2Status::try_from(s.status).unwrap();
                let values = match s.values.is_empty() {
                    true => Some(s.values),
                    false => None,
                };
                (status, values)
            })
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

impl From<fmi_proto::Fmi2StatusReturn> for Fmi2Status {
    fn from(src: fmi_proto::Fmi2StatusReturn) -> Self {
        match src.status() {
            fmi_proto::Fmi2Status::Fmi2Ok => Self::Fmi2OK,
            fmi_proto::Fmi2Status::Fmi2Warning => Self::Fmi2Warning,
            fmi_proto::Fmi2Status::Fmi2Discard => Self::Fmi2Discard,
            fmi_proto::Fmi2Status::Fmi2Error => Self::Fmi2Error,
            fmi_proto::Fmi2Status::Fmi2Fatal => Self::Fmi2Fatal,
            fmi_proto::Fmi2Status::Fmi2Pending => Self::Fmi2Pending,
        }
    }
}

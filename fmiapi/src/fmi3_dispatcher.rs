use std::{
    ffi::OsString,
    path::Path,
    convert::TryFrom,
};

use crate::fmi3_messages::{
    self, Fmi3DeserializeFmuState, Fmi3DoStep, Fmi3EmptyReturn,
    Fmi3EnterInitializationMode, Fmi3ExitInitializationMode,
    Fmi3FreeInstance, Fmi3GetBoolean, Fmi3GetBooleanReturn, Fmi3GetFloat32,
    Fmi3GetFloat32Return, Fmi3GetFloat64, Fmi3GetFloat64Return, Fmi3GetInt16,
    Fmi3GetInt16Return, Fmi3GetInt32, Fmi3GetInt32Return, Fmi3GetInt64,
    Fmi3GetInt64Return, Fmi3GetInt8, Fmi3GetInt8Return, Fmi3GetString,
    Fmi3GetStringReturn, Fmi3GetUInt16, Fmi3GetUInt16Return, Fmi3GetUInt32,
    Fmi3GetUInt32Return, Fmi3GetUInt64, Fmi3GetUInt64Return, Fmi3GetUInt8,
    Fmi3GetUInt8Return, Fmi3InstantiateCoSimulation,
    Fmi3InstantiateModelExchange, Fmi3Reset, Fmi3SerializeFmuState,
    Fmi3SerializeFmuStateReturn, Fmi3StatusReturn, Fmi3Terminate,
    Fmi3GetClock, Fmi3GetClockReturn, Fmi3GetIntervalDecimal,
    Fmi3GetIntervalDecimalReturn, Fmi3EnterStepMode, Fmi3EnterEventMode,
    Fmi3InstantiateScheduledExecution, Fmi3SetFloat32, Fmi3SetFloat64,
    Fmi3SetInt8, Fmi3SetUInt8, Fmi3SetInt16, Fmi3SetUInt16, Fmi3SetInt32,
    Fmi3SetUInt32, Fmi3SetInt64, Fmi3SetUInt64, Fmi3SetBoolean, Fmi3SetClock,
    Fmi3UpdateDiscreteStates, Fmi3UpdateDiscreteStatesReturn, Fmi3SetString,
    Fmi3SetBinary,
};

use crate::fmi3::Fmi3Status;
use crate::fmi3_messages::fmi3_command::Command as c_enum;
use crate::fmi3_messages::Fmi3Command as c_obj;

use crate::unifmu_handshake::{HandshakeStatus, HandshakeReply,};

use bytes::Bytes;
use prost::Message;
use subprocess::{ExitStatus, Popen, PopenConfig};
use tokio::runtime::Runtime;
use tokio::time::{Duration, sleep};
use tokio::select;
use tracing::{debug, error, info, span, warn, Level};
use zeromq::{Endpoint, RepSocket, Socket, SocketRecv, SocketSend};

struct BackendSocket {
    socket: zeromq::RepSocket
}

impl BackendSocket {
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

    async fn recv<R: Message + Default>(&mut self) -> Result<R, DispatcherError> {
        match self.socket.recv().await {
            Ok(buf) => {
                let buf: Bytes = buf.get(0).unwrap().to_owned();
                match R::decode(buf.as_ref()) {
                    Ok(msg) => Ok(msg),
                    Err(e) => Err(DispatcherError::DecodeError(e)),
                }
            },
            Err(_) => Err(DispatcherError::SocketError)
        }
    }

    async fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> Result<R, DispatcherError> {
        self.send(msg).await?;
        self.recv().await
    }
}

struct BackendSubprocess {
    subprocess: Popen
}

impl BackendSubprocess {
    async fn monitor_subprocess(&mut self) -> Result<(), DispatcherError> {
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
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum DispatcherError {
    DecodeError(prost::DecodeError),
    EnumDecodeError(prost::UnknownEnumValue),
    EncodeError,
    SocketError,
    SubprocessError,
    Timeout,
    BackendImplementationError,
}

pub struct Fmi3CommandDispatcher {
    socket: BackendSocket,
    rt: Runtime,
    pub endpoint: Endpoint,
    pub subprocess: BackendSubprocess,
}

#[allow(non_snake_case)]
impl Fmi3CommandDispatcher {
    pub fn await_handshake(&mut self) -> Result<(), DispatcherError>  {
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
    pub fn fmi3InstantiateModelExchange(
        &mut self,
        instance_name: String,
        instantiation_token: String,
        resource_path: String,
        visible: bool,
        logging_on: bool,
        ) -> Result<Fmi3EmptyReturn, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3InstantiateModelExchange(
                Fmi3InstantiateModelExchange {
                    instance_name,
                    instantiation_token,
                    resource_path,
                    visible,
                    logging_on,
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

    pub fn fmi3InstantiateScheduledExecution(
        &mut self,
        instance_name: String,
        instantiation_token: String,
        resource_path: String,
        visible: bool,
        logging_on: bool,
    ) -> Result<Fmi3EmptyReturn, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3InstantiateScheduledExecution(
                Fmi3InstantiateScheduledExecution {
                    instance_name,
                    instantiation_token,
                    resource_path,
                    visible,
                    logging_on,
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
        let tolerance_defined = tolerance.is_some();
        let stop_time_defined = stop_time.is_some();

        let cmd = c_obj {
            command: Some(c_enum::Fmi3EnterInitializationMode(
                Fmi3EnterInitializationMode {
                    tolerance_defined,
                    tolerance,
                    start_time,
                    stop_time_defined,
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
	
	pub fn fmi3EnterEventMode(&mut self) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3EnterEventMode(
                Fmi3EnterEventMode {},
            )),
        };
        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| s.into())
    }
	
	pub fn fmi3EnterStepMode(&mut self) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3EnterStepMode(
                Fmi3EnterStepMode {},
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
    ) -> Result<(Fmi3Status, bool, bool, bool, f64), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3DoStep(Fmi3DoStep {
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point,
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3DoStepReturn>(&cmd)
            .map(|result| {
                let event_handling_needed = result.event_handling_needed;
                let terminate_simulation = result.terminate_simulation;
                let early_return = result.early_return;
                let last_successful_time = result.last_successful_time;

                (Fmi3Status::try_from(result.status).unwrap(), event_handling_needed,
                 terminate_simulation, early_return, last_successful_time)
            })
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
	
	pub fn fmi3GetIntervalDecimal(
		&mut self,
		value_references: Vec<u32>,
	) -> Result<(Fmi3Status, Option<Vec<f64>>, Option<Vec<i32>>), DispatcherError> {
		let cmd = c_obj {
			command: Some(c_enum::Fmi3GetIntervalDecimal(Fmi3GetIntervalDecimal { value_references })),
		};
		self.send_and_recv::<_, Fmi3GetIntervalDecimalReturn>(&cmd)
			.map(|result| {
				let interval = match result.intervals.is_empty() {
					true => None,
					false => Some(result.intervals),
				};
				let qualifier = match result.qualifiers.is_empty() {
					true => None,
					false => Some(result.qualifiers),
				};
				(Fmi3Status::try_from(result.status).unwrap(), interval, qualifier)
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
        _value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<Vec<u8>>>), DispatcherError> {
        todo!()
    }

    pub fn fmi3GetClock(
        &mut self,
        value_references: Vec<u32>,
    ) -> Result<(Fmi3Status, Option<Vec<bool>>), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3GetClock(Fmi3GetClock { value_references })),
        };
        self.send_and_recv::<_, Fmi3GetClockReturn>(&cmd)
            .map(|result| {
                let values = match result.values.is_empty() {
                    true => None,
                    false => Some(result.values),
                };
                (Fmi3Status::try_from(result.status).unwrap(), values)
            })
    }

    pub fn fmi3UpdateDiscreteStates(
        &mut self
    ) -> Result<(Fmi3Status, bool, bool, bool, bool, bool, f64), DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3UpdateDiscreteStates(Fmi3UpdateDiscreteStates {
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3UpdateDiscreteStatesReturn>(&cmd)
            .map(|result| {
                let discrete_states_need_update = result.discrete_states_need_update;
                let terminate_simulation = result.terminate_simulation;
                let nominals_continuous_states_changed = result.nominals_continuous_states_changed;
                let values_continuous_states_changed = result.values_continuous_states_changed;
                let next_event_time_defined = result.next_event_time_defined;
                let next_event_time = result.next_event_time;

                (Fmi3Status::try_from(result.status).unwrap(), discrete_states_need_update,
                 terminate_simulation, nominals_continuous_states_changed,
                 values_continuous_states_changed, next_event_time_defined, next_event_time)
            })
    }

    pub fn fmi3SetFloat32(
        &mut self,
        references: &[u32],
        values: &[f32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetFloat32(Fmi3SetFloat32 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetFloat64(
        &mut self,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetFloat64(Fmi3SetFloat64 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetInt8(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetInt8(Fmi3SetInt8 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetUInt8(
        &mut self,
        references: &[u32],
        values: &[u32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetUInt8(Fmi3SetUInt8 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetInt16(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetInt16(Fmi3SetInt16 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetUInt16(
        &mut self,
        references: &[u32],
        values: &[u32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetUInt16(Fmi3SetUInt16 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetInt32(
        &mut self,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetInt32(Fmi3SetInt32 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetUInt32(
        &mut self,
        references: &[u32],
        values: &[u32],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetUInt32(Fmi3SetUInt32 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetInt64(
        &mut self,
        references: &[u32],
        values: &[i64],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetInt64(Fmi3SetInt64 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetUInt64(
        &mut self,
        references: &[u32],
        values: &[u64],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetUInt64(Fmi3SetUInt64 {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetBoolean(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetBoolean(Fmi3SetBoolean {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetString(
        &mut self,
        references: &[u32],
        values: &[String],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetString(Fmi3SetString {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }

    pub fn fmi3SetBinary(
        &mut self,
        references: &[u32],
        value_sizes: &[usize],
        binary_values: &[&[u8]],
    ) -> Result<Fmi3Status, DispatcherError> {

        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetBinary(Fmi3SetBinary {
                value_references: references.to_owned(),
                value_sizes:value_sizes.iter().map(|&size| size as u64).collect(),
                values: binary_values.iter().map(|&v| v.to_vec()).collect(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
    }


    pub fn fmi3SetClock(
        &mut self,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi3Status, DispatcherError> {
        let cmd = c_obj {
            command: Some(c_enum::Fmi3SetClock(Fmi3SetClock {
                value_references: references.to_owned(),
                values: values.to_owned(),
            })),
        };

        self.send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
            .map(|s| Fmi3Status::try_from(s.status).unwrap())
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
    pub fn new(
        resource_path: &Path,
        launch_command: &Vec<String>,
        endpoint: &str
    ) -> Result<Self, DispatcherError> {
        let mut socket = RepSocket::new();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let endpoint = rt.block_on(socket.bind(endpoint)).unwrap();        
        let endpoint_string = endpoint.to_string();
        let endpoint_port = endpoint_string
            .split(":")
            .last()
            .expect("There should be a port after the colon")
            .to_owned();
        
        // set environment variables
        info!("Collecting local environment variables.");
        let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
            OsString::from(endpoint_string),
        ));
        env_vars.push((
            OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
            OsString::from(endpoint_port),
        ));

        info!("Spawning backend process.");
        debug!("Launch command: {}", &launch_command[0]);
        debug!("Environment variables: {:#?}", env_vars);
        // spawn process
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
                panic!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", launch_command);
            }
        };

        Ok(
            Self {
                socket: BackendSocket{socket},
                rt,
                endpoint,
                subprocess: BackendSubprocess{subprocess},
            }
        )
    }

    fn send_and_recv<S: Message, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> Result<R, DispatcherError> {
        self.rt.block_on(async {
            select! {
                result = self.socket.send_and_recv::<S, R>(msg) => result,
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
            }
        })
    }

    fn send<S: Message>(&mut self, msg: &S) -> Result<(), DispatcherError> {
        self.rt.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
                result = self.socket.send::<S>(msg) => result,
            }
        })
    }

    fn recv<R: Message + Default>(&mut self) -> Result<R, DispatcherError> {
        self.rt.block_on(async {
            select! {
                Err(e) = self.subprocess.monitor_subprocess() => Err(e),
                result = self.socket.recv::<R>() => result,
            }
        })
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

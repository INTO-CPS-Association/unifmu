use crate::dispatcher::{Dispatch, Dispatcher, DispatcherError};
use crate::fmi3_messages::{
    self,
    Fmi3Command,
    fmi3_command::Command,
    Fmi3Return,
    fmi3_return
};

use std::{
    error::Error,
    ffi::CString,
    fmt::{Debug, Display}
};

use prost::Message;
use tracing::error;

pub struct Fmi3Slave {
    dispatcher: Dispatcher,
    pub last_successful_time: Option<f64>,
    pub string_buffer: Vec<CString>,
}

impl Fmi3Slave {
    pub fn new(dispatcher: Dispatcher) -> Self {
        Self {
            dispatcher,
            last_successful_time: None,
            string_buffer: Vec::new(),
        }
    }

    pub fn dispatch<R: Message>(
        &mut self,
        command: &(impl Message + Debug),
        return_matcher: fn(&mut Self, fmi3_return::ReturnMessage) -> Fmi3SlaveResult<R>
    ) -> Fmi3SlaveResult<R> {
        let return_message = self.dispatcher
            .send_and_recv::<_, Fmi3Return>(command)?
            .return_message
            .ok_or(Fmi3SlaveError::ReturnError(
                "Reply from backend didn't contain nested return message".to_string()
            ))?;

        return_matcher(
            self,
            return_message
        )            
    }

    pub fn empty_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3EmptyReturn> {
        match return_message {
            fmi3_return::ReturnMessage::Empty(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn status_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3StatusReturn> {
        match return_message {
            fmi3_return::ReturnMessage::Status(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn do_step_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3DoStepReturn> {
        match return_message {
            fmi3_return::ReturnMessage::DoStep(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn free_instance_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3FreeInstanceReturn> {
        match return_message {
            fmi3_return::ReturnMessage::FreeInstance(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_float_32_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetFloat32Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetFloat32(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_float_64_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetFloat64Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetFloat64(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_int_8_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetInt8Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetInt8(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_u_int_8_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetUInt8Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetUInt8(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_int_16_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetInt16Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetInt16(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_u_int_16_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetUInt16Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetUInt16(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_int_32_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetInt32Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetInt32(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_u_int_32_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetUInt32Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetUInt32(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_int_64_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetInt64Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetInt64(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_u_int_64_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetUInt64Return> {
        match return_message {
            fmi3_return::ReturnMessage::GetUInt64(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_boolean_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetBooleanReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetBoolean(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_string_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetStringReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetString(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_binary_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetBinaryReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetBinary(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_directional_derivative_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetDirectionalDerivativeReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetDirectionalDerivative(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_adjoint_derivative_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetAdjointDerivativeReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetAdjointDerivative(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_output_derivatives_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetOutputDerivativesReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetOutputDerivatives(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn serialize_fmu_state_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3SerializeFmuStateReturn> {
        match return_message {
            fmi3_return::ReturnMessage::SerializeFmuState(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_clock_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetClockReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetClock(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn update_discrete_states_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3UpdateDiscreteStatesReturn> {
        match return_message {
            fmi3_return::ReturnMessage::UpdateDiscreteStates(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_interval_decimal_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetIntervalDecimalReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetIntervalDecimal(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_interval_fraction_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetIntervalFractionReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetIntervalFraction(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_shift_decimal_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetShiftDecimalReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetShiftDecimal(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }

    pub fn get_shift_fraction_return_matcher(
        &mut self,
        return_message: fmi3_return::ReturnMessage
    ) -> Fmi3SlaveResult<fmi3_messages::Fmi3GetShiftFractionReturn> {
        match return_message {
            fmi3_return::ReturnMessage::GetShiftFraction(inner_message) => {
                Ok(inner_message)
            }
            _ => Err(Fmi3SlaveError::ReturnError(
                format!("Unexpected return message: {:?}", return_message)
            ))
        }
    }
}

/// Sends the fmi3FreeInstance message to the backend when the slave is dropped.
impl Drop for Fmi3Slave {
    fn drop(&mut self) {
        let cmd = Fmi3Command {
            command: Some(Command::Fmi3FreeInstance(
                fmi3_messages::Fmi3FreeInstance {}
            )),
        };

        match self.dispatcher.send(&cmd) {
            Ok(_) => (),
            Err(error) => error!(
                "Freeing instance failed with error: {:?}.", error
            ),
        };
    }
}

pub type Fmi3SlaveResult<T> = Result<T, Fmi3SlaveError>;

#[derive(Debug)]
pub enum Fmi3SlaveError {
    DispatchError(DispatcherError),
    ReturnError(String)
}

impl Display for Fmi3SlaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DispatchError(dispatcher_error) => {
                write!(f, "dispatch error: {}", dispatcher_error)
            }
            Self::ReturnError(fault_message) => {
                write!(f, "return error: {}", fault_message)
            }
        }
    }
}

impl Error for Fmi3SlaveError {}

impl From<DispatcherError> for Fmi3SlaveError {
    fn from(value: DispatcherError) -> Self {
        Self::DispatchError(value)
    }
}

pub struct SlaveState {
    pub bytes: Vec<u8>,
}
impl SlaveState {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes),
        }
    }
}

pub type Fmi3SlaveType = Box<Fmi3Slave>;
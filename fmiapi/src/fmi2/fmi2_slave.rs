use super::{
    fmi2_messages::{
        self,
        Fmi2Command,
        fmi2_command::Command,
        Fmi2Return,
        fmi2_return,
        fmi2_return::ReturnMessage
    },
    fmi2_types::Fmi2Status,
    fmi2_logger::Fmi2Logger
};

use crate::common::{
    dispatcher::{Dispatch, Dispatcher, DispatcherError},
    logger::Logger,
    protobuf_extensions::ExpectableReturn
};

use std::{
    error::Error,
    ffi::CString,
    fmt::{Debug, Display}
};

use prost::Message;

#[repr(C)]
pub struct Fmi2Slave {
    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to free or overwrite the memory as soon as another call is made to the FMI interface.
    pub string_buffer: Vec<CString>,

    /// Object performing remote procedure calls on the slave
    pub dispatcher: Dispatcher,

    pub logger: Fmi2Logger,
    pub last_successful_time: Option<f64>,
    pub pending_message: Option<String>,
    pub dostep_status: Option<Fmi2Status>
}
//  + Send + UnwindSafe + RefUnwindSafe
// impl RefUnwindSafe for Slave {}
// impl UnwindSafe for Slave {}
// unsafe impl Send for Slave {}

impl Fmi2Slave {
    pub fn new(
        dispatcher: Dispatcher,
        logger: Fmi2Logger
    ) -> Self {
        Self {
            dispatcher,
            logger,
            string_buffer: Vec::new(),
            last_successful_time: None,
            pending_message: None,
            dostep_status: None,
        }
    }

    pub fn dispatch<R>(&mut self, command: &(impl Message + Debug)) -> Fmi2SlaveResult<R>
    where
        R: Message + ExpectableReturn<ReturnMessage>
    {
        let mut return_message = self.dispatcher
            .send_and_recv::<_, Fmi2Return>(command)?
            .return_message
            .ok_or(Fmi2SlaveError::ReturnError)?;

        while let fmi2_return::ReturnMessage::Log(log_return) = return_message {
            self.handle_log_return(log_return);

            let continue_command = Fmi2Command {
                command: Some(Command::Fmi2CallbackContinue(
                    fmi2_messages::Fmi2CallbackContinue {}
                )),
            };

            return_message = self.dispatcher
                .send_and_recv::<_, Fmi2Return>(&continue_command)?
                .return_message
                .ok_or(Fmi2SlaveError::ReturnError)?;
        }

        R::extract_from(return_message)
            .ok_or(Fmi2SlaveError::ReturnError)
    }

    fn handle_log_return(
        &mut self,
        log_return: fmi2_messages::Fmi2LogReturn
    ) {
        self.logger.log(
            log_return.status().into(),
            log_return.category.into(),
            &log_return.log_message
        );
    }
}

/// Sends the fmi2FreeInstance message to the backend when the slave is dropped.
impl Drop for Fmi2Slave {
    fn drop(&mut self) {
        let cmd = Fmi2Command {
            command: Some(Command::Fmi2FreeInstance(
                fmi2_messages::Fmi2FreeInstance {}
            )),
        };

        match self.dispatcher.send(&cmd) {
            Ok(_) => self.logger.ok("Send free instance message to shut down backend"),
            Err(error) => self.logger.error(&format!(
                "Freeing instance failed with error: {}.", error
            )),
        };
    }
}

type Fmi2SlaveResult<T> = Result<T, Fmi2SlaveError>;

#[derive(Debug)]
pub enum Fmi2SlaveError {
    DispatchError(DispatcherError),
    ReturnError
}

impl Display for Fmi2SlaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DispatchError(dispatcher_error) => {
                write!(f, "dispatch error; {}", dispatcher_error)
            }
            Self::ReturnError => {
                write!(f, "unknown return message from backend")
            }
        }
    }
}

impl Error for Fmi2SlaveError {}

impl From<DispatcherError> for Fmi2SlaveError {
    fn from(value: DispatcherError) -> Self {
        Self::DispatchError(value)
    }
}

#[derive(Debug)]
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
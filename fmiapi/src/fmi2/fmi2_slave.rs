//! This module contains definitions, trait implementations and related
//! structures to the `Fmi2Slave` struct.

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

/// Struct representing an - and containing the references and subsctructures
/// of - a singular FMU instance. In FMI2 palor this is the fmi2Component.
/// 
/// Th content and structure of this struct is undefined and unknown in the
/// FMI2 standard by design, letting implementers defines and use it as needed
/// for their specific usecase. In UniFMU's case this struct is the root
/// identity of the FMU instance, with the actions of it's methods and contents
/// of it's fields related to that instance's FMU functionality, including the
/// behaviour defined in the user implemented backend. There is a one to one
/// relationship between an instant of this struct and a UniFMU backend
/// process.
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

    /// Dispatches a FMI command to the backend, handles any callbacks from the
    /// backend during command execution, and returns the return message from
    /// the backend after it has executed the command.
    /// 
    /// This method will return an error if it at any point gets and unexpected
    /// return message from the backend, or if communication with the backend
    /// is disrupted.
    /// 
    /// Currently only the 'logger' callback is handled (accepting and
    /// emitting log events from the backend) (see page 21 fo the Fmi 2.0.5
    /// specification, and the `common::logger` module for further details).
    pub fn dispatch<R>(
        &mut self,
        command: &(impl Message + Debug)
    ) -> Fmi2SlaveResult<R>
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

    /// Logs the logging event contained in the Fmi2LogReturn message using the
    /// Fmi2Slaves logger.
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
            Ok(_) => self.logger.ok("Send free instance message to shut down backend."),
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
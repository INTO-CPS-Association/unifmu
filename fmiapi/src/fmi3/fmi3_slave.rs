//! This module contains definitions, trait implementations and related
//! structures to the `Fmi3Slave` struct.

use super::{
    fmi3_logger::Fmi3Logger,
    fmi3_messages::{
        self,
        Fmi3Command,
        fmi3_command::Command,
        Fmi3Return,
        fmi3_return,
        fmi3_return::ReturnMessage
    },
    fmi3_types::Fmi3Byte
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
/// of - a singular FMU instance. In FMI3 palor this is the fmi3Instance.
/// 
/// Th content and structure of this struct is undefined and unknown in the
/// FMI2 standard by design, letting implementers defines and use it as needed
/// for their specific usecase. In UniFMU's case this struct is the root
/// identity of the FMU instance, with the actions of it's methods and contents
/// of it's fields related to that instance's FMU functionality, including the
/// behaviour defined in the user implemented backend. There is a one to one
/// relationship between an instant of this struct and a UniFMU backend
/// process.
pub struct Fmi3Slave {
    pub byte_buffer: Vec<Vec<Fmi3Byte>>,
    dispatcher: Dispatcher,
    pub logger: Fmi3Logger,
    pub last_successful_time: Option<f64>,
    pub string_buffer: Vec<CString>
}

impl Fmi3Slave {
    pub fn new(dispatcher: Dispatcher, logger: Fmi3Logger) -> Self {
        Self {
            byte_buffer: Vec::new(),
            dispatcher,
            logger,
            last_successful_time: None,
            string_buffer: Vec::new()
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
    /// Currently only the 'fmi3LogMessageCallback' callback is handled
    /// (accepting and emitting log events from the backend) (see section
    /// 2.3.1 of the FMI3 specification, and the `common::logger` module for
    /// further details).
    pub fn dispatch<R>(&mut self, command: &(impl Message + Debug)) -> Fmi3SlaveResult<R>
    where
        R: Message + ExpectableReturn<ReturnMessage>
    {
        let mut return_message = self.dispatcher
            .send_and_recv::<_, Fmi3Return>(command)?
            .return_message
            .ok_or(Fmi3SlaveError::ReturnError)?;

        while let fmi3_return::ReturnMessage::Log(log_return) = return_message {
            self.handle_log_return(log_return);

            let continue_command = Fmi3Command {
                command: Some(Command::Fmi3CallbackContinue(
                    fmi3_messages::Fmi3CallbackContinue {}
                )),
            };

            return_message = self.dispatcher
                .send_and_recv::<_, Fmi3Return>(&continue_command)?
                .return_message
                .ok_or(Fmi3SlaveError::ReturnError)?;
        }

        R::extract_from(return_message)
            .ok_or(Fmi3SlaveError::ReturnError)
    }

    /// Logs the logging event contained in the Fmi3LogReturn message using the
    /// Fmi3Slaves logger.
    fn handle_log_return(
        &mut self,
        log_return: fmi3_messages::Fmi3LogReturn
    ) {
        self.logger.log(
            log_return.status().into(),
            log_return.category.into(),
            &log_return.log_message
        );
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
            Ok(_) => self.logger.ok("Send free instance message to shut down backend."),
            Err(error) => self.logger.error(&format!(
                "Freeing instance failed with error: {}.", error
            )),
        };
    }
}

pub type Fmi3SlaveResult<T> = Result<T, Fmi3SlaveError>;

#[derive(Debug)]
pub enum Fmi3SlaveError {
    DispatchError(DispatcherError),
    ReturnError
}

impl Display for Fmi3SlaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DispatchError(dispatcher_error) => {
                write!(f, "dispatch error: {}", dispatcher_error)
            }
            Self::ReturnError => {
                write!(f, "unknown return message from backend")
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
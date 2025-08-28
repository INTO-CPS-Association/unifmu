use super::{
    fmi3_logger::Fmi3Logger,
    fmi3_messages::{
        self,
        Fmi3Command,
        fmi3_command::Command,
        Fmi3Return,
        fmi3_return,
        fmi3_return::ReturnMessage
    }
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

pub struct Fmi3Slave {
    dispatcher: Dispatcher,
    pub logger: Fmi3Logger,
    pub last_successful_time: Option<f64>,
    pub string_buffer: Vec<CString>,
}

impl Fmi3Slave {
    pub fn new(dispatcher: Dispatcher, logger: Fmi3Logger) -> Self {
        Self {
            dispatcher,
            logger,
            last_successful_time: None,
            string_buffer: Vec::new(),
        }
    }

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
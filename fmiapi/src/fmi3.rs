#![allow(non_snake_case)]
#![allow(unused_variables)]

use std::{
    ffi::{c_void, CStr, CString},
    path::{Path, PathBuf},
    slice::{from_raw_parts, from_raw_parts_mut},
    sync::LazyLock
};

use libc::{c_char, size_t};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use url::Url;
use tracing::{error, warn};
use tracing_subscriber;

use crate::dispatcher::{Dispatch, Dispatcher};
use crate::fmi3_messages::{self, Fmi3Command, fmi3_command::Command};
use crate::spawn::spawn_slave;

/// One shot function that sets up logging.
/// 
/// Checking the result runs the contained function once if it hasn't been run
/// or returns the stored result.
/// 
/// Result should be checked at entrance to instantiation functions, and an
/// error should be considered a grave error signifying something seriously
/// wrong (most probably that the global logger was already set somewhere else).
static ENABLE_LOGGING: LazyLock<Result<(), Fmi3Status>> = LazyLock::new(|| {
    if tracing_subscriber::fmt::try_init().is_err() {
        return Err(Fmi3Status::Fatal);
    }
    Ok(())
});

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3Status {
    OK = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
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

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3DependencyKind {
	Independent = 0,
	Constant = 1,
	Fixed = 2,
	Tunable = 3,
	Discrete = 4,
	Dependent = 5,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3IntervalQualifier {
    IntervalNotYetKnown = 0,
    IntervalUnchanged = 1,
    IntervalChanged = 2,
}

pub struct Fmi3Slave {
    dispatcher: Dispatcher,
    last_successful_time: Option<f64>,
    string_buffer: Vec<CString>,
}

impl Fmi3Slave {
    pub fn new(dispatcher: Dispatcher) -> Self {
        Self {
            dispatcher,
            last_successful_time: None,
            string_buffer: Vec::new(),
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

pub struct SlaveState {
    bytes: Vec<u8>,
}
impl SlaveState {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes),
        }
    }
}

type Fmi3SlaveType = Box<Fmi3Slave>;

fn c2s(c: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(c)
            .to_str()
            .expect("Should be able to convert CStr to String!") //Grave error, should abort!
            .to_owned()
    }
}

// ------------------------------------- FMI FUNCTIONS --------------------------------

static VERSION: &str = "3.0\0";

#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn fmi3SetDebugLogging(
    instance: &mut Fmi3Slave,
    logging_on: i32,
    n_categories: size_t,
    categories: *const *const c_char
) -> Fmi3Status {
    error!("fmi3SetDebugLogging is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateModelExchange(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: i32,
    logging_on: i32,
    instance_environment: *const c_void,
    log_message: *const c_void,
) -> Option<Fmi3SlaveType> {
    if (&*ENABLE_LOGGING).is_err() {
        error!("Tried to set already set global tracing subscriber.");
        return None;
    }

    error!("fmi3InstantiateModelExchange is not implemented by UNIFMU.");
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateCoSimulation(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: bool,
    logging_on: bool,
    event_mode_used: bool,
    early_return_allowed: bool,
    required_intermediate_variables: *const u32,
    n_required_intermediate_variables: size_t,
    instance_environment: *const c_void,
    log_message: *const c_void,
    intermediate_update: *const c_void,
) -> Option<Fmi3SlaveType> {
    if (&*ENABLE_LOGGING).is_err() {
        error!("Tried to set already set global tracing subscriber.");
        return None;
    }

    let instance_name = c2s(instance_name);
    let instantiation_token = c2s(instantiation_token);
    let required_intermediate_variables = unsafe {
        from_raw_parts(
            required_intermediate_variables,
            n_required_intermediate_variables,
        )
    }
    .to_owned();

    let resource_path_str = unsafe {
        match resource_path.as_ref() {
            Some(b) => match CStr::from_ptr(b).to_str() {
                Ok(s) => s.to_string(),
                Err(e) => {
                    error!("resource path was not valid utf-8");
                    return None;
                },
            },
            None => {
                error!("resourcePath was null");
                return None
            },
        }
    };

    // NOTE: In version 3 of the FMI standard, resourcePath should be a path, e.g., "C:\...". At
    // least one tool seems to still follow version 2 in that it passes a URI, e.g., starting with
    // "file:///C://..." instead. The current implementation maintains this "backwards
    // compatibility" with this incorrect implementation of version 3 of the standard.

    // Check for supported URI schemes or treat as a direct file path
    let resources_dir = if resource_path_str.starts_with("file:")
        || resource_path_str.starts_with("http:")
        || resource_path_str.starts_with("https:")
        || resource_path_str.starts_with("ftp:")
        || resource_path_str.starts_with("fmi2:")
    {
        // Parse as a URI
        let resource_uri = match Url::parse(&resource_path_str) {
            Ok(uri) => uri,
            Err(e) => {
                error!("Unable to parse uri: {:#?}", e);
                return None;
            }
        };

        if resource_uri.scheme() == "file" {
            match resource_uri.to_file_path() {
                Ok(path) => path,
                Err(_) => {
                    error!(
                        "URI was parsed but could not be converted into a file path, got: '{:?}'.",
                        resource_uri
                    );
                    return None;
                }
            }
        } else {
            error!("Unsupported URI scheme: '{}'", resource_uri.scheme());
            return None;
        }
    } else {
        // Treat it as a direct file path
        PathBuf::from(resource_path_str)
    };

    let dispatcher = match spawn_slave(&Path::new(&resources_dir)) {
        Ok(dispatcher) => dispatcher,
        Err(_) => {
            error!("Spawning fmi3 slave failed.");
            return None;
        }
    };

    let resource_path = match resources_dir.into_os_string().into_string() {
        Ok(string_path) => string_path,
        Err(e) => {
            error!("Couldn't convert resource directory path into String.");
            return None;
        }
    };

    let mut slave = Fmi3Slave::new(dispatcher);

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3InstantiateCoSimulation(
            fmi3_messages::Fmi3InstantiateCoSimulation {
                instance_name: instance_name.clone(),
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

    match slave.dispatcher.send_and_recv::<_, fmi3_messages::Fmi3EmptyReturn>(&cmd) {
        Ok(_) => Some(Box::new(slave)),
        Err(error) => {
            error!(
                "Instantiation of fmi3 slave '{}' failed with error {:?}.",
                instance_name,
                error
            );
            None
        },
    }
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateScheduledExecution(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: i32,
    logging_on: i32,
    instance_environment: *const c_void,
    log_message: *const c_void,
    clock_update: *const c_void,
    lock_preemption: *const c_void,
    unlock_preemption: *const c_void,
) -> Option<Fmi3SlaveType> {
    if (&*ENABLE_LOGGING).is_err() {
        error!("Tried to set already set global tracing subscriber.");
        return None;
    }

    error!("fmi3InstantiateScheduledExecution is not implemented by UNIFMU.");
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
}

#[no_mangle]
pub extern "C" fn fmi3DoStep(
    instance: &mut Fmi3Slave,
    current_communication_point: f64,
    communication_step_size: f64,
    no_set_fmu_state_prior_to_current_point: bool,
    event_handling_needed: *mut bool,
    terminate_simulation: *mut bool,
    early_return: *mut bool,
    last_successful_time: *mut f64,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3DoStep(
            fmi3_messages::Fmi3DoStep {
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point,
            }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3DoStepReturn>(&cmd)
    {
        Ok(result) => {
            if !last_successful_time.is_null() {
                unsafe {
                    *last_successful_time = result.last_successful_time;
                }
            }
            if !event_handling_needed.is_null() {
                unsafe {
                    *event_handling_needed = result.event_handling_needed;
                }
            }
            if !terminate_simulation.is_null() {
                unsafe {
                    *terminate_simulation = result.terminate_simulation;
                }
            }
            if !early_return.is_null() {
                unsafe {
                    *early_return = result.early_return;
                }
            }

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3DoStep failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3EnterInitializationMode(
    instance: &mut Fmi3Slave,
    tolerance_defined: bool,
    tolerance: f64,
    start_time: f64,
    stop_time_defined: bool,
    stop_time: f64,
) -> Fmi3Status {
    let tolerance = {
        if tolerance_defined {
            Some(tolerance)
        } else {
            None
        }
    };
    let stop_time = {
        if stop_time_defined {
            Some(stop_time)
        } else {
            None
        }
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterInitializationMode(
            fmi3_messages::Fmi3EnterInitializationMode {
                tolerance_defined,
                tolerance,
                start_time,
                stop_time_defined,
                stop_time,
            },
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3EnterInitializationMode failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3ExitInitializationMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3ExitInitializationMode(
            fmi3_messages::Fmi3ExitInitializationMode {},
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3ExitInitializationMode failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3EnterEventMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterEventMode(
            fmi3_messages::Fmi3EnterEventMode {},
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3EnterEventMode failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3EnterStepMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterStepMode(
            fmi3_messages::Fmi3EnterStepMode {},
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3EnterStepMode failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3GetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut f32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetFloat32(
            fmi3_messages::Fmi3GetFloat32 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetFloat32Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetFloat32 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut f64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetFloat64(
            fmi3_messages::Fmi3GetFloat64 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetFloat64Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetFloat64 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt8(
            fmi3_messages::Fmi3GetInt8 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetInt8Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<i8> = result.values
                    .iter()
                    .map(|v| *v as i8)
                    .collect();

                values_out.copy_from_slice(&values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetInt8 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetUInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt8(
            fmi3_messages::Fmi3GetUInt8 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetUInt8Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<u8> = result.values
                    .iter()
                    .map(|v| *v as u8)
                    .collect();

                values_out.copy_from_slice(&values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetUInt8 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt16(
            fmi3_messages::Fmi3GetInt16 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetInt16Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<i16> = result.values
                    .iter()
                    .map(|v| *v as i16)
                    .collect();

                values_out.copy_from_slice(&values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetInt16 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetUInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt16(
            fmi3_messages::Fmi3GetUInt16 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetUInt16Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<u16> = result.values
                    .iter()
                    .map(|v| *v as u16)
                    .collect();

                values_out.copy_from_slice(&values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetUInt16 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt32(
            fmi3_messages::Fmi3GetInt32 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetInt32Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetInt32 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetUInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt32(
            fmi3_messages::Fmi3GetUInt32 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetUInt32Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetUInt32 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt64(
            fmi3_messages::Fmi3GetInt64 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetInt64Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetInt64 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetUInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt64(
            fmi3_messages::Fmi3GetUInt64 { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetUInt64Return>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetUInt64 failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut bool,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_values)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetBoolean(
            fmi3_messages::Fmi3GetBoolean { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetBooleanReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetBoolean failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetString(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut *const c_char,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetString(
            fmi3_messages::Fmi3GetString { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetStringReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                instance.string_buffer = result.values
                    .iter()
                    .map(|s| CString::new(s.as_bytes()).unwrap())
                    .collect();
                
                unsafe {
                    for (idx, cstr)
                    in instance.string_buffer.iter().enumerate() {
                        std::ptr::write(
                            values.offset(idx as isize),
                            cstr.as_ptr()
                        );
                    }
                }
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetString failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetBinary(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    value_sizes: *const size_t,
    values: *mut *const u8,
    n_values: size_t,
) -> Fmi3Status {
    error!("fmi3GetBinary is not implemented by UNIFMU.");
    Fmi3Status::Error

    // Partially implemented: only the privious corresponding dispatcher
    // function wasn't implemented.

    /*
    let value_references =
        unsafe { from_raw_parts(value_references, n_value_references) }.to_owned();
    let values_v = unsafe { from_raw_parts(values, n_values) };
    let value_sizes = unsafe { from_raw_parts(value_sizes, n_values) };

    let values_v: Vec<Vec<u8>> = values_v
        .iter()
        .zip(value_sizes.iter())
        .map(|(v, s)| unsafe { from_raw_parts(*v, *s).to_vec() })
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::??(
            fmi3_messages::?? { ?? }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, ??>(&cmd)
        .map(|result| {
            todo!()
        })
    {
        Ok((s, v)) => match v {
            Some(vs) => unsafe {
                for (idx, v) in vs.iter().enumerate() {
                    std::ptr::write(values.offset(idx as isize), v.as_ptr());
                }
                s
            },
            None => s,
        },
        Err(error) => {
            error!("fmi3GetBinary failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
    */
}

#[no_mangle]
pub extern "C" fn fmi3GetClock(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut bool,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values_out = unsafe {
        from_raw_parts_mut(values, n_value_references)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetClock(
            fmi3_messages::Fmi3GetClock { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetClockReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetClock failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    intervals: *mut f64,
	qualifiers: *mut i32,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let intervals_out = unsafe {
        from_raw_parts_mut(intervals, n_value_references)
    };

    let qualifiers_out = unsafe {
        from_raw_parts_mut(qualifiers, n_value_references)
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetIntervalDecimal(
            fmi3_messages::Fmi3GetIntervalDecimal { value_references }
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3GetIntervalDecimalReturn>(&cmd)
    {
        Ok(result) => {
            if !result.intervals.is_empty() {
                intervals_out.copy_from_slice(&result.intervals);
            };

            if !result.qualifiers.is_empty() {
                qualifiers_out.copy_from_slice(&result.qualifiers);
            };

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
            })
        }
        Err(error) => {
            error!("fmi3GetIntervalDecimal failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    interval_counters: *const u64,
	resolution: *mut u64,
	qualifier: *mut Fmi3IntervalQualifier,
    n_values: size_t,
) -> Fmi3Status {
    error!("fmi3GetIntervalFraction is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    shifts: *const f64,
) -> Fmi3Status {
    error!("fmi3GetShiftDecimal is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    counters: *const u64,
	resolutions: *const u64,
) -> Fmi3Status {
    error!("fmi3GetShiftFraction is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    intervals: *mut f64,
) -> Fmi3Status {
    error!("fmi3SetIntervalDecimal is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    interval_counters: *const u64,
	resolutions: *mut u64,
) -> Fmi3Status {
    error!("fmi3SetIntervalFraction is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    shifts: *const f64,
) -> Fmi3Status {
    error!("fmi3SetShiftDecimal is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    counters: *const u64,
	resolutions: *const u64,
) -> Fmi3Status {
    error!("fmi3SetShiftFraction is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3EvaluateDiscreteStates(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    error!("fmi3EvaluateDiscreteStates is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3UpdateDiscreteStates(
    instance: &mut Fmi3Slave,
	discrete_states_need_update: *mut bool,
	terminate_simulation: *mut bool,
	nominals_continuous_states_changed: *mut bool,
	values_continuous_states_changed: *mut bool,
	next_event_time_defined: *mut bool,
	next_event_time: *mut f64,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3UpdateDiscreteStates(
            fmi3_messages::Fmi3UpdateDiscreteStates {}
        )),
    };

    match instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3UpdateDiscreteStatesReturn>(&cmd)
    {
        Ok(result) => {
            if !discrete_states_need_update.is_null() {
                unsafe {
                    *discrete_states_need_update = result
                        .discrete_states_need_update;
                }
            }
            if !terminate_simulation.is_null() {
                unsafe {
                    *terminate_simulation = result.terminate_simulation;
                }
            }
            if !nominals_continuous_states_changed.is_null() {
                unsafe {
                    *nominals_continuous_states_changed = result
                        .nominals_continuous_states_changed;
                }
            }
            if !values_continuous_states_changed.is_null() {
                unsafe {
                    *values_continuous_states_changed = result
                        .values_continuous_states_changed;
                }
            }
            if !next_event_time_defined.is_null() {
                unsafe {
                    *next_event_time_defined = result
                        .next_event_time_defined;
                }
            }
            if !next_event_time.is_null() {
                unsafe {
                    *next_event_time = result.next_event_time;
                }
            }

            Fmi3Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi3Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi3UpdateDiscreteStates failed with error: {:?}.", error);
            Fmi3Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3EnterContinuousTimeMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    error!("fmi3EnterContinuousTimeMode is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3CompletedIntegratorStep(
    instance: &mut Fmi3Slave,
	no_set_fmu_state_prior_to_current_point: i32,
	enter_event_mode: *const i32,
	terminate_simulation: *const i32,
) -> Fmi3Status {
    error!("fmi3CompletedIntegratorStep is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetTime(
    instance: &mut Fmi3Slave,
	time: f64,
) -> Fmi3Status {
    error!("fmi3SetTime is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    error!("fmi3SetContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetContinuousStateDerivatives(
    instance: &mut Fmi3Slave,
	derivatives: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    error!("fmi3GetContinuousStateDerivatives is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetEventIndicators(
    instance: &mut Fmi3Slave,
	event_indicators: *const f64,
	n_event_indicators: size_t,
) -> Fmi3Status {
    error!("fmi3GetEventIndicators is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    error!("fmi3GetContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNominalsOfContinuousStates(
    instance: &mut Fmi3Slave,
	nominals: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    error!("fmi3GetNominalsOfContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfEventIndicators(
    instance: &mut Fmi3Slave,
	n_event_indicators: *const size_t,
) -> Fmi3Status {
    error!("fmi3GetNumberOfEventIndicators is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfContinuousStates(
    instance: &mut Fmi3Slave,
	n_continuous_states: *const size_t,
) -> Fmi3Status {
    error!("fmi3GetNumberOfContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetOutputDerivatives(
    instance: &mut Fmi3Slave,
	value_references: *const u32,
	n_value_references: size_t,
	orders: *const i32,
	values: *const f64,
	n_values: size_t,
) -> Fmi3Status {
    error!("fmi3GetOutputDerivatives is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3ActivateModelPartition(
    instance: &mut Fmi3Slave,
	value_reference: u32,
	activation_time: f64,
) -> Fmi3Status {
    error!("fmi3ActivateModelPartition is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f32,
    n_values: size_t,
) -> Fmi3Status{
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetFloat32(
            fmi3_messages::Fmi3SetFloat32 {
                value_references,
                values,
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetFloat32 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }
        .to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetFloat64(
            fmi3_messages::Fmi3SetFloat64 {
                value_references,
                values
            }
        )),
    };
    
    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetFloat64 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<i32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as i32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt8(
            fmi3_messages::Fmi3SetInt8 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetInt8 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetUInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<u32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as u32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt8(
            fmi3_messages::Fmi3SetUInt8 {
                value_references,
                values
            }
        )),
    };
    
    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetUInt8 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<i32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as i32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt16(
            fmi3_messages::Fmi3SetInt16 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetInt16 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetUInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<u32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as u32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt16(
            fmi3_messages::Fmi3SetUInt16 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetUInt16 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt32(
            fmi3_messages::Fmi3SetInt32 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetInt32 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetUInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt32(
            fmi3_messages::Fmi3SetUInt32 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetUInt32 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt64(
            fmi3_messages::Fmi3SetInt64 {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetInt64 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetUInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt64(
            fmi3_messages::Fmi3SetUInt64 {
                value_references,
                values,
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetUInt64 failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const bool,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetBoolean(
            fmi3_messages::Fmi3SetBoolean {
                value_references,
                values
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetBoolean failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetString(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const *const c_char,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<String> = unsafe {
        from_raw_parts(values, n_values)
            .iter()
            .map(
                |v| CStr::from_ptr(*v)
                    .to_str()
                    .unwrap()
                    .to_owned()
            )
            .collect()
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetString(
            fmi3_messages::Fmi3SetString {
                value_references,
                values,
            }
        )),
    };

    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetString failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetBinary(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    value_sizes: *const size_t,
    values: *const u8,
    n_values: size_t,
) -> Fmi3Status {
    // Convert raw pointers to Rust slices
    let value_references = unsafe {
        std::slice::from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let sizes = unsafe {
        std::slice::from_raw_parts(value_sizes, n_values)
    };

    // Create a vector of slices for the binary values
    let mut binary_values: Vec<&[u8]> = Vec::with_capacity(n_value_references);

    // Use an offset to correctly slice the values based on sizes
    let mut offset = 0;
    for &size in sizes {
        let value_slice = unsafe {
            std::slice::from_raw_parts(values.add(offset), size)
        };
        binary_values.push(value_slice);
        offset += size;
    }

    let value_sizes = sizes
        .iter()
        .map(|&size| size as u64)
        .collect();

    let values = binary_values
        .iter()
        .map(|&v| v.to_vec())
        .collect();

    // Ensure the instance pointer is valid
    let instance = unsafe { &mut *(instance as *mut Fmi3Slave) };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetBinary(
            fmi3_messages::Fmi3SetBinary {
                value_references,
                value_sizes,
                values,
            }
        )),
    };

    // Call the dispatcher with references, binary values, and their sizes
    instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetBinary failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3SetClock(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const bool,
) -> Fmi3Status {
	let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

	let values = unsafe {
        from_raw_parts(values, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetClock(
            fmi3_messages::Fmi3SetClock {
                value_references,
                values,
            }
        )),
    };
	
	instance.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi3SetClock failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfVariableDependencies(
    instance: &mut Fmi3Slave,
    value_reference: *const i32,
    n_dependencies: *const size_t,
) -> Fmi3Status {
    error!("fmi3GetNumberOfVariableDependencies is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetVariableDependencies(
    instance: &mut Fmi3Slave,
    dependent: *const i32,
    element_indices_of_dependent: *const size_t,
	independents: *const i32,
	element_indices_of_independents: *const size_t,
	dependency_kinds: *const Fmi3DependencyKind,
	n_dependencies: size_t,
) -> Fmi3Status {
    error!("fmi3GetVariableDependencies is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetFMUState(
    instance: &mut Fmi3Slave,
    state: &mut Option<SlaveState>,
) -> Fmi3Status {
    error!("fmi3GetFMUState is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SetFMUState(
	instance: &mut Fmi3Slave,
	state: &SlaveState,
) -> Fmi3Status {
    error!("fmi3SetFMUState is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3FreeFMUState(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
) -> Fmi3Status {
    error!("fmi3FreeFMUState is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SerializedFMUStateSize(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
	size: *const size_t,
) -> Fmi3Status {
    error!("fmi3SerializedFMUStateSize is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3SerializeFMUState(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
	serialized_state: *const u8,
	size: size_t,
) -> Fmi3Status {
    error!("fmi3SerializeFMUState is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3DeserializeFMUState(
    instance: &mut Fmi3Slave,
	serialized_state: *const u8,
	size: size_t,
	state: &SlaveState,
) -> Fmi3Status {
    error!("fmi3DeserializeFMUState is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetDirectionalDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const u32,
	n_unknowns: size_t,
	knowns: *const u32,
	n_knowns: size_t,
	delta_knowns: *const f64,
	n_delta_knowns: size_t,
	delta_unknowns: *const f64,
	n_delta_unknowns: size_t,
) -> Fmi3Status {
    error!("fmi3GetDirectionalDerivative is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3GetAdjointDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const u32,
	n_unknowns: size_t,
	knowns: *const u32,
	n_knowns: size_t,
	delta_unknowns: *const f64,
	n_delta_unknowns: size_t,
	delta_knowns: *const f64,
	n_delta_knowns: size_t,
) -> Fmi3Status {
    error!("fmi3GetAdjointDerivative is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3EnterConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    error!("fmi3EnterConfigurationMode is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3ExitConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    error!("fmi3ExitConfigurationMode is not implemented by UNIFMU.");
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3Terminate(slave: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3Terminate(
            fmi3_messages::Fmi3Terminate {}
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|s| s.into())
        .unwrap_or_else(|error| {
            error!("Termination failed with error: {:?}.", error);
            Fmi3Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi3FreeInstance(slave: Option<Box<Fmi3Slave>>) {
    let mut slave = slave;

    match slave.as_mut() {
        Some(s) => {
            // fmi3FreeInstance message is send to backend on drop
            drop(slave)
        }
        None => {warn!("No instance given.")}
    }
}

#[no_mangle]
pub extern "C" fn fmi3Reset(slave: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3Reset(
            fmi3_messages::Fmi3Reset {}
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|s| s.into())
        .unwrap_or_else(|error| {
            error!("Error while resetting instance: {:?}.", error);
            Fmi3Status::Error
        })
}

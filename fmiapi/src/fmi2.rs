#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use crate::dispatcher::{Dispatch, Dispatcher};
use crate::fmi2_messages::{self, Fmi2Command, fmi2_command::Command};
use crate::spawn::spawn_slave;
use libc::c_double;
use libc::size_t;


use url::Url;
use tracing::{error, warn};
use tracing_subscriber;

use std::{
    ffi::{CStr, CString, NulError},
    os::raw::{c_char, c_int, c_uint, c_ulonglong, c_void},
    path::Path,
    slice::{from_raw_parts, from_raw_parts_mut},
    str::Utf8Error,
    sync::LazyLock
};

/// One shot function that sets up logging.
/// 
/// Checking the result runs the contained function once if it hasn't been run
/// or returns the stored result.
/// 
/// Result should be checked at entrance to instantiation functions, and an
/// error should be considered a grave error signifying something seriously
/// wrong (most probably that the global logger was already set somewhere else).
static ENABLE_LOGGING: LazyLock<Result<(), Fmi2Status>> = LazyLock::new(|| {
    if tracing_subscriber::fmt::try_init().is_err() {
        return Err(Fmi2Status::Fatal);
    }
    Ok(())
});

///
/// Represents the function signature of the logging callback function passsed
/// from the envrionment to the slave during instantiation.
pub type Fmi2CallbackLogger = extern "C" fn(
    component_environment: *mut c_void,
    instance_name: *const c_char,
    status: Fmi2Status,
    category: *const c_char,
    message: *const c_char, // ... variadic functions support in rust seems to be unstable
);

pub type Fmi2CallbackAllocateMemory = extern "C" fn(nobj: c_ulonglong, size: c_ulonglong);
pub type Fmi2CallbackFreeMemory = extern "C" fn(obj: *const c_void);
pub type Fmi2StepFinished = extern "C" fn(component_environment: *const c_void, status: i32);

/// From specification:
///
/// `This is a pointer to a data structure in the simulation environment that calls the FMU.
/// Using this pointer, data from the modelDescription.xml file [(for example, mapping of valueReferences to variable names)]
/// can be transferred between the simulation environment and the logger function.`
///
/// Recommended way to represent opaque pointer, i.e the c type 'void*'
/// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
pub struct ComponentEnvironment {
    _private: [u8; 0],
}

/// A set of callback functions provided by the environment
/// Note that all but the 'logger' function are optional and may only be used if the appropriate
/// flags are set in the 'modelDescription.xml' file
#[repr(C)]
pub struct Fmi2CallbackFunctions {
    pub logger: Fmi2CallbackLogger,

    pub allocate_memory: Option<Fmi2CallbackAllocateMemory>,

    pub free_memory: Option<Fmi2CallbackFreeMemory>,

    pub step_finished: Option<Fmi2StepFinished>,
    pub component_environment: &'static Option<Box<ComponentEnvironment>>,
}

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi2Status {
    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
    Pending = 5,
}

impl From<fmi2_messages::Fmi2StatusReturn> for Fmi2Status {
    fn from(src: fmi2_messages::Fmi2StatusReturn) -> Self {
        match src.status() {
            fmi2_messages::Fmi2Status::Fmi2Ok => Self::Ok,
            fmi2_messages::Fmi2Status::Fmi2Warning => Self::Warning,
            fmi2_messages::Fmi2Status::Fmi2Discard => Self::Discard,
            fmi2_messages::Fmi2Status::Fmi2Error => Self::Error,
            fmi2_messages::Fmi2Status::Fmi2Fatal => Self::Fatal,
            fmi2_messages::Fmi2Status::Fmi2Pending => Self::Pending,
        }
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi2StatusKind {
    Fmi2DoStepStatus = 0,
    Fmi2PendingStatus = 1,
    Fmi2LastSuccessfulTime = 2,
    Fmi2Terminated = 3,
}

// ====================== config =======================

#[repr(i32)]
pub enum Fmi2Type {
    Fmi2ModelExchange = 0,
    Fmi2CoSimulation = 1,
}

impl From<fmi2_messages::Fmi2Type> for Fmi2Type {
    fn from(src: fmi2_messages::Fmi2Type) -> Self {
        match src {
            fmi2_messages::Fmi2Type::Fmi2ModelExchange => Self::Fmi2ModelExchange,
            fmi2_messages::Fmi2Type::Fmi2CoSimulation => Self::Fmi2CoSimulation,
        }
    }
}

// ----------------------- Library instantiation and cleanup ---------------------------

#[repr(C)]
pub struct Fmi2Slave {
    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to free or overwrite the memory as soon as another call is made to the FMI interface.
    pub string_buffer: Vec<CString>,

    /// Object performing remote procedure calls on the slave
    pub dispatcher: Dispatcher,

    pub last_successful_time: Option<f64>,
    pub pending_message: Option<String>,
    pub dostep_status: Option<Fmi2Status>,
}
//  + Send + UnwindSafe + RefUnwindSafe
// impl RefUnwindSafe for Slave {}
// impl UnwindSafe for Slave {}
// unsafe impl Send for Slave {}

impl Fmi2Slave {
    fn new(dispatcher: Dispatcher) -> Self {
        Self {
            dispatcher,
            string_buffer: Vec::new(),
            last_successful_time: None,
            pending_message: None,
            dostep_status: None,
        }
    }
}

/// Sends the fmi3FreeInstance message to the backend when the slave is dropped.
impl Drop for Fmi2Slave {
    fn drop(&mut self) {
        let cmd = Fmi2Command {
            command: Some(Command::Fmi2FreeInstance(
                fmi2_messages::Fmi2FreeInstance {}
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

// ------------------------------------- Errors -------------------------------------

// ------------------------------------- FMI FUNCTIONS --------------------------------

pub static VERSION: &str = "2.0\0";
pub static TYPES_PLATFORM: &str = "default\0";
#[no_mangle]
pub extern "C" fn fmi2GetTypesPlatform() -> *const c_char {
    TYPES_PLATFORM.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn fmi2GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

// ------------------------------------- FMI FUNCTIONS (Life-Cycle) --------------------------------

/// Instantiates a slave instance by invoking a command in a new process
/// the command is specified by the configuration file, launch.toml, that should be located in the resources directory
/// fmi-commands are sent between the wrapper and slave(s) using a message queue library, specifically zmq.
/// 
/// # Safety
/// When calling this function, you have to ensure that either the pointer is null or the pointer is convertible to a reference to a string.
/// 
/// Furthermore if the pointer is not null:
/// * The memory pointed to must contain a valid nul terminator at the end of the string.
/// * The pointer must be [valid] for reads of bytes up to and including the nul terminator.
/// * The nul terminator must be within isize::MAX from the pointer.
#[no_mangle]
pub unsafe extern "C" fn fmi2Instantiate(
    instance_name: *const c_char, // neither allowed to be null or empty string
    fmu_type: Fmi2Type,
    fmu_guid: *const c_char, // not allowed to be null,
    fmu_resource_location: *const c_char,
    functions: &Fmi2CallbackFunctions,
    visible: c_int,
    logging_on: c_int,
) -> Option<Box<Fmi2Slave>> {
    if (*ENABLE_LOGGING).is_err() {
        error!("Tried to set already set global tracing subscriber.");
        return None;
    }

    let resource_uri = unsafe {
        match fmu_resource_location.as_ref() {
            Some(b) => match CStr::from_ptr(b).to_str() {
                Ok(s) => match Url::parse(s) {
                    Ok(url) => url,
                    Err(error) => {
                        error!("unable to parse resource url");
                        return None;
                    },
                },
                Err(e) => {
                    error!("resource url was not valid utf-8");
                    return None;
                },
            },
            None => {
                error!("fmuResourcesLocation was null");
                return None;
            },
        }
    };

    let resources_dir = match resource_uri.to_file_path() {
        Ok(resources_dir) => resources_dir,
        Err(_) => {
            error!(
                "URI was parsed but could not be converted into a file path, got: '{:?}'.",
                resource_uri
            );
            return None;
        }
    };

    let dispatcher = match spawn_slave(Path::new(&resources_dir)) {
        Ok(dispatcher) => dispatcher,
        Err(_) => {
            error!("Spawning fmi2 slave failed.");
            return None;
        }
    };

    let mut slave = Fmi2Slave::new(dispatcher);

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Instantiate(
            fmi2_messages::Fmi2Instantiate {
                instance_name: String::from("instance_name"),
                fmu_type: 0,
                fmu_guid: String::from("fmu_guid"),
                fmu_resource_location: String::from("fmu_resources_location"),
                visible: false,
                logging_on: false,
            }
        )),
    };

    match slave.dispatcher.send_and_recv::<_, fmi2_messages::Fmi2EmptyReturn>(&cmd) {
        Ok(_) => Some(Box::new(slave)),
        Err(error) => {
            error!(
                "Instantiation of fmi2 slave failed with error {:?}.",
                error
            );
            None
        },
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(slave: Option<Box<Fmi2Slave>>) {
    let mut slave = slave;

    match slave.as_mut() {
        Some(s) => {
            // fmi2FreeInstance message is send to backend on drop
            drop(slave)
        }
        None => {warn!("No instance given.")}
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    slave: &mut Fmi2Slave,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const *const c_char,
) -> Fmi2Status {
    error!("fmi2SetDebugLogging is not implemented by UNIFMU.");
    Fmi2Status::Error
}

#[no_mangle]
pub extern "C" fn fmi2SetupExperiment(
    slave: &mut Fmi2Slave,
    tolerance_defined: c_int,
    tolerance: c_double,
    start_time: c_double,
    stop_time_defined: c_int,
    stop_time: c_double,
) -> Fmi2Status {
    let tolerance = {
        if tolerance_defined != 0 {
            Some(tolerance)
        } else {
            None
        }
    };

    let stop_time = {
        if stop_time_defined != 0 {
            Some(stop_time)
        } else {
            None
        }
    };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetupExperiment(
            fmi2_messages::Fmi2SetupExperiment {
                start_time,
                stop_time,
                tolerance,
            }
        )),
    };

    slave
        .dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetupExperiment failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2EnterInitializationMode(
            fmi2_messages::Fmi2EnterInitializationMode {},
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2EnterInitializationMode failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2ExitInitializationMode(
            fmi2_messages::Fmi2ExitInitializationMode {},
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2ExitInitializationMode failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Terminate(
            fmi2_messages::Fmi2Terminate {},
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2Terminate failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2Reset(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Reset(
            fmi2_messages::Fmi2Reset {},
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2Reset failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------
#[no_mangle]
pub extern "C" fn fmi2DoStep(
    slave: &mut Fmi2Slave,
    current_time: c_double,
    step_size: c_double,
    no_step_prior: c_int,
) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2DoStep(
            fmi2_messages::Fmi2DoStep {
                current_time,
                step_size,
                no_set_fmu_state_prior_to_current_point: no_step_prior != 0,
            }
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
    {
        Ok(status) => match status {
            Fmi2Status::Ok | Fmi2Status::Warning => {
                slave.last_successful_time = Some(current_time + step_size);
                status
            }
            status => status,
        },
        Err(error) => {
            error!("fmi2DoStep failed with error {:?}.", error);
            Fmi2Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2CancelStep(
            fmi2_messages::Fmi2CancelStep {},
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2CancelStep failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_double>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_double` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_double>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetReal(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetReal(
            fmi2_messages::Fmi2GetReal {
                references,
            }
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetRealReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }

            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetReal failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetInteger(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetInteger(
            fmi2_messages::Fmi2GetInteger {
                references,
            }
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetIntegerReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }

            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetInteger failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetBoolean(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetBoolean(
            fmi2_messages::Fmi2GetBoolean {
                references,
            }
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetBooleanReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<i32> = result.values
                    .iter()
                    .map(|v| match v {
                        false => 0,
                        true => 1,
                    })
                    .collect();

                values_out.copy_from_slice(&values)
            }

            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetBoolean failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We choose to do it on an instance basis, i.e. each instance has its own string buffer.
/// 
/// # Safety
/// Behavior is undefined if any of the following are violated:
/// * `values` must be non-null, \[valid\] for writes, and properly aligned.
/// * `references` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<c_uint>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `refernces` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `references` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `references` must point to `nvr` consecutive properly initialized values
///   of type `c_uint`.
/// * The total size `nvr * mem::size_of::<c_uint>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `references` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetString(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut *const c_char,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetString(
            fmi2_messages::Fmi2GetString {
                references,
            }
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetStringReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                let conversion_result: Result<Vec<CString>, NulError> = result
                    .values
                    .iter()
                    .map(|string| CString::new(string.as_bytes()))
                    .collect();

                match conversion_result {
                    Ok(converted_values) => {
                        slave.string_buffer = converted_values
                    },
                    Err(e) =>  {
                        error!("Backend returned strings containing interior nul bytes. These cannot be converted into CStrings.");
                        return Fmi2Status::Fatal;
                    }
                }

                unsafe {
                    for (idx, cstr)
                    in slave.string_buffer.iter().enumerate()
                    {
                        std::ptr::write(
                            values.offset(idx as isize), 
                            cstr.as_ptr()
                        );
                    }
                }
            }

            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetString failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_double>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_double` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_double>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetReal(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetReal(
            fmi2_messages::Fmi2SetReal {
                references,
                values,
            }
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetReal failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetInteger(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetInteger(
            fmi2_messages::Fmi2SetInteger {
                references,
                values,
            }
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetInteger failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
/// 
/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetBoolean(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values: Vec<bool> = unsafe { from_raw_parts(values, nvr) }
        .iter()
        .map(|v| *v != 0)
        .collect();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetBoolean(
            fmi2_messages::Fmi2SetBoolean {
                references,
                values,
            }
        )),
    };
    
    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetBoolean failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_char>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_char` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_char>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetString(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();

    let conversion_result: Result<Vec<String>, Utf8Error> = unsafe {
        from_raw_parts(values, nvr)
            .iter()
            .map(|v| {
                CStr::from_ptr(*v)
                    .to_str()
                    .map(|str| str.to_owned())
                })
            .collect()
    };

    match conversion_result {
        Ok(values) => {
            let cmd = Fmi2Command {
                command: Some(Command::Fmi2SetString(
                    fmi2_messages::Fmi2SetString {
                        references,
                        values,
                    }
                )),
            };
        
            slave.dispatcher
                .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
                .map(|status| status.into())
                .unwrap_or_else(|error| {
                    error!("fmi2SetString failed with error: {:?}.", error);
                    Fmi2Status::Error
                })
        },
        Err(conversion_error) => {
            error!("The String values could not be converted to Utf-8: {:?}.", conversion_error);
            Fmi2Status::Error
        }
    }

    
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`unknown_refs`, `known_refs`, `direction_known`, `direction_unknown`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_uint`, `c_double`, `c_double`\]
/// 
/// and `SIZE_PARAMETERS` \[`nvr_unknown`, `nvr_known`, `nvr_known`, `nvr_unknown`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `SIZE_PARAMETER * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `SIZE_PARAMETER` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `SIZE_PARAMETER * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetDirectionalDerivative(
    slave: &mut Fmi2Slave,
    unknown_refs: *const c_uint,
    nvr_unknown: size_t,
    known_refs: *const c_uint,
    nvr_known: size_t,
    direction_known: *const c_double,
    direction_unknown: *mut c_double,
) -> Fmi2Status {
    let references_unknown = unsafe {
        from_raw_parts(unknown_refs, nvr_known)
    }
        .to_owned();

    let references_known = unsafe {
        from_raw_parts(known_refs, nvr_known)
    }
        .to_owned();

    let direction_known = unsafe {
        from_raw_parts(direction_known, nvr_known)
    }
        .to_owned();

    let direction_unknown = unsafe {
        from_raw_parts_mut(direction_unknown, nvr_known)
    };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetDirectionalDerivatives(
            fmi2_messages::Fmi2GetDirectionalDerivatives {
                references_unknown,
                references_known,
                direction_known,
            },
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetDirectionalDerivativesReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                direction_unknown.copy_from_slice(&result.values);
                
                Fmi2Status::try_from(result.status)
                    .unwrap_or_else(|_| {
                        error!("Unknown status returned from backend.");
                        Fmi2Status::Fatal
                    })
            } else {
                todo!();
            }
        },
        Err(error) => {
            error!("fmi2GetDirectionalDerivative failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`references`, `orders`, `values`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_int`, `c_double`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `nvr` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `nvr * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetRealInputDerivatives(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const c_int,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let orders = unsafe { from_raw_parts(orders, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetRealInputDerivatives(
            fmi2_messages::Fmi2SetRealInputDerivatives {
                references,
                orders,
                values
            },
        )),
    };

    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetRealInputDerivatives failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`references`, `orders`, `values`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_int`, `c_double`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `nvr` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `nvr * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetRealOutputDerivatives(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const c_int,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let orders = unsafe { from_raw_parts(orders, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetRealOutputDerivatives(
            fmi2_messages::Fmi2GetRealOutputDerivatives {
                references,
                orders,
            },
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2GetRealOutputDerivativesReturn>(&cmd)
    {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }
            
            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetRealOutputDerivatives failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------
#[no_mangle]
pub extern "C" fn fmi2SetFMUstate(slave: &mut Fmi2Slave, state: &SlaveState) -> Fmi2Status {
    let state = state.bytes.to_owned();
    
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2DeserializeFmuState(
            fmi2_messages::Fmi2DeserializeFmuState {
                state,
            }
        )),
    };
    
    slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            error!("fmi2SetRealInputDerivatives failed with error: {:?}.", error);
            Fmi2Status::Error
        })
}

//

#[no_mangle]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub extern "C" fn fmi2GetFMUstate(
    slave: &mut Fmi2Slave,
    state: &mut Option<SlaveState>,
) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SerializeFmuState(
            fmi2_messages::Fmi2SerializeFmuState {}
        )),
    };

    match slave.dispatcher
        .send_and_recv::<_, fmi2_messages::Fmi2SerializeFmuStateReturn>(&cmd)
    {
        Ok(result) => {
            match state.as_mut() {
                Some(state) => {
                    state.bytes = result.state;
                }
                None => {
                    *state = Some(SlaveState::new(&result.state));
                }
            }

            Fmi2Status::try_from(result.status)
                .unwrap_or_else(|_| {
                    error!("Unknown status returned from backend.");
                    Fmi2Status::Fatal
                })
        }
        Err(error) => {
            error!("fmi2GetFMUstate failed with error: {:?}.", error);
            Fmi2Status::Error
        }
    }
}

/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
#[no_mangle]
pub extern "C" fn fmi2FreeFMUstate(
    slave: &mut Fmi2Slave,
    state: Option<Box<SlaveState>>,
) -> Fmi2Status {
    match state {
        Some(state) => drop(state),
        None => warn!("fmi2FreeFMUstate called with state pointing to null!")
    }
    Fmi2Status::Ok
}

/// Copies the state of a slave into a buffer provided by the environment
///
/// Oddly, the length of the buffer is also provided,
/// as i would expect the environment to have enquired about the state size by calling fmi2SerializedFMUstateSize.
#[no_mangle]
/// We assume that the buffer is sufficiently large
pub extern "C" fn fmi2SerializeFMUstate(
    _slave: &Fmi2Slave,
    state: &SlaveState,
    data: *mut u8,
    size: size_t,
) -> Fmi2Status {
    let serialized_state_len = state.bytes.len();

    if serialized_state_len > size {
        error!("Error while calling fmi2SerializeFMUstate: FMUstate too big to be contained in given byte vector.");
        return Fmi2Status::Error;
    }

    unsafe { std::ptr::copy(
        state.bytes.as_ptr(),
        data.cast(),
        serialized_state_len
    ) };

    Fmi2Status::Ok
}

/// # Safety
/// Behavior is undefined if any of the following are violated:
/// * `serialized_state` must be non-null, \[valid\] for reads for
///   `size * mem::size_of::<u8>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `refernces` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `serialized_state` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `serialized_state` must point to `size` consecutive properly initialized values
///   of type `u8`.
/// * The total size `size * mem::size_of::<u8>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `serialized_state` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2DeSerializeFMUstate(
    slave: &mut Fmi2Slave,
    serialized_state: *const u8,
    size: size_t,
    state: &mut Box<Option<SlaveState>>,
) -> Fmi2Status {
    let serialized_state = unsafe { from_raw_parts(serialized_state, size) };

    *state = Box::new(Some(SlaveState::new(serialized_state)));
    Fmi2Status::Ok
}

#[no_mangle]
pub extern "C" fn fmi2SerializedFMUstateSize(
    slave: &Fmi2Slave,
    state: &SlaveState,
    size: &mut size_t,
) -> Fmi2Status {
    *size = state.bytes.len();
    Fmi2Status::Ok
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------
#[no_mangle]
pub extern "C" fn fmi2GetStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Status,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2DoStepStatus => match slave.dostep_status {
            Some(status) => status,
            None => {
                error!("'fmi2GetStatus' called with fmi2StatusKind 'Fmi2DoStepStatus' before 'fmi2DoStep' has returned pending.");
                Fmi2Status::Error
            }
        },
        _ => {
            error!(
                "'fmi2GetStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            );
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if `value` points outside of address space and if it
/// is dereferenced after function call.
#[no_mangle]
pub unsafe extern "C" fn fmi2GetRealStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut c_double,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2LastSuccessfulTime => match slave.last_successful_time {
            Some(last_time) => {
                unsafe {
                    *value = last_time;
                };
                Fmi2Status::Ok
            }
            None => {
                error!("'fmi2GetRealStatus' can not be called before 'Fmi2DoStep'");
                Fmi2Status::Error
            }
        },
        _ => {
            error!(
                "'fmi2GetRealStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            );
            Fmi2Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetIntegerStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2Status {
    error!("No 'fmi2StatusKind' exist for which 'fmi2GetIntegerStatus' can be called");
    Fmi2Status::Error
}

#[no_mangle]
pub extern "C" fn fmi2GetBooleanStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut c_int,
) -> Fmi2Status {
    error!("fmi2GetBooleanStatus is not implemented by UNIFMU.");
    Fmi2Status::Discard
}

#[no_mangle]
pub extern "C" fn fmi2GetStringStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_char,
) -> Fmi2Status {
    error!("fmi2GetStringStatus is not implemented by UNIFMU.");
    Fmi2Status::Error
}

#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::dispatcher::CommandDispatcher;
use libc::c_double;
use libc::size_t;

use subprocess::Popen;
use subprocess::PopenConfig;
use url::Url;

use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_ulonglong;
use std::os::raw::c_void;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;

use crate::config::LaunchConfig;

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
use serde_repr::{Deserialize_repr, Serialize_repr};
#[repr(i32)]
#[derive(
    Debug, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, IntoPrimitive, TryFromPrimitive,
)]

pub enum Fmi2Status {
    Fmi2OK = 0,
    Fmi2Warning = 1,
    Fmi2Discard = 2,
    Fmi2Error = 3,
    Fmi2Fatal = 4,
    Fmi2Pending = 5,
}

#[repr(i32)]
#[derive(
    Debug, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, IntoPrimitive, TryFromPrimitive,
)]
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

// ----------------------- Library instantiation and cleanup ---------------------------

#[repr(C)]
pub struct Slave {
    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to free or overwrite the memory as soon as another call is made to the FMI interface.
    string_buffer: Vec<CString>,

    /// Object performing remote procedure calls on the slave
    dispatcher: Box<CommandDispatcher>,

    popen: Popen,

    last_successful_time: Option<f64>,
    pending_message: Option<String>,
    dostep_status: Option<Fmi2Status>,
}
//  + Send + UnwindSafe + RefUnwindSafe
impl RefUnwindSafe for Slave {}
impl UnwindSafe for Slave {}
unsafe impl Send for Slave {}

impl Slave {
    fn new(dispatcher: Box<CommandDispatcher>, popen: Popen) -> Self {
        Self {
            dispatcher,
            string_buffer: Vec::new(),
            popen,
            last_successful_time: None,
            pending_message: None,
            dostep_status: None,
        }
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

#[no_mangle]
pub static VERSION: &str = "2.0\0";
pub static TYPES_PLATFORM: &str = "default";

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

#[no_mangle]
pub extern "C" fn fmi2Instantiate(
    instance_name: *const c_char, // neither allowed to be null or empty string
    fmu_type: Fmi2Type,
    fmu_guid: *const c_char, // not allowed to be null,
    fmu_resource_location: *const c_char,
    functions: &Fmi2CallbackFunctions,
    visible: c_int,
    logging_on: c_int,
) -> Option<Box<Slave>> {
    let resource_uri = unsafe {
        match fmu_resource_location.as_ref() {
            Some(b) => match CStr::from_ptr(b).to_str() {
                Ok(s) => match Url::parse(s) {
                    Ok(url) => url,
                    Err(e) => panic!("unable to parse resource url"),
                },
                Err(e) => panic!("resource url was not valid utf-8"),
            },
            None => panic!("fmuResourcesLocation was null"),
        }
    };

    let resources_dir = resource_uri.to_file_path().expect(&format!(
        "URI was parsed but could not be converted into a file path, got: '{:?}'.",
        resource_uri
    ));

    let config_path = resources_dir.join("launch.toml");

    let config = read_to_string(&config_path).expect(&format!(
             "Unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
             config_path
         ));

    let config: LaunchConfig = toml::from_str(config.as_str())
        .expect("configuration file was opened, but the contents does not appear to be valid");

    let mut dispatcher = Box::new(CommandDispatcher::new("tcp://127.0.0.1:0"));

    let endpoint = dispatcher.endpoint.to_owned();
    let endpoint_port = endpoint
        .split(":")
        .last()
        .expect("There should be a port after the colon")
        .to_owned();

    // set environment variables
    let mut env_vars: Vec<(OsString, OsString)> = std::env::vars_os().collect();

    env_vars.push((
        OsString::from("UNIFMU_DISPATCHER_ENDPOINT"),
        OsString::from(endpoint),
    ));
    env_vars.push((
        OsString::from("UNIFMU_DISPATCHER_ENDPOINT_PORT"),
        OsString::from(endpoint_port),
    ));

    // grab launch command for host os
    let launch_command = match std::env::consts::OS {
        "windows" => match config.windows {
            Some(cmd) => cmd,
            None => return None,
        },
        "macos" => match config.macos {
            Some(cmd) => cmd,
            None => return None,
        },
        _other => match config.linux {
            Some(cmd) => cmd,
            None => return None,
        },
    };

    // spawn process
    let popen = match Popen::create(
        &launch_command,
        PopenConfig {
            cwd: Some(resources_dir.as_os_str().to_owned()),
            env: Some(env_vars),
            ..Default::default()
        },
    ) {
        Ok(popen) => popen,
        Err(e) => {
            eprintln!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", launch_command);
            return None;
        }
    };

    match dispatcher.await_handshake() {
        Ok(handshake) => (),
        Err(e) => {
            eprint!("Error ocurred during handshake");
            return None;
        }
    };

    dispatcher
        .fmi2Instantiate(
            "instance_name",
            fmu_type,
            "fmu_guid",
            "fmu_resources_location",
            false,
            false,
        )
        .unwrap();

    Some(Box::new(Slave::new(dispatcher, popen)))
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(slave: Option<Box<Slave>>) {
    let mut slave = slave;

    match slave.as_mut() {
        Some(s) => {
            match s.dispatcher.fmi2FreeInstance() {
                Ok(result) => (),
                Err(e) => eprintln!("An error ocurred when freeing slave"),
            };

            drop(slave)
        }
        None => {}
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    slave: &mut Slave,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const *const c_char,
) -> Fmi2Status {
    let categories: Vec<String> = unsafe {
        core::slice::from_raw_parts(categories, n_categories)
            .iter()
            .map(|s| CStr::from_ptr(*s).to_str().unwrap().to_owned())
            .collect()
    };

    slave
        .dispatcher
        .fmi2SetDebugLogging(&categories, logging_on != 0)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2SetupExperiment(
    slave: &mut Slave,
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

    slave
        .dispatcher
        .fmi2SetupExperiment(start_time, stop_time, tolerance)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(slave: &mut Slave) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2EnterInitializationMode()
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(slave: &mut Slave) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2ExitInitializationMode()
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(slave: &mut Slave) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2Terminate()
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2Reset(slave: &mut Slave) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2Reset()
        .unwrap_or(Fmi2Status::Fmi2Error)
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2DoStep(
    slave: &mut Slave,
    current_time: c_double,
    step_size: c_double,
    no_step_prior: c_int,
) -> Fmi2Status {
    match slave
        .dispatcher
        .fmi2DoStep(current_time, step_size, no_step_prior != 0)
    {
        Ok(s) => match s {
            Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => {
                slave.last_successful_time = Some(current_time + step_size);
                s
            }
            s => s,
        },
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(slave: &mut Slave) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2CancelStep()
        .unwrap_or(Fmi2Status::Fmi2Error)
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetReal(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    match slave.dispatcher.fmi2GetReal(references) {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetInteger(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    match slave.dispatcher.fmi2GetInteger(references) {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetBoolean(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    match slave.dispatcher.fmi2GetBoolean(references) {
        Ok((status, values)) => {
            match values {
                Some(values) => {
                    let values: Vec<i32> = values
                        .iter()
                        .map(|v| match v {
                            false => 0,
                            true => 1,
                        })
                        .collect();
                    values_out.copy_from_slice(&values)
                }
                None => (),
            };
            status
        }
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We choose to do it on an instance basis, i.e. each instance has its own string buffer.

#[no_mangle]
pub extern "C" fn fmi2GetString(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut *const c_char,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };

    match slave.dispatcher.fmi2GetString(references) {
        Ok((status, vals)) => {
            match vals {
                Some(vals) => {
                    slave.string_buffer = vals
                        .iter()
                        .map(|s| CString::new(s.as_bytes()).unwrap())
                        .collect();

                    unsafe {
                        for (idx, cstr) in slave.string_buffer.iter().enumerate() {
                            std::ptr::write(values.offset(idx as isize), cstr.as_ptr());
                        }
                    }
                }
                None => (),
            };
            status
        }
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetReal(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) };
    let values = unsafe { from_raw_parts(values, nvr) };

    slave
        .dispatcher
        .fmi2SetReal(references, values)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2SetInteger(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) };
    let values = unsafe { from_raw_parts(values, nvr) };

    slave
        .dispatcher
        .fmi2SetInteger(references, values)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.

#[no_mangle]
pub extern "C" fn fmi2SetBoolean(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let values: Vec<bool> = unsafe { from_raw_parts(values, nvr) }
        .iter()
        .map(|v| *v != 0)
        .collect();

    slave
        .dispatcher
        .fmi2SetBoolean(references, &values)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2SetString(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) };

    let values: Vec<String> = unsafe {
        from_raw_parts(values, nvr)
            .iter()
            .map(|v| CStr::from_ptr(*v).to_str().unwrap().to_owned())
            .collect()
    };
    slave
        .dispatcher
        .fmi2SetString(references, &values)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetDirectionalDerivative(
    slave: &mut Slave,
    unknown_refs: *const c_uint,
    nvr_unknown: size_t,
    known_refs: *const c_uint,
    nvr_known: size_t,
    direction_known: *const c_double,
    direction_unknown: *mut c_double,
) -> Fmi2Status {
    let references_unknown = unsafe { from_raw_parts(unknown_refs, nvr_known) };
    let references_known = unsafe { from_raw_parts(known_refs, nvr_known) };
    let direction_known = unsafe { from_raw_parts(direction_known, nvr_known) };
    let direction_unknown = unsafe { from_raw_parts_mut(direction_unknown, nvr_known) };

    match slave.dispatcher.fmi2GetDirectionalDerivative(
        references_known,
        references_unknown,
        direction_known,
    ) {
        Ok((status, values)) => match values {
            Some(values) => {
                direction_unknown.copy_from_slice(&values);
                status
            }
            None => todo!(),
        },
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetRealInputDerivatives(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const c_int,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let orders = unsafe { from_raw_parts(orders, nvr) };
    let values = unsafe { from_raw_parts(values, nvr) };

    slave
        .dispatcher
        .fmi2SetRealInputDerivatives(references, orders, values)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

#[no_mangle]
pub extern "C" fn fmi2GetRealOutputDerivatives(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const c_int,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) };
    let orders = unsafe { from_raw_parts(orders, nvr) };
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    match slave
        .dispatcher
        .fmi2GetRealOutputDerivatives(references, orders)
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi2Status::Fmi2Error,
    }
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2SetFMUstate(slave: &mut Slave, state: &SlaveState) -> Fmi2Status {
    slave
        .dispatcher
        .fmi2ExtDeserializeSlave(&state.bytes)
        .unwrap_or(Fmi2Status::Fmi2Error)
}

//
#[no_mangle]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub extern "C" fn fmi2GetFMUstate(slave: &mut Slave, state: &mut Option<SlaveState>) -> Fmi2Status {
    match slave.dispatcher.unifmuSerialize() {
        Ok((status, bytes)) => match state.as_mut() {
            Some(state) => {
                state.bytes = bytes;
                status
            }
            None => {
                *state = Some(SlaveState::new(&bytes));
                status
            }
        },
        Err(e) => Fmi2Status::Fmi2Error,
    }
}
/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification

#[no_mangle]
pub extern "C" fn fmi2FreeFMUstate(
    slave: &mut Slave,
    state: Option<Box<SlaveState>>,
) -> Fmi2Status {
    match state {
        Some(s) => drop(s),
        None => {}
    }
    Fmi2Status::Fmi2OK
}

/// Copies the state of a slave into a buffer provided by the environment
///
/// Oddly, the length of the buffer is also provided,
/// as i would expect the environment to have enquired about the state size by calling fmi2SerializedFMUstateSize.

#[no_mangle]
/// We assume that the buffer is sufficiently large
pub extern "C" fn fmi2SerializeFMUstate(
    _slave: &Slave,
    state: &SlaveState,
    data: *mut u8,
    size: size_t,
) -> Fmi2Status {
    let serialized_state_len = state.bytes.len();

    if serialized_state_len > size {
        return Fmi2Status::Fmi2Error;
    }

    unsafe { std::ptr::copy(state.bytes.as_ptr(), data.cast(), serialized_state_len) };

    Fmi2Status::Fmi2OK
}

//
// #[repr(C)]

#[no_mangle]
pub extern "C" fn fmi2DeSerializeFMUstate(
    slave: &mut Slave,
    serialized_state: *const u8,
    size: size_t,
    state: &mut Box<Option<SlaveState>>,
) -> Fmi2Status {
    let serialized_state = unsafe { from_raw_parts(serialized_state, size) };

    *state = Box::new(Some(SlaveState::new(serialized_state)));
    Fmi2Status::Fmi2OK
}

#[no_mangle]
pub extern "C" fn fmi2SerializedFMUstateSize(
    slave: &Slave,
    state: &SlaveState,
    size: &mut size_t,
) -> Fmi2Status {
    *size = state.bytes.len();
    Fmi2Status::Fmi2OK
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetStatus(
    slave: &mut Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Status,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2DoStepStatus => match slave.dostep_status {
            Some(s) => s,
            None => {
                eprintln!("'fmi2GetStatus' called with fmi2StatusKind 'Fmi2DoStepStatus' before 'fmi2DoStep' has returned pending.");
                Fmi2Status::Fmi2Error
            }
        },
        _ => {
            eprintln!(
                "'fmi2GetStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            );
            return Fmi2Status::Fmi2Error;
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetRealStatus(
    slave: &mut Slave,
    status_kind: Fmi2StatusKind,
    value: *mut c_double,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2LastSuccessfulTime => match slave.last_successful_time {
            Some(last_time) => {
                unsafe {
                    *value = last_time;
                };
                Fmi2Status::Fmi2OK
            }
            None => {
                eprintln!("'fmi2GetRealStatus' can not be called before 'Fmi2DoStep'");
                Fmi2Status::Fmi2Error
            }
        },
        _ => {
            eprintln!(
                "'fmi2GetRealStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            );
            return Fmi2Status::Fmi2Error;
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetIntegerStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2Status {
    eprintln!("No 'fmi2StatusKind' exist for which 'fmi2GetIntegerStatus' can be called");
    return Fmi2Status::Fmi2Error;
}

#[no_mangle]
pub extern "C" fn fmi2GetBooleanStatus(
    slave: &mut Slave,
    status_kind: Fmi2StatusKind,
    value: *mut c_int,
) -> Fmi2Status {
    eprintln!("Not currently implemented by UniFMU");
    return Fmi2Status::Fmi2Discard;
}

#[no_mangle]
pub extern "C" fn fmi2GetStringStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_char,
) -> Fmi2Status {
    todo!();

    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

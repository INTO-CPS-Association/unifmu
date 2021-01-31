#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(dead_code)]

use libc::c_double;
use libc::size_t;

use ::safer_ffi::prelude::*;
use num_enum::TryFromPrimitive;
use rpc::{config::RpcConfig, initialize_slave_from_config, Fmi2CommandRPC};
use safer_ffi::{
    c,
    char_p::{char_p_raw, char_p_ref},
};

use std::ffi::CString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_ulonglong;
use std::os::raw::c_void;
use std::panic::UnwindSafe;
use std::{ffi::CStr, panic::RefUnwindSafe};

use toml;
use url::Url;
pub mod rpc;

///
/// Represents the function signature of the logging callback function passsed
/// from the envrionment to the slave during instantiation.
pub type Fmi2CallbackLogger = extern "C" fn(
    component_environment: *mut c_void,
    instance_name: char_p_raw,
    status: Fmi2Status,
    category: char_p_raw,
    message: char_p_raw,
    // ... variadic functions support in rust seems to be unstable
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
#[derive_ReprC]
#[ReprC::opaque]
pub struct ComponentEnvironment {
    _private: [u8; 0],
}

#[derive_ReprC]
#[repr(C)]
/// A set of callback functions provided by the environment
/// Note that all but the 'logger' function are optional and may only be used if the appropriate
/// flags are set in the 'modelDescription.xml' file
pub struct Fmi2CallbackFunctions {
    pub logger: Fmi2CallbackLogger,
    pub allocate_memory: Option<Fmi2CallbackAllocateMemory>,
    pub free_memory: Option<Fmi2CallbackFreeMemory>,
    pub step_finished: Option<Fmi2StepFinished>,
    pub component_environment: &'static Option<repr_c::Box<ComponentEnvironment>>,
}

// ====================== config =======================

/// Represents the possible status codes which are returned from the slave

#[derive_ReprC]
#[repr(i32)]
#[derive(serde::Deserialize, TryFromPrimitive, Debug, PartialEq)]
pub enum Fmi2Status {
    Fmi2OK,
    Fmi2Warning,
    Fmi2Discard,
    Fmi2Error,
    Fmi2Fatal,
    Fmi2Pending,
}

#[derive_ReprC]
#[repr(i32)]
pub enum Fmi2StatusKind {
    Fmi2DoStepStatus = 0,
    Fmi2PendingStatus = 1,
    Fmi2LastSuccessfulTime = 2,
    Fmi2Terminated = 3,
}

#[derive_ReprC]
#[repr(i32)]
pub enum Fmi2Type {
    Fmi2ModelExchange = 0,
    Fmi2CoSimulation = 1,
}

// ----------------------- Library instantiation and cleanup ---------------------------

#[derive_ReprC]
#[ReprC::opaque]
pub struct Slave {
    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to  free or overwrite the memory as soon as another call is made to the FMI interface.
    string_buffer: Vec<CString>,

    /// Buffer storing 0 or more past state of the slave.

    /// Object performing remote procedure calls on the slave
    rpc: Box<dyn Fmi2CommandRPC + Send + UnwindSafe + RefUnwindSafe>,
}

impl Slave {
    fn new(rpc: Box<dyn Fmi2CommandRPC + Send + UnwindSafe + RefUnwindSafe>) -> Self {
        Self {
            rpc,
            string_buffer: Vec::new(),
        }
    }
}

#[derive_ReprC]
#[ReprC::opaque]
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

// trait InsertNext<V> {
//     /// Insert value into map at the next available entry and return a handle to the element
//     fn insert_next(&mut self, value: V) -> Result<i32, String>;
// }

// impl<V> InsertNext<V> for HashMap<i32, V> {
//     fn insert_next(&mut self, value: V) -> Result<i32, String> {
//         for i in 0..std::i32::MAX {
//             if !self.contains_key(&i) {
//                 self.insert(i, value);
//                 return Ok(i);
//             }
//         }

//         Err(String::from("No free keys available"))
//     }
// }

// -------------------------------------------------------------------------------------------------------------

// ------------------------------------- FMI FUNCTIONS --------------------------------
#[ffi_export]
pub fn fmi2GetTypesPlatform() -> char_p_ref<'static> {
    c!("default")
}

#[ffi_export]
pub fn fmi2GetVersion() -> char_p_ref<'static> {
    c!("2.0")
}

// ------------------------------------- FMI FUNCTIONS (Life-Cycle) --------------------------------

/// Instantiates a slave instance by invoking a command in a new process
/// the command is specified by the configuration file, launch.toml, that should be located in the resources directory
/// fmi-commands are sent between the wrapper and slave(s) using a message queue library, specifically zmq.
///
/// The protocol for instantiating a slave can be defined as:
/// 1. read the launch.toml file
/// 2. wrapper creates a single handshake socket
/// 2. wrapper invokes the launch-command defined for the specific OS, the handshake-endpoint is appended to the defined command
/// 3. slave opens two socket, handshake and command
/// 4. slave uses handshake-socket to send the a endpoint of the command socket back to the wrapper
///
///
/// Now the connection has been establihed between the wrapper and the newly instantiated slave.
/// The protocol for sending fmi-commands between wrapper and slave can be defined as
/// 1. C-API fmi-function is invoked on wrapper
/// 2. C-types are converted into native Rust-types
/// 3. Rust types are serialized
/// 4. Wrapper sends message to command_socket of slave
/// 5. Slave deserializes message and responds
/// 6. step 4+5 are repeated until fmi2FreeInstance
///
/// Notes:
/// * The slave choses decides the following:
///     * transport layer
///     * port and endpoint
///     * serialization format (Json, FlexBuffers, Pickle, etc)
///

#[ffi_export]
pub fn fmi2Instantiate<'callback>(
    _instance_name: char_p_ref, // not allowed to be null, also cannot be non-empty
    _fmu_type: Fmi2Type,
    _fmu_guid: char_p_ref, // not allowed to be null,
    fmu_resource_location: char_p_ref,
    _functions: &Fmi2CallbackFunctions,
    _visible: c_int,
    _logging_on: c_int,
) -> Option<repr_c::Box<Slave>> {
    let resource_uri = Url::parse(fmu_resource_location.to_str()).expect(&format!(
        "Unable to parse the specified file URI, got: '{:?}'.",
        fmu_resource_location,
    ));

    let resources_dir = resource_uri.to_file_path().expect(&format!(
        "URI was parsed but could not be converted into a file path, got: '{:?}'.",
        resource_uri
    ));

    let config_path = resources_dir.join("launch.toml");

    let config = read_to_string(&config_path).expect(&format!(
            "unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
            config_path
        ));

    let config: RpcConfig = toml::from_str(config.as_str())
        .expect("configuration file was opened, but contents were not valid toml");

    // creating a handshake-socket which is used by the slave-process to pass connection strings back to the wrapper

    let rpc = initialize_slave_from_config(config, resources_dir).expect(&format!(
            "Something went wrong during instantiation of the slave using the configuration defined in 'launch.toml' file. Common causes are missing runtime dependencies or incorrect paths."
        ));

    Some(repr_c::Box::new(Slave::new(rpc)))
}

// #[ffi_export] temporarily disabled macro, see issue: https://github.com/getditto/safer_ffi/issues/30
pub fn fmi2FreeInstance(mut slave: Option<repr_c::Box<Slave>>) {
    match slave.as_mut() {
        Some(s) => {
            s.rpc.fmi2FreeInstance();
            drop(slave)
        }
        None => {}
    }
}

#[ffi_export]
pub fn fmi2SetDebugLogging<'call>(
    slave: &mut Slave,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const char_p_ref,
) -> Fmi2Status {
    let categories: Vec<&str> = unsafe { core::slice::from_raw_parts(categories, n_categories) }
        .iter()
        .map(|s| s.to_str())
        .collect();

    slave.rpc.fmi2SetDebugLogging(&categories, logging_on == 1)
}

#[ffi_export]
pub fn fmi2SetupExperiment(
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
        .rpc
        .fmi2SetupExperiment(start_time, stop_time, tolerance)
}

#[ffi_export]
pub fn fmi2EnterInitializationMode(slave: &mut Slave) -> Fmi2Status {
    slave.rpc.fmi2EnterInitializationMode()
}

#[ffi_export]
pub fn fmi2ExitInitializationMode(slave: &mut Slave) -> Fmi2Status {
    slave.rpc.fmi2ExitInitializationMode()
}

#[ffi_export]
pub fn fmi2Terminate(slave: &mut Slave) -> Fmi2Status {
    slave.rpc.fmi2Terminate()
}

#[ffi_export]
pub fn fmi2Reset(slave: &mut Slave) -> Fmi2Status {
    slave.rpc.fmi2Reset()
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[ffi_export]
pub fn fmi2DoStep(
    slave: &mut Slave,
    current: c_double,
    step_size: c_double,
    no_set_prior: c_int,
) -> Fmi2Status {
    slave.rpc.fmi2DoStep(current, step_size, no_set_prior == 1)
}

#[ffi_export]
pub fn fmi2CancelStep(slave: &mut Slave) -> Fmi2Status {
    slave.rpc.fmi2CancelStep()
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

#[ffi_export]
pub fn fmi2GetReal(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) };

    let (status, values_recv) = slave.rpc.fmi2GetReal(references);

    // copy if status is less severe than discard "discard"
    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => {
            unsafe {
                std::ptr::copy(values_recv.unwrap().as_ptr(), values, nvr);
            };
        }
        _ => {}
    }

    status
}

#[ffi_export]
pub fn fmi2GetInteger(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let (status, values_recv) = slave.rpc.fmi2GetInteger(references);

    // copy if status is less severe than discard "discard"
    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => unsafe {
            std::ptr::copy(values_recv.unwrap().as_ptr(), values, nvr);
        },
        _ => {}
    }

    status
}

#[ffi_export]
pub fn fmi2GetBoolean(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let (status, values_recv) = slave.rpc.fmi2GetBoolean(references);

    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => unsafe {
            for (idx, v) in values_recv.unwrap().iter().enumerate() {
                std::ptr::write(values.offset(idx as isize), *v as c_int);
            }
        },
        _ => {}
    }

    status
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We choose to do it on an instance basis, i.e. each instance has its own string buffer.
#[ffi_export]
pub fn fmi2GetString<'call>(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut *const c_char,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) };

    let (status, values_recv) = slave.rpc.fmi2GetString(references);

    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => {
            // Convert rust strings to owned c-strings and store in a buffer
            slave.string_buffer = values_recv
                .unwrap()
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect::<Vec<_>>();

            // write pointers to the newly allocated strings into the buffer allocated above
            unsafe {
                for (idx, cstr) in slave.string_buffer.iter().enumerate() {
                    std::ptr::write(values.offset(idx as isize), cstr.as_ptr());
                }
            }
        }
        _ => {}
    }

    status
}

#[ffi_export]
pub fn fmi2SetReal(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
    let values = unsafe { std::slice::from_raw_parts(values, nvr) };

    slave.rpc.fmi2SetReal(references, values)
}

#[ffi_export]

pub fn fmi2SetInteger(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
    let values = unsafe { std::slice::from_raw_parts(values, nvr) };

    slave.rpc.fmi2SetInteger(references, values)
}

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
#[ffi_export]
pub fn fmi2SetBoolean(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) };
    let values: Vec<bool> = unsafe { std::slice::from_raw_parts(values, nvr) }
        .iter()
        .map(|v| *v != 0)
        .collect();

    slave.rpc.fmi2SetBoolean(references, &values)
}

#[ffi_export]
pub fn fmi2SetString(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let mut values_vec: Vec<&str> = Vec::with_capacity(nvr);

    unsafe {
        for i in 0..nvr {
            let cstr = CStr::from_ptr(*values.offset(i as isize))
                .to_str()
                .expect("Unable to convert C-string to Rust compatible string");
            values_vec.insert(i, cstr);
        }
    }

    slave.rpc.fmi2SetString(references, &values_vec)
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

#[ffi_export]
pub fn fmi2GetDirectionalDerivative(
    c: *const c_int,
    unknown_refs: *const c_int,
    nvr_unknown: size_t,
    known_refs: *const c_int,
    nvr_known: size_t,
    values_known: *const c_double,
    values_unkown: *mut c_double,
) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

#[ffi_export]
pub fn fmi2SetRealInputDerivatives(slave: &mut Slave, vr: *const c_uint) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

#[ffi_export]
pub fn fmi2GetRealOutputDerivatives(slave: &mut Slave) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------

#[ffi_export]
pub fn fmi2SetFMUstate(slave: &mut Slave, state: &SlaveState) -> Fmi2Status {
    slave.rpc.deserialize(state.bytes.as_slice().into())
}

//#[ffi_export]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub fn fmi2GetFMUstate(slave: &mut Slave, state: &mut Option<SlaveState>) -> Fmi2Status {
    let (status, bytes) = slave.rpc.serialize();

    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => {
            let bytes = bytes.unwrap();

            // Whether a new buffer should be allocated depends on state's value:
            // * state points to null: allocate a new buffer and return a pointer to this
            // * state points to existing state: overwrite that buffer with current state
            match state.as_mut() {
                Some(s) => s.bytes = bytes,
                None => *state = Some(SlaveState::new(&bytes)),
            }
        }
        _ => {}
    }

    status
}
/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
#[ffi_export]
pub fn fmi2FreeFMUstate(slave: &mut Slave, state: Option<repr_c::Box<SlaveState>>) -> Fmi2Status {
    match state {
        Some(s) => drop(s),
        None => {}
    }
    Fmi2Status::Fmi2OK
}

#[ffi_export]
/// Copies the state of a slave into a buffer provided by the environment
///
/// Oddly, the length of the buffer is also provided,
/// as i would expect the environment to have enquired about the state size by calling fmi2SerializedFMUstateSize.
/// We assume that the buffer is sufficiently large
pub fn fmi2SerializeFMUstate(
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

//#[ffi_export]
pub fn fmi2DeSerializeFMUstate(
    slave: &mut Slave,
    serialized_state: *const u8,
    size: size_t,
    state: &mut repr_c::Box<Option<SlaveState>>,
) -> Fmi2Status {
    let serialized_state = unsafe { std::slice::from_raw_parts(serialized_state, size) };

    *state = repr_c::Box::new(Some(SlaveState::new(serialized_state)));
    Fmi2Status::Fmi2OK
}

#[ffi_export]
pub fn fmi2SerializedFMUstateSize(
    slave: &Slave,
    state: &SlaveState,
    size: &mut size_t,
) -> Fmi2Status {
    *size = state.bytes.len();
    Fmi2Status::Fmi2OK
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------

#[ffi_export]
pub fn fmi2GetRealStatus(
    slave: &mut Slave,
    status_kind: c_int,
    value: *mut c_double,
) -> Fmi2Status {
    todo!();

    // match catch_unwind(|| {
    //     let handle = unsafe { *slave_handle };
    //     let slaves = SLAVES.lock().unwrap();
    //     let slave = slaves.get(&handle).unwrap();

    //     let (status_value, status) = execute_fmi_command_return::<_, (f64, i32)>(
    //         slave,
    //         (FMI2FunctionCode::GetXXXStatus, status_kind),
    //     )
    //     .unwrap();
    //     unsafe { *value = status_value };
    //     status
    // }) {
    //     Ok(s) => s,
    //     Err(_) => Fmi2Status::Fmi2Fatal as i32,
    // }
}

#[ffi_export]
pub fn fmi2GetStatus(
    slave_handle: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2Status {
    todo!();

    // match catch_unwind(|| {
    //     let handle = unsafe { *slave_handle };
    //     let slaves = SLAVES.lock().unwrap();
    //     let slave = slaves.get(&handle).unwrap();

    //     let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
    //         slave,
    //         (FMI2FunctionCode::GetXXXStatus, status_kind),
    //     )
    //     .unwrap();
    //     unsafe { *value = status_value };
    //     status
    // }) {
    //     Ok(s) => s,
    //     Err(_) => Fmi2Status::Fmi2Fatal as i32,
    // }
}

#[ffi_export]
pub fn fmi2GetIntegerStatus(c: *const c_int, status_kind: c_int, value: *mut c_int) -> Fmi2Status {
    todo!();

    // match catch_unwind(|| {
    //     let handle = unsafe { *c };
    //     let slaves = SLAVES.lock().unwrap();
    //     let slave = slaves.get(&handle).unwrap();

    //     let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
    //         slave,
    //         (FMI2FunctionCode::GetXXXStatus, status_kind),
    //     )
    //     .unwrap();
    //     unsafe { *value = status_value };
    //     status
    // }) {
    //     Ok(s) => s,
    //     Err(_) => Fmi2Status::Fmi2Fatal as i32,
    // }
}

#[ffi_export]
pub fn fmi2GetBooleanStatus(c: *const c_int, status_kind: c_int, value: *mut c_int) -> Fmi2Status {
    todo!();

    // match catch_unwind(|| {
    //     let handle = unsafe { *c };
    //     let slaves = SLAVES.lock().unwrap();
    //     let slave = slaves.get(&handle).unwrap();

    //     let (status_value, status) = execute_fmi_command_return::<_, (bool, i32)>(
    //         slave,
    //         (FMI2FunctionCode::GetXXXStatus, status_kind),
    //     )
    //     .unwrap();
    //     unsafe { *value = status_value as i32 };
    //     status
    // }) {
    //     Ok(s) => s,
    //     Err(_) => Fmi2Status::Fmi2Fatal as i32,
    // }
}

#[ffi_export]
pub fn fmi2GetStringStatus(c: *const c_int, status_kind: c_int, value: *mut c_char) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

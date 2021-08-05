#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub mod config;

use common::Fmi2Status;
use libc::c_double;
use libc::size_t;

use ::safer_ffi::prelude::*;
use rpc::socket_dispatcher::new_boxed_socket_dispatcher;
use rpc::Fmi2Command;
use rpc::Fmi2CommandDispatcher;
use safer_ffi::{
    c,
    char_p::{char_p_raw, char_p_ref},
};
use subprocess::Popen;
use subprocess::PopenConfig;
use url::Url;

use std::ffi::CStr;
use std::ffi::CString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_ulonglong;
use std::os::raw::c_void;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

use crate::config::LaunchConfig;

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
    /// The reason for this recommendation is that a FMU is allowed to free or overwrite the memory as soon as another call is made to the FMI interface.
    string_buffer: Vec<CString>,

    /// Object performing remote procedure calls on the slave
    dispatcher: Box<dyn Fmi2CommandDispatcher>,

    popen: Popen,
}
//  + Send + UnwindSafe + RefUnwindSafe
impl RefUnwindSafe for Slave {}
impl UnwindSafe for Slave {}
unsafe impl Send for Slave {}

impl Slave {
    fn new(dispatcher: Box<dyn Fmi2CommandDispatcher>, popen: Popen) -> Self {
        Self {
            dispatcher,
            string_buffer: Vec::new(),
            popen,
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
pub fn fmi2Instantiate(
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
             "Unable to read configuration file stored at path: '{:?}', ensure that 'launch.toml' exists in the resources directory of the fmu.",
             config_path
         ));

    let config: LaunchConfig = toml::from_str(config.as_str())
        .expect("configuration file was opened, but the contents does not appear to be valid");

    let (command, mut dispatcher) = match config.active_dispatcher {
        config::ActiveDispatcher::Socket => {
            let cfg = config.socket_config.unwrap();
            match cfg.socket_type {
                config::SocketType::Zmq => {
                    let (endpoint, dispatcher) = new_boxed_socket_dispatcher(cfg.format);

                    let mut cmds = match std::env::consts::OS {
                        "windows" => cfg.windows,
                        "macos" => cfg.macos,
                        _ => cfg.linux,
                    };

                    cmds.append(&mut vec![String::from("--dispatcher-endpoint"), endpoint]);

                    (cmds, dispatcher)
                }
            }
        }
        config::ActiveDispatcher::Grpc => todo!(),
    };

    let popen = Popen::create(
        &command,
        PopenConfig {
            cwd: Some(resources_dir.as_os_str().to_owned()),
            ..Default::default()
        },
    )
    .expect(&format!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", command));

    println!("awaiting handshake from slave");
    dispatcher.await_handshake();
    println!("received handshake");

    Some(repr_c::Box::new(Slave::new(dispatcher, popen)))
}

#[ffi_export]
pub extern "C" fn fmi2FreeInstance(slave: Option<repr_c::Box<Slave>>) {
    let mut slave = slave; // see issue: https://github.com/getditto/safer_ffi/issues/30

    match slave.as_mut() {
        Some(s) => {
            s.dispatcher.invoke_command(&Fmi2Command::Fmi2FreeInstance);
            drop(slave)
        }
        None => {}
    }
}

#[ffi_export]
pub fn fmi2SetDebugLogging(
    slave: &mut Slave,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const char_p_ref,
) -> Fmi2Status {
    todo!();

    let categories: Vec<&str> = unsafe { core::slice::from_raw_parts(categories, n_categories) }
        .iter()
        .map(|s| s.to_str())
        .collect();
    todo!();
    // slave.rpc.fmi2SetDebugLogging(&categories, logging_on == 1)
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
    println!("doing the setup");
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
        .invoke_command(&Fmi2Command::Fmi2SetupExperiment {
            start_time,
            stop_time,
            tolerance,
        });

    Fmi2Status::Fmi2OK
}

#[ffi_export]
pub fn fmi2EnterInitializationMode(slave: &mut Slave) -> Fmi2Status {
    let ret = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2EnterInitializationMode);

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2ExitInitializationMode(slave: &mut Slave) -> Fmi2Status {
    let ret = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2ExitInitializationMode);

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2Terminate(slave: &mut Slave) -> Fmi2Status {
    let ret = slave.dispatcher.invoke_command(&Fmi2Command::Fmi2Terminate);

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2Reset(slave: &mut Slave) -> Fmi2Status {
    let ret = slave.dispatcher.invoke_command(&Fmi2Command::Fmi2Reset);

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[ffi_export]
pub fn fmi2DoStep(
    slave: &mut Slave,
    current_time: c_double,
    step_size: c_double,
    no_step_prior: c_int,
) -> Fmi2Status {
    let ret = slave.dispatcher.invoke_command(&Fmi2Command::Fmi2DoStep {
        current_time,
        step_size,
        no_step_prior: no_step_prior == 0,
    });

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2CancelStep(slave: &mut Slave) -> Fmi2Status {
    let ret = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2CancelStep);

    match ret {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

#[ffi_export]
pub fn fmi2GetReal(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { std::slice::from_raw_parts_mut(values, nvr) };

    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2GetReal { references });

    match res {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        rpc::Fmi2Return::Fmi2GetRealReturn { status, values } => {
            values_out.copy_from_slice(values.as_slice());
            status
        }
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2GetInteger(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { std::slice::from_raw_parts_mut(values, nvr) };

    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2GetInteger { references });

    match res {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        rpc::Fmi2Return::Fmi2GetIntegerReturn { status, values } => {
            values_out.copy_from_slice(values.as_slice());
            status
        }
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2GetBoolean(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { std::slice::from_raw_parts_mut(values, nvr) };

    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2GetBoolean { references });

    match res {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        rpc::Fmi2Return::Fmi2GetBooleanReturn { status, values } => {
            let values_i32: Vec<i32> = values.iter().map(|v| *v as i32).collect();
            values_out.copy_from_slice(&values_i32);

            status
        }
        _ => Fmi2Status::Fmi2Error,
    }
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We choose to do it on an instance basis, i.e. each instance has its own string buffer.
#[ffi_export]
pub fn fmi2GetString(
    slave: &mut Slave,
    references: *const c_uint,
    nvr: size_t,
    valuess: *mut *const c_char,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(references, nvr) }.to_owned();

    match slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2GetString { references })
    {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        rpc::Fmi2Return::Fmi2GetStringReturn { status, values } => {
            slave.string_buffer = values
                .iter()
                .map(|s| CString::new(s.as_bytes()).unwrap())
                .collect();

            unsafe {
                for (idx, cstr) in slave.string_buffer.iter().enumerate() {
                    std::ptr::write(valuess.offset(idx as isize), cstr.as_ptr());
                }
            }
            status
        }
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2SetReal(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { std::slice::from_raw_parts(values, nvr) }.to_owned();
    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2SetReal { references, values });

    match res {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]

pub fn fmi2SetInteger(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { std::slice::from_raw_parts(values, nvr) }.to_owned();
    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2SetInteger { references, values });

    match res {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
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
    let references = unsafe { std::slice::from_raw_parts(references, nvr) }.to_owned();
    let values_bool: Vec<bool> = unsafe { std::slice::from_raw_parts(values, nvr) }
        .iter()
        .map(|v| *v == 0)
        .collect();

    match slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2SetBoolean {
            references,
            values: values_bool,
        }) {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

#[ffi_export]
pub fn fmi2SetString(
    slave: &mut Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) }.to_owned();

    let values: Vec<String> = unsafe {
        std::slice::from_raw_parts(values, nvr)
            .iter()
            .map(|v| CStr::from_ptr(*v).to_str().unwrap().to_owned())
            .collect()
    };

    match slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2SetString { references, values })
    {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
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
    todo!();

    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

#[ffi_export]
pub fn fmi2SetRealInputDerivatives(slave: &mut Slave, vr: *const c_uint) -> Fmi2Status {
    todo!();

    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

#[ffi_export]
pub fn fmi2GetRealOutputDerivatives(slave: &mut Slave) -> Fmi2Status {
    todo!();

    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------

#[ffi_export]
pub fn fmi2SetFMUstate(slave: &mut Slave, state: &SlaveState) -> Fmi2Status {
    match slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2ExtDeserializeSlave {
            state: state.bytes.to_owned(),
        }) {
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
}

//#[ffi_export]
#[no_mangle]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub extern "C" fn fmi2GetFMUstate(
    slave: &mut Slave,
    statee: &mut Option<SlaveState>,
) -> Fmi2Status {
    let res = slave
        .dispatcher
        .invoke_command(&Fmi2Command::Fmi2ExtSerializeSlave);

    match res {
        rpc::Fmi2Return::Fmi2ExtSerializeSlaveReturn { status, state } => {
            match statee.as_mut() {
                Some(s) => s.bytes = state,
                None => *statee = Some(SlaveState::new(&state)),
            }
            status
        }
        rpc::Fmi2Return::Fmi2StatusReturn { status } => status,
        _ => Fmi2Status::Fmi2Error,
    }
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
// #[repr(C)]
#[no_mangle]
pub extern "C" fn fmi2DeSerializeFMUstate(
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
    todo!();

    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

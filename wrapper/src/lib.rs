#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(dead_code)]

use libc::c_double;
use libc::size_t;

use ::safer_ffi::prelude::*;
use rpc::{config::RpcConfig, initialize_slave_from_config, Fmi2CommandRPC};
use safer_ffi::char_p::char_p_ref;

use std::ffi::CString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_ulonglong;
use std::os::raw::c_void;
use std::panic::catch_unwind;
use std::ptr::null_mut;
use std::result::Result;
use std::sync::Mutex;
use std::{collections::HashMap, panic::UnwindSafe};
use std::{ffi::CStr, panic::RefUnwindSafe};

use lazy_static::lazy_static;

use toml;
use url::Url;
pub mod rpc;

///

/// Represents the function signature of the logging callback function passsed
/// from the envrionment to the slave during instantiation.
pub type Fmi2CallbackLogger = extern "C" fn(
    component_environment: *mut c_void,
    instance_name: *const c_char,
    status: Fmi2Status,
    category: *const c_char,
    message: *const c_char,
    // ... variadic functions support in rust seems to be unstable
);
pub type Fmi2CallbackAllocateMemory = extern "C" fn(nobj: c_ulonglong, size: c_ulonglong);
pub type Fmi2CallbackFreeMemory = extern "C" fn(obj: *const c_void);
pub type Fmi2StepFinished = extern "C" fn(component_environment: *const c_void, status: i32);

#[derive_ReprC]
#[repr(C)]
pub struct Fmi2CallbackFunctions {
    pub logger: Fmi2CallbackLogger,
    pub allocate_memory: Option<Fmi2CallbackAllocateMemory>,
    pub free_memory: Option<Fmi2CallbackFreeMemory>,
    pub step_finished: Fmi2StepFinished,
    pub component_environment: *const c_void,
}

// ====================== config =======================

/// Represents the possible status codes which are returned from the slave
#[derive_ReprC]
#[repr(i32)]
pub enum Fmi2Status {
    Fmi2OK,
    Fmi2Warning,
    Fmi2Discard,
    Fmi2Error,
    Fmi2Fatal,
    Fmi2Pending,
}

impl From<i32> for Fmi2Status {
    fn from(value: i32) -> Self {
        match value {
            0 => Fmi2Status::Fmi2OK,
            1 => Fmi2Status::Fmi2Warning,
            2 => Fmi2Status::Fmi2Discard,
            3 => Fmi2Status::Fmi2Error,
            4 => Fmi2Status::Fmi2Fatal,
            5 => Fmi2Status::Fmi2Pending,
            _ => {
                panic! {"This should never happen!"}
            }
        }
    }
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

/// An identifier that can be used to uniquely identify a slave within the context of a specific backend.
pub type SlaveHandle = i32;
pub type StateHandle = i32;

struct Slave {
    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to  free or overwrite the memory as soon as another call is made to the FMI interface.
    string_buffer: Vec<CString>,

    /// Buffer storing 0 or more past state of the slave.
    serialization_buffer: HashMap<StateHandle, Vec<u8>>,

    /// Object performing remote procedure calls on the slave
    rpc: Box<dyn Fmi2CommandRPC + Send + UnwindSafe + RefUnwindSafe>,
}

impl Slave {
    fn new(rpc: Box<dyn Fmi2CommandRPC + Send + UnwindSafe + RefUnwindSafe>) -> Self {
        Self {
            rpc,
            string_buffer: Vec::new(),
            serialization_buffer: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref SLAVES: Mutex<HashMap<SlaveHandle, Slave>> = Mutex::new(HashMap::new());
}

trait InsertNext<V> {
    /// Insert value into map at the next available entry and return a handle to the element
    fn insert_next(&mut self, value: V) -> Result<i32, String>;
}

impl<V> InsertNext<V> for HashMap<i32, V> {
    fn insert_next(&mut self, value: V) -> Result<i32, String> {
        for i in 0..std::i32::MAX {
            if !self.contains_key(&i) {
                self.insert(i, value);
                return Ok(i);
            }
        }

        Err(String::from("No free keys available"))
    }
}

// -------------------------------------------------------------------------------------------------------------

// ------------------------------------- FMI FUNCTIONS --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetTypesPlatform() -> *const c_char {
    b"default\0".as_ptr().cast()
}

#[no_mangle]
pub extern "C" fn fmi2GetVersion() -> *const c_char {
    b"2.0\0".as_ptr().cast()
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
fn fmi2Instantiate(
    _instance_name: *const c_char, // not allowed to be null, also cannot be non-empty
    _fmu_type: Fmi2Type,
    _fmu_guid: *const c_char, // not allowed to be null,
    fmu_resource_location: *const c_char,
    _functions: &Fmi2CallbackFunctions,
    _visible: c_int,
    _logging_on: c_int,
) -> *const SlaveHandle {
    let panic_result: Result<i32, _> = catch_unwind(|| {
        // To load the fmu we need to extract the resources directory from the uri path to instantiate
        let resource_location_cstr = unsafe { CStr::from_ptr(fmu_resource_location) };
        let resource_location_utf8 = resource_location_cstr.to_str().expect(&format!(
            "Unable to convert resource uri c-string to utf8, got '{:?}'.",
            &resource_location_cstr
        ));

        let resource_uri = Url::parse(&resource_location_utf8).expect(&format!(
            "Unable to parse the specified file URI, got: '{:?}'.",
            resource_location_utf8,
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

        let mut slaves = SLAVES.lock().unwrap();
        slaves.insert_next(Slave::new(rpc)).unwrap()
    });

    match panic_result {
        Ok(h) => Box::into_raw(Box::new(h)),
        Err(_) => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(slave_handle: *mut SlaveHandle) {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2FreeInstance();

    slaves.remove(&handle).expect(&format!("Failed freeing slave with handle id '{:?}. Ensure that it has been instantiated and not previously freed.", handle));
    unsafe { Box::from_raw(slave_handle) }; // Free handle allocated to the heap by fmi2Instantiate
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    slave_handle: *const SlaveHandle,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const *const c_char,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

    let mut categories_vec: Vec<&str> = Vec::with_capacity(n_categories);
    for i in 0..(n_categories as isize) {
        let cat = unsafe { CStr::from_ptr(*categories.offset(i)).to_str().unwrap() };
        categories_vec.push(cat);
    }
    slave
        .rpc
        .fmi2SetDebugLogging(categories_vec, logging_on == 1)
}

#[ffi_export]
fn fmi2SetupExperiment(
    slave_handle: *const SlaveHandle,
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

    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

    slave
        .rpc
        .fmi2SetupExperiment(start_time, stop_time, tolerance)
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(slave_handle: *const SlaveHandle) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2EnterInitializationMode()
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(slave_handle: *const SlaveHandle) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2ExitInitializationMode()
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(slave_handle: *const SlaveHandle) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2Terminate()
}

#[no_mangle]
pub extern "C" fn fmi2Reset(slave_handle: *const SlaveHandle) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2Reset()
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2DoStep(
    slave_handle: *const SlaveHandle,
    current: c_double,
    step_size: c_double,
    no_set_prior: c_int,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    slave.rpc.fmi2DoStep(current, step_size, no_set_prior == 1)
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(slave_handle: *const SlaveHandle) -> Fmi2Status {
    let slave_handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&slave_handle).unwrap();
    slave.rpc.fmi2CancelStep()
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetReal(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

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

#[no_mangle]
pub extern "C" fn fmi2GetInteger(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

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

#[no_mangle]
pub extern "C" fn fmi2GetBoolean(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

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
#[no_mangle]
pub extern "C" fn fmi2GetString(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut *const c_char,
) -> Fmi2Status {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

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

#[no_mangle]
pub extern "C" fn fmi2SetReal(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
    let values = unsafe { std::slice::from_raw_parts(values, nvr) };

    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

    slave.rpc.fmi2SetReal(references, values)
}

#[no_mangle]

pub extern "C" fn fmi2SetInteger(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
    let values = unsafe { std::slice::from_raw_parts(values, nvr) };

    slave.rpc.fmi2SetInteger(references, values)
}

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
#[no_mangle]
pub extern "C" fn fmi2SetBoolean(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
    let values = unsafe { std::slice::from_raw_parts(values, nvr) };

    let values: Vec<bool> = values.iter().map(|v| *v != 0).collect();

    slave.rpc.fmi2SetBoolean(references, &values)
}

#[no_mangle]
pub extern "C" fn fmi2SetString(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();
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

#[no_mangle]
pub extern "C" fn fmi2GetDirectionalDerivative(
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

#[no_mangle]
pub extern "C" fn fmi2SetRealInputDerivatives(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

#[no_mangle]
pub extern "C" fn fmi2GetRealOutputDerivatives(slave_handle: *const SlaveHandle) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2SetFMUstate(
    slave_handle: *const SlaveHandle,
    state_handle: *const StateHandle,
) -> Fmi2Status {
    let slave_handle: i32 = unsafe { *slave_handle };
    let state_handle: i32 = unsafe { *state_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&slave_handle).unwrap();

    let bytes = slave.serialization_buffer.get(&state_handle)
        .expect(&format!("Unable to find state indicated by the handle '{:?}' for slave '{:?}'. Ensure that the state has previously been stored by 'fmi2GetFMUState' or 'fmi2DeSerializeFMUstate'", state_handle, slave_handle));

    slave.rpc.deserialize(bytes)
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub extern "C" fn fmi2GetFMUstate(
    slave_handle: *const c_int,
    state_handle_or_none: *mut *mut SlaveHandle,
) -> Fmi2Status {
    let handle = unsafe { *slave_handle };
    let mut slaves = SLAVES.lock().unwrap();
    let slave = slaves.get_mut(&handle).unwrap();

    let (status, bytes) = slave.rpc.serialize();

    match status {
        Fmi2Status::Fmi2OK | Fmi2Status::Fmi2Warning => {
            let bytes = bytes.unwrap();

            // Whether a new buffer should be allocated depends on state's value:
            // * state points to null: allocate a new buffer and return a pointer to this
            // * state points to existing state: overwrite that buffer with current state
            unsafe {
                match (*state_handle_or_none).as_ref() {
                    Some(h) => {
                        slave.serialization_buffer.insert(*h, bytes);
                    }
                    None => {
                        let state_handle = slave.serialization_buffer.insert_next(bytes).unwrap();
                        std::ptr::write(
                            state_handle_or_none,
                            Box::into_raw(Box::new(state_handle)),
                        );
                    }
                };
            }
        }
        _ => {}
    }

    status
}

#[no_mangle]
/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
pub extern "C" fn fmi2FreeFMUstate(
    slave_handle: *const SlaveHandle,
    state: *mut *mut StateHandle,
) -> Fmi2Status {
    match catch_unwind(|| match unsafe { state.as_ref() } {
        None => (),
        Some(s) => {
            let slave_handle = unsafe { *slave_handle };
            let state_handle = unsafe { *(*s as *mut i32) };
            let mut slaves = SLAVES.lock().unwrap();
            let slave = slaves.get_mut(&slave_handle).unwrap();

            slave.serialization_buffer.remove(&state_handle)
            .expect(&format!("Unable to free FMU state indicated by handle {:?} for slave {:?}. Ensure that it has not already been freed",state_handle,slave_handle));

            unsafe {
                Box::from_raw(*state);
                *state = null_mut()
            };
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK,
        Err(_) => Fmi2Status::Fmi2Fatal,
    }
}

#[no_mangle]
/// Copies the state of a slave into a buffer provided by the environment
///
/// Oddly, the length of the buffer is also provided,
/// as i would expect the environment to have enquired about the state size by calling fmi2SerializedFMUstateSize.
/// We assume that the buffer is sufficiently large
pub extern "C" fn fmi2SerializeFMUstate(
    slave_handle: *const SlaveHandle,
    state_handle: *mut StateHandle,
    data: *mut c_char,
    _size: size_t,
) -> Fmi2Status {
    match catch_unwind(|| {
        let slave_handle: i32 = unsafe { *slave_handle };
        let state_handle: i32 = unsafe { *state_handle };
        let slaves = SLAVES.lock().unwrap();

        let bytes = slaves
            .get(&slave_handle)
            .unwrap()
            .serialization_buffer
            .get(&state_handle)
            .unwrap();

        unsafe { std::ptr::copy(bytes.as_ptr(), data.cast(), bytes.len()) };
    }) {
        Ok(_) => Fmi2Status::Fmi2OK,
        Err(_) => Fmi2Status::Fmi2Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi2DeSerializeFMUstate(
    slave_handle: *const SlaveHandle,
    serialized_state: *const c_char,
    size: size_t,
    state: *mut *mut StateHandle,
) -> Fmi2Status {
    match catch_unwind(|| {
        let bytes: Vec<u8> = unsafe {
            std::ptr::slice_from_raw_parts(serialized_state.cast(), size)
                .as_ref()
                .unwrap()
                .to_vec()
        };

        let slave_handle: i32 = unsafe { *slave_handle };
        let mut slaves = SLAVES.lock().unwrap();
        let slave = slaves.get_mut(&slave_handle).unwrap();

        let state_handle = slave.serialization_buffer.insert_next(bytes).unwrap();
        unsafe {
            *state = Box::into_raw(Box::new(state_handle));
        };
    }) {
        Ok(_) => Fmi2Status::Fmi2OK,
        Err(_) => Fmi2Status::Fmi2Fatal,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SerializedFMUstateSize(
    slave_handle: *const SlaveHandle,
    state_handle: *const StateHandle,
    size: *mut size_t,
) -> Fmi2Status {
    match catch_unwind(|| {
        let slave_handle: i32 = unsafe { *slave_handle };
        let state_handle: i32 = unsafe { *state_handle };
        let slaves = SLAVES.lock().unwrap();

        let state_size = slaves
            .get(&slave_handle)
            .unwrap()
            .serialization_buffer
            .get(&state_handle)
            .unwrap()
            .len();

        unsafe {
            *size = state_size;
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK,
        Err(_) => Fmi2Status::Fmi2Fatal,
    }
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetRealStatus(
    slave_handle: *const SlaveHandle,
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

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStatus(
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

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetIntegerStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2Status {
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

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetBooleanStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2Status {
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

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStringStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_char,
) -> Fmi2Status {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error
}

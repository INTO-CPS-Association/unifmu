#![allow(non_snake_case)]
use crate::config::FullConfig;
use crate::config::LaunchConfig;
use crate::fmi2::Fmi2Status;
use libc::c_double;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_void;
use std::panic::catch_unwind;
use std::ptr::null_mut;
use std::result::Result;
use std::sync::atomic::AtomicI32;
use std::sync::Mutex;
use std::time::Duration;

use lazy_static::lazy_static;
use subprocess::Popen;
use subprocess::PopenConfig;
use toml;
use url::Url;

pub mod config;
pub mod fmi2;
pub mod serialization;
use crate::config::HandshakeInfo;
use crate::config::SerializationFormat;
use crate::fmi2::Fmi2CallbackFunctions;
use crate::fmi2::Fmi2Type;
use crate::serialization::BindToRandom;
use crate::serialization::JsonReceiver;
use crate::serialization::ObjectSender;
use crate::serialization::PickleReceiver;
use once_cell::sync::OnceCell;

/// An identifier that can be used to uniquely identify a slave within the context of a specific backend.
pub type SlaveHandle = i32;

// -------------------------- ZMQ -----------------------------------

lazy_static! {
    static ref CONTEXT: zmq::Context = zmq::Context::new();
}

lazy_static! {
    static ref HANDLE_TO_SOCKETS: Mutex<HashMap<SlaveHandle, Mutex<zmq::Socket>>> =
        Mutex::new(HashMap::new());
}

// ----------------------------- PROCESS -----------------------------------

lazy_static! {
    static ref HANDLE_TO_PROCESS: Mutex<HashMap<SlaveHandle, Popen>> = Mutex::new(HashMap::new());
}

static WRAPPER_CONFIG: OnceCell<config::FullConfig> = OnceCell::new();

// --------------------------- Utilities functions for communicating with slave ------------------------------

fn execute_fmi_command_status<T>(handle_ptr: *const c_int, value: T) -> i32
where
    T: serde::ser::Serialize + std::panic::UnwindSafe,
{
    match execute_fmi_command_return::<_, i32>(handle_ptr, value) {
        Ok(status) => status as i32,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

fn execute_fmi_command_return<T, V>(handle_ptr: *const c_int, value: T) -> Result<V, ()>
where
    T: serde::ser::Serialize + std::panic::UnwindSafe,
    V: serde::de::DeserializeOwned,
{
    let result_or_panic = catch_unwind(|| {
        let handle = unsafe { *handle_ptr };

        let map = HANDLE_TO_SOCKETS.lock().unwrap();
        let command_socket = map.get(&handle).unwrap().lock().unwrap();
        let format = WRAPPER_CONFIG
            .get()
            .expect("the configuration is not ready. do not invoke this function prior to setting configuration")
            .handshake_info
            .serialization_format;

        command_socket.send_as_object(value, format, None).unwrap();
        let bytes = command_socket.recv_bytes(0).unwrap();
        let res: V = match format {
            SerializationFormat::Pickle => {
                serde_pickle::from_slice(&bytes).expect("unable to un-pickle object")
            }
            SerializationFormat::Json => {
                serde_json::from_slice(&bytes).expect("unable to decode json object")
            }
        };
        res
    });

    match result_or_panic {
        Ok(value) => Ok(value),
        Err(_panic) => {
            eprintln!("Failed sending command to slave. a panic was raised during the call");
            Err(())
        }
    }
}

// -------------------------------------------------------------------------------------------------------------
use serde_repr::Serialize_repr;

#[repr(i32)]
#[derive(Serialize_repr)]
enum FMI2FunctionCode {
    SetDebugLogging = 0,
    SetupExperiement = 1,
    EnterInitializationMode = 2,
    ExitInitializationMode = 3,
    Terminate = 4,
    Reset = 5,
    SetXXX = 6,
    GetXXX = 7,
    DoStep = 8,
    FreeInstance = 9,
}

// ------------------------------------- FMI FUNCTIONS --------------------------------

#[no_mangle]
pub extern "C" fn fmi2GetTypesPlatform() -> *const c_char {
    b"default\0".as_ptr() as *const i8
}

#[no_mangle]
pub extern "C" fn fmi2GetVersion() -> *const c_char {
    b"2.0\0".as_ptr() as *const i8
}

// ------------------------------------- FMI FUNCTIONS (Life-Cycle) --------------------------------

/// Instantiates a slave instance by invoking a command in a new process
/// the command is specified by the configuration file, launch.toml, that should be located in the resources directory
/// fmi-commands are sent between the wrapper and slave(s) using a message queue library, specifically zmq.
///
/// The protocol for instantiating a slave can be defined as:
/// 1. read the launch.toml file
/// 2. wrapper crates a single handshake socket
/// 2. wrapper invokes the launch-command defined for the specific OS, the handshake-endpoint is appended to the defined command
/// 3. slave opens two socket, handshake and command
/// 4. slave uses handshake-socket to send the a endpoint of the command socket back to the wrapper
///
///
/// Not the connection has been establihed between the wrapper and the newly instantiated slave.
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
#[no_mangle]
pub extern "C" fn fmi2Instantiate(
    instance_name: *const c_char,
    fmu_type: c_int,
    fmu_guid: *const c_char,
    fmu_resource_location: *const c_char,
    functions: Fmi2CallbackFunctions,
    visible: c_int,
    logging_on: c_int,
) -> *mut i32 {
    let panic_result: Result<i32, _> = catch_unwind(|| {
        let _ = unsafe { CStr::from_ptr(instance_name) }
            .to_str()
            .expect("Unable to convert instance name to a string");

        let _ = Fmi2Type::try_from(fmu_type).expect("Unrecognized FMU type");

        let _ = unsafe { CStr::from_ptr(fmu_guid) }
            .to_str()
            .expect("Unable to convert guid to a string");

        let resource_location = unsafe { CStr::from_ptr(fmu_resource_location) }
            .to_str()
            .expect("Unable to convert resource location to a string");

        // let _ = functions.logger.expect(
        //     "logging function appears to be null, this is not permitted by the FMI specification.",
        // );

        let _ = visible != 0;
        let _ = logging_on != 0;

        // locate resource directory
        let resources_dir = Url::parse(resource_location)
            .expect("unable to parse uri")
            .to_file_path()
            .expect("unable to map URI to path");

        let config_path = resources_dir.join("launch.toml");

        assert!(config_path.is_file());
        let config = read_to_string(config_path).expect("unable to read configuration file");

        let config: config::LaunchConfig = toml::from_str(config.as_str())
            .expect("configuration file was opened, but contents were not valid toml");

        // creating a handshake-socket which is used by the slave-process to pass connection strings back to the wrapper

        let handshake_socket = CONTEXT
            .socket(zmq::PULL)
            .expect("Unable to create handshake");

        let handshake_port = handshake_socket.bind_to_random_port("*").unwrap();

        // to start the slave-process the os-specific launch command is read from the launch.toml file
        // in addition to the manually specified arguements, the endpoint of the wrappers handshake socket is appended to the args
        // specifically the argument appended is  "--handshake-endpoint tcp://..."
        let mut command = match std::env::consts::OS {
            "windows" => config.command.windows.clone(),
            "macos" => config.command.macos.clone(),
            _ => config.command.linux.clone(),
        };

        command.push("--handshake-endpoint".to_string());
        let endpoint = format!("tcp://localhost:{:?}", handshake_port);
        command.push(endpoint.to_string());

        let res = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect("Unable to start process using command");

        let handshake_info: HandshakeInfo = handshake_socket
            .recv_from_json()
            .expect("Failed to read and parse handshake information sent by slave");

        // intialize configuration, in case it has not been done before.
        // In practice just try to set it and ignore the potential error indicating that it was full
        let _ = WRAPPER_CONFIG.set(FullConfig {
            launch_config: config.clone(),
            handshake_info: handshake_info.clone(),
        });
        let command_socket = CONTEXT.socket(zmq::REQ).unwrap();
        command_socket
            .connect(&handshake_info.command_endpoint)
            .expect("unable to establish a connection to the slave's command socket");

        // associate a numerical id with each slave and its corresponding socket(s)
        let mut handle_to_sock = HANDLE_TO_SOCKETS.lock().unwrap();

        let mut handle: SlaveHandle = 0;
        while handle_to_sock.contains_key(&handle) {
            handle += 1;
        }

        handle_to_sock.insert(handle, Mutex::new(command_socket));
        HANDLE_TO_PROCESS.lock().unwrap().insert(handle, res);
        handle
    });

    match panic_result {
        Ok(h) => Box::into_raw(Box::new(h)),
        Err(_) => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(c: *mut c_int) {
    let _ = catch_unwind(|| {
        let handle = unsafe { *c };
        execute_fmi_command_status(c, (FMI2FunctionCode::FreeInstance,));

        // ensure that process is terminated
        HANDLE_TO_PROCESS.lock().unwrap().remove(&handle).unwrap();

        HANDLE_TO_SOCKETS
            .lock()
            .unwrap()
            .remove(&handle)
            .expect("the sockets associated with the slave appears to be missing");

        unsafe { Box::from_raw(c) };
    });
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    c: *const SlaveHandle,
    logging_on: c_int,
    n_categories: usize,
    categories: *const *const c_char,
) -> c_int {
    let mut categories_vec: Vec<&str> = vec![];
    let n_categories = n_categories as isize;
    for i in 0..n_categories {
        let cat = unsafe { CStr::from_ptr(*categories.offset(i)).to_str().unwrap() };
        categories_vec.push(cat);
    }
    execute_fmi_command_status(
        c,
        (
            FMI2FunctionCode::SetDebugLogging,
            categories_vec,
            logging_on == 0,
        ),
    )
}

#[no_mangle]
pub extern "C" fn fmi2SetupExperiment(
    c: *const SlaveHandle,
    tolerance_defined: c_int,
    tolerance: c_double,
    start_time: c_double,
    stop_time_defined: c_int,
    stop_time: c_double,
) -> c_int {
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

    execute_fmi_command_status(
        c,
        (
            FMI2FunctionCode::SetupExperiement,
            tolerance,
            start_time,
            stop_time,
        ),
    ) as i32
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::EnterInitializationMode,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::ExitInitializationMode,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::Terminate,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2Reset(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::Reset,)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2DoStep(
    c: *const SlaveHandle,
    current: c_double,
    step_size: c_double,
    no_set_prior: c_int,
) -> c_int {
    execute_fmi_command_status(
        c,
        (
            FMI2FunctionCode::DoStep,
            current,
            step_size,
            no_set_prior == 0,
        ),
    ) as i32
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(c: *const c_int) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

fn fmi2GetXXX<T>(c: *const SlaveHandle, vr: *const c_uint, nvr: usize, values: *mut T) -> c_int
where
    T: serde::de::DeserializeOwned,
{
    let result_or_panic = catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

        execute_fmi_command_return::<_, Vec<T>>(c, (FMI2FunctionCode::GetXXX, references)).unwrap()
    });

    match result_or_panic {
        Ok(values_slave) => {
            unsafe {
                std::ptr::copy(values_slave.as_ptr(), values, nvr);
            }
            Fmi2Status::Fmi2OK as i32
        }
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetReal(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut c_double,
) -> c_int {
    fmi2GetXXX(c, vr, nvr, values)
}

#[no_mangle]
pub extern "C" fn fmi2GetInteger(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut c_int,
) -> c_int {
    fmi2GetXXX(c, vr, nvr, values)
}

#[no_mangle]

pub extern "C" fn fmi2GetBoolean(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut c_int,
) -> c_int {
    match catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let bools =
            execute_fmi_command_return::<_, Vec<bool>>(c, (FMI2FunctionCode::GetXXX, references))
                .unwrap();

        for i in 0..nvr {
            unsafe {
                *values.offset(i as isize) = bools[i] as i32;
            }
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

struct StringBuffer {
    array: Vec<*mut c_char>,
}
unsafe impl Send for StringBuffer {}
unsafe impl Sync for StringBuffer {}

// https://users.rust-lang.org/t/share-mut-t-between-threads-wrapped-in-mutex/19621/2
lazy_static! {
    static ref STRING_BUFFER: Mutex<StringBuffer> = Mutex::new(StringBuffer { array: Vec::new() });
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We chose to do it on an instance basis, e.g. each instance has its own string buffer.
#[no_mangle]
pub extern "C" fn fmi2GetString(
    c: *const c_int,
    vr: *const c_uint,
    nvr: usize,
    values: *mut *mut *mut c_char,
) -> c_int {
    match catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let strings =
            execute_fmi_command_return::<_, Vec<String>>(c, (FMI2FunctionCode::GetXXX, references))
                .unwrap();

        // Convert vector of strings into c-string double pointer
        // Note that we intentionally omit dropping the memory
        // This should be cleared by next call to a FMI function (currently is not the case)
        let mut vec_cstr = strings
            .into_iter()
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect::<Vec<_>>();

        vec_cstr.shrink_to_fit();
        assert!(vec_cstr.len() == vec_cstr.capacity());

        unsafe {
            let mut string_buffer = STRING_BUFFER.lock().unwrap();
            // free previously allocated using CString::into_raw
            for v in string_buffer.array.iter() {
                CString::from_raw(*v);
            }

            string_buffer.array = vec_cstr;

            std::ptr::write(values, string_buffer.array.as_mut_ptr());
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

// ------------------------------------- FMI FUNCTIONS (Setters) --------------------------------

fn fmi2SetXXX<T>(c: *const c_int, vr: *const c_uint, nvr: usize, values: *const T) -> c_int
where
    T: serde::ser::Serialize + std::panic::RefUnwindSafe,
{
    let result_or_panic = catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let values = unsafe { std::slice::from_raw_parts(values, nvr) };

        execute_fmi_command_status(c, (FMI2FunctionCode::SetXXX, references, values)) as i32
    });

    match result_or_panic {
        Ok(status) => status,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetReal(
    c: *const c_int,
    vr: *const c_uint,
    nvr: usize,
    values: *const c_double,
) -> c_int {
    fmi2SetXXX(c, vr, nvr, values)
}

#[no_mangle]

pub extern "C" fn fmi2SetInteger(
    c: *const c_int,
    vr: *const c_uint,
    nvr: usize,
    values: *const c_int,
) -> c_int {
    fmi2SetXXX(c, vr, nvr, values)
}

#[no_mangle]

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
pub extern "C" fn fmi2SetBoolean(
    c: *const c_int,
    vr: *const c_uint,
    nvr: usize,
    values: *const c_int,
) -> c_int {
    match catch_unwind(|| {
        let integers = unsafe { std::slice::from_raw_parts(values, nvr) };
        let bools = integers.iter().map(|&x| x == 1).collect::<Vec<bool>>();
        fmi2SetXXX(c, vr, nvr, bools.as_ptr())
    }) {
        Ok(status) => status,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

#[no_mangle]

pub extern "C" fn fmi2SetString(
    c: *const c_int,
    vr: *const c_uint,
    nvr: usize,
    values: *const *const c_char,
) -> c_int {
    let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

    let mut vec: Vec<&str> = Vec::with_capacity(nvr);

    for i in 0..nvr {
        unsafe {
            let cstr = CStr::from_ptr(*values.offset(i as isize))
                .to_str()
                .expect("Unable to convert C-string to Rust compatible string");
            vec.insert(i, cstr);
        };
    }

    execute_fmi_command_status(c, (FMI2FunctionCode::SetXXX, references, vec)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetDirectionalDerivative(
    c: *const c_int,
    unknown_refs: *const c_int,
    nvr_unknown: usize,
    known_refs: *const c_int,
    nvr_known: usize,
    values_known: *const c_double,
    values_unkown: *mut c_double,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SetRealInputDerivatives(c: *const c_int, vr: *const c_uint) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetRealOutputDerivatives(c: *const c_int) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SetFMUstate(c: *const c_int, state: *const c_void) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SerializeFMUstate(
    c: *const c_int,
    state: *mut c_void,
    data: *const c_char,
    size: usize,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2DeSerializeFMUstate(
    c: *const c_int,
    serialized_state: *const c_char,
    size: usize,
    state: *mut c_void,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SerializedFMUstateSize(c: *const c_int, state: *mut *mut c_void) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetFMUstate(c: *const c_int, state: *mut *mut c_void) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2FreeFMUstate(c: *const c_int) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetRealStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_double,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStatus(
    c: *const c_int,
    status_kind: c_int,
    Fmi2Status: *mut c_int,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetIntegerStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetBooleanStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStringStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_char,
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

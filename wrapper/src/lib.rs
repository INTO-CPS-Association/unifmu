#![allow(non_snake_case)]

use libc::c_double;
use libc::size_t;
use once_cell::sync::OnceCell;
use serde_bytes::ByteBuf;
use serde_bytes::Bytes;
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
use std::sync::Mutex;
use std::{collections::HashMap, ptr::null};

use lazy_static::lazy_static;
use serde_repr::Serialize_repr;
use subprocess::Popen;
use subprocess::PopenConfig;
use toml;
use url::Url;

pub mod config;
pub mod fmi2;
pub mod serialization;

use crate::config::FullConfig;
use crate::config::HandshakeInfo;
use crate::config::SerializationFormat;
use crate::fmi2::Fmi2CallbackFunctions;
use crate::fmi2::Fmi2Status;
use crate::serialization::BindToRandom;
use crate::serialization::JsonReceiver;
use crate::serialization::ObjectSender;

// ----------------------- Library instantiation and cleanup ---------------------------

/// An identifier that can be used to uniquely identify a slave within the context of a specific backend.
pub type SlaveHandle = i32;
pub type StateHandle = i32;
pub type Fmi2StatusT = c_int;

// https://users.rust-lang.org/t/share-mut-t-between-threads-wrapped-in-mutex/19621/2
struct StringBuffer {
    array: Vec<CString>,
}
unsafe impl Send for StringBuffer {}
unsafe impl Sync for StringBuffer {}

struct Slave {
    socket: zmq::Socket,

    popen: Popen,

    /// Buffer storing the c-strings returned by `fmi2GetStrings`.
    /// The specs states that the caller should copy the strings to its own memory immidiately after the call has been made.
    /// The reason for this recommendation is that a FMU is allowed to  free or overwrite the memory as soon as another call is made to the FMI interface.
    string_buffer: StringBuffer,

    /// Buffer storing 0 or more past state of the slave.
    serialization_buffer: HashMap<StateHandle, Vec<u8>>,
}

impl Slave {
    fn new(socket: zmq::Socket, popen: Popen) -> Self {
        Self {
            socket,
            popen,
            string_buffer: StringBuffer { array: Vec::new() },
            serialization_buffer: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref ZMQ_CONTEXT: zmq::Context = zmq::Context::new();
    static ref SLAVES: Mutex<HashMap<SlaveHandle, Slave>> = Mutex::new(HashMap::new());
}

/// Contents of configuration file located in resources directory are written into this variable
/// This happens in the first call to `fmi2Instantiate`
static WRAPPER_CONFIG: OnceCell<config::FullConfig> = OnceCell::new();

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

// --------------------------- Utilities functions for communicating with slave ------------------------------

fn execute_fmi_command_status<T>(slave: &Slave, value: T) -> i32
where
    T: serde::ser::Serialize + std::panic::UnwindSafe,
{
    match execute_fmi_command_return::<_, i32>(slave, value) {
        Ok(status) => status as i32,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

fn execute_fmi_command_return<T, V>(slave: &Slave, value: T) -> Result<V, ()>
where
    T: serde::ser::Serialize + std::panic::UnwindSafe,
    V: serde::de::DeserializeOwned,
{
    let result_or_panic = catch_unwind(|| {
        let format = WRAPPER_CONFIG
            .get()
            .expect("the configuration is not ready. do not invoke this function prior to setting configuration")
            .handshake_info
            .serialization_format;

        slave.socket.send_as_object(value, format, None).unwrap();

        let bytes = slave.socket.recv_bytes(0).unwrap();
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

#[repr(i32)]
#[derive(Serialize_repr)]
enum FMI2FunctionCode {
    // ----- common functions ------
    // GetTypesPlatform <- implemented by wrapper
    // GetVersion <- implemented by wrapper
    SetDebugLogging = 0,
    // Instantiate <- implemented by wrapper
    SetupExperiement = 1,
    FreeInstance = 2,
    EnterInitializationMode = 3,
    ExitInitializationMode = 4,
    Terminate = 5,
    Reset = 6,
    SetXXX = 7,
    GetXXX = 8,
    Serialize = 9,
    Deserialize = 10,
    GetDirectionalDerivative = 11,
    // model-exchange (not implemented)
    // ----- cosim ------
    SetRealInputDerivatives = 12,
    GetRealOutputDerivatives = 13,
    DoStep = 14,
    CancelStep = 15,
    GetXXXStatus = 16,
}

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
#[no_mangle]
pub extern "C" fn fmi2Instantiate(
    _instance_name: *const c_char,
    _fmu_type: c_int,
    _fmu_guid: *const c_char,
    fmu_resource_location: *const c_char,
    _functions: Fmi2CallbackFunctions,
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

        // locate resource directory
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

        let config: config::LaunchConfig = toml::from_str(config.as_str())
            .expect("configuration file was opened, but contents were not valid toml");

        // creating a handshake-socket which is used by the slave-process to pass connection strings back to the wrapper

        let handshake_socket = ZMQ_CONTEXT
            .socket(zmq::PULL)
            .expect("Unable to create handshake");
        let command_socket = ZMQ_CONTEXT.socket(zmq::REQ).unwrap();
        let handshake_port = handshake_socket.bind_to_random_port("*").unwrap();

        // to start the slave-process the os-specific launch command is read from the launch.toml file
        // in addition to the manually specified arguments, the endpoint of the wrappers handshake socket is appended to the args
        // specifically the argument appended is  "--handshake-endpoint tcp://..."
        let mut command = match std::env::consts::OS {
            "windows" => config.command.windows.clone(),
            "macos" => config.command.macos.clone(),
            _ => config.command.linux.clone(),
        };

        command.push("--handshake-endpoint".to_string());
        let endpoint = format!("tcp://localhost:{:?}", handshake_port);
        command.push(endpoint.to_string());

        let popen = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect(&format!("Unable to start the process using the specified command '{:?}'. Ensure that you can invoke the command directly from a shell", command));

        let handshake_info: HandshakeInfo = handshake_socket
            .recv_from_json()
            .expect("Failed to read and parse handshake information sent by slave");

        // intialize configuration, in case it has not been done before.
        // In practice just try to set it and ignore the potential error indicating that it was full
        let _ = WRAPPER_CONFIG.set(FullConfig {
            launch_config: config.clone(),
            handshake_info: handshake_info.clone(),
        });
        command_socket
            .connect(&handshake_info.command_endpoint)
            .expect(&format!(
            "unable to establish a connection to the slave's command socket with endpoint '{:?}'",
            handshake_info.command_endpoint
        ));

        // associate a numerical id with each slave and its corresponding socket(s)

        let mut slaves = SLAVES.lock().unwrap();

        let handle = slaves
            .insert_next(Slave::new(command_socket, popen))
            .unwrap();

        handle
    });

    match panic_result {
        Ok(h) => Box::into_raw(Box::new(h)),
        Err(_) => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(slave_handle: *mut SlaveHandle) {
    let _ = catch_unwind(|| {
        let handle = unsafe { *slave_handle };
        let mut slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        execute_fmi_command_status(&slave, (FMI2FunctionCode::FreeInstance,));

        slaves.remove(&handle).expect(&format!("Failed freeing slave with handle id '{:?}. Ensure that it has been instantiated and not previously freed.", handle));

        unsafe { Box::from_raw(slave_handle) }; // Free handle allocated to the heap by fmi2Instantiate
    });
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    slave_handle: *const SlaveHandle,
    logging_on: c_int,
    n_categories: size_t,
    categories: *const *const c_char,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();

    let mut categories_vec: Vec<&str> = Vec::with_capacity(n_categories);
    for i in 0..(n_categories as isize) {
        let cat = unsafe { CStr::from_ptr(*categories.offset(i)).to_str().unwrap() };
        categories_vec.push(cat);
    }
    execute_fmi_command_status(
        &slave,
        (
            FMI2FunctionCode::SetDebugLogging,
            categories_vec,
            logging_on == 0,
        ),
    )
}

#[no_mangle]
pub extern "C" fn fmi2SetupExperiment(
    slave_handle: *const SlaveHandle,
    tolerance_defined: c_int,
    tolerance: c_double,
    start_time: c_double,
    stop_time_defined: c_int,
    stop_time: c_double,
) -> Fmi2StatusT {
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
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();

    execute_fmi_command_status(
        slave,
        (
            FMI2FunctionCode::SetupExperiement,
            tolerance,
            start_time,
            stop_time,
        ),
    ) as i32
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::EnterInitializationMode,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::ExitInitializationMode,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::Terminate,)) as i32
}

#[no_mangle]
pub extern "C" fn fmi2Reset(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::Reset,)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[no_mangle]
pub extern "C" fn fmi2DoStep(
    slave_handle: *const SlaveHandle,
    current: c_double,
    step_size: c_double,
    no_set_prior: c_int,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();

    execute_fmi_command_status(
        slave,
        (
            FMI2FunctionCode::DoStep,
            current,
            step_size,
            no_set_prior == 0,
        ),
    ) as i32
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::CancelStep,)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

fn fmi2GetXXX<T>(slave: &Slave, vr: *const c_uint, nvr: size_t, values: *mut T) -> Fmi2StatusT
where
    T: serde::de::DeserializeOwned,
{
    let result_or_panic = catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };

        execute_fmi_command_return::<_, Vec<T>>(slave, (FMI2FunctionCode::GetXXX, references))
            .unwrap()
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
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_double,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    fmi2GetXXX(&slave, vr, nvr, values)
}

#[no_mangle]
pub extern "C" fn fmi2GetInteger(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    fmi2GetXXX(&slave, vr, nvr, values)
}

#[no_mangle]
pub extern "C" fn fmi2GetBoolean(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *mut c_int,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let handle = unsafe { *slave_handle };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        let bools = execute_fmi_command_return::<_, Vec<bool>>(
            &slave,
            (FMI2FunctionCode::GetXXX, references),
        )
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
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *slave_handle };

        let mut slaves = SLAVES.lock().unwrap();
        let slave = slaves.get_mut(&handle).unwrap();

        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let strings = execute_fmi_command_return::<_, Vec<String>>(
            slave,
            (FMI2FunctionCode::GetXXX, references),
        )
        .unwrap();

        slave.string_buffer.array = strings
            .into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect::<Vec<_>>();

        unsafe {
            for (idx, cstr) in slave.string_buffer.array.iter().enumerate() {
                std::ptr::write(values.offset(idx as isize), cstr.as_ptr());
            }
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

// ------------------------------------- FMI FUNCTIONS (Setters) --------------------------------

/// Generic implementation for `fmi2SetReal`, `fmi2SetInteger`, and `fmi2SetBoolean`, i.e this function is not part of the FMI API
fn fmi2SetXXX<T>(slave: &Slave, vr: *const c_uint, nvr: size_t, values: *const T) -> Fmi2StatusT
where
    T: serde::ser::Serialize + std::panic::RefUnwindSafe,
{
    let result_or_panic = catch_unwind(|| {
        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let values = unsafe { std::slice::from_raw_parts(values, nvr) };

        execute_fmi_command_status(slave, (FMI2FunctionCode::SetXXX, references, values)) as i32
    });

    match result_or_panic {
        Ok(status) => status,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetReal(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_double,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();

    fmi2SetXXX(slave, vr, nvr, values)
}

#[no_mangle]

pub extern "C" fn fmi2SetInteger(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2StatusT {
    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();

    fmi2SetXXX(slave, vr, nvr, values)
}

#[no_mangle]

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
pub extern "C" fn fmi2SetBoolean(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const c_int,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *slave_handle };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();
        let integers = unsafe { std::slice::from_raw_parts(values, nvr) };
        let bools = integers.iter().map(|&x| x == 1).collect::<Vec<bool>>();

        fmi2SetXXX(slave, vr, nvr, bools.as_ptr())
    }) {
        Ok(status) => status,
        Err(_) => Fmi2Status::Fmi2Error as i32,
    }
}

#[no_mangle]

pub extern "C" fn fmi2SetString(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
    nvr: size_t,
    values: *const *const c_char,
) -> Fmi2StatusT {
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

    let handle = unsafe { *slave_handle };
    let slaves = SLAVES.lock().unwrap();
    let slave = slaves.get(&handle).unwrap();
    execute_fmi_command_status(slave, (FMI2FunctionCode::SetXXX, references, vec)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetDirectionalDerivative(
    c: *const c_int,
    unknown_refs: *const c_int,
    nvr_unknown: size_t,
    known_refs: *const c_int,
    nvr_known: size_t,
    values_known: *const c_double,
    values_unkown: *mut c_double,
) -> Fmi2StatusT {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SetRealInputDerivatives(
    slave_handle: *const SlaveHandle,
    vr: *const c_uint,
) -> Fmi2StatusT {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetRealOutputDerivatives(slave_handle: *const SlaveHandle) -> Fmi2StatusT {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SetFMUstate(
    slave_handle: *const SlaveHandle,
    state_handle: *const StateHandle,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let slave_handle: i32 = unsafe { *slave_handle };
        let state_handle: i32 = unsafe { *state_handle };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&slave_handle).unwrap();

        let bytes = slave.serialization_buffer.get(&state_handle)
        .expect(&format!("Unable to find state indicated by the handle '{:?}' for slave '{:?}'. Ensure that the state has previously been stored by 'fmi2GetFMUState' or 'fmi2DeSerializeFMUstate'", state_handle, slave_handle));

        let status = execute_fmi_command_return::<_, i32>(
            slave,
            (FMI2FunctionCode::Deserialize, Bytes::new(bytes)),
        )
        .unwrap();
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
pub extern "C" fn fmi2GetFMUstate(
    slave_handle: *const c_int,
    state_handle_or_none: *mut *mut SlaveHandle,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *slave_handle };
        let mut slaves = SLAVES.lock().unwrap();
        let slave = slaves.get_mut(&handle).unwrap();

        let (bytes, status) =
            execute_fmi_command_return::<_, (ByteBuf, i32)>(slave, (FMI2FunctionCode::Serialize,))
                .unwrap();

        // Whether a new buffer should be allocated depends on state's value:
        // * state points to null: allocate a new buffer and return a pointer to this
        // * state points to previously state: overwrite that buffer with current state
        unsafe {
            match (*state_handle_or_none).as_ref() {
                Some(h) => {
                    println!("A");
                    slave.serialization_buffer.insert(*h, bytes.to_vec());
                }
                None => {
                    println!("B");
                    let state_handle = slave
                        .serialization_buffer
                        .insert_next(bytes.to_vec())
                        .unwrap();
                    std::ptr::write(state_handle_or_none, Box::into_raw(Box::new(state_handle)));
                }
            };
        }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
pub extern "C" fn fmi2FreeFMUstate(
    slave_handle: *const SlaveHandle,
    state: *mut *mut c_void,
) -> Fmi2StatusT {
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
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
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
) -> Fmi2StatusT {
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
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
pub extern "C" fn fmi2DeSerializeFMUstate(
    slave_handle: *const SlaveHandle,
    serialized_state: *const c_char,
    size: size_t,
    state: *mut *mut StateHandle,
) -> Fmi2StatusT {
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
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2SerializedFMUstateSize(
    slave_handle: *const SlaveHandle,
    state_handle: *const StateHandle,
    size: *mut size_t,
) -> Fmi2StatusT {
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
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetRealStatus(
    slave_handle: *const SlaveHandle,
    status_kind: c_int,
    value: *mut c_double,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *slave_handle };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        let (status_value, status) = execute_fmi_command_return::<_, (f64, i32)>(
            slave,
            (FMI2FunctionCode::GetXXXStatus, status_kind),
        )
        .unwrap();
        unsafe { *value = status_value };
        status
    }) {
        Ok(s) => s,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStatus(
    slave_handle: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *slave_handle };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
            slave,
            (FMI2FunctionCode::GetXXXStatus, status_kind),
        )
        .unwrap();
        unsafe { *value = status_value };
        status
    }) {
        Ok(s) => s,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetIntegerStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *c };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
            slave,
            (FMI2FunctionCode::GetXXXStatus, status_kind),
        )
        .unwrap();
        unsafe { *value = status_value };
        status
    }) {
        Ok(s) => s,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetBooleanStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_int,
) -> Fmi2StatusT {
    match catch_unwind(|| {
        let handle = unsafe { *c };
        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

        let (status_value, status) = execute_fmi_command_return::<_, (bool, i32)>(
            slave,
            (FMI2FunctionCode::GetXXXStatus, status_kind),
        )
        .unwrap();
        unsafe { *value = status_value as i32 };
        status
    }) {
        Ok(s) => s,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2GetStringStatus(
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_char,
) -> Fmi2StatusT {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

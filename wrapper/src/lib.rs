#![allow(non_snake_case)]
use crate::config::FullConfig;
use crate::fmi2::Fmi2Status;
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
use crate::serialization::BindToRandom;
use crate::serialization::JsonReceiver;
use crate::serialization::ObjectSender;

// ----------------------- Library instantiation and cleanup ---------------------------

/// An identifier that can be used to uniquely identify a slave within the context of a specific backend.
pub type SlaveHandle = i32;

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
    serialization_buffer: HashMap<i32, Vec<u8>>,
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
    // static ref HANDLE_TO_SOCKET: Mutex<HashMap<SlaveHandle, zmq::Socket>> =
    //     Mutex::new(HashMap::new());
    // static ref HANDLE_TO_PROCESS: Mutex<HashMap<SlaveHandle, Popen>> = Mutex::new(HashMap::new());
    // static ref STRING_BUFFER: Mutex<StringBuffer> = Mutex::new(StringBuffer { array: Vec::new() });

    // /// Stores 0 or more snapshots of the individual slaves that support serialization
    // static ref SERIALIZATION_BUFFER: Mutex<HashMap<SlaveHandle, Mutex<HashMap<i32,Vec<u8>>>>> =
    //     Mutex::new(HashMap::new());
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

// -------------------------- ZMQ -----------------------------------

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

        let format = WRAPPER_CONFIG
            .get()
            .expect("the configuration is not ready. do not invoke this function prior to setting configuration")
            .handshake_info
            .serialization_format;

        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&handle).unwrap();

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
use serde_repr::Serialize_repr;

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
#[allow(non_snake_case)]
pub extern "C" fn fmi2GetTypesPlatform() -> *const c_char {
    b"default\0".as_ptr() as *const i8
}

#[no_mangle]
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
pub extern "C" fn fmi2Instantiate(
    _instance_name: *const c_char,
    _fmu_type: c_int,
    _fmu_guid: *const c_char,
    fmu_resource_location: *const c_char,
    _functions: Fmi2CallbackFunctions,
    _visible: c_int,
    _logging_on: c_int,
) -> *mut i32 {
    let panic_result: Result<i32, _> = catch_unwind(|| {
        let resource_location_cstr = unsafe { CStr::from_ptr(fmu_resource_location) };
        let resource_location_utf8 = resource_location_cstr.to_str().expect(&format!(
            "Unable to convert resource uri c-string to utf8, got {:?}",
            &resource_location_cstr
        ));

        // locate resource directory
        let resource_uri = Url::parse(&resource_location_utf8).expect(&format!(
            "Unable to parse the specified file URI, got: {:?}.",
            resource_location_utf8,
        ));

        let resources_dir = resource_uri.to_file_path().expect(&format!(
            "URI was parsed but could not be converted into a file path, got: {:?}",
            resource_uri
        ));

        let config_path = resources_dir.join("launch.toml");

        let config = read_to_string(&config_path).expect(&format!(
            "unable to read configuration file stored at path: {:?}",
            config_path
        ));

        let config: config::LaunchConfig = toml::from_str(config.as_str())
            .expect("configuration file was opened, but contents were not valid toml");

        // creating a handshake-socket which is used by the slave-process to pass connection strings back to the wrapper

        let handshake_socket = ZMQ_CONTEXT
            .socket(zmq::PULL)
            .expect("Unable to create handshake");
        let command_socket = ZMQ_CONTEXT.socket(zmq::REQ).unwrap();
        // command_socket.set_linger().unwrap();
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

        let popen = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect("Unable to start the process using the specified command. Ensure that you can invoke the command directly from a shell");

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
            .expect("unable to establish a connection to the slave's command socket");

        // associate a numerical id with each slave and its corresponding socket(s)

        let mut handle: SlaveHandle = 0;
        let mut slaves = SLAVES.lock().unwrap();
        while slaves.contains_key(&handle) {
            handle += 1;
        }

        slaves.insert(handle, Slave::new(command_socket, popen));

        handle
    });

    match panic_result {
        Ok(h) => Box::into_raw(Box::new(h)),
        Err(_) => null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2FreeInstance(c: *mut c_int) {
    match catch_unwind(|| {
        let handle = unsafe { *c };
        execute_fmi_command_status(c, (FMI2FunctionCode::FreeInstance,));

        SLAVES.lock().unwrap().remove(&handle).unwrap();

        unsafe { Box::from_raw(c) }; // Free handle allocated to the heap by fmi2Instantiate
    }) {
        Ok(_) => (),
        Err(_) => eprintln!("Failed freeing slave"),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2SetDebugLogging(
    c: *const SlaveHandle,
    logging_on: c_int,
    n_categories: usize,
    categories: *const *const c_char,
) -> c_int {
    let mut categories_vec: Vec<&str> = vec![];
    for i in 0..(n_categories as isize) {
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
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
pub extern "C" fn fmi2EnterInitializationMode(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::EnterInitializationMode,)) as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2ExitInitializationMode(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::ExitInitializationMode,)) as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2Terminate(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::Terminate,)) as i32
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2Reset(c: *const SlaveHandle) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::Reset,)) as i32
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------

#[no_mangle]
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
pub extern "C" fn fmi2CancelStep(c: *const c_int) -> c_int {
    execute_fmi_command_status(c, (FMI2FunctionCode::CancelStep,)) as i32
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
#[allow(non_snake_case)]
pub extern "C" fn fmi2GetReal(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut c_double,
) -> c_int {
    fmi2GetXXX(c, vr, nvr, values)
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn fmi2GetInteger(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut c_int,
) -> c_int {
    fmi2GetXXX(c, vr, nvr, values)
}

#[no_mangle]
#[allow(non_snake_case)]
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

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We chose to do it on an instance basis, e.g. each instance has its own string buffer.
#[no_mangle]
pub extern "C" fn fmi2GetString(
    c: *const SlaveHandle,
    vr: *const c_uint,
    nvr: usize,
    values: *mut *const c_char,
) -> c_int {
    match catch_unwind(|| {
        let handle = unsafe { *c };

        let mut slaves = SLAVES.lock().unwrap();
        let slave = slaves.get_mut(&handle).unwrap();

        let references = unsafe { std::slice::from_raw_parts(vr, nvr) };
        let strings =
            execute_fmi_command_return::<_, Vec<String>>(c, (FMI2FunctionCode::GetXXX, references))
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
pub extern "C" fn fmi2SetFMUstate(c: *const c_int, state: *const c_int) -> c_int {
    match catch_unwind(|| {
        let slave_handle: i32 = unsafe { *c };
        let state_handle: i32 = unsafe { *state };

        let slaves = SLAVES.lock().unwrap();
        let slave = slaves.get(&slave_handle).unwrap();

        let bytes = slave.serialization_buffer.get(&state_handle).unwrap();

        let status = execute_fmi_command_return::<_, i32>(
            c,
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
/// Whether a new buffer should be allocated depends on state's value:
/// * state points to null: allocate a new buffer and return a pointer to this
/// * state points to previously state: overwrite that buffer with current state
pub extern "C" fn fmi2GetFMUstate(slave_handle: *const c_int, state: *mut *mut c_int) -> c_int {
    match catch_unwind(|| {
        let (bytes, status) = execute_fmi_command_return::<_, (ByteBuf, i32)>(
            slave_handle,
            (FMI2FunctionCode::Serialize,),
        )
        .unwrap();

        let slave_handle: i32 = unsafe { *slave_handle };
        let mut slaves = SLAVES.lock().unwrap();

        let state_handle = slaves
            .get_mut(&slave_handle)
            .unwrap()
            .serialization_buffer
            .insert_next(bytes.to_vec())
            .unwrap();

        unsafe { std::ptr::write(state, Box::into_raw(Box::new(state_handle))) }
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
pub extern "C" fn fmi2FreeFMUstate(c: *const c_int, state: *mut *mut c_void) -> c_int {
    match catch_unwind(|| match unsafe { state.as_ref() } {
        None => (),
        Some(s) => {
            let slave_handle = unsafe { *c };
            let state_handle = unsafe { *(*s as *mut i32) };
            let mut slaves = SLAVES.lock().unwrap();

            let buffer_lock = slaves
                .get_mut(&slave_handle)
                .unwrap()
                .serialization_buffer
                .remove(&state_handle);

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
#[allow(non_snake_case)]
/// Copies the state of a slave into a buffer provided by the environment
///
/// Oddly, the length of the buffer is also provided,
/// as i would expect the environment to have enquired about the state size by calling fmi2SerializedFMUstateSize.
/// We assume that the buffer is sufficiently large
pub extern "C" fn fmi2SerializeFMUstate(
    slave_handle: *const c_int,
    state_handle: *mut c_int,
    data: *mut u8,
    _size: usize,
) -> c_int {
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

        unsafe { std::ptr::copy(bytes.as_ptr(), data, bytes.len()) };
    }) {
        Ok(_) => Fmi2Status::Fmi2OK as i32,
        Err(_) => Fmi2Status::Fmi2Fatal as i32,
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "C" fn fmi2DeSerializeFMUstate(
    slave_handle: *const c_int,
    serialized_state: *const u8,
    size: usize,
    state: *mut *mut c_int,
) -> c_int {
    match catch_unwind(|| {
        let bytes = unsafe {
            std::ptr::slice_from_raw_parts(serialized_state, size)
                .as_ref()
                .unwrap()
                .to_vec()
        };

        let slave_handle: i32 = unsafe { *slave_handle };
        let mut slaves = SLAVES.lock().unwrap();

        let state_handle = slaves
            .get_mut(&slave_handle)
            .unwrap()
            .serialization_buffer
            .insert_next(bytes)
            .unwrap();
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
    slave_handle: *const c_int,
    state_handle: *const c_int,
    size: *mut size_t,
) -> c_int {
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
    c: *const c_int,
    status_kind: c_int,
    value: *mut c_double,
) -> c_int {
    match catch_unwind(|| {
        let (status_value, status) = execute_fmi_command_return::<_, (f64, i32)>(
            c,
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
pub extern "C" fn fmi2GetStatus(c: *const c_int, status_kind: c_int, value: *mut c_int) -> c_int {
    match catch_unwind(|| {
        let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
            c,
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
) -> c_int {
    match catch_unwind(|| {
        let (status_value, status) = execute_fmi_command_return::<_, (i32, i32)>(
            c,
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
) -> c_int {
    match catch_unwind(|| {
        let (status_value, status) = execute_fmi_command_return::<_, (bool, i32)>(
            c,
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
) -> c_int {
    eprintln!("NOT IMPLEMENTED");
    Fmi2Status::Fmi2Error.into()
}

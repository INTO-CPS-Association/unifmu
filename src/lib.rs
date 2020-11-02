use crate::config::FullConfig;
use crate::config::LaunchConfig;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::os::raw::c_char;
use std::os::raw::c_int;
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

lazy_static! {
    static ref CONTEXT: zmq::Context = zmq::Context::new();
}

lazy_static! {
    static ref HANDLE_TO_SOCKETS: Mutex<HashMap<SlaveHandle, Mutex<zmq::Socket>>> =
        Mutex::new(HashMap::new());
}

static WRAPPER_CONFIG: OnceCell<config::FullConfig> = OnceCell::new();

// --------------------------- Utilities functions for communicating with slave ------------------------------
fn send_command_to_slave<T, V>(handle: SlaveHandle, value: T) -> Result<V, String>
where
    T: serde::ser::Serialize,
    V: serde::de::DeserializeOwned,
{
    let map = HANDLE_TO_SOCKETS.lock().unwrap();
    let command_socket = map.get(&handle).unwrap().lock().unwrap();

    let format = WRAPPER_CONFIG
        .get()
        .expect("the configuration is not ready. do not invoke this function prior to setting configuration")
        .handshake_info
        .serialization_format;

    command_socket.send_as_object(value, format, None).unwrap();

    let bytes = command_socket.recv_bytes(0).unwrap();

    let res = match format {
        SerializationFormat::Pickle => {
            serde_pickle::from_slice(&bytes).expect("unable to un-pickle object")
        }
        SerializationFormat::Json => {
            serde_json::from_slice(&bytes).expect("unable to decode json object")
        }
    };

    Ok(res)
    //Ok(command_socket.recv_from_pickle().unwrap())
}

// -------------------------------------------------------------------------------------------------------------

#[repr(i32)]
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

#[allow(non_snake_case)]
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
        // convert C-style fmi2 parameters to rust types
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

        let _ = functions.logger.expect(
            "logging function appears to be null, this is not permitted by the FMI specification.",
        );

        let _ = visible != 0;
        let _ = logging_on != 0;

        // locate resource directory
        let resources_dir = Url::parse(resource_location)
            .expect("unable to parse uri")
            .to_file_path()
            .expect("unable to map URI to path");

        let config_path = resources_dir.join("launch.toml");

        println!("loading configuration file from: {:?}", resources_dir);

        assert!(config_path.is_file());
        let config = read_to_string(config_path).expect("unable to read configuration file");

        let config: config::LaunchConfig = toml::from_str(config.as_str())
            .expect("configuration file was opened, but contents were not valid toml");

        println!("config is: {:?}", config);

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

        let _res = Popen::create(
            &command,
            PopenConfig {
                cwd: Some(resources_dir.as_os_str().to_owned()),
                ..Default::default()
            },
        )
        .expect("Unable to start process using command")
        .detach();

        let handshake_info: HandshakeInfo = handshake_socket
            .recv_from_json()
            .expect("Failed to read and parse handshake information sent by slave");

        println!(
            "connection string recieved by wrapper: {:?}",
            handshake_info
        );

        WRAPPER_CONFIG
            .set(FullConfig {
                launch_config: config.clone(),
                handshake_info: handshake_info.clone(),
            })
            .unwrap();

        let command_socket = CONTEXT.socket(zmq::REQ).unwrap();
        command_socket
            .connect(&handshake_info.command_endpoint)
            .expect("unable to establish a connection to the slave's command socket");

        println!("now connected to command socket!");
        // associate a numerical id with each slave and its corresponding socket(s)
        let mut handle_to_sock = HANDLE_TO_SOCKETS.lock().unwrap();

        let mut handle: SlaveHandle = 0;
        while handle_to_sock.contains_key(&handle) {
            handle += 1;
        }

        handle_to_sock.insert(handle, Mutex::new(command_socket));

        handle
    });

    match panic_result {
        Ok(h) => {
            println!("slave instantiated!");
            Box::into_raw(Box::new(h))
        }
        Err(e) => {
            eprintln!("Failed to instantiate slave due to {:?}", e);
            null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(c: *mut c_int) {
    let status_or_panic = catch_unwind(|| {
        let handle = unsafe { *c };

        let _: i32 =
            send_command_to_slave(handle, (FMI2FunctionCode::FreeInstance as i32,)).unwrap();

        println!("received response");
    });

    match status_or_panic {
        Ok(_) => println!("slave successfully freed"),
        Err(_) => eprint!("failed to remove slave"),
    }
}

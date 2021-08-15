#![allow(non_snake_case)]
use std::{ffi::CString, path::Path};

use libc::{c_char, c_double, c_int, c_void};
use libloading::{Library, Symbol};
use std::ptr::null;
use url::Url;

type fmi2CallbackFunctions = *const c_void;

type fmi2InstantiateType = unsafe extern "C" fn(
    *const c_char,
    c_int,
    *const c_char,
    *const c_char,
    fmi2CallbackFunctions,
    c_int,
    c_int,
) -> *mut c_void;

type fmi2SetupExperiment =
    unsafe extern "C" fn(*mut c_void, c_int, c_double, c_double, c_int, c_double) -> c_int;
type fmi2EnterInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2ExitInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2Terminate = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2Reset = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2FreeInstanceType = unsafe extern "C" fn(*mut c_void);
type fmi2DoStep = unsafe extern "C" fn(*mut c_void, c_double, c_double, c_int) -> c_int;

// pub fn fmi2Instantiate(
//     instance_name: char_p_ref, // neither allowed to be null or empty string
//     fmu_type: Fmi2Type,
//     fmu_guid: char_p_ref, // not allowed to be null,
//     fmu_resource_location: char_p_ref,
//     _functions: &Fmi2CallbackFunctions,
//     visible: c_int,
//     logging_on: c_int,
// ) -> Option<repr_c::Box<Slave>>

#[derive(Debug)]
pub enum ValidationError {
    NoLibrary,
    MissingMethods { methods: Vec<String> },
}

pub struct ValidationConfig {}

pub struct FmuDescription {
    _supports_async: bool,
}

pub fn validate(path: &Path, _config: &ValidationConfig) -> Result<(), ValidationError> {
    unsafe {
        let binary_path = match std::env::consts::OS {
            "windows" => "win64/unifmu.dll",
            "macos" => "darwin64/unifmu.dylib",
            _other => "linux64/unifmu.so",
        };

        let lib = match Library::new(path.join("resources").join(binary_path)) {
            Ok(l) => l,
            Err(_) => return Err(ValidationError::NoLibrary),
        };

        let fmi2Instantiate: Symbol<fmi2InstantiateType> = lib.get(b"fmi2Instantiate").unwrap();

        let fmi2EnterInitializationMode: Symbol<fmi2EnterInitializationModeType> =
            lib.get(b"fmi2EnterInitializationMode").unwrap();
        let fmi2ExitInitializationMode: Symbol<fmi2ExitInitializationModeType> =
            lib.get(b"fmi2ExitInitializationMode").unwrap();

        let fmi2FreeInstance: Symbol<fmi2FreeInstanceType> = lib.get(b"fmi2FreeInstance").unwrap();

        // instantiate slave
        let instance_name = CString::new("a").unwrap();

        let resources_path = path.join("resources");
        let url = Url::from_file_path(resources_path).unwrap();
        let resources_uri = CString::new(url.as_str()).unwrap();
        let fmu_type: c_int = 1;

        let guid = CString::new("1234").unwrap();

        let visible: c_int = 1;
        let logging_on: c_int = 1;

        let handle = fmi2Instantiate(
            instance_name.as_ptr(),
            fmu_type,
            guid.as_ptr(),
            resources_uri.as_ptr(),
            null(),
            visible,
            logging_on,
        );

        fmi2EnterInitializationMode(handle);

        fmi2ExitInitializationMode(handle);

        fmi2FreeInstance(handle);
    }

    Ok(())
}

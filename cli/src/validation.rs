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

type fmi2SetupExperimentType =
    unsafe extern "C" fn(*mut c_void, c_int, c_double, c_double, c_int, c_double) -> c_int;
type fmi2EnterInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2ExitInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2TerminateType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2ResetType = unsafe extern "C" fn(*mut c_void) -> c_int;
type fmi2FreeInstanceType = unsafe extern "C" fn(*mut c_void);
type fmi2DoStepType = unsafe extern "C" fn(*mut c_void, c_double, c_double, c_int) -> c_int;

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
pub enum ValidateError {
    /// Unable to locate the binary for the current host operating system
    BinaryNotFound,
    MissingMethods {
        methods: Vec<String>,
    },

    Fmi2InstantiateFailed,
    Fmi2SetupExperimentFailed(i32),
    Fmi2EnterInitializationModeFailed(i32),
    Fmi2ExitInitializationModeFailed(i32),
    Fmi2TerminateFailed(i32),
    Fmi2ResetFailed(i32),
}

pub struct ValidationConfig {
    check_instantiate: bool,
}

impl ValidationConfig {
    pub fn new(check_instantiate: bool) -> Self {
        Self { check_instantiate }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_instantiate: true,
        }
    }
}

pub struct FmuDescription {
    _supports_async: bool,
}

#[allow(non_snake_case)]
struct Fmi2Binary<'a> {
    // lib: Library,
    pub fmi2Instantiate: Symbol<'a, fmi2InstantiateType>,
    pub fmi2SetupExperiment: Symbol<'a, fmi2SetupExperimentType>,
    pub fmi2EnterInitializationMode: Symbol<'a, fmi2EnterInitializationModeType>,
    pub fmi2ExitInitializationMode: Symbol<'a, fmi2ExitInitializationModeType>,
    pub fmi2Terminate: Symbol<'a, fmi2TerminateType>,
    pub fmi2Reset: Symbol<'a, fmi2ResetType>,
    pub fmi2FreeInstance: Symbol<'a, fmi2FreeInstanceType>,
    pub fmi2DoStep: Symbol<'a, fmi2DoStepType>,
}

pub fn validate(rootdir: &Path, _config: &ValidationConfig) -> Result<(), ValidateError> {
    unsafe {
        let binary_path = match std::env::consts::OS {
            "windows" => "win64/unifmu.dll",
            "macos" => "darwin64/unifmu.dylib",
            _other => "linux64/unifmu.so",
        };
        let path = rootdir.join("binaries").join(binary_path);

        let lib = match Library::new(path) {
            Ok(l) => l,
            Err(_) => return Err(ValidateError::BinaryNotFound),
        };

        // load methods

        unsafe fn load_helper<'a, T>(
            l: &'a Library,
            missing: &mut Vec<String>,
            symbol: &str,
        ) -> Result<Symbol<'a, T>, libloading::Error> {
            let sym = l.get::<T>(symbol.as_bytes());

            if sym.is_err() {
                missing.push(symbol.to_owned());
            };
            sym
        }

        let mut missing: Vec<String> = Vec::new();

        let fmi2Instantiate =
            load_helper::<fmi2InstantiateType>(&lib, &mut missing, "fmi2Instantiate");
        let fmi2EnterInitializationMode = load_helper::<fmi2EnterInitializationModeType>(
            &lib,
            &mut missing,
            "fmi2EnterInitializationMode",
        );
        let fmi2ExitInitializationMode = load_helper::<fmi2ExitInitializationModeType>(
            &lib,
            &mut missing,
            "fmi2ExitInitializationMode",
        );
        let fmi2Reset = load_helper::<fmi2ResetType>(&lib, &mut missing, "fmi2Reset");
        let fmi2Terminate = load_helper::<fmi2TerminateType>(&lib, &mut missing, "fmi2Terminate");
        let fmi2FreeInstance =
            load_helper::<fmi2FreeInstanceType>(&lib, &mut missing, "fmi2FreeInstance");
        let fmi2DoStep = load_helper::<fmi2DoStepType>(&lib, &mut missing, "fmi2DoStep");
        let fmi2SetupExperiment =
            load_helper::<fmi2SetupExperimentType>(&lib, &mut missing, "fmi2SetupExperiment");

        // if all methods are found we can unwrap for convenience
        if !missing.is_empty() {
            return Err(ValidateError::MissingMethods { methods: missing });
        }

        let fmi2SetupExperiment = fmi2SetupExperiment.unwrap();
        let fmi2EnterInitializationMode = fmi2EnterInitializationMode.unwrap();
        let fmi2ExitInitializationMode = fmi2ExitInitializationMode.unwrap();
        let fmi2Terminate = fmi2Terminate.unwrap();
        let fmi2Reset = fmi2Reset.unwrap();
        let fmi2FreeInstance = fmi2FreeInstance.unwrap();
        let fmi2DoStep = fmi2DoStep.unwrap();
        let fmi2Instantiate = fmi2Instantiate.unwrap();

        // instantiate slave
        let instance_name = CString::new("a").unwrap();

        let resources_path = rootdir.join("resources");
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

        if handle.is_null() {
            return Err(ValidateError::Fmi2InstantiateFailed);
        }

        // Basic run of lifecycle

        match fmi2SetupExperiment(handle, 0, 0.001, 0.0, 1, 10.0) {
            0 => (),
            s => return Err(ValidateError::Fmi2SetupExperimentFailed(s)),
        }

        match fmi2EnterInitializationMode(handle) {
            0 => (),
            s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
        }

        match fmi2ExitInitializationMode(handle) {
            0 => (),
            s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
        }

        fmi2DoStep(handle, 0.0, 0.001, 0);

        match fmi2Terminate(handle) {
            0 => (),
            s => return Err(ValidateError::Fmi2TerminateFailed(s)),
        }

        match fmi2Reset(handle) {
            0 => (),
            s => return Err(ValidateError::Fmi2ResetFailed(s)),
        }

        fmi2FreeInstance(handle);

        Ok(())
    }
}

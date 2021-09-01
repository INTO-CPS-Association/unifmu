#![allow(non_snake_case, non_camel_case_types)]
use std::{
    ffi::CString,
    path::{Path, PathBuf},
};

extern crate dlopen;

use dlopen::wrapper::{Container, WrapperApi};

use common::md::{Fmi2ModelDescription, Fmi2Variable};
use libc::{c_char, c_double, c_int, c_void, size_t};
// use libloading::{Library, Symbol};

use log::{error, info};
use std::ptr::null;
use url::Url;

type fmi2CallbackFunctions = *const c_void;

// type fmi2InstantiateType = unsafe extern "C" fn(
//     *const c_char,
//     c_int,
//     *const c_char,
//     *const c_char,
//     fmi2CallbackFunctions,
//     c_int,
//     c_int,
// ) -> *mut c_void;

// type fmi2SetupExperimentType =
//     unsafe extern "C" fn(*mut c_void, c_int, c_double, c_double, c_int, c_double) -> c_int;
// type fmi2EnterInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
// type fmi2ExitInitializationModeType = unsafe extern "C" fn(*mut c_void) -> c_int;
// type fmi2TerminateType = unsafe extern "C" fn(*mut c_void) -> c_int;
// type fmi2ResetType = unsafe extern "C" fn(*mut c_void) -> c_int;
// type fmi2FreeInstanceType = unsafe extern "C" fn(*mut c_void);
// type fmi2DoStepType = unsafe extern "C" fn(*mut c_void, c_double, c_double, c_int) -> c_int;

#[derive(WrapperApi)]
struct Fmi2CoSimulationApi {
    fmi2Instantiate: unsafe extern "C" fn(
        name: *const c_char,
        fmu_type: c_int,
        guid: *const c_char,
        path: *const c_char,
        callbacks: fmi2CallbackFunctions,
        visible: c_int,
        logging: c_int,
    ) -> *mut c_void,

    fmi2SetupExperiment: unsafe extern "C" fn(
        instance: *mut c_void,
        tolerance_defined: c_int,
        tolerance: c_double,
        start_time: c_double,
        stop_time_defined: c_int,
        stop_time: c_double,
    ) -> c_int,
    fmi2EnterInitializationMode: unsafe extern "C" fn(instance: *mut c_void) -> c_int,
    fmi2ExitInitializationMode: unsafe extern "C" fn(instance: *mut c_void) -> c_int,
    fmi2Terminate: unsafe extern "C" fn(instance: *mut c_void) -> c_int,
    fmi2Reset: unsafe extern "C" fn(instance: *mut c_void) -> c_int,
    fmi2FreeInstance: unsafe extern "C" fn(instance: *mut c_void),
    fmi2DoStep: unsafe extern "C" fn(
        instance: *mut c_void,
        current_time: c_double,
        step_size: c_double,
        no_step_prior: c_int,
    ) -> c_int,

    fmi2GetReal: unsafe extern "C" fn(
        instance: *mut c_void,
        vrefs: *const u32,
        values: *mut f64,
        nvr: size_t,
    ) -> i32,

    fmi2GetInteger: unsafe extern "C" fn(
        instance: *mut c_void,
        vrefs: *const u32,
        values: *mut i32,
        nvr: size_t,
    ) -> i32,

    fmi2GetBoolean: unsafe extern "C" fn(
        instance: *mut c_void,
        vrefs: *const u32,
        values: *mut i32,
        nvr: size_t,
    ) -> i32,

    fmi2GetString: unsafe extern "C" fn(
        instance: *mut c_void,
        vrefs: *const u32,
        values: *mut *const c_char,
        nvr: size_t,
    ) -> i32,
}

#[derive(Debug)]
pub enum ValidateError {
    /// Unable to locate the binary for the current host operating system
    BinaryNotFound,
    ModelDescriptionNotFound,
    ModelDescriptionUnableToRead,
    ModelDescriptionUnableToParse,
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

#[derive(Debug)]
pub struct ValidationConfig {
    check_instantiate: bool,

    /// ensure that the start values specified by the modelDescription.xml file are identical to the values defined by the FMU.
    start_values: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_instantiate: true,
            start_values: true,
        }
    }
}

pub struct FmuDescription {
    md: Fmi2ModelDescription,
}

/// Convenience wrapper for a single instance of an FMI2 FMU.
struct Fmi2CoSimulationFMU {
    api: Container<Fmi2CoSimulationApi>,
    handle: *mut c_void,
}

impl Fmi2CoSimulationFMU {
    pub unsafe fn fmi2Instantate(
        api: Container<Fmi2CoSimulationApi>,
        instance_name: &str,
        guid: &str,
        resources_uri: &str,
        visible: bool,
        logging: bool,
    ) -> Result<Self, ValidateError> {
        let instance_name = CString::new(instance_name).unwrap();
        let guid = CString::new(guid).unwrap();

        let resources_uri = CString::new(resources_uri).unwrap();
        let fmu_type: c_int = 1;
        let callbacks = null();

        let visible: c_int = 1;
        let logging_on: c_int = 1;

        let handle = api.fmi2Instantiate(
            instance_name.as_ptr(),
            fmu_type,
            guid.as_ptr(),
            resources_uri.as_ptr(),
            callbacks,
            visible,
            logging_on,
        );

        if handle.is_null() {
            return Err(ValidateError::Fmi2InstantiateFailed);
        }

        Ok(Self { api, handle })
    }

    pub fn fmi2EnterInitializationMode(&self) -> i32 {
        unsafe { self.api.fmi2EnterInitializationMode(self.handle) }
    }
    pub fn fmi2ExitInitializationMode(&self) -> i32 {
        unsafe { self.api.fmi2ExitInitializationMode(self.handle) }
    }

    pub fn fmi2SetupExperiment(
        &self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> i32 {
        let tolerance_defined = match tolerance {
            Some(_) => 1,
            None => 0,
        };
        let tolerance = tolerance.unwrap_or_else(|| 0.0);
        let stop_time_defined = match stop_time {
            Some(_) => 1,
            None => 0,
        };
        let stop_time = stop_time.unwrap_or_else(|| 0.0);
        unsafe {
            self.api.fmi2SetupExperiment(
                self.handle,
                tolerance_defined,
                tolerance,
                start_time,
                stop_time_defined,
                stop_time,
            )
        }
    }

    pub fn fmi2DoStep(&self, current_time: f64, step_size: f64, no_step_prior: i32) -> i32 {
        unsafe {
            self.api
                .fmi2DoStep(self.handle, current_time, step_size, no_step_prior)
        }
    }

    pub fn fmi2Terminate(&self) -> i32 {
        unsafe { self.api.fmi2Terminate(self.handle) }
    }

    pub fn fmi2Reset(&self) -> i32 {
        unsafe { self.api.fmi2Reset(self.handle) }
    }

    pub fn fmi2FreeInstance(&self) -> () {
        unsafe { self.api.fmi2FreeInstance(self.handle) }
    }

    pub fn fmi2GetReal(&self, vrefs: &[u32], values: &mut [f64]) -> i32 {
        unsafe {
            self.api.fmi2GetReal(
                self.handle,
                vrefs.as_ptr(),
                values.as_mut_ptr(),
                values.len(),
            )
        }
    }

    pub fn fmi2GetInteger(&self, vrefs: &[u32], values: &mut [i32]) -> i32 {
        unsafe {
            self.api.fmi2GetInteger(
                self.handle,
                vrefs.as_ptr(),
                values.as_mut_ptr(),
                values.len(),
            )
        }
    }

    pub fn fmi2GetBoolean(&self, vrefs: &[u32], values: &mut [bool]) -> i32 {
        let mut c_values: Vec<i32> = values.into_iter().map(|v| *v as i32).collect();

        let status = unsafe {
            self.api.fmi2GetBoolean(
                self.handle,
                vrefs.as_ptr(),
                c_values.as_mut_ptr(),
                values.len(),
            )
        };

        for (b, i) in values.into_iter().zip(c_values) {
            *b = i != 0;
        }

        status
    }

    pub fn fmi2GetString(&self, vrefs: &[u32], values: &mut [String]) -> i32 {
        let c_values: Vec<CString> = values
            .into_iter()
            .map(|v| CString::new(v.as_str()).unwrap())
            .collect();

        let mut c_values_ptr: Vec<*const c_char> = c_values.iter().map(|v| v.as_ptr()).collect();

        let ptrs = c_values_ptr.as_mut_ptr();

        let status = unsafe {
            self.api
                .fmi2GetString(self.handle, vrefs.as_ptr(), ptrs, values.len())
        };

        for (c, cs) in values.into_iter().zip(c_values) {
            *c = cs.to_str().unwrap().to_owned();
        }

        status
    }
}

pub fn validate(rootdir: &Path, config: &ValidationConfig) -> Result<(), ValidateError> {
    let md_path = rootdir.join("modelDescription.xml");

    if !md_path.exists() {
        return Err(ValidateError::ModelDescriptionNotFound);
    }

    let md = match std::fs::read_to_string(md_path) {
        Ok(s) => match Fmi2ModelDescription::from_slice(s.as_bytes()) {
            Ok(md) => md,
            Err(_) => return Err(ValidateError::ModelDescriptionUnableToParse),
        },
        Err(_) => return Err(ValidateError::ModelDescriptionUnableToRead),
    };

    if md.cosimulation.is_none() {
        todo!("Only validation of FMI2 cosimulation FMUs are currently supported.")
    }

    let model_identifier = md.cosimulation.unwrap().model_identifier;

    // load library
    unsafe {
        let binary_path = rootdir.join("binaries");

        let binary_path = match std::env::consts::OS {
            "windows" => binary_path
                .join("win64")
                .join(format!("{}.dll", model_identifier)),
            "macos" => binary_path
                .join("darwin645")
                .join(format!("{}.dylib", model_identifier)),

            _other => binary_path
                .join("linux64")
                .join(format!("{}.so", model_identifier)),
        };

        info!("attempting to load binary from {:?}", binary_path);

        let api: Container<Fmi2CoSimulationApi> = match Container::load(binary_path) {
            Ok(l) => l,
            Err(_) => return Err(ValidateError::BinaryNotFound),
        };

        info!("binary loaded successfully");

        // instantiate slave
        let instance_name = CString::new("a").unwrap();
        let resources_path = rootdir.join("resources");

        info!(
            "attempting to generate URI for based on path to resources directory {:?}",
            resources_path
        );

        let url = Url::from_file_path(resources_path).unwrap();
        let resources_uri = CString::new(url.as_str()).unwrap();
        let fmu_type: c_int = 1;
        let callbacks: *const c_void = null();

        let guid = CString::new(md.guid).unwrap();

        let visible: c_int = 1;
        let logging_on: c_int = 1;

        info!(
                "instantiating slave with instance name {:?} with type {:?}, guid {:?}, resources uri {:?}, callbacks: {:?}, visible {:?} and logging {:?}",
                instance_name, fmu_type, guid, resources_uri, callbacks, visible, logging_on
            );

        let s = match Fmi2CoSimulationFMU::fmi2Instantate(
            api,
            &"abc",
            &"1234",
            &url.as_str(),
            true,
            false,
        ) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        // Basic run of lifecycle

        match s.fmi2SetupExperiment(None, 0.001, None) {
            0 => (),
            s => return Err(ValidateError::Fmi2SetupExperimentFailed(s)),
        }

        match s.fmi2EnterInitializationMode() {
            0 => (),
            s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
        }

        match s.fmi2ExitInitializationMode() {
            0 => (),
            s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
        }

        let mut current_time = 0.0;
        let step_size = 0.01;
        let n_steps = 100;
        for _ in 0..n_steps {
            let status = s.fmi2DoStep(current_time, step_size, 0);
            current_time += step_size;
        }

        match s.fmi2Terminate() {
            0 => (),
            s => return Err(ValidateError::Fmi2TerminateFailed(s)),
        }

        match s.fmi2Reset() {
            0 => (),
            s => return Err(ValidateError::Fmi2ResetFailed(s)),
        }

        // check start values
        if config.start_values {
            let mut real_vrefs: Vec<(u32, f64)> = Vec::new();
            let mut integer_vrefs: Vec<(u32, i32)> = Vec::new();
            let mut boolean_vrefs: Vec<(u32, bool)> = Vec::new();
            let mut string_vrefs: Vec<(u32, String)> = Vec::new();
            for v in md.model_variables.variables {
                match v.var {
                    Fmi2Variable::Real { start } => match start {
                        Some(s) => real_vrefs.push((v.value_reference, s)),
                        None => (),
                    },
                    Fmi2Variable::Integer { start } => match start {
                        Some(s) => integer_vrefs.push((v.value_reference, s)),
                        None => (),
                    },
                    Fmi2Variable::Boolean { start } => match start {
                        Some(s) => boolean_vrefs.push((v.value_reference, s)),
                        None => (),
                    },
                    Fmi2Variable::String { start } => match start {
                        Some(s) => string_vrefs.push((v.value_reference, s)),
                        None => (),
                    },
                };
            }

            match s.fmi2EnterInitializationMode() {
                0 => (),
                s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
            }

            match s.fmi2ExitInitializationMode() {
                0 => (),
                s => return Err(ValidateError::Fmi2EnterInitializationModeFailed(s)),
            }
        }

        //
        s.fmi2FreeInstance();

        Ok(())
    }
}

fn validate_common() {}

fn validate_cosimulation() {}

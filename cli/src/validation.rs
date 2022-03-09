#![allow(non_snake_case, non_camel_case_types)]
use std::ffi::{CStr, CString};
use std::path::Path;

extern crate dlopen;

use dlopen::wrapper::{Container, WrapperApi};

use common::{
    fmi2_md::{parse_fmi2_model_description, Fmi2ModelDescription, Fmi2Variable},
    fmi3_md::{parse_fmi3_model_description, Fmi3ModelDescription},
    get_model_description_major_version,
};
use libc::{c_char, c_double, c_int, c_void, size_t};

use log::{error, info};
use std::ptr::null;
use url::Url;

type fmi2CallbackFunctions = *const c_void;

#[derive(WrapperApi)]
struct Fmi3Api {
    fmi3InstantiateCoSimulation: extern "C" fn(
        instance_name: *const c_char,
        instantiation_token: *const c_char,
        resource_path: *const c_char,
        visible: i32,
        logging_on: i32,
        event_mode_used: i32,
        early_return_allowed: i32,
        required_intermediate_variables: *const u32,
        n_required_intermediate_variables: size_t,
        instance_environment: *const c_void,
        log_message: *const c_void,
        intermediate_update: *const c_void,
    ) -> *mut c_void,
    fmi3EnterInitializationMode: unsafe extern "C" fn(
        instance: *const c_void,
        tolerance_defined: i32,
        tolerance: f64,
        start_time: f64,
        stop_time_defined: i32,
        stop_time: f64,
    ) -> i32,
    fmi3ExitInitializationMode: unsafe extern "C" fn(instance: *mut c_void) -> i32,

    fmi3FreeInstance: unsafe extern "C" fn(instance: *mut c_void),

    fmi3Reset: unsafe extern "C" fn(instance: *mut c_void) -> i32,
    // fmi3GetVersion: unsafe extern "C" fn() -> *const c_char,
}

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

struct Fmi3Fmu {
    api: Container<Fmi3Api>,
    handle: *mut c_void,
}

impl Fmi3Fmu {
    pub fn fmi3InstantiateCoSimulation(
        api: Container<Fmi3Api>,
        instance_name: &str,
        instantiation_token: &str,
        resource_path: &str,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: &[u32],
        n_required_intermediate_variables: size_t,
        instance_environment: *const c_void,
        log_message: *const c_void,
        intermediate_update: *const c_void,
    ) -> Result<Self, ()> {
        let instance_name = CString::new(instance_name).unwrap();
        let instantiation_token = CString::new(instantiation_token).unwrap();
        let resource_path = CString::new(resource_path).unwrap();

        let handle = api.fmi3InstantiateCoSimulation(
            instance_name.as_ptr(),
            instantiation_token.as_ptr(),
            resource_path.as_ptr(),
            visible as i32,
            logging_on as i32,
            event_mode_used as i32,
            early_return_allowed as i32,
            required_intermediate_variables.as_ptr(),
            n_required_intermediate_variables,
            instance_environment,
            log_message,
            intermediate_update,
        );

        if handle.is_null() {
            return Err(());
        }
        Ok(Self { api, handle })
    }

    pub fn fmi3GetVersion(&self) -> String {
        todo!();
        // unsafe { CStr::from_ptr(self.api.fmi3GetVersion()) }
        //     .to_str()
        //     .unwrap()
        //     .to_owned()
    }

    pub fn fmi3FreeInstance(&self) {
        unsafe { self.api.fmi3FreeInstance(self.handle) }
    }
    pub fn fmi3Reset(&self) -> i32 {
        unsafe { self.api.fmi3Reset(self.handle) }
    }

    pub fn fmi3ExitInitializationMode(&self) -> i32 {
        unsafe { self.api.fmi3ExitInitializationMode(self.handle) }
    }

    pub fn fmi3EnterInitializationMode(
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
            self.api.fmi3EnterInitializationMode(
                self.handle,
                tolerance_defined,
                tolerance,
                start_time,
                stop_time_defined,
                stop_time,
            )
        }
    }
}

/// Convenience wrapper for a single instance of an FMI2 FMU.
struct Fmi2Fmu {
    api: Container<Fmi2CoSimulationApi>,
    handle: *mut c_void,
}

impl Fmi2Fmu {
    pub fn fmi2Instantate(
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

        let handle = unsafe {
            api.fmi2Instantiate(
                instance_name.as_ptr(),
                fmu_type,
                guid.as_ptr(),
                resources_uri.as_ptr(),
                callbacks,
                visible as i32,
                logging as i32,
            )
        };

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
fn validate_fmi3_fmu(rootdir: &Path, md: Fmi3ModelDescription) -> Result<(), ValidateError> {
    let model_identifier = md.cosimulation.unwrap().model_identifier;
    let binary_path = rootdir.join("binaries");

    let binary_path = match std::env::consts::OS {
        "windows" => binary_path
            .join("x86_64-windows")
            .join(format!("{}.dll", model_identifier)),
        "macos" => binary_path
            .join("x86_64-darwin")
            .join(format!("{}.dylib", model_identifier)),

        _other => binary_path
            .join("x86_64-linux")
            .join(format!("{}.so", model_identifier)),
    };

    info!("attempting to load binary from {:?}", binary_path);

    let api: Container<Fmi3Api> = match unsafe { Container::load(&binary_path) } {
        Ok(l) => l,
        Err(e) => {
            error!(
                "Unable to load binary for platform {:?} at {:?} due to error {:?}",
                std::env::consts::OS,
                binary_path,
                e
            );
            return Err(ValidateError::BinaryNotFound);
        }
    };

    // info!("calling 'fmi3GetVersion'");
    // unsafe { api.fmi3GetVersion() };

    let resources_path = rootdir.join("resources");
    let resources_path = resources_path.to_str().unwrap();

    info!("calling 'fmi3InstantiateCoSimulation'");
    let s = Fmi3Fmu::fmi3InstantiateCoSimulation(
        api,
        "abc",
        "instantiation_token",
        resources_path,
        false,
        false,
        false,
        false,
        &[],
        0,
        null(),
        null(),
        null(),
    )
    .unwrap();

    info!("calling 'fmi3EnterInitializationMode'");
    s.fmi3EnterInitializationMode(None, 0.0, None);
    info!("calling 'fmi3ExitInitializationMode'");
    s.fmi3ExitInitializationMode();
    info!("calling 'fmi3Reset'");
    s.fmi3Reset();
    info!("calling 'fmi3FreeInstance'");
    s.fmi3FreeInstance();

    Ok(())
}

fn validate_fmi2_fmu(
    rootdir: &Path,
    md: Fmi2ModelDescription,
    config: &ValidationConfig,
) -> Result<(), ValidateError> {
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

        let s = match Fmi2Fmu::fmi2Instantate(api, &"abc", &"1234", &url.as_str(), true, false) {
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
        };

        //
        s.fmi2FreeInstance();

        Ok(())
    }
}

pub fn validate(rootdir: &Path, config: &ValidationConfig) -> Result<(), ()> {
    let md_path = rootdir.join("modelDescription.xml");

    if !md_path.exists() {
        error!("Unable to locate 'modelDescription.xml' at {:?}", md_path);
        return Err(());
    }

    let s = match std::fs::read_to_string(md_path) {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Unable to read contents of 'modelDescription.xml' due to error {}",
                e
            );
            return Err(());
        }
    };

    info!("Detecting FMI version from the 'fmiVersion' field of the 'modelDescription.xml'");

    let md_major_version = get_model_description_major_version(s.as_bytes());

    let v = match md_major_version {
        Ok(v) => v,
        Err(e) => {
            error!("Unable to identify 'fmiVersion' by inspection of the 'modelDescription.xml'");
            return Err(());
        }
    };

    info!("Major version {:?} detected", v);

    let res = match v {
        common::FmiVersion::Fmi3 => match parse_fmi3_model_description(s.as_bytes()) {
            Ok(md) => validate_fmi3_fmu(rootdir, md),
            Err(e) => {
                error!("Unable to parse the contents of the 'modelDescription.xml' file");
                return Err(());
            }
        },
        common::FmiVersion::Fmi2 => match parse_fmi2_model_description(s.as_bytes()) {
            Ok(md) => validate_fmi2_fmu(rootdir, md, config),
            Err(e) => {
                error!("Unable to parse the contents of the 'modelDescription.xml' file");
                return Err(());
            }
        },
        common::FmiVersion::Fmi1 => {
            error!("Validation of FMI1 FMUs is not supported");
            return Err(());
        }
    };

    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

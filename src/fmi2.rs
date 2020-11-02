use anyhow::Error;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde;
use serde::Deserialize;
use serde::Serialize;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_ulonglong;
use std::os::raw::c_void;

// ------------------------------------- LOGGING (types) -------------------------------------

/// Represents the function signature of the logging callback function passsed
/// from the envrionment to the slave during instantiation.
pub type Fmi2CallbackLogger = extern "C" fn(
    component_environment: *mut c_void,
    instance_name: *const c_char,
    status: c_int,
    category: *const c_char,
    message: *const c_char,
    // ... variadic functions support in rust seems to be unstable
);
pub type Fmi2CallbackAllocateMemory = extern "C" fn(nobj: c_ulonglong, size: c_ulonglong);
pub type Fmi2CallbackFreeMemory = extern "C" fn(obj: *const c_void);
pub type Fmi2StepFinished = extern "C" fn(component_environment: *mut c_void, status: Fmi2Status);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Fmi2CallbackFunctions {
    pub logger: Option<Fmi2CallbackLogger>,
    pub allocate_memory: Option<Fmi2CallbackAllocateMemory>,
    pub free_memory: Option<Fmi2CallbackFreeMemory>,
    pub step_finished: Option<Fmi2StepFinished>,
    pub component_environment: Option<*mut c_void>,
}

// ====================== config =======================

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PyfmuConfig {
    #[serde(rename = "backend.active")]
    pub backend_active: String,
    #[serde(rename = "backend.embedded_cpython.libpython")]
    pub backend_embedded_cpython_libpython: String,
    #[serde(rename = "backend.interpreter_msgqueue.executable")]
    pub backend_interpreter_msgqueue_executable: String,
    #[serde(rename = "backend.interpreter_msgqueue.protocol")]
    pub backend_interpreter_msgqueue_protocol: String,
    #[serde(rename = "log_stdout")]
    pub log_stdout: bool,
}

// ==================== status codes =====================

/// Represents the possible status codes which are returned from the slave
#[repr(i32)]
#[derive(
    Serialize, Deserialize, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd, Eq,
)]
pub enum Fmi2Status {
    Fmi2OK,
    Fmi2Warning,
    Fmi2Discard,
    Fmi2Error,
    Fmi2Fatal,
    Fmi2Pending,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd, Eq)]
#[repr(i32)]
pub enum Fmi2StatusKind {
    Fmi2DoStepStatus = 0,
    Fmi2PendingStatus = 1,
    Fmi2LastSuccessfulTime = 2,
    Fmi2Terminated = 3,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, PartialOrd, Eq)]
#[repr(i32)]
pub enum Fmi2Type {
    Fmi2ModelExchange = 0,
    Fmi2CoSimulation = 1,
}

/// Callback to envrioment used for logging
pub trait FMI2Logger {
    fn log(self, instance_name: &str, status: Fmi2Status, category: &str, message: &str);
}

/// An identifier that can be used to uniquely identify a slave within the context of a specific backend.
pub type SlaveHandle = i32;

/// Represents an manager of multiple Python slave instances.
/// Each instance is assoicated with an integer handle returned by the function
pub trait PyFmuBackend {
    // ------------ Lifecycle --------------

    fn instantiate(
        &mut self,
        instance_name: &str,
        fmu_type: Fmi2Type,
        fmu_guid: &str,
        resource_location: &str,
        callback_functions: Fmi2CallbackLogger,
        visible: bool,
        logging_on: bool,
    ) -> Result<SlaveHandle, Error>;

    fn free_instance(&mut self, handle: SlaveHandle) -> Result<(), Error>;

    fn set_debug_logging(
        &self,
        handle: SlaveHandle,
        logging_on: bool,
        categories: Vec<&str>,
    ) -> Result<Fmi2Status, Error>;
    fn setup_experiment(
        &self,
        handle: SlaveHandle,
        start_time: f64,
        tolerance: Option<f64>,
        stop_time: Option<f64>,
    ) -> Result<Fmi2Status, Error>;
    fn enter_initialization_mode(&self, handle: SlaveHandle) -> Result<Fmi2Status, Error>;
    fn exit_initialization_mode(&self, handle: SlaveHandle) -> Result<Fmi2Status, Error>;
    fn terminate(&self, handle: SlaveHandle) -> Result<Fmi2Status, Error>;
    fn reset(&self, handle: SlaveHandle) -> Result<Fmi2Status, Error>;

    // ------------ Getters --------------

    fn get_real(
        &self,
        handle: SlaveHandle,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<f64>>), Error>;
    fn get_integer(
        &self,
        handle: SlaveHandle,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<i32>>), Error>;
    fn get_boolean(
        &self,
        handle: SlaveHandle,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<bool>>), Error>;
    fn get_string(
        &self,
        handle: SlaveHandle,
        references: &[u32],
    ) -> Result<(Fmi2Status, Option<Vec<String>>), Error>;

    // ------------ Setters --------------
    fn set_real(
        &self,
        handle: SlaveHandle,
        references: &[u32],
        values: &[f64],
    ) -> Result<Fmi2Status, Error>;

    fn set_integer(
        &self,
        handle: SlaveHandle,
        references: &[u32],
        values: &[i32],
    ) -> Result<Fmi2Status, Error>;

    fn set_boolean(
        &self,
        handle: SlaveHandle,
        references: &[u32],
        values: &[bool],
    ) -> Result<Fmi2Status, Error>;

    fn set_string(
        &self,
        handle: SlaveHandle,
        references: &[u32],
        values: &[&str],
    ) -> Result<Fmi2Status, Error>;

    // fn fmi2SetRealInputDerivatives(&self) -> Result<Fmi2Status, Error>;
    // fn fmi2GetRealOutputDerivatives(&self) -> Result<Fmi2Status, Error>;
    fn do_step(
        &self,
        handle: SlaveHandle,
        current_communication_point: f64,
        communication_step: f64,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<Fmi2Status, Error>;
}

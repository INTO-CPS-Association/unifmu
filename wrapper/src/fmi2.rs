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

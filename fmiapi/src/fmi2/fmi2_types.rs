//! Type definitions for parameters in functions crossing the ABI boundary
//! betweeen C and Rust.

use crate::common::category_filter::LogCategory;

use std::{cmp::PartialEq, ffi::{c_char, c_double, c_int}, fmt::{Debug, Display}};

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub type Fmi2Real = c_double;
pub type Fmi2Integer = c_int;
/// Should be checked for out of range value and converted to bool before use
/// if given as a function argument.
pub type Fmi2Boolean = c_int;
pub type Fmi2Char = c_char;
/// Must be checked for null-ness and converted to Rust str before use if given
/// as a function argument.
pub type Fmi2String = *const Fmi2Char;
pub type Fmi2Byte = c_char;

#[repr(i32)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi2Status {
    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
    Pending = 5,
}

/// From specification:
///
/// `This is a pointer to a data structure in the simulation environment that
/// calls the FMU.
/// Using this pointer, data from the modelDescription.xml file [(for example,
/// mapping of valueReferences to variable names)] can be transferred between
/// the simulation environment and the logger function.`
///
/// Recommended way to represent opaque pointer, i.e the c type 'void*'
/// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
/// 
/// Representing it this way lets us have type safety without knowing the
/// structure of the type.
#[repr(C)]
pub struct ComponentEnvironment {
    _data: [u8; 0]
}

/// Represents the function signature of the logging callback function passsed
/// from the environment to the slave during instantiation.
pub type Fmi2CallbackLogger = unsafe extern "C" fn(
    component_environment: *const ComponentEnvironment,
    instance_name: Fmi2String,
    status: Fmi2Status,
    category: Fmi2String,
    message: Fmi2String,
    ...
);

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum Fmi2LogCategory {
    LogEvents,
    LogSingularLinearSystems,
    LogNonlinearSystems,
    LogDynamicStateSelection,
    LogStatusWarning,
    LogStatusDiscard,
    LogStatusError,
    LogStatusFatal,
    LogStatusPending,
    #[default] LogAll,
    LogUserDefined(String)
}

impl LogCategory for Fmi2LogCategory {
    fn str_name(&self) -> &str {
        match self {
            Fmi2LogCategory::LogEvents => "logEvents",
            Fmi2LogCategory::LogSingularLinearSystems => "logSingularLinearSystems",
            Fmi2LogCategory::LogNonlinearSystems => "logNonlinearSystems",
            Fmi2LogCategory::LogDynamicStateSelection => "logDynamicStateSelection",
            Fmi2LogCategory::LogStatusWarning => "logStatusWarning",
            Fmi2LogCategory::LogStatusDiscard => "logStatusDiscard",
            Fmi2LogCategory::LogStatusError => "logStatusError",
            Fmi2LogCategory::LogStatusFatal => "logStatusFatal",
            Fmi2LogCategory::LogStatusPending => "logStatusPending",
            Fmi2LogCategory::LogAll => "logAll",
            Fmi2LogCategory::LogUserDefined(name) => name,
        }
    }
}

impl Display for Fmi2LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str_name())
    }
}

impl From<&str> for Fmi2LogCategory {
    fn from(value: &str) -> Self {
        match value {
            "logEvents" => Fmi2LogCategory::LogEvents,
            "logSingularLinearSystems" => Fmi2LogCategory::LogSingularLinearSystems,
            "logNonlinearSystems" => Fmi2LogCategory::LogNonlinearSystems,
            "logDynamicStateSelection" => Fmi2LogCategory::LogDynamicStateSelection,
            "logStatusWarning" => Fmi2LogCategory::LogStatusWarning,
            "logStatusDiscard" => Fmi2LogCategory::LogStatusDiscard,
            "logStatusError" => Fmi2LogCategory::LogStatusError,
            "logStatusFatal" => Fmi2LogCategory::LogStatusFatal,
            "logStatusPending" => Fmi2LogCategory::LogStatusPending,
            "logAll" => Fmi2LogCategory::LogAll,
            _ => Fmi2LogCategory::LogUserDefined(String::from(value))
        }
    }
}

impl From<String> for Fmi2LogCategory {
    fn from(value: String) -> Self {
        Self::from(&value as &str)
    }
}

//pub type Fmi2CallbackAllocateMemory = extern "C" fn(nobj: c_ulonglong, size: c_ulonglong);
//pub type Fmi2CallbackFreeMemory = extern "C" fn(obj: *const c_void);
pub type Fmi2StepFinished = unsafe extern "C" fn(
    component_environment: *const ComponentEnvironment,
    status: Fmi2Status
);

/// A set of callback functions provided by the environment
/// Note that all but the 'logger' function are optional and may only be used if the appropriate
/// flags are set in the 'modelDescription.xml' file
#[repr(C)]
pub struct Fmi2CallbackFunctions {
    pub logger: Fmi2CallbackLogger,

    // Memory Management functions aren't feasible in Rust.
    pub _allocate_memory: Option<unsafe extern "C" fn(...)>,
    pub _free_memory: Option<unsafe extern "C" fn(...)>,

    pub step_finished: Option<Fmi2StepFinished>,
    pub component_environment: ComponentEnvironment,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi2StatusKind {
    Fmi2DoStepStatus = 0,
    Fmi2PendingStatus = 1,
    Fmi2LastSuccessfulTime = 2,
    Fmi2Terminated = 3,
}

#[repr(i32)]
pub enum Fmi2Type {
    Fmi2ModelExchange = 0,
    Fmi2CoSimulation = 1,
}
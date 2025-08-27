use super::fmi3_messages;

use crate::common::category_filter::LogCategory;

use std::{
    error::Error,
    ffi::{CStr, c_char},
    fmt::Display,
    str::Utf8Error
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub type Fmi3Float32 = f32;
pub type Fmi3Float64 = f64;
pub type Fmi3Int8 = i8;
pub type Fmi3UInt8 = u8;
pub type Fmi3Int16 = i16;
pub type Fmi3UInt16 = u16;
pub type Fmi3Int32 = i32;
pub type Fmi3UInt32 = u32;
pub type Fmi3Int64 = i64;
pub type Fmi3UInt64 = u64;
pub type Fmi3Boolean = bool; // As of the 2018 edition of rust, Rust's bool is equal to the C bool.
pub type Fmi3Char = c_char;
pub type Fmi3String = *const Fmi3Char;
pub type Fmi3Byte = u8;
pub type Fmi3Binary = *const Fmi3Byte;
pub type Fmi3Clock = bool; // As of the 2018 edition of rust, Rust's bool is equal to the C bool.

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3Status {
    Fmi3OK = 0,
    Fmi3Warning = 1,
    Fmi3Discard = 2,
    Fmi3Error = 3,
    Fmi3Fatal = 4,
}

impl From<fmi3_messages::Fmi3StatusReturn> for Fmi3Status {
    fn from(src: fmi3_messages::Fmi3StatusReturn) -> Self {
        match src.status() {
            fmi3_messages::Fmi3Status::Fmi3Ok => Self::Fmi3OK,
            fmi3_messages::Fmi3Status::Fmi3Warning => Self::Fmi3Warning,
            fmi3_messages::Fmi3Status::Fmi3Discard => Self::Fmi3Discard,
            fmi3_messages::Fmi3Status::Fmi3Error => Self::Fmi3Error,
            fmi3_messages::Fmi3Status::Fmi3Fatal => Self::Fmi3Fatal,
        }
    }
}

impl From<fmi3_messages::Fmi3Status> for Fmi3Status {
    fn from(s: fmi3_messages::Fmi3Status) -> Self {
        match s {
            fmi3_messages::Fmi3Status::Fmi3Ok => Self::Fmi3OK,
            fmi3_messages::Fmi3Status::Fmi3Warning => Self::Fmi3Warning,
            fmi3_messages::Fmi3Status::Fmi3Discard => Self::Fmi3Discard,
            fmi3_messages::Fmi3Status::Fmi3Error => Self::Fmi3Error,
            fmi3_messages::Fmi3Status::Fmi3Fatal => Self::Fmi3Fatal,
        }
    }
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3IntervalQualifier {
    Fmi3IntervalNotYetKnown = 0,
    Fmi3IntervalUnchanged = 1,
    Fmi3IntervalChanged = 2,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3DependencyKind {
	Fmi3Independent = 0,
    Fmi3Constant = 1,
	Fmi3Fixed = 2,
	Fmi3Tunable = 3,
	Fmi3Discrete = 4,
	Fmi3Dependent = 5,
}

/// From specification:
///
/// "[This] is a pointer that must be passed to
/// `fmi3IntermediateUpdateCallback`, `fmi3ClockUpdateCallback` and
/// `fmi3LogMessageCallback` to allow the simulation environment an efficient
/// way to identify the calling FMU."
///
/// Recommended way to represent opaque pointer, i.e the c type 'void*'
/// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
/// 
/// Representing it this way lets us have type safety without knowing the
/// structure of the type.
#[repr(C)]
pub struct Fmi3InstanceEnvironment {
    _data: [u8; 0]
}

pub type Fmi3ValueReference = u32;

/// Represents the function signature of the logging callback function passsed
/// from the environment to the slave during instantiation.
pub type Fmi3LogMessageCallback = unsafe extern "C" fn(
    instance_environment: *const Fmi3InstanceEnvironment,
    status: Fmi3Status,
    category: Fmi3String,
    message: Fmi3String
);

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub enum Fmi3LogCategory {
    #[default] LogEvents,
    LogSingularLinearSystems,
    LogNonlinearSystems,
    LogDynamicStateSelection,
    LogStatusWarning,
    LogStatusDiscard,
    LogStatusError,
    LogStatusFatal,
    LogUserDefined(String)
}

impl LogCategory for Fmi3LogCategory {
    fn str_name(&self) -> &str {
        match self {
            Fmi3LogCategory::LogEvents => "logEvents",
            Fmi3LogCategory::LogSingularLinearSystems => "logSingularLinearSystems",
            Fmi3LogCategory::LogNonlinearSystems => "logNonlinearSystems",
            Fmi3LogCategory::LogDynamicStateSelection => "logDynamicStateSelection",
            Fmi3LogCategory::LogStatusWarning => "logStatusWarning",
            Fmi3LogCategory::LogStatusDiscard => "logStatusDiscard",
            Fmi3LogCategory::LogStatusError => "logStatusError",
            Fmi3LogCategory::LogStatusFatal => "logStatusFatal",
            Fmi3LogCategory::LogUserDefined(name) => name,
        }
    }
}

impl Display for Fmi3LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str_name())
    }
}

impl From<&str> for Fmi3LogCategory {
    fn from(value: &str) -> Self {
        match value {
            "logEvents" => Fmi3LogCategory::LogEvents,
            "logSingularLinearSystems" => Fmi3LogCategory::LogSingularLinearSystems,
            "logNonlinearSystems" => Fmi3LogCategory::LogNonlinearSystems,
            "logDynamicStateSelection" => Fmi3LogCategory::LogDynamicStateSelection,
            "logStatusWarning" => Fmi3LogCategory::LogStatusWarning,
            "logStatusDiscard" => Fmi3LogCategory::LogStatusDiscard,
            "logStatusError" => Fmi3LogCategory::LogStatusError,
            "logStatusFatal" => Fmi3LogCategory::LogStatusFatal,
            _ => Fmi3LogCategory::LogUserDefined(String::from(value))
        }
    }
}

impl From<String> for Fmi3LogCategory {
    fn from(value: String) -> Self {
        Self::from(&value as &str)
    }
}

/// NOT CURRENTLY FULLY SUPPORTED!
/// ~ "This is not my final form" ~
pub type Fmi3IntermediateUpdateCallback = unsafe extern "C" fn(
    instance_environment: *const Fmi3InstanceEnvironment,
    intermediate_update_time: Fmi3Float64,
    intermediate_variable_set_requested: Fmi3Boolean,
    intermediate_variable_get_allowed: Fmi3Boolean,
    intermediate_step_finished: Fmi3Boolean,
    can_return_early: Fmi3Boolean,
    early_return_requested: *const Fmi3Boolean, //might be *mut instead depending on how its used
    early_return_time: *const Fmi3Float64 //might be *mut instead depending on how its used
);

/// Callbacks of this type are part of the FMI3 API that is not supported by
/// UniFMU.
pub type UnsupportedCallback = unsafe extern "C" fn(...);
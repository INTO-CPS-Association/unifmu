//! FMI3 specific type definitions, including types crossing the ABI boundary.
//! For types with a one to one relation to types in the FMI3 specification
//! their corrosponding names in the specification will be denoted as a
//! comment on the type in the following form:
//! FMI3 Spec name: <CORROSPONDING NAME>

use super::fmi3_messages::{self, fmi3_return::ReturnMessage};

use crate::common::{
    logger::{
        log_category::LogCategory,
        log_status::LogStatus
    },
    protobuf_extensions::{
        ExpectableReturn,
        implement_expectable_return
    }
};

use std::{
    ffi::c_char,
    fmt::Display
};

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// FMI3 Spec name: fmi3Float32
pub type Fmi3Float32 = f32;
/// FMI3 Spec name: fmi3Float64
pub type Fmi3Float64 = f64;
/// FMI3 Spec name: fmi3Int8
pub type Fmi3Int8 = i8;
/// FMI3 Spec name: fmi3UInt8
pub type Fmi3UInt8 = u8;
/// FMI3 Spec name: fmi3Int16
pub type Fmi3Int16 = i16;
/// FMI3 Spec name: fmi3UInt16
pub type Fmi3UInt16 = u16;
/// FMI3 Spec name: fmi3Int32
pub type Fmi3Int32 = i32;
/// FMI3 Spec name: fmi3UInt32
pub type Fmi3UInt32 = u32;
/// FMI3 Spec name: fmi3Int64
pub type Fmi3Int64 = i64;
/// FMI3 Spec name: fmi3UInt64
pub type Fmi3UInt64 = u64;
/// FMI3 Spec name: fmi3Boolean
pub type Fmi3Boolean = bool; // As of the 2018 edition of rust, Rust's bool is equal to the C bool.
/// FMI3 Spec name: fmi3Char
pub type Fmi3Char = c_char;
/// FMI3 Spec name: fmi3String
pub type Fmi3String = *const Fmi3Char;
/// FMI3 Spec name: fmi3Byte
pub type Fmi3Byte = u8;
/// FMI3 Spec name: fmi3Binary
pub type Fmi3Binary = *const Fmi3Byte;
/// FMI3 Spec name: fmi3Clock
pub type Fmi3Clock = bool; // As of the 2018 edition of rust, Rust's bool is equal to the C bool.

/// FMI3 Spec name: fmi3Status
#[repr(i32)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3Status {
    Fmi3OK = 0,
    Fmi3Warning = 1,
    Fmi3Discard = 2,
    Fmi3Error = 3,
    Fmi3Fatal = 4,
}

impl Fmi3Status {
    fn is_fault(&self) -> bool {
        return self > Self::Fmi3Warning
    }
}

impl LogStatus for Fmi3Status {
    fn ok() -> Self {
        Self::Fmi3OK
    }

    fn warning() -> Self {
        Self::Fmi3Warning
    }

    fn error() -> Self {
        Self::Fmi3Error
    }

    fn fatal() -> Self {
        Self::Fmi3Fatal
    }

    #[cfg(feature = "fmt_logging")]
    fn fmt_log_prefix(&self) -> String {
        match self {
            Self::Fmi3OK => String::from("[OK] "),
            Self::Fmi3Warning => String::from("[WARN] "),
            Self::Fmi3Error => String::from("[ERROR] "),
            Self::Fmi3Fatal => String::from("[FATAL] "),
            Self::Fmi3Discard => String::from("[DISCARD] ")
        }
    }

    #[cfg(feature = "fmt_logging")]
    fn is_ok(&self) -> bool {
        matches!(self, Self::Fmi3OK)
    }
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

/// FMI3 Spec name: fmi3IntervalQualifier
#[allow(clippy::enum_variant_names)]
#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3IntervalQualifier {
    Fmi3IntervalNotYetKnown = 0,
    Fmi3IntervalUnchanged = 1,
    Fmi3IntervalChanged = 2,
}

/// FMI3 Spec name: fmi3DependencyKind
#[allow(clippy::enum_variant_names)]
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
/// 
/// FMI3 Spec name: fmi3InstanceEnvironment
#[repr(C)]
pub struct Fmi3InstanceEnvironment {
    _data: [u8; 0]
}

/// FMI3 Spec name: fmi3ValueReference
pub type Fmi3ValueReference = u32;

/// Represents the function signature of the logging callback function passsed
/// from the environment to the slave during instantiation.
/// 
/// FMI3 Spec name: fmi3LogMessageCallback
pub type Fmi3LogMessageCallback = unsafe extern "C" fn(
    instance_environment: *const Fmi3InstanceEnvironment,
    status: Fmi3Status,
    category: Fmi3String,
    message: Fmi3String
);

/// See section "2.4.5. LogCategories" of the FMI3 Specification.
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
    LogUnifmuMessages,
    LogUserDefined(String)
}

impl LogCategory for Fmi3LogCategory {
    fn str_name(&self) -> &str {
        match self {
            Self::LogEvents => "logEvents",
            Self::LogSingularLinearSystems => "logSingularLinearSystems",
            Self::LogNonlinearSystems => "logNonlinearSystems",
            Self::LogDynamicStateSelection => "logDynamicStateSelection",
            Self::LogStatusWarning => "logStatusWarning",
            Self::LogStatusDiscard => "logStatusDiscard",
            Self::LogStatusError => "logStatusError",
            Self::LogStatusFatal => "logStatusFatal",
            Self::LogUnifmuMessages => "logUnifmuMessages",
            Self::LogUserDefined(name) => name,
        }
    }

    fn ok() -> Self {
        Self::LogEvents
    }

    fn warning() -> Self {
        Self::LogStatusWarning
    }

    fn error() -> Self {
        Self::LogStatusError
    }

    fn fatal() -> Self {
        Self::LogStatusFatal
    }

    fn unifmu_message() -> Self {
        Self::LogUnifmuMessages
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
            "logEvents" => Self::LogEvents,
            "logSingularLinearSystems" => Self::LogSingularLinearSystems,
            "logNonlinearSystems" => Self::LogNonlinearSystems,
            "logDynamicStateSelection" => Self::LogDynamicStateSelection,
            "logStatusWarning" => Self::LogStatusWarning,
            "logStatusDiscard" => Self::LogStatusDiscard,
            "logStatusError" => Self::LogStatusError,
            "logStatusFatal" => Self::LogStatusFatal,
            "logUnifmuMessages" => Self::LogUnifmuMessages,
            _ => Self::LogUserDefined(String::from(value))
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
/// 
/// FMI3 Spec name: fmi3IntermediateUpdateCallback
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

// ----------------------- Protocol Buffer Trait decorations ---------------------------
// The trait ExpectableReturn extends the Return message with an extract
// function that let's us pattern match and unwrap the inner type of a
// ReturnMessage.
implement_expectable_return!(fmi3_messages::Fmi3EmptyReturn, ReturnMessage, Empty);
implement_expectable_return!(fmi3_messages::Fmi3StatusReturn, ReturnMessage, Status);
implement_expectable_return!(fmi3_messages::Fmi3DoStepReturn, ReturnMessage, DoStep);
implement_expectable_return!(fmi3_messages::Fmi3FreeInstanceReturn, ReturnMessage, FreeInstance);
implement_expectable_return!(fmi3_messages::Fmi3GetFloat32Return, ReturnMessage, GetFloat32);
implement_expectable_return!(fmi3_messages::Fmi3GetFloat64Return, ReturnMessage, GetFloat64);
implement_expectable_return!(fmi3_messages::Fmi3GetInt8Return, ReturnMessage, GetInt8);
implement_expectable_return!(fmi3_messages::Fmi3GetUInt8Return, ReturnMessage, GetUInt8);
implement_expectable_return!(fmi3_messages::Fmi3GetInt16Return, ReturnMessage, GetInt16);
implement_expectable_return!(fmi3_messages::Fmi3GetUInt16Return, ReturnMessage, GetUInt16);
implement_expectable_return!(fmi3_messages::Fmi3GetInt32Return, ReturnMessage, GetInt32);
implement_expectable_return!(fmi3_messages::Fmi3GetUInt32Return, ReturnMessage, GetUInt32);
implement_expectable_return!(fmi3_messages::Fmi3GetInt64Return, ReturnMessage, GetInt64);
implement_expectable_return!(fmi3_messages::Fmi3GetUInt64Return, ReturnMessage, GetUInt64);
implement_expectable_return!(fmi3_messages::Fmi3GetBooleanReturn, ReturnMessage, GetBoolean);
implement_expectable_return!(fmi3_messages::Fmi3GetStringReturn, ReturnMessage, GetString);
implement_expectable_return!(fmi3_messages::Fmi3GetBinaryReturn, ReturnMessage, GetBinary);
implement_expectable_return!(fmi3_messages::Fmi3GetDirectionalDerivativeReturn, ReturnMessage, GetDirectionalDerivative);
implement_expectable_return!(fmi3_messages::Fmi3GetAdjointDerivativeReturn, ReturnMessage, GetAdjointDerivative);
implement_expectable_return!(fmi3_messages::Fmi3GetOutputDerivativesReturn, ReturnMessage, GetOutputDerivatives);
implement_expectable_return!(fmi3_messages::Fmi3SerializeFmuStateReturn, ReturnMessage, SerializeFmuState);
implement_expectable_return!(fmi3_messages::Fmi3GetClockReturn, ReturnMessage, GetClock);
implement_expectable_return!(fmi3_messages::Fmi3UpdateDiscreteStatesReturn, ReturnMessage, UpdateDiscreteStates);
implement_expectable_return!(fmi3_messages::Fmi3GetIntervalDecimalReturn, ReturnMessage, GetIntervalDecimal);
implement_expectable_return!(fmi3_messages::Fmi3GetIntervalFractionReturn, ReturnMessage, GetIntervalFraction);
implement_expectable_return!(fmi3_messages::Fmi3GetShiftDecimalReturn, ReturnMessage, GetShiftDecimal);
implement_expectable_return!(fmi3_messages::Fmi3GetShiftFractionReturn, ReturnMessage, GetShiftFraction);
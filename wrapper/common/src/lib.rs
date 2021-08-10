use num_enum::{IntoPrimitive, TryFromPrimitive};
use safer_ffi::derive_ReprC;
use serde_repr::{Deserialize_repr, Serialize_repr};
#[derive_ReprC]
#[repr(i32)]
#[derive(
    Debug, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, IntoPrimitive, TryFromPrimitive,
)]

pub enum Fmi2Status {
    Fmi2OK = 0,
    Fmi2Warning = 1,
    Fmi2Discard = 2,
    Fmi2Error = 3,
    Fmi2Fatal = 4,
    Fmi2Pending = 5,
}

#[derive_ReprC]
#[repr(i32)]
#[derive(
    Debug, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr, IntoPrimitive, TryFromPrimitive,
)]
pub enum Fmi2StatusKind {
    Fmi2DoStepStatus = 0,
    Fmi2PendingStatus = 1,
    Fmi2LastSuccessfulTime = 2,
    Fmi2Terminated = 3,
}

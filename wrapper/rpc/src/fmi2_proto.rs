#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetReal {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetInteger {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetBoolean {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetString {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(string, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetReal {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetInteger {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetBoolean {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetString {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2DoStep {
    #[prost(double, tag="1")]
    pub current_time: f64,
    #[prost(double, tag="2")]
    pub step_size: f64,
    #[prost(bool, tag="3")]
    pub no_step_prior: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetupExperiment {
    #[prost(double, tag="1")]
    pub start_time: f64,
    #[prost(double, optional, tag="2")]
    pub stop_time: ::core::option::Option<f64>,
    /// bool has_stop_time = 4;
    /// bool has_tolerance = 5;
    #[prost(double, optional, tag="3")]
    pub tolerance: ::core::option::Option<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2CancelStep {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2EnterInitializationMode {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExitInitializationMode {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2FreeInstance {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2Terminate {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2Reset {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetDirectionalDerivatives {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetInputDerivatives {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetOutputDerivatives {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetXxxStatus {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetDebugLogging {
    #[prost(string, repeated, tag="1")]
    pub categories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, tag="2")]
    pub logging_on: bool,
}
// ----------------------- RETURN VALUES ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2StatusReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetRealReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetIntegerReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetBooleanReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(bool, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetStringReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(string, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2FreeInstanceReturn {
}
// ----------------------- Non FMI2 standard messages ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExtSerializeSlave {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExtDeserializeSlave {
    #[prost(bytes="vec", tag="1")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExtHandshakeReturn {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExtSerializeSlaveReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(bytes="vec", tag="2")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
/// enumeration listing all possible command that are sent from binary to slave
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2Command {
    #[prost(oneof="fmi2_command::Command", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19")]
    pub command: ::core::option::Option<fmi2_command::Command>,
}
/// Nested message and enum types in `Fmi2Command`.
pub mod fmi2_command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag="1")]
        Fmi2DoStep(super::Fmi2DoStep),
        #[prost(message, tag="2")]
        Fmi2SetReal(super::Fmi2SetReal),
        #[prost(message, tag="3")]
        Fmi2SetInteger(super::Fmi2SetInteger),
        #[prost(message, tag="4")]
        Fmi2SetBoolean(super::Fmi2SetBoolean),
        #[prost(message, tag="5")]
        Fmi2SetString(super::Fmi2SetString),
        #[prost(message, tag="6")]
        Fmi2GetReal(super::Fmi2GetReal),
        #[prost(message, tag="7")]
        Fmi2GetInteger(super::Fmi2GetInteger),
        #[prost(message, tag="8")]
        Fmi2GetBoolean(super::Fmi2GetBoolean),
        #[prost(message, tag="9")]
        Fmi2GetString(super::Fmi2GetString),
        #[prost(message, tag="10")]
        Fmi2SetupExperiment(super::Fmi2SetupExperiment),
        #[prost(message, tag="11")]
        Fmi2EnterInitializationMode(super::Fmi2EnterInitializationMode),
        #[prost(message, tag="12")]
        Fmi2ExitInitializationMode(super::Fmi2ExitInitializationMode),
        #[prost(message, tag="13")]
        Fmi2FreeInstance(super::Fmi2FreeInstance),
        #[prost(message, tag="14")]
        Fmi2Reset(super::Fmi2Reset),
        #[prost(message, tag="15")]
        Fmi2Terminate(super::Fmi2Terminate),
        #[prost(message, tag="16")]
        Fmi2CancelStep(super::Fmi2CancelStep),
        #[prost(message, tag="17")]
        Fmi2ExtSerializeSlave(super::Fmi2ExtSerializeSlave),
        #[prost(message, tag="18")]
        Fmi2ExtDeserializeSlave(super::Fmi2ExtDeserializeSlave),
        #[prost(message, tag="19")]
        Fmi2SetDebugLogging(super::Fmi2SetDebugLogging),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2Return {
    #[prost(oneof="fmi2_return::Result", tags="1, 2, 3, 4, 5, 6, 7, 8")]
    pub result: ::core::option::Option<fmi2_return::Result>,
}
/// Nested message and enum types in `Fmi2Return`.
pub mod fmi2_return {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        #[prost(message, tag="1")]
        Fmi2StatusReturn(super::Fmi2StatusReturn),
        #[prost(message, tag="2")]
        Fmi2GetRealReturn(super::Fmi2GetRealReturn),
        #[prost(message, tag="3")]
        Fmi2GetIntegerReturn(super::Fmi2GetIntegerReturn),
        #[prost(message, tag="4")]
        Fmi2GetBooleanReturn(super::Fmi2GetBooleanReturn),
        #[prost(message, tag="5")]
        Fmi2GetStringReturn(super::Fmi2GetStringReturn),
        #[prost(message, tag="6")]
        Fmi2FreeInstanceReturn(super::Fmi2FreeInstanceReturn),
        #[prost(message, tag="7")]
        Fmi2ExtHandshakeReturn(super::Fmi2ExtHandshakeReturn),
        #[prost(message, tag="8")]
        Fmi2ExtSerializeSlaveReturn(super::Fmi2ExtSerializeSlaveReturn),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Fmi2Status {
    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
    Pending = 5,
}

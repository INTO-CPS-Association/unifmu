#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3InstantiateModelExchange {
    #[prost(string, tag="1")]
    pub instance_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub instantiation_token: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub resource_path: ::prost::alloc::string::String,
    #[prost(bool, tag="4")]
    pub visible: bool,
    #[prost(bool, tag="5")]
    pub logging_on: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3InstantiateCoSimulation {
    #[prost(string, tag="1")]
    pub instance_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub instantiation_token: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub resource_path: ::prost::alloc::string::String,
    #[prost(bool, tag="4")]
    pub visible: bool,
    #[prost(bool, tag="5")]
    pub logging_on: bool,
    #[prost(bool, tag="6")]
    pub event_mode_used: bool,
    #[prost(bool, tag="7")]
    pub early_return_allowed: bool,
    #[prost(uint32, repeated, tag="8")]
    pub required_intermediate_variables: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3InstantiateScheduledExecution {
    #[prost(string, tag="1")]
    pub instance_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub instantiation_token: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub resource_path: ::prost::alloc::string::String,
    #[prost(bool, tag="4")]
    pub visible: bool,
    #[prost(bool, tag="5")]
    pub logging_on: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3DoStep {
    #[prost(double, tag="1")]
    pub current_communication_point: f64,
    #[prost(double, tag="2")]
    pub communication_step_size: f64,
    #[prost(bool, tag="3")]
    pub no_set_fmu_state_prior_to_current_point: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetDebugLogging {
    #[prost(bool, tag="1")]
    pub logging_on: bool,
    #[prost(string, repeated, tag="2")]
    pub categories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3EnterInitializationMode {
    #[prost(double, optional, tag="1")]
    pub tolerance: ::core::option::Option<f64>,
    #[prost(double, tag="2")]
    pub start_time: f64,
    #[prost(double, optional, tag="3")]
    pub stop_time: ::core::option::Option<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3ExitInitializationMode {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3FreeInstance {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3Terminate {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3Reset {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SerializeFmuState {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3DeserializeFmuState {
    #[prost(bytes="vec", tag="1")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
// ----------------------- FMI3 Getters ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetFloat32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetFloat64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetBoolean {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetString {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmiGetBinary {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetDirectionalDerivative {
    #[prost(uint32, repeated, tag="1")]
    pub unknowns: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub knowns: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="3")]
    pub seed: ::prost::alloc::vec::Vec<f64>,
    #[prost(double, repeated, tag="4")]
    pub sensitivity: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetAdjointDerivative {
    #[prost(uint32, repeated, tag="1")]
    pub unknowns: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub knowns: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="3")]
    pub seed: ::prost::alloc::vec::Vec<f64>,
    #[prost(double, repeated, tag="4")]
    pub sensitivity: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetOutputDerivatives {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub orders: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="3")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
// ----------------------- FMI3 Setters ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetFloat32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(float, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetFloat64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int64, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint64, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetBoolean {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(bool, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetString {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(string, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmiSetBinary {
    #[prost(uint32, repeated, tag="1")]
    pub value_references: ::prost::alloc::vec::Vec<u32>,
    #[prost(bytes="vec", repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
// ----------------------- FMI3 Return Values ----------------------

/// For methods that do not return a status code
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3EmptyReturn {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3StatusReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3FreeInstanceReturn {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetFloat32Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(float, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetFloat64Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt8Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt8Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt16Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt16Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt32Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt32Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetInt64Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(int64, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetUInt64Return {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(uint64, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetBooleanReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(bool, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetStringReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(string, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmiGetBinaryReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(bytes="vec", repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetDirectionalDerivativeReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetAdjointDerivativeReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3GetOutputDerivativesReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SerializeFmuStateReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(bytes="vec", tag="2")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
// ----------------------- FMI Command Wrapper ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3Command {
    #[prost(oneof="fmi3_command::Command", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43")]
    pub command: ::core::option::Option<fmi3_command::Command>,
}
/// Nested message and enum types in `Fmi3Command`.
pub mod fmi3_command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        // FMI3

        #[prost(message, tag="1")]
        Fmi3InstantiateModelExchange(super::Fmi3InstantiateModelExchange),
        #[prost(message, tag="2")]
        Fmi3InstantiateCoSimulation(super::Fmi3InstantiateCoSimulation),
        #[prost(message, tag="3")]
        Fmi3InstantiateScheduledExecution(super::Fmi3InstantiateScheduledExecution),
        #[prost(message, tag="4")]
        Fmi3DoStep(super::Fmi3DoStep),
        #[prost(message, tag="5")]
        Fmi3SetDebugLogging(super::Fmi3SetDebugLogging),
        #[prost(message, tag="6")]
        Fmi3EnterInitializationMode(super::Fmi3EnterInitializationMode),
        #[prost(message, tag="7")]
        Fmi3ExitInitializationMode(super::Fmi3ExitInitializationMode),
        #[prost(message, tag="8")]
        Fmi3FreeInstance(super::Fmi3FreeInstance),
        #[prost(message, tag="9")]
        Fmi3Terminate(super::Fmi3Terminate),
        #[prost(message, tag="10")]
        Fmi3Reset(super::Fmi3Reset),
        #[prost(message, tag="13")]
        Fmi3GetFloat32(super::Fmi3GetFloat32),
        #[prost(message, tag="14")]
        Fmi3GetFloat64(super::Fmi3GetFloat64),
        #[prost(message, tag="15")]
        Fmi3GetInt8(super::Fmi3GetInt8),
        #[prost(message, tag="16")]
        Fmi3GetUInt8(super::Fmi3GetUInt8),
        #[prost(message, tag="17")]
        Fmi3GetInt16(super::Fmi3GetInt16),
        #[prost(message, tag="18")]
        Fmi3GetUInt16(super::Fmi3GetUInt16),
        #[prost(message, tag="19")]
        Fmi3GetInt32(super::Fmi3GetInt32),
        #[prost(message, tag="20")]
        Fmi3GetUInt32(super::Fmi3GetUInt32),
        #[prost(message, tag="21")]
        Fmi3GetInt64(super::Fmi3GetInt64),
        #[prost(message, tag="22")]
        Fmi3GetUInt64(super::Fmi3GetUInt64),
        #[prost(message, tag="23")]
        Fmi3GetBoolean(super::Fmi3GetBoolean),
        #[prost(message, tag="24")]
        Fmi3GetString(super::Fmi3GetString),
        #[prost(message, tag="25")]
        FmiGetBinary(super::FmiGetBinary),
        #[prost(message, tag="26")]
        Fmi3GetDirectionalDerivative(super::Fmi3GetDirectionalDerivative),
        #[prost(message, tag="27")]
        Fmi3GetAdjointDerivative(super::Fmi3GetAdjointDerivative),
        #[prost(message, tag="28")]
        Fmi3GetOutputDerivatives(super::Fmi3GetOutputDerivatives),
        #[prost(message, tag="29")]
        Fmi3SetFloat32(super::Fmi3SetFloat32),
        #[prost(message, tag="30")]
        Fmi3SetFloat64(super::Fmi3SetFloat64),
        #[prost(message, tag="31")]
        Fmi3SetInt8(super::Fmi3SetInt8),
        #[prost(message, tag="32")]
        Fmi3SetUInt8(super::Fmi3SetUInt8),
        #[prost(message, tag="33")]
        Fmi3SetInt16(super::Fmi3SetInt16),
        #[prost(message, tag="34")]
        Fmi3SetUInt16(super::Fmi3SetUInt16),
        #[prost(message, tag="35")]
        Fmi3SetInt32(super::Fmi3SetInt32),
        #[prost(message, tag="36")]
        Fmi3SetUInt32(super::Fmi3SetUInt32),
        #[prost(message, tag="37")]
        Fmi3SetInt64(super::Fmi3SetInt64),
        #[prost(message, tag="38")]
        Fmi3SetUInt64(super::Fmi3SetUInt64),
        #[prost(message, tag="39")]
        Fmi3SetBoolean(super::Fmi3SetBoolean),
        #[prost(message, tag="40")]
        Fmi3SetString(super::Fmi3SetString),
        #[prost(message, tag="41")]
        FmiSetBinary(super::FmiSetBinary),
        #[prost(message, tag="42")]
        Fmi3SerializeFmuState(super::Fmi3SerializeFmuState),
        #[prost(message, tag="43")]
        Fmi3DeserializeFmuState(super::Fmi3DeserializeFmuState),
    }
}
// ----------------------- FMI3 ----------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Fmi3Status {
    Fmi3Ok = 0,
    Fmi3Warning = 1,
    Fmi3Discard = 2,
    Fmi3Error = 3,
    Fmi3Fatal = 4,
}

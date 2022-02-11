// ----------------------- Common ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnifmuSerialize {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnifmuDeserialize {
    #[prost(bytes="vec", tag="1")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmptyReturn {
}
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
    #[prost(bool, tag="4")]
    pub event_handling_needed: bool,
    #[prost(bool, tag="5")]
    pub terminate_simulation: bool,
    #[prost(bool, tag="6")]
    pub early_return: bool,
    #[prost(double, tag="7")]
    pub last_successful_time: f64,
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
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
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
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(float, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetFloat64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt8 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt16 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt32 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
    #[prost(int64, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi3SetUInt64 {
    #[prost(uint32, repeated, tag="1")]
    pub value_reference: ::prost::alloc::vec::Vec<u32>,
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
pub struct UnifmuFmi3SerializeReturn {
    #[prost(enumeration="Fmi3Status", tag="1")]
    pub status: i32,
    #[prost(bytes="vec", tag="2")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2Instantiate {
    #[prost(string, tag="1")]
    pub instance_name: ::prost::alloc::string::String,
    #[prost(enumeration="Fmi2Type", tag="2")]
    pub fmu_type: i32,
    #[prost(string, tag="3")]
    pub fmu_guid: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub fmu_resource_location: ::prost::alloc::string::String,
    #[prost(bool, tag="5")]
    pub visible: bool,
    #[prost(bool, tag="6")]
    pub logging_on: bool,
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
pub struct Fmi2SetDebugLogging {
    #[prost(string, repeated, tag="1")]
    pub categories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, tag="2")]
    pub logging_on: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2SetupExperiment {
    #[prost(double, tag="1")]
    pub start_time: f64,
    #[prost(double, optional, tag="2")]
    pub stop_time: ::core::option::Option<f64>,
    #[prost(double, optional, tag="3")]
    pub tolerance: ::core::option::Option<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2EnterInitializationMode {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2ExitInitializationMode {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2CancelStep {
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
// ----------------------- FMI2 Getters ----------------------

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
pub struct Fmi2GetDirectionalDerivatives {
    #[prost(uint32, repeated, tag="1")]
    pub references_unknown: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag="2")]
    pub references_known: ::prost::alloc::vec::Vec<u32>,
    #[prost(double, repeated, tag="3")]
    pub direction_known: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetRealOutputDerivatives {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub orders: ::prost::alloc::vec::Vec<i32>,
}
// ----------------------- FMI2 Setters ----------------------

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
pub struct Fmi2SetRealInputDerivatives {
    #[prost(uint32, repeated, tag="1")]
    pub references: ::prost::alloc::vec::Vec<u32>,
    #[prost(int32, repeated, tag="2")]
    pub orders: ::prost::alloc::vec::Vec<i32>,
    #[prost(double, repeated, tag="3")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
// ----------------------- FMI2 Return Values ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2StatusReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2FreeInstanceReturn {
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
pub struct Fmi2GetRealOutputDerivativesReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fmi2GetDirectionalDerivativesReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(double, repeated, tag="2")]
    pub values: ::prost::alloc::vec::Vec<f64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnifmuFmi2SerializeReturn {
    #[prost(enumeration="Fmi2Status", tag="1")]
    pub status: i32,
    #[prost(bytes="vec", tag="2")]
    pub state: ::prost::alloc::vec::Vec<u8>,
}
// ----------------------- FMI Command Wrapper ----------------------

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmiCommand {
    #[prost(oneof="fmi_command::Command", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64")]
    pub command: ::core::option::Option<fmi_command::Command>,
}
/// Nested message and enum types in `FmiCommand`.
pub mod fmi_command {
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
        // FMI2

        #[prost(message, tag="42")]
        Fmi2DoStep(super::Fmi2DoStep),
        #[prost(message, tag="43")]
        Fmi2SetDebugLogging(super::Fmi2SetDebugLogging),
        #[prost(message, tag="44")]
        Fmi2SetupExperiment(super::Fmi2SetupExperiment),
        #[prost(message, tag="45")]
        Fmi2EnterInitializationMode(super::Fmi2EnterInitializationMode),
        #[prost(message, tag="46")]
        Fmi2ExitInitializationMode(super::Fmi2ExitInitializationMode),
        #[prost(message, tag="47")]
        Fmi2FreeInstance(super::Fmi2FreeInstance),
        #[prost(message, tag="48")]
        Fmi2CancelStep(super::Fmi2CancelStep),
        #[prost(message, tag="49")]
        Fmi2Terminate(super::Fmi2Terminate),
        #[prost(message, tag="50")]
        Fmi2Reset(super::Fmi2Reset),
        #[prost(message, tag="51")]
        Fmi2GetReal(super::Fmi2GetReal),
        #[prost(message, tag="52")]
        Fmi2GetInteger(super::Fmi2GetInteger),
        #[prost(message, tag="53")]
        Fmi2GetBoolean(super::Fmi2GetBoolean),
        #[prost(message, tag="54")]
        Fmi2GetString(super::Fmi2GetString),
        #[prost(message, tag="55")]
        Fmi2GetDirectionalDerivatives(super::Fmi2GetDirectionalDerivatives),
        #[prost(message, tag="56")]
        Fmi2GetRealOutputDerivatives(super::Fmi2GetRealOutputDerivatives),
        #[prost(message, tag="57")]
        Fmi2SetReal(super::Fmi2SetReal),
        #[prost(message, tag="58")]
        Fmi2SetInteger(super::Fmi2SetInteger),
        #[prost(message, tag="59")]
        Fmi2SetBoolean(super::Fmi2SetBoolean),
        #[prost(message, tag="60")]
        Fmi2SetString(super::Fmi2SetString),
        #[prost(message, tag="61")]
        Fmi2SetRealInputDerivatives(super::Fmi2SetRealInputDerivatives),
        #[prost(message, tag="62")]
        Fmi2Instantiate(super::Fmi2Instantiate),
        // Common

        #[prost(message, tag="63")]
        UnifmuSerialize(super::UnifmuSerialize),
        #[prost(message, tag="64")]
        UnifmuDeserialize(super::UnifmuDeserialize),
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
// ----------------------- FMI2 ----------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Fmi2Status {
    Fmi2Ok = 0,
    Fmi2Warning = 1,
    Fmi2Discard = 2,
    Fmi2Error = 3,
    Fmi2Fatal = 4,
    Fmi2Pending = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Fmi2Type {
    Fmi2ModelExchange = 0,
    Fmi2CoSimulation = 1,
}

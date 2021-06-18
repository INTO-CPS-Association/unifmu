use std::path::PathBuf;

use serde::Deserialize;

use crate::{google_rpc::GRPCConfig, schemaless_rpc::ZMQConfig};
use crate::{google_rpc::ProtobufGRPC, schemaless_rpc::ZMQSchemalessRPC, Fmi2Status};

#[derive(Deserialize, Clone, Debug)]
pub enum RpcConfigType {
    #[serde(rename = "zmq")]
    Zmq,
    #[serde(rename = "grpc")]
    Grpc,
}

/// Data-structure representing the contents of the launch.toml file,
/// located in the 'resources' directory of an FMU.
/// The backend such as `grpc` or `zmq` is choosen based on the concrete contents.
#[derive(Deserialize, Clone, Debug)]
pub struct RpcConfig {
    backend: RpcConfigType,
    zmq: Option<ZMQConfig>,
    grpc: Option<GRPCConfig>,
}

/// Trait implemented by objects that provide a way to communicate with FMUs using 'Remote Procedure Call' (RPC)
///
/// Current implementations are:
///
/// * grpc: google remote procedure call
/// * zmq: schemaless protocol defined by struct + serde that are transmitted over `zmq`
pub trait Fmi2CommandRPC {
    fn fmi2DoStep(&mut self, current_time: f64, step_size: f64, no_step_prior: bool) -> Fmi2Status;
    fn fmi2CancelStep(&mut self) -> Fmi2Status;
    fn fmi2SetDebugLogging(&mut self, categories: &[&str], logging_on: bool) -> Fmi2Status;
    fn fmi2SetupExperiment(
        &mut self,
        start_time: f64,
        stop_time: Option<f64>,
        tolerance: Option<f64>,
    ) -> Fmi2Status;
    fn fmi2EnterInitializationMode(&mut self) -> Fmi2Status;
    fn fmi2ExitInitializationMode(&mut self) -> Fmi2Status;
    fn fmi2Terminate(&mut self) -> Fmi2Status;
    fn fmi2Reset(&mut self) -> Fmi2Status;

    fn fmi2SetReal(&mut self, references: &[u32], values: &[f64]) -> Fmi2Status;
    fn fmi2SetInteger(&mut self, references: &[u32], values: &[i32]) -> Fmi2Status;
    fn fmi2SetBoolean(&mut self, references: &[u32], values: &[bool]) -> Fmi2Status;
    fn fmi2SetString(&mut self, references: &[u32], values: &[&str]) -> Fmi2Status;

    // TODO add status to return
    fn fmi2GetReal(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<f64>>);
    fn fmi2GetInteger(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<i32>>);
    fn fmi2GetBoolean(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<bool>>);
    fn fmi2GetString(&mut self, references: &[u32]) -> (Fmi2Status, Option<Vec<String>>);

    fn serialize(&mut self) -> (Fmi2Status, Option<Vec<u8>>);
    fn deserialize(&mut self, bytes: &[u8]) -> Fmi2Status;

    fn fmi2FreeInstance(&mut self);
}

/// Instantiate a slave using the procedure specified by the configuration.
pub fn initialize_slave_from_config(
    config: RpcConfig,
    resources_dir: PathBuf,
) -> Result<Box<dyn Fmi2CommandRPC>, String> {
    match config.backend {
        RpcConfigType::Zmq => ZMQSchemalessRPC::new(
            &config
                .zmq
                .expect("The 'zmq' backend does not define any launch commands."),
            &resources_dir,
        ),
        RpcConfigType::Grpc => ProtobufGRPC::new(
            &config
                .grpc
                .expect("The 'grpc' backend does not define any launch commands."),
            &resources_dir,
        ),
    }
}

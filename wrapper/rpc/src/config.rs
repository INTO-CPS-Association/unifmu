// #[derive(Deserialize, Clone, Debug)]
// pub enum RpcConfigType {
//     #[serde(rename = "zmq")]
//     Zmq,
//     #[serde(rename = "grpc")]
//     Grpc,
// }

// /// Data-structure representing the contents of the launch.toml file,
// /// located in the 'resources' directory of an FMU.
// /// The backend such as `grpc` or `zmq` is choosen based on the concrete contents.
// #[derive(Deserialize, Clone, Debug)]
// pub struct RpcConfig {
//     backend: RpcConfigType,
//     zmq: Option<ZMQConfig>,
//     grpc: Option<GRPCConfig>,
// }

// #[derive(serde::Deserialize, Debug, Clone)]
// pub struct GRPCConfig {
//     pub windows: Vec<String>,
//     pub linux: Vec<String>,
//     pub macos: Vec<String>,
// }

// #[derive(serde::Deserialize, Debug, Clone)]
// pub struct ZMQConfig {
//     pub windows: Vec<String>,
//     pub linux: Vec<String>,
//     pub macos: Vec<String>,
//     pub serialization_format: SerializationFormat,
// }

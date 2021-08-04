use rpc::socket_dispatcher::SerializationFormat;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum ActiveDispatcher {
    #[serde(alias = "socket")]
    Socket,
    #[serde(alias = "grpc")]
    Grpc,
}

#[derive(Deserialize)]
pub struct LaunchConfig {
    pub active_dispatcher: ActiveDispatcher,

    #[serde(alias = "socket")]
    pub socket_config: Option<SocketConfig>,
    #[serde(alias = "grpc")]
    pub grpc_config: Option<GrpcConfig>,
}

#[derive(Deserialize)]
enum ZmqTransportType {
    TCP,
    IPC,
}

#[derive(Deserialize)]
pub enum SocketType {
    #[serde(alias = "zmq")]
    Zmq,
}

#[derive(Deserialize)]
pub struct SocketConfig {
    #[serde(alias = "type")]
    pub socket_type: SocketType,
    pub format: SerializationFormat,
    pub linux: Vec<String>,
    pub windows: Vec<String>,
    pub macos: Vec<String>,
}
#[derive(Deserialize)]
pub struct GrpcConfig {
    pub linux: Vec<String>,
    pub windows: Vec<String>,
    pub macos: Vec<String>,
}

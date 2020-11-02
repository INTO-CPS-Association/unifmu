use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SerializationFormat {
    Pickle,
    //FlexBuffers,
    Json,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Command {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
    pub macos: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Timeout {
    pub command: i32,
    pub launch: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LaunchConfig {
    pub command: Command,
    pub timeout: Timeout,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HandshakeInfo {
    pub serialization_format: SerializationFormat,
    pub command_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct FullConfig {
    pub launch_config: LaunchConfig,
    pub handshake_info: HandshakeInfo,
}

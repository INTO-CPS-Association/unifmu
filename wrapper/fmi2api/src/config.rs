use serde::Deserialize;

// #[derive(Deserialize)]
// pub enum ActiveDispatcher {
//     #[serde(alias = "socket")]
//     Socket,
//     #[serde(alias = "grpc")]
//     Grpc,
// }

#[derive(Deserialize)]
pub struct LaunchConfig {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>,
    pub macos: Option<Vec<String>>,
}

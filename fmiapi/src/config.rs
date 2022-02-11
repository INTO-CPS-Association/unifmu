use serde::Deserialize;


#[derive(Deserialize)]
pub struct LaunchConfig {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>,
    pub macos: Option<Vec<String>>,
}

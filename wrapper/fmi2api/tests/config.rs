const VALID_ZMQ_CONFIG: &str = r#"
active_dispatcher = "socket" # {socket, grpc}

[socket]
type = "zmq"      # {zmq, tcp}
format = "pickle" # {protobuf, pickle, json}
linux = ["python3", "launch.py"]
windows = [""]
macos = [""]


[grpc]
linux = [""]
windows = [""]
macos = [""]
"#;

#[cfg(test)]

mod tests {

    use super::*;
    use fmi2api::config::LaunchConfig;
    #[test]

    fn parse_toml() {
        let _ = toml::from_str::<LaunchConfig>(VALID_ZMQ_CONFIG).unwrap();
    }
}

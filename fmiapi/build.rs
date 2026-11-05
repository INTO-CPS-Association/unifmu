fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::default()
        .out_dir("src/fmi2")
        .compile_protos(
            &["fmi2_messages.proto"],
            &["../schemas"],
        )
        .unwrap();

    prost_build::Config::default()
        .out_dir("src/fmi3")
        .compile_protos(
            &["fmi3_messages.proto"],
            &["../schemas"],
        )
        .unwrap();

    prost_build::Config::default()
        .out_dir("src/common")
        // Retain the unused request in generated bindings for schema compatibility.
        .type_attribute(".unifmu_handshake.HandshakeRequest", "#[allow(dead_code)]")
        .compile_protos(
            &["unifmu_handshake.proto"],
            &["../schemas"],
        )
        .unwrap();

    Ok(())
}

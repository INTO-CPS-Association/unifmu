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
        .compile_protos(
            &["unifmu_handshake.proto"],
            &["../schemas"],
        )
        .unwrap();

    Ok(())
}

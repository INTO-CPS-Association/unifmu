fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::default()
        .out_dir("src")
        .compile_protos(
            &["fmi2_messages.proto", "fmi3_messages.proto"],
            &["../schemas"],
        )
        .unwrap();

    Ok(())
}

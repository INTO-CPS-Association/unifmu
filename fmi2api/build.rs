fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::default()
        .out_dir("src")
        .compile_protos(&["unifmu_fmi.proto"], &["../schemas"])
        .unwrap();

    Ok(())
}

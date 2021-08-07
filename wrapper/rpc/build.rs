fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::default()
        .out_dir("src")
        .compile_protos(&["unifmu_fmi2.proto"], &["../../schemas"])
        .unwrap();

    Ok(())

    // tonic_build::configure()
    //     .build_client(true)
    //     .out_dir("src")
    //     .compile(&["unifmu_fmi2.proto"], &["../../schemas"])?;
    // Ok(())
}

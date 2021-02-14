use std::env;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("OUT_DIR", "src");
    tonic_build::configure()
        .build_client(true)
        .out_dir("src")
        .compile(&["../schemas/unifmu_fmi2.proto"], &["../schemas"])?;
    Ok(())
}

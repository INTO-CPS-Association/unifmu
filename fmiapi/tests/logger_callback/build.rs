fn main() {
    println!("cargo:rerun-if-changed=callback.c");
    cc::Build::new()
        .file("callback.c")
        .compile("fmi2_logger_test_callback");
}

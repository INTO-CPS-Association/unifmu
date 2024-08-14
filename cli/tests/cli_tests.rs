use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

fn get_generated_fmus_dir() -> PathBuf {
    // cwd starts at cli folder, so move to parent and get to generated_fmus
    let generated_fmus = std::env::current_dir().unwrap().parent().unwrap().join("generated_fmus");
    assert!(generated_fmus.exists(), "The directory {} does not exist", generated_fmus.display());
    generated_fmus
}

fn get_vdm_check2_jar() -> PathBuf {
    // cwd starts at cli folder, so move to parent and get to vdm_check
    let test_dependencies = std::env::current_dir().unwrap().parent().unwrap().join("test_dependencies");
    assert!(test_dependencies.exists(), "The directory {} does not exist", test_dependencies.display());

    // Search for folder starting with vdmcheck. Ensures that we can easily get new version of vdm_check
    let not_found_msg = format!("No vdm_check directory found in {}", test_dependencies.display());
    let vdm_check_dir = test_dependencies.read_dir().unwrap()
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().starts_with("vdmcheck"))
        .expect(&not_found_msg)
        .path();

    let vdm_check_jar = vdm_check_dir.join("vdmcheck2.jar");
    assert!(vdm_check_jar.exists(), "vdmcheck2.jar not found in {}", vdm_check_dir.display());
    
    vdm_check_jar
}

#[test]
fn test_python_fmi2() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();
    
    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", "python", "pythonfmu.fmu", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let python_fmu = generated_fmus_dir.join("pythonfmu.fmu");
    assert!(python_fmu.exists(), "The file {} does not exist", python_fmu.display());

    let vdm_check_2_jar = get_vdm_check2_jar();
    
    let mut vdm_check_cmd: Command = Command::new("java");
    
    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(python_fmu.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found."));
}

use std::{fs::File, path::PathBuf};
use assert_cmd::Command as AssertCommand;
use std::process::Command as StdCommand;
use predicates::str::contains;
//use assert_cmd::{assert};

#[test]
fn test_python_fmi2() {
    test_fmi2(&"python");
}

#[test]
fn test_java_fmi2() {
    test_fmi2(&"java");
}

#[test]
fn test_csharp_fmi2() {
    test_fmi2(&"c-sharp");
}

fn test_fmi2(language: &str) {
    let mut unifmu_cmd: AssertCommand = AssertCommand::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    let test_fmu_filename = format!("{}_fmi2.fmu", language);

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", language, &test_fmu_filename, "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let fmu_path: PathBuf = generated_fmus_dir.join(test_fmu_filename);

    test_fmu_fmi2(fmu_path);
}

fn get_generated_fmus_dir() -> PathBuf {
    // cwd starts at cli folder, so move to parent and get to generated_fmus
    let generated_fmus = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("generated_fmus");
    assert!(
        generated_fmus.exists(),
        "The directory {} does not exist",
        generated_fmus.display()
    );
    return generated_fmus;
}

fn get_vdm_check_jar(version: &str) -> PathBuf {
    assert!(version == "2" || version == "3", "Invalid vdmcheck version requested: {}", version);
    // cwd starts at cli folder, so move to parent and get to vdm_check
    let test_dependencies = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_dependencies");
    assert!(
        test_dependencies.exists(),
        "The directory {} does not exist",
        test_dependencies.display()
    );

    // Search for folder starting with vdmcheck. Ensures that we can easily get new version of vdm_check
    let not_found_msg = format!(
        "No vdm_check directory found in {}",
        test_dependencies.display()
    );
    let vdm_check_dir = test_dependencies
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().starts_with(&format!("vdmcheck{}", version)))
        .expect(&not_found_msg)
        .path();

    let vdm_check_jar = vdm_check_dir.join(&format!("vdmcheck{}.jar", version));
    assert!(
        vdm_check_jar.exists(),
        "vdmcheck{}.jar not found in {}",
        version,
        vdm_check_dir.display()
    );

    vdm_check_jar
}


fn test_fmu_fmi2(fmu_path: PathBuf) {
    assert!(fmu_path.exists(),
    "The FMU file {} does not exist",
    fmu_path.display());
    
    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: AssertCommand = AssertCommand::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(fmu_path.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found"));

    // Run test script with fmu_path as argument
    let script_filename = "test_fmi2.py";
    let py_script_path = "./tests/test_scripts/".to_string() + script_filename;
    let output = StdCommand::new("python")
        .arg(py_script_path)
        .arg(fmu_path.as_path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    println!("Python script output: {:?}", output);
}
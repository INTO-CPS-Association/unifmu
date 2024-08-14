use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_with_argument() {
    let mut cmd: Command = Command::cargo_bin("unifmu").unwrap();
    
    // cwd starts at cli folder, so move to parent and get to generated_fmus
    let cwd = std::env::current_dir().unwrap().parent().unwrap().join("generated_fmus");
    assert!(cwd.exists(), "The directory {} does not exist", cwd.display());

    cmd
        .current_dir(cwd.as_path())
        .args(&["generate", "python", "pythonfmu.fmu", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));
}

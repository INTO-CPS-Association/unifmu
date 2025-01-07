use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    thread
};

use assert_cmd::Command;
use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{CoSimulation, Common},
    },
    import,
    schema::fmi2::ScalarVariable,
    traits::{FmiImport, FmiStatus},
};
use gag::BufferRedirect;
use predicates::str::contains;

mod common;

use common::{
    RemoteBackend, RemoteFileStructure, TestableFmu, ZippedTestableFmu
};

static DISTRIBUTED_FMU: Mutex<Executor> = Mutex::new(Executor);

#[derive(Clone, Copy)]
struct Executor;

impl Executor {
    fn run_test(self, f: impl FnOnce()) {
        f();
    }
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
    generated_fmus
}

fn get_tests_dir() -> PathBuf {
    // cwd starts at cli folder, so move to parent and get to generated_fmus
    let tests_dir = std::env::current_dir()
        .unwrap()
        .join("tests");
    assert!(
        tests_dir.exists(),
        "The directory {} does not exist",
        tests_dir.display()
    );
    tests_dir
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

#[test]
fn test_python_fmi2() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", "python", "pythonfmu_fmi2.fmu", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let python_fmu: PathBuf = generated_fmus_dir.join("pythonfmu_fmi2.fmu");

    test_fmu_fmi2(python_fmu);
}

#[test]
#[should_panic(expected = "Expected panic!: Instantiation")]
fn test_python_backend_subprocess_unexpected_exit_in_handshake() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    fmu.break_model();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let _instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Expected panic!");
}

#[test]
#[should_panic(expected = "Expected panic!: Error")]
fn test_python_backend_subprocess_unexpected_exit_during_fmi3_command() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    fmu.break_do_step_function();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Should be able to instantiate FMU.");

    instance.do_step(
        0.0,
        1.0,
        true
    )
        .ok()
        .expect("Expected panic!");
}

#[test]
fn test_java_fmi2() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", "java", "javafmu_fmi2.fmu", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let java_fmu: PathBuf = generated_fmus_dir.join("javafmu_fmi2.fmu");

    test_fmu_fmi2(java_fmu);
}

#[test]
#[should_panic(expected = "Expected panic!: Instantiation")]
fn test_java_backend_subprocess_unexpected_exit_in_handshake() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    fmu.break_model();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let _instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Expected panic!");
}

#[test]
#[should_panic(expected = "Expected panic!: Error")]
fn test_java_backend_subprocess_unexpected_exit_during_fmi3_command() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    fmu.break_do_step_function();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Should be able to instantiate FMU.");

    instance.do_step(
        0.0,
        1.0,
        true
    )
        .ok()
        .expect("Expected panic!");
}

#[test]
fn test_csharp_fmi2() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", "c-sharp", "csharpfmu_fmi2.fmu", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let fmu_file: PathBuf = generated_fmus_dir.join("csharpfmu_fmi2.fmu");

    test_fmu_fmi2(fmu_file);
}

#[test]
#[should_panic(expected = "Expected panic!: Instantiation")]
fn test_csharp_backend_subprocess_unexpected_exit_in_handshake() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    fmu.break_model();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let _instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Expected panic!");
}

#[test]
#[should_panic(expected = "Expected panic!: Error")]
fn test_csharp_backend_subprocess_unexpected_exit_during_fmi3_command() {
    let fmu = common::LocalFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    fmu.break_do_step_function();

    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    let instance = import.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("Should be able to instantiate FMU.");

    instance.do_step(
        0.0,
        1.0,
        true
    )
        .ok()
        .expect("Expected panic!");
}

fn test_fmu_fmi2(fmu_path: PathBuf){
    assert!(
        fmu_path.exists(),
        "The file {} was not generated successfully.",
        fmu_path.display()
    );

    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(fmu_path.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let fmu_file = File::open(fmu_path).unwrap();

    let import: Fmi2Import = import::new::<File, Fmi2Import>(fmu_file).unwrap();

    assert_eq!(import.model_description().fmi_version, "2.0");

    let mut cs_instance: fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS> = import.instantiate_cs("instance", false, true).unwrap();
    assert_eq!(
        fmi::fmi2::instance::Common::get_version(&cs_instance),
        "2.0"
    );

    cs_instance
        .setup_experiment(Some(1.0e-6_f64), 0.0, None)
        .ok()
        .expect("setup_experiment");

    cs_instance
        .enter_initialization_mode()
        .ok()
        .expect("enter_initialization_mode");

    cs_instance
        .exit_initialization_mode()
        .ok()
        .expect("exit_initialization_mode");

    // Check for initial outputs as they are calculated

    let mut real_c = get_real(&import, &mut cs_instance, "real_c");
    assert_eq!(real_c, 0.0);

    let mut integer_c = get_integer(&import, &mut cs_instance, "integer_c");
    assert_eq!(integer_c, 0);

    cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");

    set_real(&import, &mut cs_instance, "real_a", 1.0);
    set_real(&import, &mut cs_instance, "real_b", 2.0);

    set_integer(&import, &mut cs_instance, "integer_a", 1);
    set_integer(&import, &mut cs_instance, "integer_b", 2);

    cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");

    real_c = get_real(&import, &mut cs_instance, "real_c");
    assert_eq!(real_c, 3.0);

    integer_c = get_integer(&import, &mut cs_instance, "integer_c");
    assert_eq!(integer_c, 3);

    cs_instance.terminate().ok().expect("terminate");
}

// Default fmu utility functions
fn get_real(import: &Fmi2Import, cs_instance: &mut fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS>, var: &str) -> f64 {
    let real_sv: &ScalarVariable = import
        .model_description()
        .model_variable_by_name(var)
        .unwrap();

    let mut real: [f64; 1] = [0.0];

    cs_instance
        .get_real(&[real_sv.value_reference], &mut real)
        .ok()
        .unwrap();

    real[0]
}

fn get_integer(import: &Fmi2Import, cs_instance: &mut fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS>, var: &str) -> i32 {
    let integer_sv: &ScalarVariable = import
        .model_description()
        .model_variable_by_name(var)
        .unwrap();

    let mut integer: [i32; 1] = [0];

    cs_instance
        .get_integer(&[integer_sv.value_reference], &mut integer)
        .ok()
        .unwrap();

    integer[0]
}

fn set_real(import: &Fmi2Import, cs_instance: &mut fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS>, var_name: &str, value: f64) {
    let sv: &ScalarVariable = import
        .model_description()
        .model_variable_by_name(var_name)
        .unwrap();

    let error_msg = format!("set_real failed for {}", var_name);

    cs_instance
        .set_real(
            &[sv.value_reference],
            &[value],
        )
        .ok()
        .expect(&error_msg);
}

fn set_integer(import: &Fmi2Import, cs_instance: &mut fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS>, var_name: &str, value: i32) {
    let sv: &ScalarVariable = import
        .model_description()
        .model_variable_by_name(var_name)
        .unwrap();

    let error_msg = format!("set_integer failed for {}", var_name);

    cs_instance
        .set_integer(
            &[sv.value_reference],
            &[value],
        )
        .ok()
        .expect(&error_msg);
}


/***** Tests for distributed FMUs *****/
#[test]
fn test_fmu_creation_csharp_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    assert!(fmu.model_file_path().exists());
}

#[test]
fn test_fmu_creation_java_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    assert!(fmu.model_file_path().exists());
}

#[test]
fn test_fmu_creation_python_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    assert!(fmu.model_file_path().exists());
}

#[test]
fn test_fmu_is_well_formatted_csharp_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    is_fmu_well_formatted(
        fmu.zipped()
            .proxy_directory_path()
            .as_path()
    );
}

#[test]
fn test_fmu_is_well_formatted_java_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    is_fmu_well_formatted(
        fmu.zipped()
            .proxy_directory_path()
            .as_path()
    );
}

#[test]
fn test_fmu_is_well_formatted_python_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    is_fmu_well_formatted(
        fmu.zipped()
            .proxy_directory_path()
            .as_path()
    );
}

fn is_fmu_well_formatted(fmu_zipfile_path: &Path) {
    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(fmu_zipfile_path)
        .assert()
        .success()
        .stdout(contains("No errors found."));
}

#[test]
fn test_instantiate_csharp_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    basic_remote_testing_procedure(fmu, just_instantiate);
}

#[test]
fn test_instantiate_java_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    basic_remote_testing_procedure(fmu, just_instantiate);
}

#[test]
fn test_instantiate_python_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    basic_remote_testing_procedure(fmu, just_instantiate);
}

fn basic_remote_testing_procedure(
    fmu: (impl TestableFmu + RemoteBackend),
    fmu_operations: fn(Fmi2Import)
) {
    let import = import::new::<File, Fmi2Import>(fmu.zipped().file())
        .expect("Should be able to import FMU.");

    DISTRIBUTED_FMU.lock().unwrap().run_test(move || {
        // Redirects the contained codes stdout to a buffer, enabling us to catch
        // the portnumber when the backend instantiates.
        let mut stdout_buffer = BufferRedirect::stdout()
            .expect("Should be able to redirect stdout.");

        let proxy_thread = thread::spawn(move || {
            fmu_operations(import);
        });

        let mut port_string = String::new();

        // Wait for output and parse it if it has the right format.
        // It should contain the portnumber.
        loop {
            let mut caught_stdout = String::new();
            stdout_buffer.read_to_string(&mut caught_stdout)
                .expect("Should be able to read redirected stdout.");

            // Current humanreadable port message is:
            //
            // Connect remote backend to dispatcher via endpoint tcp://0.0.0.0:XXXXX
            //
            // Where XXXXX is the port number, and where the message is followed with a newline
            // (type of newline might be platform dependent).
            if caught_stdout.contains("Connect remote backend to dispatcher via endpoint") {
                port_string = caught_stdout[64..69].to_string();
                break;
            }
        }

        drop(stdout_buffer);

        fmu.run_remote_backend(port_string);

        proxy_thread.join().unwrap();
    });
}

fn just_instantiate(imported_fmu: Fmi2Import) {
    let _instance = imported_fmu.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("should be able to instantiate FMU.");
}

#[test]
fn test_fmu_functionality_csharp_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::CSharp
    );

    basic_remote_testing_procedure(fmu, run_through_full_functionality);
}

#[test]
fn test_fmu_functionality_java_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Java
    );

    basic_remote_testing_procedure(fmu, run_through_full_functionality);
}

#[test]
fn test_fmu_functionality_python_fmi2_distributed() {
    let fmu = common::RemoteFmu::get_clone(
        &common::FmiVersion::Fmi2, 
        &common::FmuBackendImplementationLanguage::Python
    );

    basic_remote_testing_procedure(fmu, run_through_full_functionality);
}

fn run_through_full_functionality(imported_fmu: Fmi2Import) {
    assert_eq!(imported_fmu.model_description().fmi_version, "2.0");

    let mut cs_instance = imported_fmu.instantiate_cs(
        "instance", 
        false,
        true
    )
        .expect("should be able to instantiate FMU.");

    assert_eq!(
        fmi::fmi2::instance::Common::get_version(&cs_instance),
        "2.0"
    );

    cs_instance
        .setup_experiment(Some(1.0e-6_f64), 0.0, None)
        .ok()
        .expect("setup_experiment");

    cs_instance
        .enter_initialization_mode()
        .ok()
        .expect("enter_initialization_mode");

    cs_instance
        .exit_initialization_mode()
        .ok()
        .expect("exit_initialization_mode");

    // Check for initial outputs as they are calculated

    let mut real_c = get_real(&imported_fmu, &mut cs_instance, "real_c");
    assert_eq!(real_c, 0.0);

    let mut integer_c = get_integer(&imported_fmu, &mut cs_instance, "integer_c");
    assert_eq!(integer_c, 0);

    cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");

    set_real(&imported_fmu, &mut cs_instance, "real_a", 1.0);
    set_real(&imported_fmu, &mut cs_instance, "real_b", 2.0);

    set_integer(&imported_fmu, &mut cs_instance, "integer_a", 1);
    set_integer(&imported_fmu, &mut cs_instance, "integer_b", 2);

    cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");

    real_c = get_real(&imported_fmu, &mut cs_instance, "real_c");
    assert_eq!(real_c, 3.0);

    integer_c = get_integer(&imported_fmu, &mut cs_instance, "integer_c");
    assert_eq!(integer_c, 3);

    cs_instance.terminate().ok().expect("terminate");
}

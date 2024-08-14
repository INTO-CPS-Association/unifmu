use std::{fs::File, path::PathBuf};

use assert_cmd::Command;
use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{CoSimulation, Common},
    },
    fmi3::{
        import::Fmi3Import,
    },
    import,
    schema::fmi2::ScalarVariable,
    traits::{FmiImport, FmiStatus},
};
use predicates::str::contains;

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
    assert!(
        python_fmu.exists(),
        "The file {} does not exist",
        python_fmu.display()
    );

    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(python_fmu.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let python_fmu_file = File::open(python_fmu).unwrap();

    let import: Fmi2Import = import::new::<File, Fmi2Import>(python_fmu_file).unwrap();

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
        .expect("enter_initialization_mode");

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

#[test]
fn test_python_fmi3() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate", "python", "pythonfmu_fmi3.fmu", "fmi3", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMU was generated successfully"));

    let python_fmu: PathBuf = generated_fmus_dir.join("pythonfmu_fmi3.fmu");
    assert!(
        python_fmu.exists(),
        "The file {} does not exist",
        python_fmu.display()
    );

    let vdm_check_3_jar = get_vdm_check_jar("3");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_3_jar.as_path())
        .arg(python_fmu.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let reference_fmu: PathBuf = generated_fmus_dir.join("Reference-FMUs-0.0.32").join("3.0").join("Feedthrough.fmu");
    let python_fmu_file = File::open(python_fmu).unwrap();
    //let python_fmu_file = File::open(reference_fmu).unwrap();

    let import: Fmi3Import = import::new::<File, Fmi3Import>(python_fmu_file).unwrap();

    assert!(import.model_description().fmi_version.starts_with("3.0"));

    let launch_toml: PathBuf = PathBuf::from(import.archive_path());

    let mut cs_instance: fmi::fmi3::instance::Instance<fmi::fmi3::instance::CS> = import.instantiate_cs("instance", false, true, false, false, &[]).unwrap();
    assert_eq!(
        fmi::fmi3::instance::Common::get_version(&cs_instance),
        "3.0"
    );
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

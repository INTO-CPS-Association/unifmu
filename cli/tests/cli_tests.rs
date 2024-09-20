use std::{fs::File, path::PathBuf, path::Path, process::{Command as processCommand, Stdio}, io::{Read, Write, BufRead, BufReader}};

use assert_cmd::{assert, Command};
use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{CoSimulation, Common},
    },
    import,
    schema::fmi2::ScalarVariable,
    traits::{FmiImport, FmiStatus},
};
use predicates::str::contains;

#[cfg(test)]
use std::{println as info, println as warn, println as debug};

use std::thread;

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


/***** Tests distributed FMUs *****/
#[test]
fn test_python_fmi2_distributed() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate-virtual", "python", "pythonfmu_fmi2_distributed","127.0.0.1", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMUs were generated successfully"));

    let python_fmu: PathBuf = generated_fmus_dir.join("pythonfmu_fmi2_distributed_proxy.fmu");
    let python_backend_directory: PathBuf = generated_fmus_dir.join("pythonfmu_fmi2_distributed_private/resources/backend.py");

    test_fmu_fmi2_distributed_python(python_fmu,python_backend_directory);
}

#[test]
fn test_java_fmi2_distributed() {
    let mut unifmu_cmd: Command = Command::cargo_bin("unifmu").unwrap();

    let generated_fmus_dir = get_generated_fmus_dir();

    unifmu_cmd
        .current_dir(generated_fmus_dir.as_path())
        .args(&["generate-virtual", "java", "javafmu_fmi2_distributed", "127.0.0.1", "--zipped"])
        .assert()
        .success()
        .stderr(contains("the FMUs were generated successfully"));

    let java_fmu: PathBuf = generated_fmus_dir.join("javafmu_fmi2_distributed_proxy.fmu");
    let java_backend_directory: PathBuf = generated_fmus_dir.join("javafmu_fmi2_distributed_private/resources/gradlew");


    test_fmu_fmi2_distributed_java(java_fmu,java_backend_directory);
}

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    std::env::current_dir()
}

fn test_fmu_fmi2_distributed_python(fmu_path: PathBuf, backend_directory: PathBuf){
    // Checking first the FMU (proxy)


    assert!(
        fmu_path.exists(),
        "The file {} was not generated successfully.",
        fmu_path.display()
    );

    let fmu_path_str = fmu_path.as_path();

    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(fmu_path_str)
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let fmu_file = File::open(fmu_path.clone()).unwrap();

    let import: Fmi2Import = import::new::<File, Fmi2Import>(fmu_file).unwrap();

    assert_eq!(import.model_description().fmi_version, "2.0");


    // Start the proxy
    println!("Initializing FMU proxy backend...");

    /*let proxy_fmu_thread = thread::spawn(move || {
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
    });*/

    let tests_dir = get_tests_dir();
    let fmpy_test_fmi2: PathBuf = tests_dir.join("test_fmi2.py");


    let mut python_cmd_proxy = processCommand::new("python3")
        .args(&[fmpy_test_fmi2,fmu_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().expect("Failed to start external test with fmpy.");

    let out_proxy = BufReader::new(python_cmd_proxy.stdout.take().unwrap());
    let err_proxy = BufReader::new(python_cmd_proxy.stderr.take().unwrap());

    //static mut connection_port: u16 = 0;
    let proxy_fmu_thread = thread::spawn(move || {
        out_proxy.lines().for_each(|line_out|{
            let log_out = line_out.unwrap();
            println!("out: {}", log_out);
            let idx_0 = log_out.find("'").unwrap_or(0);
            let idx_1 = log_out.rfind("'").unwrap_or(0);
            let substring = &log_out[idx_0+1..idx_1];
            println!("substring: {}", substring);
            let port_number = substring.parse::<u16>();
            match port_number {
                Ok(ok) => {
                    //connection_port = port_number.unwrap();
                    println!("Initializing private backend...");
                    let mut python_cmd: Command = Command::new("python3");

                    python_cmd
                        .arg(&backend_directory)
                        .arg(format!("{}",port_number.unwrap()))
                        .assert()
                        .success()
                        .stdout(contains("Socket connected successfully."));
                },
                Err(e) => {},
            }

        });
        err_proxy.lines().for_each(|line_err|{
            println!("err: {}", line_err.unwrap());
        });
    });

    // Start the backend
    // let port = 5555; // to be updated
    // println!("Initializing private backend...");
    // let mut python_cmd: Command = Command::new("python3");
    //
    // python_cmd
    //     .arg(backend_directory)
    //     .arg(format!("{}",port))
    //     .assert()
    //     .success()
    //     .stdout(contains("Socket connected successfully."));



    // cs_instance
    //     .setup_experiment(Some(1.0e-6_f64), 0.0, None)
    //     .ok()
    //     .expect("setup_experiment");
    //
    // cs_instance
    //     .enter_initialization_mode()
    //     .ok()
    //     .expect("enter_initialization_mode");
    //
    // cs_instance
    //     .exit_initialization_mode()
    //     .ok()
    //     .expect("exit_initialization_mode");
    //
    // // Check for initial outputs as they are calculated
    //
    // let mut real_c = get_real(&import, &mut cs_instance, "real_c");
    // assert_eq!(real_c, 0.0);
    //
    // let mut integer_c = get_integer(&import, &mut cs_instance, "integer_c");
    // assert_eq!(integer_c, 0);
    //
    // cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");
    //
    // set_real(&import, &mut cs_instance, "real_a", 1.0);
    // set_real(&import, &mut cs_instance, "real_b", 2.0);
    //
    // set_integer(&import, &mut cs_instance, "integer_a", 1);
    // set_integer(&import, &mut cs_instance, "integer_b", 2);
    //
    // cs_instance.do_step(0.0, 0.01, false).ok().expect("do_step");
    //
    // real_c = get_real(&import, &mut cs_instance, "real_c");
    // assert_eq!(real_c, 3.0);
    //
    // integer_c = get_integer(&import, &mut cs_instance, "integer_c");
    // assert_eq!(integer_c, 3);

    // cs_instance.terminate().ok().expect("terminate");

    //proxy_fmu_thread.join().unwrap();
}

fn test_fmu_fmi2_distributed_java(fmu_path: PathBuf, backend_directory: PathBuf){
    // Checking first the FMU (proxy)
    assert!(
        fmu_path.exists(),
        "The file {} was not generated successfully.",
        fmu_path.display()
    );

    let fmu_path_str = fmu_path.as_path();

    let vdm_check_2_jar = get_vdm_check_jar("2");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_2_jar.as_path())
        .arg(fmu_path_str)
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let fmu_file = File::open(fmu_path.clone()).unwrap();

    let import: Fmi2Import = import::new::<File, Fmi2Import>(fmu_file).unwrap();

    assert_eq!(import.model_description().fmi_version, "2.0");

    let mut cs_instance: fmi::fmi2::instance::Instance<fmi::fmi2::instance::CS> = import.instantiate_cs("instance", false, true).unwrap();
    assert_eq!(
        fmi::fmi2::instance::Common::get_version(&cs_instance),
        "2.0"
    );

    let port = 5555; // to be updated

    // Start the backend
    //let source_directory_folder = get_current_working_dir().unwrap();
    // let backend_directory_folder = Path::new(&backend_directory);
    // assert!(std::env::set_current_dir(&backend_directory_folder).is_ok());
    // println!("Successfully changed working directory to {}.", &backend_directory);
    let mut gradle_cmd: Command = Command::new("sh");

    gradle_cmd
        .arg(backend_directory)
        .arg("run")
        .arg(format!("--args='{}'",port))
        .assert()
        .success()
        .stdout(contains("Socket connected successfully."));

    // assert!(std::env::set_current_dir(&source_directory_folder).is_ok());
    // println!("Successfully changed to root directory.");

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

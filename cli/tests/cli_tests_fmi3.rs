use std::{fs::File, path::PathBuf};

use assert_cmd::{assert, Command};
use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{CoSimulation, InstanceCS, Common},
    },
    import,
    schema::fmi3::AbstractVariableTrait,
    traits::{FmiImport, FmiStatus},
};
use fmi::schema::Error;
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
    test_fmu_fmi3(python_fmu)
}

fn test_fmu_fmi3(fmu_path: PathBuf) {
    assert!(
        fmu_path.exists(),
        "The file {} does not exist",
        fmu_path.display()
    );

    let vdm_check_3_jar = get_vdm_check_jar("3");

    let mut vdm_check_cmd: Command = Command::new("java");

    vdm_check_cmd
        .arg("-jar")
        .arg(vdm_check_3_jar.as_path())
        .arg(fmu_path.as_path())
        .assert()
        .success()
        .stdout(contains("No errors found."));

    // Load FMU and interact with it

    let fmu_file = File::open(fmu_path).unwrap();

    let import: Fmi3Import = import::new::<File, Fmi3Import>(fmu_file).unwrap();

    assert!(import.model_description().fmi_version.starts_with("3.0"));


    // Instantiate FMU, using event mode (event_mode_used = True)
    let mut cs_instance: InstanceCS = import.instantiate_cs("instance", false, true, true, false, &[]).unwrap();
    assert_eq!(
        cs_instance.get_version(),
        "3.0"
    );

    // Enter initialization mode
    match cs_instance.enter_initialization_mode(Some(1.0e-6_f64), 0.0, None).ok() {
        Ok(..) => {},
        Err(e) => panic!("enter_initialization_mode failed: {:?}", e),
    }

    // Get interval of periodic input clock
    let (interval, qualifier): (f64, i32) = get_interval_decimal(&import, &mut cs_instance, &1001);
    assert_eq!(interval, 1.0);
    assert_eq!(qualifier, 2);

    // Exit initialization mode
    match cs_instance.exit_initialization_mode().ok() {
        Ok(..) => {},
        Err(e) => panic!("exit_initialization_mode failed: {:?}", e),
    }

    // Using event mode, so we should now be in event mode
    // Call update discrete states to signal end of event mode
    let mut next_event_time: Option<f64> = None;
    match cs_instance.update_discrete_states(&mut false, &mut false, &mut false, &mut false, &mut next_event_time).ok() {
        Ok(..) => {
            assert_eq!(
                next_event_time,
                Some(1.0),
                "Expected next_event_time to be Some(1.0), but got: {:?}",
                next_event_time
            );
        },
        Err(e) => panic!("do_step failed: {:?}", e),
    }

    // Enter step mode instead
    match cs_instance.enter_step_mode().ok() {
        Ok(..) => {},
        Err(e) => panic!("enter_step_mode failed: {:?}", e),
    }

    // Do step
    let mut last_successful_time = 0.0;
    match cs_instance.do_step(0.0, 1.0, false, &mut false, &mut false, &mut false, &mut last_successful_time).ok() {
        Ok(..) => {
            assert_eq!(
                last_successful_time,
                1.0,
                "Expected last_successful_time to be 1.0, but got: {}",
                last_successful_time
            );
        },
        Err(e) => panic!("do_step failed: {:?}", e),
    }

    // Check for initial outputs as they are calculated
    let float32_c: f32 = get_float32(&import, &mut cs_instance, "float32_c");
    assert_eq!(float32_c, 0.0);

    let float64_c: f64 = get_float64(&import, &mut cs_instance, "float64_c");
    assert_eq!(float64_c, 0.0);

    let int8_c: i8 = get_int8(&import, &mut cs_instance, "int8_c");
    assert_eq!(int8_c, 0);

    let uint8_c: u8 = get_uint8(&import, &mut cs_instance, "uint8_c");
    assert_eq!(uint8_c, 0);

    let int16_c: i16 = get_int16(&import, &mut cs_instance, "int16_c");
    assert_eq!(int16_c, 0);

    let uint16_c: u16 = get_uint16(&import, &mut cs_instance, "uint16_c");
    assert_eq!(uint16_c, 0);

    let int32_c: i32 = get_int32(&import, &mut cs_instance, "int32_c");
    assert_eq!(int32_c, 0);

    let uint32_c: u32 = get_uint32(&import, &mut cs_instance, "uint32_c");
    assert_eq!(uint32_c, 0);

    let int64_c: i64 = get_int64(&import, &mut cs_instance, "int64_c");
    assert_eq!(int64_c, 0);

    let uint64_c: u64 = get_uint64(&import, &mut cs_instance, "uint64_c");
    assert_eq!(uint64_c, 0);

    let boolean_c: bool = get_boolean(&import, &mut cs_instance, "boolean_c");
    assert_eq!(boolean_c, false);

    let clock_c: bool = get_clock(&import, &mut cs_instance, &1003);
    assert_eq!(clock_c, false);

    let string_c: String = get_string(&import, &mut cs_instance, "string_c");
    assert_eq!(string_c, "");

    // Set inputs
    set_float32(&import, &mut cs_instance, "float32_a", 1.0);
    set_float32(&import, &mut cs_instance, "float32_b", 2.0);
    set_float64(&import, &mut cs_instance, "float64_a", 1.0);
    set_float64(&import, &mut cs_instance, "float64_b", 2.0);
    set_int8(&import, &mut cs_instance, "int8_a", 1);
    set_int8(&import, &mut cs_instance, "int8_b", 2);
    set_uint8(&import, &mut cs_instance, "uint8_a", 1);
    set_uint8(&import, &mut cs_instance, "uint8_b", 2);
    set_int16(&import, &mut cs_instance, "int16_a", 1);
    set_int16(&import, &mut cs_instance, "int16_b", 2);
    set_uint16(&import, &mut cs_instance, "uint16_a", 1);
    set_uint16(&import, &mut cs_instance, "uint16_b", 2);
    set_int32(&import, &mut cs_instance, "int32_a", 1);
    set_int32(&import, &mut cs_instance, "int32_b", 2);
    set_uint32(&import, &mut cs_instance, "uint32_a", 1);
    set_uint32(&import, &mut cs_instance, "uint32_b", 2);
    set_int64(&import, &mut cs_instance, "int64_a", 1);
    set_int64(&import, &mut cs_instance, "int64_b", 2);
    set_uint64(&import, &mut cs_instance, "uint64_a", 1);
    set_uint64(&import, &mut cs_instance, "uint64_b", 2);
    set_boolean(&import, &mut cs_instance, "boolean_a", true);
    set_boolean(&import, &mut cs_instance, "boolean_b", false);
    set_string(&import, &mut cs_instance, "string_a", "Hello ".to_string());
    set_string(&import, &mut cs_instance, "string_b", "World!".to_string());
    set_binary(&import, &mut cs_instance, "binary_a", vec![1]);
    set_binary(&import, &mut cs_instance, "binary_b", vec![2]);

    // Do step
    match cs_instance.do_step(last_successful_time, 1.0, false, &mut false, &mut false, &mut false, &mut last_successful_time).ok() {
        Ok(..) => {
            assert_eq!(
                last_successful_time,
                2.0,
                "Expected last_successful_time to be 2.0, but got: {}",
                last_successful_time
            );
        },
        Err(e) => panic!("do_step failed: {:?}", e),
    }

    // Check outputs
    let float32_c = get_float32(&import, &mut cs_instance, "float32_c");
    assert_eq!(float32_c, 3.0);

    let float64_c: f64 = get_float64(&import, &mut cs_instance, "float64_c");
    assert_eq!(float64_c, 3.0);

    let int8_c: i8 = get_int8(&import, &mut cs_instance, "int8_c");
    assert_eq!(int8_c, 3);

    let uint8_c: u8 = get_uint8(&import, &mut cs_instance, "uint8_c");
    assert_eq!(uint8_c, 3);

    let int16_c: i16 = get_int16(&import, &mut cs_instance, "int16_c");
    assert_eq!(int16_c, 3);

    let uint16_c: u16 = get_uint16(&import, &mut cs_instance, "uint16_c");
    assert_eq!(uint16_c, 3);

    let int32_c: i32 = get_int32(&import, &mut cs_instance, "int32_c");
    assert_eq!(int32_c, 3);

    let uint32_c: u32 = get_uint32(&import, &mut cs_instance, "uint32_c");
    assert_eq!(uint32_c, 3);

    let int64_c: i64 = get_int64(&import, &mut cs_instance, "int64_c");
    assert_eq!(int64_c, 3);

    let uint64_c: u64 = get_uint64(&import, &mut cs_instance, "uint64_c");
    assert_eq!(uint64_c, 3);

    let boolean_c: bool = get_boolean(&import, &mut cs_instance, "boolean_c");
    assert_eq!(boolean_c, true);

    let string_c: String = get_string(&import, &mut cs_instance, "string_c");
    assert_eq!(string_c, "Hello World!");

    // Enter event mode
    cs_instance.enter_event_mode().ok().expect("enter_event_mode failed");

    // Tick input clocks
    set_clock(&import, &mut cs_instance, &1001, true);
    set_clock(&import, &mut cs_instance, &1002, true);

    // Get output clocks
    let clock_c: bool = get_clock(&import, &mut cs_instance, &1003);
    assert_eq!(clock_c, true);

    // Terminate
    cs_instance.terminate().ok().expect("terminate failed");
}

// Default fmu utility functions
fn get_float32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> f32 {
    let mut val: [f32; 1] = [0.0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_float32 failed for {}", var);
    cs_instance.get_float32(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_float64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> f64 {
    let mut val: [f64; 1] = [0.0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_float64 failed for {}", var);
    cs_instance.get_float64(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_int8(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> i8 {
    let mut val: [i8; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_int8 failed for {}", var);
    cs_instance.get_int8(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_uint8(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> u8 {
    let mut val: [u8; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_uint8 failed for {}", var);
    cs_instance.get_uint8(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_int16(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> i16 {
    let mut val: [i16; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_int16 failed for {}", var);
    cs_instance.get_int16(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_uint16(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> u16 {
    let mut val: [u16; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_uint16 failed for {}", var);
    cs_instance.get_uint16(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_int32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> i32 {
    let mut val: [i32; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_int32 failed for {}", var);
    cs_instance.get_int32(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_uint32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> u32 {
    let mut val: [u32; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_uint32 failed for {}", var);
    cs_instance.get_uint32(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_int64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> i64 {
    let mut val: [i64; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_int64 failed for {}", var);
    cs_instance.get_int64(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_uint64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> u64 {
    let mut val: [u64; 1] = [0];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_uint64 failed for {}", var);
    cs_instance.get_uint64(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_boolean(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> bool {
    let mut val: [bool; 1] = [false];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_boolean failed for {}", var);
    cs_instance.get_boolean(&[vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_string(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str) -> String {
    let mut val: [String; 1] = [String::new()];
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("get_string failed for {}", var);
    cs_instance.get_string(&[vr], &mut val).ok().expect(&error_msg);

    val[0].clone()
}

fn get_clock(import: &Fmi3Import, cs_instance: &mut InstanceCS, vr: &u32) -> bool {
    let mut val: [bool; 1] = [false];
    let error_msg = format!("get_clock failed for {}", vr);
    cs_instance.get_clock(&[*vr], &mut val).ok().expect(&error_msg);

    val[0]
}

fn get_interval_decimal(import: &Fmi3Import, cs_instance: &mut InstanceCS, vr: &u32) -> (f64, i32) {
    let mut interval: [f64; 1] = [0.0];
    let mut qualifier: [i32; 1] = [0];
    let error_msg = format!("get_interval_decimal failed for {}", vr);
    cs_instance.get_interval_decimal(&[*vr], &mut interval, &mut qualifier).ok().expect(&error_msg);

    (interval[0], qualifier[0])
}

fn set_float32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: f32) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_float32 failed for {}", var);
    cs_instance.set_float32(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_float64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: f64) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_float64 failed for {}", var);
    cs_instance.set_float64(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_int8(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: i8) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_int8 failed for {}", var);
    cs_instance.set_int8(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_uint8(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: u8) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_uint8 failed for {}", var);
    cs_instance.set_uint8(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_int16(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: i16) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_int16 failed for {}", var);
    cs_instance.set_int16(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_uint16(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: u16) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_uint16 failed for {}", var);
    cs_instance.set_uint16(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_int32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: i32) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_int32 failed for {}", var);
    cs_instance.set_int32(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_uint32(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: u32) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_uint32 failed for {}", var);
    cs_instance.set_uint32(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_int64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: i64) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_int64 failed for {}", var);
    cs_instance.set_int64(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_uint64(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: u64) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_uint64 failed for {}", var);
    cs_instance.set_uint64(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_boolean(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: bool) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_boolean failed for {}", var);
    cs_instance.set_boolean(&[vr], &[value]).ok().expect(&error_msg);
}

fn set_string(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: String) {
    let vr: u32 = get_model_variable_reference_by_name(&import, var);
    let error_msg = format!("set_boolean failed for {}", var);
    // Wrap the value in an iterator that yields references to str
    let value_iter = std::iter::once(value.as_str());

    // Pass the value iterator to cs_instance.set_string
    cs_instance.set_string(&[vr], value_iter).ok().expect(&error_msg);
}

fn set_binary(import: &Fmi3Import, cs_instance: &mut InstanceCS, var: &str, value: Vec<u8>
) {
    let vr: u32 = get_model_variable_reference_by_name(import, var);
    let error_msg = format!("set_binary failed for {}", var);
    // Convert the Vec<u8> into an iterator of &[u8]
    let value_iter = std::iter::once(value.as_slice());

    // Call the set_binary function with the value reference and binary data
    cs_instance.set_binary(&[vr], value_iter).ok().expect(&error_msg);
}


fn set_clock(import: &Fmi3Import, cs_instance: &mut InstanceCS, vr: &u32, value: bool) {
    let error_msg = format!("set_clock failed for {}", vr);
    cs_instance.set_clock(&[*vr], &[value]).ok().expect(&error_msg);
}

fn get_model_variable_reference_by_name(import: &Fmi3Import, name: &str) -> u32 {
    import.model_description()
        .model_variables.iter_abstract()
        .find(|var| var.name() == name)
        .ok_or_else(|| Error::VariableNotFound(name.to_owned()))
        .unwrap()
        .value_reference()
}

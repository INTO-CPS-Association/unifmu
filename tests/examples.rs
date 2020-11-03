use core::ptr::null_mut;
use libc::puts;
use std::env::current_dir;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_double;
use std::os::raw::c_int;
use std::os::raw::c_void;
use wrapper::fmi2::{Fmi2CallbackFunctions, Fmi2Status};
use wrapper::{
    fmi2CancelStep, fmi2DoStep, fmi2EnterInitializationMode, fmi2ExitInitializationMode,
    fmi2FreeInstance, fmi2GetBoolean, fmi2GetInteger, fmi2GetReal, fmi2GetString, fmi2Instantiate,
    fmi2SetBoolean, fmi2SetInteger, fmi2SetReal, fmi2SetupExperiment,
};

use url::Url;

pub unsafe fn cstr_to_string(cstr: *const c_char) -> String {
    CStr::from_ptr(cstr).to_string_lossy().into_owned()
}

pub fn get_example_resources_uri(example_name: &str) -> String {
    let path = current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("exported")
        .join(example_name)
        .join("resources");

    assert!(
        path.is_dir(),
        "Examples directory does not exist, ensure that examples have already been exported"
    );

    Url::from_file_path(path)
        .expect("unable to convert path to file URI")
        .to_string()
}

extern "C" fn logger(
    _component_environment: *mut c_void,
    instance_name: *const c_char,
    status: c_int,
    category: *const c_char,
    message: *const c_char,
) {
    let instance_name = unsafe { cstr_to_string(instance_name) };
    let category = unsafe { cstr_to_string(category) };
    let message = unsafe { cstr_to_string(message) };
    println!(
        "Callback:{}:{}:{}:{}",
        instance_name, category, status, message
    )
}

fn test_fmu(name: &str) {
    let instance_name = b"a\0";
    let instance_name_ptr = instance_name.as_ptr();

    let fmu_type = 1;
    let guid = b"1234\0";
    let guid_ptr = guid.as_ptr();

    let fmu_resources_path = current_dir()
        .unwrap()
        .join("tests")
        .join("examples")
        .join(name)
        .join("resources");

    let fmu_resources_uri_cstr = CString::new(
        Url::from_directory_path(fmu_resources_path)
            .unwrap()
            .as_str(),
    )
    .unwrap();

    let functions = Fmi2CallbackFunctions {
        logger: Some(logger),
        allocate_memory: None,
        free_memory: None,
        step_finished: None,
        component_environment: None,
    };
    let visible: c_int = 0;
    let logging_on: c_int = 0;

    let handle = fmi2Instantiate(
        instance_name_ptr as *const i8,
        fmu_type,
        guid_ptr as *const i8,
        fmu_resources_uri_cstr.as_ptr() as *const i8,
        functions,
        visible,
        logging_on,
    );

    assert_ne!(handle, null_mut());

    assert_eq!(
        fmi2SetupExperiment(handle, 0, 0.0, 0.0, 0, 0.0),
        Fmi2Status::Fmi2OK as i32
    );

    assert_eq!(
        fmi2EnterInitializationMode(handle),
        Fmi2Status::Fmi2OK as i32
    );

    assert_eq!(
        fmi2ExitInitializationMode(handle),
        Fmi2Status::Fmi2OK as i32
    );
    // ============== real ===============
    let mut values: [c_double; 3] = [0.0, 0.0, 0.0];
    fmi2GetReal(
        handle,
        [1, 2, 3].as_ptr(),
        values.len(),
        values.as_mut_ptr(),
    );
    assert_eq!(values, [0.0, 0.0, 0.0]);

    let mut values: [c_double; 2] = [10.0, 20.0];
    fmi2SetReal(handle, [0, 1].as_ptr(), values.len(), values.as_mut_ptr());

    fmi2DoStep(handle, 0.0, 1.0, 0);

    let mut values: [c_double; 1] = [0.0];
    fmi2GetReal(handle, [2].as_ptr(), values.len(), values.as_mut_ptr());
    assert_eq!(values, [30.0]);

    // ============== integer ===============
    let mut values: [c_int; 3] = [0, 0, 0];
    fmi2GetInteger(
        handle,
        [3, 4, 5].as_ptr(),
        values.len(),
        values.as_mut_ptr(),
    );
    assert_eq!(values, [0, 0, 0]);

    let mut values: [c_int; 2] = [10, 20];
    fmi2SetInteger(handle, [3, 4].as_ptr(), values.len(), values.as_mut_ptr());

    fmi2DoStep(handle, 0.0, 1.0, 0);

    let mut values: [c_int; 1] = [0];
    fmi2GetInteger(handle, [5].as_ptr(), values.len(), values.as_mut_ptr());
    assert_eq!(values, [30]);

    // ============== boolean ===============
    let mut values: [c_int; 3] = [0, 0, 0];
    fmi2GetBoolean(
        handle,
        [6, 7, 8].as_ptr(),
        values.len(),
        values.as_mut_ptr(),
    );
    assert_eq!(values, [0, 0, 0]);

    let mut values: [c_int; 2] = [1, 1];
    fmi2SetBoolean(handle, [6, 7].as_ptr(), values.len(), values.as_mut_ptr());

    fmi2DoStep(handle, 0.0, 1.0, 0);

    let mut values: [c_int; 1] = [0];
    fmi2GetBoolean(handle, [8].as_ptr(), values.len(), values.as_mut_ptr());
    assert_eq!(values, [1]);

    // cleanup

    fmi2FreeInstance(handle);
}

#[test]
fn python_fmu() {
    test_fmu("python_fmu")
}

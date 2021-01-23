#![allow(non_snake_case)]

extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;

use dlopen::wrapper::{Container, WrapperApi};
use libc::{c_char, c_double, c_int, c_uint, size_t};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr::null_mut,
};
use unifmu::{fmi2::Fmi2CallbackFunctions, SlaveHandle};

#[derive(WrapperApi)]
struct Fmi2Api {
    fmi2GetTypesPlatform: extern "C" fn() -> *const c_char,
    fmi2GetVersion: extern "C" fn() -> *const c_char,
    fmi2Instantiate: extern "C" fn(
        _instance_name: *const c_char,
        _fmu_type: c_int,
        _fmu_guid: *const c_char,
        fmu_resource_location: *const c_char,
        _functions: Option<Fmi2CallbackFunctions>,
        _visible: c_int,
        _logging_on: c_int,
    ) -> *mut SlaveHandle,

    fmi2SetupExperiment: extern "C" fn(
        slave_handle: *const SlaveHandle,
        tolerance_defined: c_int,
        tolerance: c_double,
        start_time: c_double,
        stop_time_defined: c_int,
        stop_time: c_double,
    ) -> c_int,

    fmi2FreeInstance: extern "C" fn(c: *mut c_int),

    fmi2EnterInitializationMode: extern "C" fn(c: *const SlaveHandle) -> c_int,
    fmi2ExitInitializationMode: extern "C" fn(c: *const SlaveHandle) -> c_int,
    fmi2Terminate: extern "C" fn(c: *const SlaveHandle) -> c_int,
    fmi2Reset: extern "C" fn(c: *const SlaveHandle) -> c_int,
    fmi2GetFMUstate: extern "C" fn(slave_handle: *const c_int, state: *mut *mut c_int) -> c_int,
    fmi2SerializedFMUstateSize: extern "C" fn(
        slave_handle: *const c_int,
        state_handle: *const c_int,
        size: *mut size_t,
    ) -> c_int,

    fmi2GetReal: extern "C" fn(
        c: *const SlaveHandle,
        vr: *const c_uint,
        nvr: usize,
        values: *mut c_double,
    ) -> c_int,

    fmi2GetInteger: extern "C" fn(
        c: *const SlaveHandle,
        vr: *const c_uint,
        nvr: usize,
        values: *mut c_int,
    ) -> c_int,

    fmi2GetBoolean: extern "C" fn(
        c: *const SlaveHandle,
        vr: *const c_uint,
        nvr: usize,
        values: *mut c_int,
    ) -> c_int,

    fmi2GetString: extern "C" fn(
        c: *const SlaveHandle,
        vr: *const c_uint,
        nvr: usize,
        values: *mut *const c_char,
    ) -> c_int,

    fmi2SetReal: extern "C" fn(
        c: *const c_int,
        vr: *const c_uint,
        nvr: usize,
        values: *const c_double,
    ) -> c_int,

    fmi2SetInteger: extern "C" fn(
        c: *const c_int,
        vr: *const c_uint,
        nvr: usize,
        values: *const c_int,
    ) -> c_int,

    fmi2SetBoolean: extern "C" fn(
        c: *const c_int,
        vr: *const c_uint,
        nvr: usize,
        values: *const c_int,
    ) -> c_int,

    fmi2SetString: extern "C" fn(
        c: *const c_int,
        vr: *const c_uint,
        nvr: usize,
        values: *const *const c_char,
    ) -> c_int,
}

/// credits https://stackoverflow.com/questions/62118412/is-it-possible-to-build-a-hashmap-of-str-referencing-environment-variables
fn os_env_hashmap() -> HashMap<String, String> {
    let mut map = HashMap::new();
    use std::env;
    for (key, val) in env::vars_os() {
        // Use pattern bindings instead of testing .is_some() followed by .unwrap()
        if let (Ok(k), Ok(v)) = (key.into_string(), val.into_string()) {
            map.insert(k, v);
        }
    }
    return map;
}

#[test]
fn test_adder() {
    let env_values = os_env_hashmap();
    let resource_uri = env_values.get("UNIFMU_ADDER_RESOURCES_URI")
    .expect("To run integration tests an environement variable must be defined `UNIFMU_ADDER_RESOURCES_URI`. Please invoke `python build.py --update-wrapper ----test-integration` in the root of the repo instead of running the tests manually");
    let shared_object_path = env_values.get("UNIFMU_ADDER_LIBRARY")
    .expect(
        "To run integration tests an environement variable must be defined `UNIFMU_ADDER_LIBRARY`. Please invoke `python build.py --update-wrapper ----test-integration` in the root of the repo instead of running the tests manually",
    );

    format!("{:?}", resource_uri);
    format!("{:?}", shared_object_path);

    let f: Container<Fmi2Api> = unsafe { Container::load(shared_object_path) }
        .expect("Could not open library or load symbols");

    unsafe {
        assert_eq!(
            CStr::from_ptr(f.fmi2GetTypesPlatform()),
            CStr::from_ptr(b"default\0".as_ptr().cast())
        );
        assert_eq!(
            CStr::from_ptr(f.fmi2GetVersion()),
            CStr::from_ptr(b"2.0\0".as_ptr().cast())
        );

        let name = b"adder\0".as_ptr();
        let guid = "\0".as_ptr();

        let resource_uri = CString::new(resource_uri.to_owned()).unwrap();
        let ptr = resource_uri.as_ptr();

        let handle = f.fmi2Instantiate(name.cast(), 1, guid.cast(), ptr, None, 0, 0);
        assert_ne!(handle, std::ptr::null_mut());

        let get_and_set = || {
            // reals
            {
                let mut vals: [c_double; 2] = [-1.0, -1.0];
                let mut refs: [c_uint; 2] = [0, 1];

                assert_eq!(
                    f.fmi2GetReal(handle, refs.as_ptr(), 2, vals.as_mut_ptr()),
                    0
                );
                assert!(vals[0] == 0.0 && vals[1] == 0.0); // 0.0 is default
                vals[0] = 1.0;
                vals[1] = 1.0;
                assert!(f.fmi2SetReal(handle, refs.as_ptr(), 2, vals.as_ptr()) == 0);
                refs[0] = 2;
                assert!(f.fmi2GetReal(handle, refs.as_ptr(), 1, vals.as_mut_ptr()) == 0);
                assert_eq!(vals[0], 2.0);
            }
        };

        let t_start: c_double = 0.0;
        let t_end: c_double = 1.0;
        assert!(f.fmi2SetupExperiment(handle, 0, 0.0, t_start, 1, t_end) == 0);
        assert!(f.fmi2EnterInitializationMode(handle) == 0);
        assert!(f.fmi2ExitInitializationMode(handle) == 0);

        get_and_set();

        // let mut state_ptr = [null_mut()];

        // assert!(state_ptr[0] == null_mut());
        // assert!(f.fmi2GetFMUstate(handle, state_ptr.as_mut_ptr()) == 0);
        // assert!(state_ptr[0] != null_mut());

        // let state_size: *mut size_t = [0].as_mut_ptr();

        // assert!(f.fmi2SerializedFMUstateSize(handle, *state_ptr.as_mut_ptr(), state_size) == 0);
        // assert!(state_size != null_mut());
        // assert!(*state_size > 0); // not required by spec, but is reasonable assumption

        // No way to check if actually freed
        f.fmi2FreeInstance(handle);
    }
}

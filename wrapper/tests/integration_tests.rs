#![allow(non_snake_case)]

use libc::{c_char, c_double, c_int, c_uint, size_t};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr::null_mut,
};
use unifmu::{Fmi2CallbackFunctions, *};

#[cfg(test)]
mod tests {

    use libc::c_void;
    use safer_ffi::{
        char_p::{char_p_raw, char_p_ref},
        prelude::repr_c,
    };
    use unifmu::Fmi2Status;

    use super::*;

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

        // let f: Container<Fmi2Api> = unsafe { Container::load(shared_object_path) }
        //     .expect("Could not open library or load symbols");

        use safer_ffi::c;
        assert_eq!(fmi2GetTypesPlatform(), c!("default")); // rust analyzer might flag c! as undefined macro
        assert_eq!(fmi2GetVersion(), c!("2.0"));

        extern "C" fn step_finished(_component_environment: *const c_void, _status: i32) {}

        extern "C" fn logger_func(
            _component_environment: *mut c_void,
            instance_name: char_p_raw,
            status: Fmi2Status,
            category: char_p_raw,
            message: char_p_raw,
            // ... variadic functions support in rust seems to be unstable
        ) {
            let instance_name: char_p_ref = unsafe { std::mem::transmute(instance_name) };
            let message: char_p_ref = unsafe { std::mem::transmute(message) };
            let category: char_p_ref = unsafe { std::mem::transmute(category) };

            println!(
                "slave: '{}' sent message: '{}' of category {} with severity: '{:?}'",
                instance_name.to_str(),
                message.to_str(),
                category.to_str(),
                status
            );
        }
        let resource_uri = CString::new(resource_uri.to_owned()).unwrap();
        let resources_cstr = resource_uri.as_c_str();
        let resource_uri = char_p_ref::from(resources_cstr);

        let callbacks = Fmi2CallbackFunctions {
            logger: logger_func,
            allocate_memory: None,
            free_memory: None,
            step_finished: Some(step_finished),
            component_environment: &None,
        };
        let name = c!("adder");
        let guid = c!("");

        let slave = fmi2Instantiate(
            name,
            Fmi2Type::Fmi2CoSimulation,
            guid,
            resource_uri,
            &callbacks,
            0,
            0,
        );

        // assert!(&slave.is_none());

        let mut slave = slave.unwrap();

        // Adder has two inputs and a single output for each data type
        // for real and integers the output is the sum, for booleans it they are logic AND'ed and strings are concatenated

        let check_input_outputs = |slave: &mut repr_c::Box<Slave>| {
            // reals
            {
                let mut vals: [c_double; 2] = [-1.0, -1.0];
                let mut refs: [c_uint; 2] = [0, 1];

                assert_eq!(
                    fmi2GetReal(slave, refs.as_ptr(), 2, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK
                );
                assert!(vals[0] == 0.0 && vals[1] == 0.0); // 0.0 is default
                vals[0] = 1.0;
                vals[1] = 1.0;
                assert_eq!(
                    fmi2SetReal(slave, refs.as_ptr(), 2, vals.as_ptr()),
                    Fmi2Status::Fmi2OK
                );
                refs[0] = 2;
                assert_eq!(
                    fmi2GetReal(slave, refs.as_ptr(), 1, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK
                );
                assert_eq!(vals[0], 2.0);
            }
            // integer
            {
                let mut vals: [c_int; 2] = [-1, -1];
                let mut refs: [c_uint; 2] = [3, 4];

                assert_eq!(
                    fmi2GetInteger(slave, refs.as_ptr(), 2, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK,
                );
                assert!(vals[0] == 0 && vals[1] == 0); // 0.0 is default
                vals[0] = 1;
                vals[1] = 1;
                assert_eq!(
                    fmi2SetInteger(slave, refs.as_ptr(), 2, vals.as_ptr()),
                    Fmi2Status::Fmi2OK
                );
                refs[0] = 5;
                assert_eq!(
                    fmi2GetInteger(slave, refs.as_ptr(), 1, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK
                );
                assert_eq!(vals[0], 2);
            }
            // boolean
            {
                let mut vals: [c_int; 2] = [-1, -1];
                let mut refs: [c_uint; 2] = [6, 7];

                assert_eq!(
                    fmi2GetBoolean(slave, refs.as_ptr(), 2, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK
                );
                assert_eq!(vals[0], 0);
                assert_eq!(vals[1], 0); // 0.0 is default
                vals[0] = 1;
                vals[1] = 1;
                assert_eq!(
                    fmi2SetBoolean(slave, refs.as_ptr(), 2, vals.as_ptr()),
                    Fmi2Status::Fmi2OK
                );
                refs[0] = 8;
                assert_eq!(
                    fmi2GetBoolean(slave, refs.as_ptr(), 1, vals.as_mut_ptr()),
                    Fmi2Status::Fmi2OK
                );
                assert_eq!(vals[0], 1);
            }

            // strings
            {
                let mut vals: [*mut c_char; 3] = [null_mut(), null_mut(), null_mut()];
                let mut refs: [c_uint; 3] = [9, 10, 11];

                assert_eq!(
                    fmi2GetString(slave, refs.as_ptr(), 3, vals.as_mut_ptr().cast()),
                    Fmi2Status::Fmi2OK
                );

                let streq = |a: *const c_char, b: *const c_char| -> bool {
                    let a_cstr = unsafe { CStr::from_ptr(a) };
                    let b_cstr = unsafe { CStr::from_ptr(b) };
                    a_cstr == b_cstr
                };

                let expected = b"\0";
                assert!(streq(expected.as_ptr().cast(), vals[0]));
                assert!(streq(expected.as_ptr().cast(), vals[1]));
                assert!(streq(expected.as_ptr().cast(), vals[2]));

                let abc = b"abc\0";
                let def = b"def\0";
                let vals: [*const c_char; 2] = [abc.as_ptr().cast(), def.as_ptr().cast()];

                assert_eq!(
                    fmi2SetString(slave, refs.as_ptr(), 2, vals.as_ptr()),
                    Fmi2Status::Fmi2OK
                );
                refs[0] = 11;

                let mut vals: [*mut c_char; 2] = [null_mut(), null_mut()];
                assert_eq!(
                    fmi2GetString(slave, refs.as_ptr(), 1, vals.as_mut_ptr().cast()),
                    Fmi2Status::Fmi2OK
                );
                let expected = b"abcdef\0";
                streq(expected.as_ptr().cast(), vals[0]);
            }
        };

        let t_start: c_double = 0.0;
        let t_end: c_double = 1.0;
        let step_size: c_double = 0.01;
        assert_eq!(
            fmi2SetupExperiment(&mut slave, 0, 0.0, t_start, 1, t_end),
            Fmi2Status::Fmi2OK
        );
        assert_eq!(fmi2EnterInitializationMode(&mut slave), Fmi2Status::Fmi2OK);
        assert_eq!(fmi2ExitInitializationMode(&mut slave), Fmi2Status::Fmi2OK);

        let mut state: repr_c::Box<Option<SlaveState>> = repr_c::Box::new(None);
        // let mut state_ptr: [*mut i32; 1] = [null_mut()];

        // assert_eq!(state_ptr[0], null_mut());

        assert_eq!(fmi2GetFMUstate(&mut slave, &mut state), Fmi2Status::Fmi2OK);
        assert!(!&state.is_none());

        // assert_ne!(state_ptr[0], null_mut());

        check_input_outputs(&mut slave);

        let mut cur_time: c_double = t_start;

        for _ in 0..100 {
            assert_eq!(
                fmi2DoStep(&mut slave, cur_time, step_size, 0),
                Fmi2Status::Fmi2OK
            );
            cur_time += step_size;
        }

        // // roll back to initial state, then check if it behaves as newly intialized

        let state = repr_c::Box::new(state.take().unwrap());

        assert_eq!(fmi2SetFMUstate(&mut slave, &state), Fmi2Status::Fmi2OK);

        check_input_outputs(&mut slave);

        let mut state_size: size_t = 0;

        assert_eq!(
            fmi2SerializedFMUstateSize(&mut slave, &state, &mut state_size),
            Fmi2Status::Fmi2OK
        );
        assert!(state_size > 0); // not defined by spec, but is reasonable assumption

        let mut state_buffer: Vec<u8> = Vec::with_capacity(state_size);

        assert_eq!(
            fmi2SerializeFMUstate(&mut slave, &state, state_buffer.as_mut_ptr(), state_size),
            Fmi2Status::Fmi2OK,
        );

        assert_eq!(
            fmi2FreeFMUstate(&mut slave, Some(state)),
            Fmi2Status::Fmi2OK
        );

        let mut state: repr_c::Box<Option<SlaveState>> = repr_c::Box::new(None);

        assert_eq!(
            fmi2DeSerializeFMUstate(&mut slave, state_buffer.as_ptr(), state_size, &mut state),
            Fmi2Status::Fmi2OK,
        );

        assert!(!state.is_none());

        let state = repr_c::Box::new(state.take().unwrap());
        assert_eq!(fmi2SetFMUstate(&mut slave, &state), Fmi2Status::Fmi2OK);

        fmi2FreeFMUstate(&mut slave, Some(state));

        check_input_outputs(&mut slave);

        fmi2Terminate(&mut slave);
        fmi2Reset(&mut slave);

        // // No way to check if actually freed

        fmi2FreeInstance(Some(slave));
    }
}

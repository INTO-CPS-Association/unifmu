#![allow(non_snake_case)]

use unifmu::generate;

#[cfg(test)]
mod tests {

    use fmi2api::{
        fmi2DeSerializeFMUstate, fmi2DoStep, fmi2EnterInitializationMode,
        fmi2ExitInitializationMode, fmi2FreeFMUstate, fmi2FreeInstance, fmi2GetBoolean,
        fmi2GetFMUstate, fmi2GetInteger, fmi2GetReal, fmi2GetString, fmi2GetTypesPlatform,
        fmi2GetVersion, fmi2Instantiate, fmi2Reset, fmi2SerializeFMUstate,
        fmi2SerializedFMUstateSize, fmi2SetBoolean, fmi2SetFMUstate, fmi2SetInteger, fmi2SetReal,
        fmi2SetString, fmi2SetupExperiment, fmi2Terminate, Fmi2CallbackFunctions, Fmi2Status,
        Fmi2Type, Slave, SlaveState,
    };
    use libc::{c_char, c_double, c_int, c_uint, c_void, size_t};
    use safer_ffi::{
        c,
        char_p::{char_p_raw, char_p_ref},
        prelude::repr_c,
    };
    use std::{
        ffi::{CStr, CString},
        path::Path,
        ptr::null_mut,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tempfile::TempDir;
    use unifmu::Language;
    use url::Url;

    use super::*;

    #[test]
    fn test_placeholder_python() {
        let tmpdir = TempDir::new().unwrap();
        generate(&Language::Python, tmpdir.path(), false, false).unwrap();
        test_placeholder_functionality(tmpdir.path());
    }

    #[test]
    fn test_placeholder_python_dockerize() {
        let tmpdir = TempDir::new().unwrap();
        generate(&Language::Python, tmpdir.path(), false, true).unwrap();
        test_placeholder_functionality(tmpdir.path());
    }

    #[test]
    fn test_placeholder_csharp() {
        let tmpdir = TempDir::new().unwrap();
        generate(&Language::CSharp, tmpdir.path(), false, false).unwrap();
        test_placeholder_functionality(tmpdir.path());
    }

    fn test_placeholder_functionality(rootdir: &Path) {
        let resources_path = rootdir.join("resources");
        let url = Url::from_file_path(resources_path).unwrap();
        let resource_uri = url.as_str();

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

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        // let guid = c!("abc");

        let guid = CString::new(since_the_epoch).unwrap();
        let guid = guid.as_c_str();

        let slave = fmi2Instantiate(
            name,
            Fmi2Type::Fmi2CoSimulation,
            guid.into(),
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
                fmi2DoStep(slave, 0.0, 0.0, 0);
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
                fmi2DoStep(slave, 0.0, 0.0, 0);
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
                vals[1] = 0;
                assert_eq!(
                    fmi2SetBoolean(slave, refs.as_ptr(), 2, vals.as_ptr()),
                    Fmi2Status::Fmi2OK
                );
                fmi2DoStep(slave, 0.0, 0.0, 0);
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
                fmi2DoStep(slave, 0.0, 0.0, 0);
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

        // rollback to initial state, then check if it behaves as newly intialized

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

        // No way to check if actually freed

        fmi2FreeInstance(Some(slave));
    }
}

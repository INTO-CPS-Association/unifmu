use super::*;
use crate::common::dispatcher::Dispatcher;
use fmi2_logger_test as _;
use std::ptr;

extern "C" {
    fn fmi2_test_logger(
        environment: *const fmi2_types::ComponentEnvironment,
        instance_name: Fmi2String,
        status: Fmi2Status,
        category: Fmi2String,
        format: Fmi2String,
        ...
    );
}

fn slave() -> Fmi2Slave {
    Fmi2Slave::new(
        Dispatcher::remote(|_| {}).unwrap(),
        Fmi2Logger::new(fmi2_test_logger, c"state-test".as_ptr(), ptr::null(), false),
    )
}

#[test]
fn free_deserialized_fmu_state_clears_handle_and_can_be_repeated() {
    let mut slave = slave();
    let mut state = ptr::null_mut();
    let bytes = [1, 2, 3];
    assert_eq!(
        unsafe { fmi2DeSerializeFMUstate(&mut slave, bytes.as_ptr(), bytes.len(), &mut state) },
        Fmi2Status::Ok
    );
    assert!(!state.is_null());
    assert_eq!(unsafe { &(*state).bytes }, &bytes);
    assert_eq!(fmi2FreeFMUstate(&mut slave, &mut state), Fmi2Status::Ok);
    assert!(state.is_null());
    assert_eq!(fmi2FreeFMUstate(&mut slave, &mut state), Fmi2Status::Ok);
}

#[test]
fn free_null_fmu_state_is_a_no_op() {
    let mut slave = slave();
    assert_eq!(
        fmi2FreeFMUstate(&mut slave, ptr::null_mut()),
        Fmi2Status::Ok
    );
    let mut state = ptr::null_mut();
    assert_eq!(fmi2FreeFMUstate(&mut slave, &mut state), Fmi2Status::Ok);
    assert!(state.is_null());
    assert_eq!(
        fmi2FreeFMUstate(ptr::null_mut(), &mut state),
        Fmi2Status::Ok
    );
}

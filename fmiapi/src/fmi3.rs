#![allow(non_snake_case)]
#![allow(unused_variables)]

use libc::{c_char, size_t};
use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::path::Path;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;
use subprocess::Popen;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::fmi3_dispatcher::Fmi3CommandDispatcher;
use crate::spawn::spawn_fmi3_slave;

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3Status {
    OK = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
}

pub struct Fmi3Slave {
    dispatcher: Fmi3CommandDispatcher,
    last_successful_time: Option<f64>,
    string_buffer: Vec<CString>,
    popen: Popen,
}

impl Fmi3Slave {
    pub fn new(dispatcher: Fmi3CommandDispatcher, popen: Popen) -> Self {
        Self {
            dispatcher,
            popen,
            last_successful_time: None,
            string_buffer: Vec::new(),
        }
    }
}

fn c2s(c: *const c_char) -> String {
    unsafe { CStr::from_ptr(c).to_str().unwrap().to_owned() }
}

// ------------------------------------- FMI FUNCTIONS --------------------------------

static VERSION: &str = "2.0\0";

#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

type Fmi3SlaveType = Box<Fmi3Slave>;
#[no_mangle]
pub extern "C" fn fmi3InstantiateCoSimulation(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: i32,
    logging_on: i32,
    event_mode_used: i32,
    early_return_allowed: i32,
    required_intermediate_variables: *const u32,
    n_required_intermediate_variables: size_t,
    instance_environment: *const c_void,
    log_message: *const c_void,
    intermediate_update: *const c_void,
) -> Option<Fmi3SlaveType> {
    let instance_name = c2s(instance_name);
    let instantiation_token = c2s(instantiation_token);
    let resource_path = c2s(resource_path);
    let required_intermediate_variables = unsafe {
        from_raw_parts(
            required_intermediate_variables,
            n_required_intermediate_variables,
        )
    }
    .to_owned();

    let (mut dispatcher, popen) = spawn_fmi3_slave(&Path::new(&resource_path)).unwrap();

    match dispatcher.fmi3InstantiateCoSimulation(
        instance_name,
        instantiation_token,
        resource_path,
        visible != 0,
        logging_on != 0,
        event_mode_used != 0,
        early_return_allowed != 0,
        required_intermediate_variables,
    ) {
        Ok(_) => Some(Box::new(Fmi3Slave::new(dispatcher, popen))),
        Err(_) => None,
    }
}
#[no_mangle]
pub extern "C" fn fmi3DoStep(
    instance: &mut Fmi3Slave,
    current_communication_point: f64,
    communication_step_size: f64,
    no_set_fmu_state_prior_to_current_point: i32,
    event_handling_needed: *const i32,
    terminate_simulation: *const i32,
    early_return: *const i32,
    last_successful_time: f64,
) -> Fmi3Status {
    match instance.dispatcher.fmi3DoStep(
        current_communication_point,
        communication_step_size,
        no_set_fmu_state_prior_to_current_point != 0,
    ) {
        Ok(s) => match s {
            Fmi3Status::OK | Fmi3Status::Warning => {
                instance.last_successful_time =
                    Some(current_communication_point + communication_step_size);
                s
            }
            s => s,
        },
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3EnterInitializationMode(
    instance: &mut Fmi3Slave,
    tolerance_defined: i32,
    tolerance: f64,
    start_time: f64,
    stop_time_defined: i32,
    stop_time: f64,
) -> Fmi3Status {
    let tolerance = {
        if tolerance_defined != 0 {
            Some(tolerance)
        } else {
            None
        }
    };
    let stop_time = {
        if stop_time_defined != 0 {
            Some(stop_time)
        } else {
            None
        }
    };

    match instance
        .dispatcher
        .fmi3EnterInitializationMode(tolerance, start_time, stop_time)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3ExitInitializationMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    match instance.dispatcher.fmi3ExitInitializationMode() {
        Ok(s) => s,
        Err(_) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut f32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetFloat32(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut f64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetFloat64(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance.dispatcher.fmi3GetInt8(value_references.to_owned()) {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetUint8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetUInt8(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetInt16(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetUint16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetUInt16(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetInt32(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetUint32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetUInt32(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetInt64(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetUint64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut u64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetUInt64(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(&values),
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut i32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values_out = unsafe { from_raw_parts_mut(values, n_values) };

    match instance
        .dispatcher
        .fmi3GetBoolean(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(
                    &values
                        .iter()
                        .map(|v| match v {
                            true => 0,
                            false => 1,
                        })
                        .collect::<Vec<i32>>(),
                ),

                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetString(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *mut *const c_char,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe { from_raw_parts(value_references, n_values) };

    match instance
        .dispatcher
        .fmi3GetString(value_references.to_owned())
    {
        Ok((status, vals)) => {
            match vals {
                Some(vals) => {
                    instance.string_buffer = vals
                        .iter()
                        .map(|s| CString::new(s.as_bytes()).unwrap())
                        .collect();

                    unsafe {
                        for (idx, cstr) in instance.string_buffer.iter().enumerate() {
                            std::ptr::write(values.offset(idx as isize), cstr.as_ptr());
                        }
                    }
                }
                None => (),
            };
            status
        }
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetBinary(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    value_sizes: *const size_t,
    values: *mut *const u8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references =
        unsafe { from_raw_parts(value_references, n_value_references) }.to_owned();
    let values_v = unsafe { from_raw_parts(values, n_values) };
    let value_sizes = unsafe { from_raw_parts(value_sizes, n_values) };

    let values_v: Vec<Vec<u8>> = values_v
        .iter()
        .zip(value_sizes.iter())
        .map(|(v, s)| unsafe { from_raw_parts(*v, *s).to_vec() })
        .collect();

    match instance.dispatcher.fmi3GetBinary(value_references) {
        Ok((s, v)) => match v {
            Some(vs) => unsafe {
                for (idx, v) in vs.iter().enumerate() {
                    std::ptr::write(values.offset(idx as isize), v.as_ptr());
                }
                s
            },
            None => s,
        },
        Err(e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f32,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f64,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i8,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetUint8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u8,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetInt16(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i16,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetUint16(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u16,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetInt32(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetUint32(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u32,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetInt64(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i64,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetUint64(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u64,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetBoolean(
    instance: *mut c_void,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}
#[no_mangle]
pub extern "C" fn fmi3SetBinary(
    instance: *mut c_void,
    value_references: *const i32,
    n_value_references: size_t,
    valueSizes: *const size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

#[no_mangle]
pub extern "C" fn fmi3Terminate(slave: &mut Fmi3Slave) -> Fmi3Status {
    slave
        .dispatcher
        .fmi3Terminate()
        .unwrap_or(Fmi3Status::Error)
}

#[no_mangle]
pub extern "C" fn fmi3FreeInstance(slave: Option<Box<Fmi3Slave>>) {
    let mut slave = slave;

    match slave.as_mut() {
        Some(s) => {
            match s.dispatcher.fmi3FreeInstance() {
                Ok(result) => (),
                Err(e) => eprintln!("An error ocurred when freeing slave"),
            };

            drop(slave)
        }
        None => {}
    }
}
#[no_mangle]
pub extern "C" fn fmi3Reset(slave: &mut Fmi3Slave) -> Fmi3Status {
    slave.dispatcher.fmi3Reset().unwrap_or(Fmi3Status::Error)
}

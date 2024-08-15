#![allow(non_snake_case)]
#![allow(unused_variables)]

use libc::{c_char, size_t};
use url::Url;
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

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3DependencyKind {
	Independent = 0,
	Constant = 1,
	Fixed = 2,
	Tunable = 3,
	Discrete = 4,
	Dependent = 5,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi3IntervalQualifier {
    IntervalNotYetKnown = 0,
    IntervalUnchanged = 1,
    IntervalChanged = 2,
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

pub struct SlaveState {
    bytes: Vec<u8>,
}
impl SlaveState {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes),
        }
    }
}

fn c2s(c: *const c_char) -> String {
    unsafe { CStr::from_ptr(c).to_str().unwrap().to_owned() }
}

// ------------------------------------- FMI FUNCTIONS --------------------------------

static VERSION: &str = "3.0\0";

#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn fmi3SetDebugLogging(
    instance: &mut Fmi3Slave,
    logging_on: i32,
    n_categories: size_t,
    categories: *const *const c_char
) -> Fmi3Status {
    Fmi3Status::Error
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateModelExchange(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: i32,
    logging_on: i32,
    instance_environment: *const c_void,
    log_message: *const c_void,
) -> Option<Fmi3SlaveType> {
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
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
    let required_intermediate_variables = unsafe {
        from_raw_parts(
            required_intermediate_variables,
            n_required_intermediate_variables,
        )
    }
    .to_owned();

    let resource_uri = unsafe {
        match resource_path.as_ref() {
            Some(b) => match CStr::from_ptr(b).to_str() {
                Ok(s) => match Url::parse(s) {
                    Ok(url) => url,
                    Err(e) => panic!("unable to parse resource url"),
                },
                Err(e) => panic!("resource url was not valid utf-8"),
            },
            None => panic!("fmuResourcesLocation was null"),
        }
    };

    let resources_dir = resource_uri.to_file_path().expect(&format!(
        "URI was parsed but could not be converted into a file path, got: '{:?}'.",
        resource_uri
    ));

    let (mut dispatcher, popen) = spawn_fmi3_slave(&Path::new(&resources_dir)).unwrap();

    match dispatcher.fmi3InstantiateCoSimulation(
        instance_name,
        instantiation_token,
        resources_dir.into_os_string().into_string().unwrap(),
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
pub extern "C" fn fmi3InstantiateScheduledExecution(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: i32,
    logging_on: i32,
    instance_environment: *const c_void,
    log_message: *const c_void,
    clock_update: *const c_void,
    lock_preemption: *const c_void,
    unlock_preemption: *const c_void,
) -> Option<Fmi3SlaveType> {
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
}
#[no_mangle]
pub extern "C" fn fmi3DoStep(
    instance: &mut Fmi3Slave,
    current_communication_point: f64,
    communication_step_size: f64,
    no_set_fmu_state_prior_to_current_point: bool,
    event_handling_needed: *mut bool,
    terminate_simulation: *mut bool,
    early_return: *mut bool,
    last_successful_time: *mut f64,
) -> Fmi3Status {
    match instance.dispatcher.fmi3DoStep(
        current_communication_point,
        communication_step_size,
        no_set_fmu_state_prior_to_current_point,
    ) {
        Ok((status, event_handling, terminate, early, successful_time)) => {
            if !last_successful_time.is_null() {
                unsafe {
                    *last_successful_time = successful_time;
                }
            }
            if !event_handling_needed.is_null() {
                unsafe {
                    *event_handling_needed = event_handling;
                }
            }
            if !terminate_simulation.is_null() {
                unsafe {
                    *terminate_simulation = terminate;
                }
            }
            if !early_return.is_null() {
                unsafe {
                    *early_return = early;
                }
            }
            status
        }
        Err(_) => Fmi3Status::Error,
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
pub extern "C" fn fmi3EnterEventMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    match instance.dispatcher.fmi3EnterEventMode() {
        Ok(s) => s,
        Err(_) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3EnterStepMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    match instance.dispatcher.fmi3EnterStepMode() {
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
pub extern "C" fn fmi3GetUInt8(
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
pub extern "C" fn fmi3GetUInt16(
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
pub extern "C" fn fmi3GetUInt32(
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
pub extern "C" fn fmi3GetUInt64(
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
pub extern "C" fn fmi3GetClock(
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
        .fmi3GetClock(value_references.to_owned())
    {
        Ok((status, values)) => {
            match values {
                Some(values) => values_out.copy_from_slice(
                    &values
                        .iter()
                        .map(|v| match v {
                            true => 1,
                            false => 0,
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
pub extern "C" fn fmi3GetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    interval: *mut f64,
	qualifier: *mut i32,
    n_values: size_t,
) -> Fmi3Status {
	let value_references = unsafe { from_raw_parts(value_references, n_value_references) };
	let interval_out = unsafe { from_raw_parts_mut(interval, n_values) };
	let qualifier_out = unsafe { from_raw_parts_mut(qualifier, n_values) };
	
	match instance
		.dispatcher
		.fmi3GetIntervalDecimal(value_references.to_owned())
	{
		Ok((status, interval, qualifier)) => {
			match interval {
				Some(interval) => interval_out.copy_from_slice(&interval),
				None => (),
			};
			match qualifier {
				Some(qualifier) => qualifier_out.copy_from_slice(&qualifier),
				None => (),
			};
			status
		}
		Err(e) => Fmi3Status::Error,
	}
}
#[no_mangle]
pub extern "C" fn fmi3GetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    interval_counters: *const u64,
	resolution: *mut u64,
	qualifier: *mut Fmi3IntervalQualifier,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    shifts: *const f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    counters: *const u64,
	resolutions: *const u64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    intervals: *mut f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    interval_counters: *const u64,
	resolutions: *mut u64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    shifts: *const f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    counters: *const u64,
	resolutions: *const u64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3EvaluateDiscreteStates(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3UpdateDiscreteStates(
    instance: &mut Fmi3Slave,
	discrete_states_need_update: *const i32,
	terminate_simulation: *const i32,
	nominals_continuous_states_changed: *const i32,
	values_continuous_states_changed: *const i32,
	next_event_time_defined: *const i32,
	next_event_time: *const f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3EnterContinuousTimeMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3CompletedIntegratorStep(
    instance: &mut Fmi3Slave,
	no_set_fmu_state_prior_to_current_point: i32,
	enter_event_mode: *const i32,
	terminate_simulation: *const i32,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetTime(
    instance: &mut Fmi3Slave,
	time: f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetContinuousStateDerivatives(
    instance: &mut Fmi3Slave,
	derivatives: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetEventIndicators(
    instance: &mut Fmi3Slave,
	event_indicators: *const f64,
	n_event_indicators: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetNominalsOfContinuousStates(
    instance: &mut Fmi3Slave,
	nominals: *const f64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetNumberOfEventIndicators(
    instance: &mut Fmi3Slave,
	n_event_indicators: *const size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetNumberOfContinuousStates(
    instance: &mut Fmi3Slave,
	n_continuous_states: *const size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetOutputDerivatives(
    instance: &mut Fmi3Slave,
	value_references: *const u32,
	n_value_references: size_t,
	orders: *const i32,
	values: *const f64,
	n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3ActivateModelPartition(
    instance: &mut Fmi3Slave,
	value_reference: u32,
	activation_time: f64,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f32,
    n_values: size_t,
) -> Fmi3Status{
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetFloat32(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const f64,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetFloat64(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i8,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values: Vec<i32> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|&v| v as i32)
        .collect();

    match instance
        .dispatcher
        .fmi3SetInt8(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetUInt8(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u8,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values: Vec<u32> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|&v| v as u32)
        .collect();

    match instance
        .dispatcher
        .fmi3SetUInt8(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i16,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values: Vec<i32> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|&v| v as i32)
        .collect();

    match instance
        .dispatcher
        .fmi3SetInt16(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetUInt16(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u16,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values: Vec<u32> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|&v| v as u32)
        .collect();

    match instance
        .dispatcher
        .fmi3SetUInt16(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetInt32(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetUInt32(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u32,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetUInt32(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i64,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetInt64(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetUInt64(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const u64,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values = unsafe { from_raw_parts(values, n_values) };

    match instance
        .dispatcher
        .fmi3SetUInt64(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
    let references = unsafe { from_raw_parts(value_references, n_value_references) };
    let values: Vec<bool> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|v| *v != 0)
        .collect();

    match instance
        .dispatcher
        .fmi3SetBoolean(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3SetString(
    instance: *mut c_void,
    value_references: *const i32,
    n_value_references: size_t,
    values: *const *const c_char,
    n_values: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
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
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetClock(
    instance: &mut Fmi3Slave,
    value_references: *const u32,
    n_value_references: size_t,
    values: *const i32,
    n_values: size_t,
) -> Fmi3Status {
	let references = unsafe { from_raw_parts(value_references, n_value_references) };
	let values: Vec<bool> = unsafe { from_raw_parts(values, n_values) }
        .iter()
        .map(|v| *v != 0)
        .collect();
	
	match instance
        .dispatcher
        .fmi3SetClock(references, &values)
    {
        Ok(s) => s,
        Err(_e) => Fmi3Status::Error,
    }
}
#[no_mangle]
pub extern "C" fn fmi3GetNumberOfVariableDependencies(
    instance: *mut c_void,
    value_reference: *const i32,
    n_dependencies: *const size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetVariableDependencies(
    instance: *mut c_void,
    dependent: *const i32,
    element_indices_of_dependent: *const size_t,
	independents: *const i32,
	element_indices_of_independents: *const size_t,
	dependency_kinds: *const Fmi3DependencyKind,
	n_dependencies: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetFMUState(
    instance: &mut Fmi3Slave,
    state: &mut Option<SlaveState>,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SetFMUState(
	instance: &mut Fmi3Slave,
	state: &SlaveState,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3FreeFMUState(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SerializedFMUStateSize(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
	size: *const size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3SerializeFMUState(
    instance: &mut Fmi3Slave,
    state: Option<Box<SlaveState>>,
	serialized_state: *const u8,
	size: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3DeserializeFMUState(
    instance: &mut Fmi3Slave,
	serialized_state: *const u8,
	size: size_t,
	state: &SlaveState,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetDirectionalDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const u32,
	n_unknowns: size_t,
	knowns: *const u32,
	n_knowns: size_t,
	delta_knowns: *const f64,
	n_delta_knowns: size_t,
	delta_unknowns: *const f64,
	n_delta_unknowns: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3GetAdjointDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const u32,
	n_unknowns: size_t,
	knowns: *const u32,
	n_knowns: size_t,
	delta_unknowns: *const f64,
	n_delta_unknowns: size_t,
	delta_knowns: *const f64,
	n_delta_knowns: size_t,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3EnterConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    Fmi3Status::Error
}
#[no_mangle]
pub extern "C" fn fmi3ExitConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    Fmi3Status::Error
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

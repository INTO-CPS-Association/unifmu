use crate::fmi2::Slave;
use libc::{c_char, size_t};
use std::{ffi::c_void, ptr::null};

#[repr(i32)]
pub enum Fmi3Status {
    OK = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
}

pub extern "C" fn fmi3InstantiateCoSimulation(
    instanceName: *const c_char,
    instantiationToken: *const c_char,
    resourcePath: *const c_char,
    visible: i32,
    loggingOn: i32,
    eventModeUsed: i32,
    earlyReturnAllowed: i32,
    requiredIntermediateVariables: *const i32,
    nRequiredIntermediateVariables: size_t,
    instanceEnvironment: *const c_void,
    logMessage: *const c_void,
    intermediateUpdate: *const c_void,
) -> *const c_void {
    null()
}

pub extern "C" fn fmi3DoStep(
    instance: &mut Slave,
    currentCommunicationPoint: f64,
    communicationStepSize: f64,
    noSetFMUStatePriorToCurrentPoint: i32,
    eventHandlingNeeded: *const i32,
    terminateSimulation: *const i32,
    earlyReturn: *const i32,
    lastSuccessfulTime: f64,
) -> Fmi3Status {
    match instance.dispatcher.fmi3DoStep(
        currentCommunicationPoint,
        communicationStepSize,
        noSetFMUStatePriorToCurrentPoint != 0,
    ) {
        Ok(s) => match s {
            Fmi3Status::OK | Fmi3Status::Warning => {
                instance.last_successful_time =
                    Some(currentCommunicationPoint + communicationStepSize);
                s
            }
            s => s,
        },
        Err(e) => Fmi3Status::Error,
    }
}

pub extern "C" fn fmi3EnterInitializationMode(
    instance: *mut c_void,
    toleranceDefined: i32,
    tolerance: f64,
    startTime: f64,
    stopTimeDefined: f64,
    stopTime: f64,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3ExitInitializationMode() -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetFloat32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const f32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetFloat64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const f64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetInt8(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i8,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetUint8(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u8,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetInt16(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i16,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetUint16(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u16,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetInt32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetUint32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetInt64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetUint64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetBoolean(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3GetBinary(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    valueSizes: *const size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetFloat32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const f32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetFloat64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const f64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetInt8(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i8,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetUint8(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u8,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetInt16(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i16,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetUint16(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u16,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetInt32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetUint32(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetInt64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetUint64(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const u64,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetBoolean(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

pub extern "C" fn fmi3SetBinary(
    instance: *mut c_void,
    valueReferences: *const i32,
    nValueReferences: size_t,
    valueSizes: *const size_t,
    values: *const i32,
    nValues: size_t,
) -> Fmi3Status {
    Fmi3Status::OK
}

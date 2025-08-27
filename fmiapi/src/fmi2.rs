#![allow(non_snake_case)]
#![allow(unused_variables)]
#![deny(unsafe_op_in_unsafe_fn)]

use crate::fmi2_messages::{
    self,
    Fmi2Command,
    fmi2_command::Command,
    fmi2_return::ReturnMessage
};
use crate::fmi2_slave::{
    Fmi2Slave,
    SlaveState
};
use crate::fmi2_types::{
    Fmi2Real,
    Fmi2Integer,
    Fmi2Boolean,
    Fmi2String,
    Fmi2Status,
    Fmi2CallbackFunctions,
    Fmi2LogCategory,
    Fmi2StatusKind,
    Fmi2Type
};
use crate::fmi2_logger::Fmi2Logger;
use crate::logger::Logger;
use crate::protobuf_extensions::{
    ExpectableReturn,
    implement_expectable_return
};
use crate::spawn::spawn_slave;
use crate::string_conversion::{c2s, c2non_empty_s};

use libc::size_t;
use url::Url;

use std::{
    ffi::{CStr, CString, NulError},
    os::raw::{c_char, c_uint},
    path::Path,
    slice::{from_raw_parts, from_raw_parts_mut},
    str::Utf8Error
};

// ---------------------- Protocol Buffer Transformations ---------------------
impl From<fmi2_messages::Fmi2Type> for Fmi2Type {
    fn from(src: fmi2_messages::Fmi2Type) -> Self {
        match src {
            fmi2_messages::Fmi2Type::Fmi2ModelExchange => Self::Fmi2ModelExchange,
            fmi2_messages::Fmi2Type::Fmi2CoSimulation => Self::Fmi2CoSimulation,
        }
    }
}

impl From<fmi2_messages::Fmi2StatusReturn> for Fmi2Status {
    fn from(src: fmi2_messages::Fmi2StatusReturn) -> Self {
        src.status().into()
    }
}

impl From<fmi2_messages::Fmi2Status> for Fmi2Status {
    fn from(src: fmi2_messages::Fmi2Status) -> Self {
        match src {
            fmi2_messages::Fmi2Status::Fmi2Ok => Self::Ok,
            fmi2_messages::Fmi2Status::Fmi2Warning => Self::Warning,
            fmi2_messages::Fmi2Status::Fmi2Discard => Self::Discard,
            fmi2_messages::Fmi2Status::Fmi2Error => Self::Error,
            fmi2_messages::Fmi2Status::Fmi2Fatal => Self::Fatal,
            fmi2_messages::Fmi2Status::Fmi2Pending => Self::Pending,
        }
    }
}

// ----------------------- Protocol Buffer Trait decorations ---------------------------
// The trait ExpectableReturn extends the Return message with an extract
// function that let's us pattern match and unwrap the inner type of a
// ReturnMessage.
implement_expectable_return!(fmi2_messages::Fmi2EmptyReturn, ReturnMessage, Empty);
implement_expectable_return!(fmi2_messages::Fmi2StatusReturn, ReturnMessage, Status);
implement_expectable_return!(fmi2_messages::Fmi2FreeInstanceReturn, ReturnMessage, FreeInstance);
implement_expectable_return!(fmi2_messages::Fmi2GetRealReturn, ReturnMessage, GetReal);
implement_expectable_return!(fmi2_messages::Fmi2GetIntegerReturn, ReturnMessage, GetInteger);
implement_expectable_return!(fmi2_messages::Fmi2GetBooleanReturn, ReturnMessage, GetBoolean);
implement_expectable_return!(fmi2_messages::Fmi2GetStringReturn, ReturnMessage, GetString);
implement_expectable_return!(fmi2_messages::Fmi2GetRealOutputDerivativesReturn, ReturnMessage, GetRealOutputDerivatives);
implement_expectable_return!(fmi2_messages::Fmi2GetDirectionalDerivativesReturn, ReturnMessage, GetDirectionalDerivatives);
implement_expectable_return!(fmi2_messages::Fmi2SerializeFmuStateReturn, ReturnMessage, SerializeFmuState);

// ------------------------------------- FMI FUNCTIONS --------------------------------

pub static VERSION: &str = "2.0\0";
pub static TYPES_PLATFORM: &str = "default\0";

#[no_mangle]
pub extern "C" fn fmi2GetTypesPlatform() -> *const c_char {
    TYPES_PLATFORM.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn fmi2GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

// ------------------------------------- FMI FUNCTIONS (Life-Cycle) --------------------------------

/// Instantiates a slave instance by invoking a command in a new process
/// the command is specified by the configuration file, launch.toml, that should be located in the resources directory
/// fmi-commands are sent between the wrapper and slave(s) using a message queue library, specifically zmq.
/// 
/// # Safety
/// When calling this function, you have to ensure that either the pointer is null or the pointer is convertible to a reference to a string.
/// 
/// Furthermore if the pointer is not null:
/// * The memory pointed to must contain a valid nul terminator at the end of the string.
/// * The pointer must be [valid] for reads of bytes up to and including the nul terminator.
/// * The nul terminator must be within isize::MAX from the pointer.
#[no_mangle]
pub unsafe extern "C" fn fmi2Instantiate(
    instance_name: Fmi2String,
    fmu_type: Fmi2Type,
    fmu_guid: Fmi2String,
    fmu_resource_location: Fmi2String,
    functions: *const Fmi2CallbackFunctions,
    visible: Fmi2Boolean,
    logging_on: Fmi2Boolean,
) -> Option<Box<Fmi2Slave>> {

    let functions = match unsafe { functions.as_ref() } {
        None => {
            /* TODO - replace with own fmt logger call.
            error!(
                category = %Fmi2LogCategory::LogStatusError,
                "Pointer to callback functions was null."
            );
             */
            return None;
        }
        Some(functions_reference) => functions_reference
    };

    let logging_on = match logging_on {
        0 => false,
        1 => true,
        _ => {
            /* TODO - replace with own fmt logger call.
            error!(
                category = %Fmi2LogCategory::LogStatusError,
                "Invalid value passed to 'logging_on'."
            );
            */
            return None;
        }
    };

    let logger = Fmi2Logger::new(
        functions.logger,
        instance_name,
        &(functions.component_environment),
        logging_on
    );

    // Erroring out in case the importer tries to instantiate the FMU for
    // Model Exchange as that is not yet implemented.
    if let Fmi2Type::Fmi2ModelExchange = fmu_type {
        logger.error("Model Exchange is not implemented for UNIFMU.");
        return None
    }

    let instance_name = match c2non_empty_s(instance_name) {
        Ok(name) => name,
        Err(error) => {
            logger.error(&format!(
                "Could not parse instance_name; {}", error
            ));
            return None
        }
    };
    
    let fmu_guid = match c2s(fmu_guid) {
        Ok(guid) => guid,
        Err(error) => {
            logger.error(&format!(
                "Could not convert fmu_guid to String; {}", error
            ));
            return None
        }
    };

    let fmu_resource_location = match c2non_empty_s(fmu_resource_location) {
        Ok(location) => location,
        Err(error) => {
            logger.error(&format!(
                "Could not parse fmu_resource_location; {}", error
            ));
            return None
        }
    };

    let resource_uri = match Url::parse(&fmu_resource_location) {
        Err(error) => {
            logger.error(&format!("Unable to parse argument 'fmu_resource_location' as url; {}.", error));
            return None;
        }
        Ok(url) => url
    };

    let resources_dir = match resource_uri.to_file_path() {
        Err(_) => {
            logger.error(&format!(
                "URI was parsed but could not be converted into a file path, got: '{:?}'.",
                resource_uri
            ));
            return None;
        }
        Ok(resources_dir) => resources_dir
    };

    let dispatcher = match spawn_slave(
        Path::new(&resources_dir),
        |message| logger.ok(message)
    ) {
        Err(error) => {
            logger.error(&format!("Spawning fmi2 slave failed; {}", error));
            return None;
        }
        Ok(dispatcher) => dispatcher
    };

    let mut slave = Fmi2Slave::new(dispatcher, logger);

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Instantiate(
            fmi2_messages::Fmi2Instantiate {
                instance_name,
                fmu_type: 0,
                fmu_guid,
                fmu_resource_location,
                visible: false,
                logging_on,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2EmptyReturn>(&cmd) {
        Err(error) => {
            slave.logger.error(&format!(
                "Instantiation of fmi2 slave failed; {}.",
                error
            ));
            None
        },
        Ok(_) => Some(Box::new(slave))
    }
}

#[no_mangle]
pub extern "C" fn fmi2FreeInstance(slave: Option<Box<Fmi2Slave>>) {
    let mut slave = slave;

    if let Some(_s) = slave.as_mut() {
        drop(slave)
    }
}

#[no_mangle]
pub extern "C" fn fmi2SetDebugLogging(
    slave: &mut Fmi2Slave,
    logging_on: Fmi2Boolean,
    n_categories: size_t,
    categories: *const Fmi2String,
) -> Fmi2Status {
    let logging_on = match logging_on {
        0 => false,
        1 => true,
        _ => {
            slave.logger.error("Invalid value passed to 'logging_on'.");
            return Fmi2Status::Error;
        }
    };

    let mut string_categories: Vec<String> = Vec::new();

    if n_categories > 0 {
        match unsafe { from_raw_parts(categories, n_categories) }
            .iter()
            .map(|category| {
                match unsafe { category.as_ref() } {
                    None => {
                        Err("one of the categories was null")
                    }
                    Some(category_reference) => match unsafe { CStr::from_ptr(category_reference).to_str() } {
                        Err(_) => {
                            Err("one of the categories could not be parsed as an utf-8 formatted string")
                        }
                        Ok(category_str) => Ok(Fmi2LogCategory::from(category_str))
                    }
                }
            })
            .collect::<Result<Vec<Fmi2LogCategory>, &str>>()
        {
            Err(error) => {
                slave.logger.error(&format!(
                    "Couldn't parse categories; {}", error
                ));
                return Fmi2Status::Error;
            }
            Ok(categories) => {
                string_categories = categories.iter()
                    .map(|category| category.to_string())
                    .collect();

                if logging_on {
                    slave.logger.enable_categories(categories);
                } else {
                    slave.logger.disable_categories(categories);
                }
            }
        }
    } else if logging_on {
        slave.logger.enable_all_categories();
    } else {
        slave.logger.disable_all_categories();
    }

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetDebugLogging(
            fmi2_messages::Fmi2SetDebugLogging {
                categories: string_categories,
                logging_on
            }
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "Fmi2SetDebugLogging failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2SetupExperiment(
    slave: &mut Fmi2Slave,
    tolerance_defined: Fmi2Boolean,
    tolerance: Fmi2Real,
    start_time: Fmi2Real,
    stop_time_defined: Fmi2Boolean,
    stop_time: Fmi2Real,
) -> Fmi2Status {
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

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetupExperiment(
            fmi2_messages::Fmi2SetupExperiment {
                start_time,
                stop_time,
                tolerance,
            }
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2SetupExperiment failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2EnterInitializationMode(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2EnterInitializationMode(
            fmi2_messages::Fmi2EnterInitializationMode {},
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2EnterInitializationMode failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2ExitInitializationMode(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2ExitInitializationMode(
            fmi2_messages::Fmi2ExitInitializationMode {},
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2ExitInitializationMode failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2Terminate(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Terminate(
            fmi2_messages::Fmi2Terminate {},
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2Terminate failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

#[no_mangle]
pub extern "C" fn fmi2Reset(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2Reset(
            fmi2_messages::Fmi2Reset {},
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2Reset failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

// ------------------------------------- FMI FUNCTIONS (Stepping) --------------------------------
#[no_mangle]
pub extern "C" fn fmi2DoStep(
    slave: &mut Fmi2Slave,
    current_time: Fmi2Real,
    step_size: Fmi2Real,
    no_step_prior: Fmi2Boolean,
) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2DoStep(
            fmi2_messages::Fmi2DoStep {
                current_time,
                step_size,
                no_set_fmu_state_prior_to_current_point: no_step_prior != 0,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
    {
        Ok(status) => match status {
            Fmi2Status::Ok | Fmi2Status::Warning => {
                slave.last_successful_time = Some(current_time + step_size);
                status
            }
            status => status,
        },
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2DoStep failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2CancelStep(slave: &mut Fmi2Slave) -> Fmi2Status {
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2CancelStep(
            fmi2_messages::Fmi2CancelStep {},
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2CancelStep failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

// ------------------------------------- FMI FUNCTIONS (Getters) --------------------------------

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_double>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_double` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_double>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetReal(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut Fmi2Real,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetReal(
            fmi2_messages::Fmi2GetReal {
                references,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetRealReturn>(&cmd) {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }
            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetReal failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetInteger(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut Fmi2Integer,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetInteger(
            fmi2_messages::Fmi2GetInteger {
                references,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetIntegerReturn>(&cmd) {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }
            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetInteger failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetBoolean(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut Fmi2Boolean,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetBoolean(
            fmi2_messages::Fmi2GetBoolean {
                references,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetBooleanReturn>(&cmd) {
        Ok(result) => {
            if !result.values.is_empty() {
                let values: Vec<i32> = result.values
                    .iter()
                    .map(|v| match v {
                        false => 0,
                        true => 1,
                    })
                    .collect();

                values_out.copy_from_slice(&values)
            }
            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetBoolean failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// Reads strings from FMU
///
/// Note:
/// To ensure that c-strings returned by fmi2GetString can be used by the envrionment,
/// they must remain valid until another FMI function is invoked. see 2.1.7 p.23.
/// We choose to do it on an instance basis, i.e. each instance has its own string buffer.
/// 
/// # Safety
/// Behavior is undefined if any of the following are violated:
/// * `values` must be non-null, \[valid\] for writes, and properly aligned.
/// * `references` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<c_uint>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `refernces` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `references` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `references` must point to `nvr` consecutive properly initialized values
///   of type `c_uint`.
/// * The total size `nvr * mem::size_of::<c_uint>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `references` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetString(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *mut Fmi2String,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetString(
            fmi2_messages::Fmi2GetString {
                references,
            }
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetStringReturn>(&cmd) {
        Ok(result) => {
            if !result.values.is_empty() {
                let conversion_result: Result<Vec<CString>, NulError> = result
                    .values
                    .iter()
                    .map(|string| CString::new(string.as_bytes()))
                    .collect();

                match conversion_result {
                    Ok(converted_values) => {
                        slave.string_buffer = converted_values
                    },
                    Err(_) =>  {
                        slave.logger.fatal(
                            "Backend returned strings containing interior nul bytes. These cannot be converted into CStrings."
                        );
                        return Fmi2Status::Fatal;
                    }
                }

                unsafe {
                    for (idx, cstr)
                    in slave.string_buffer.iter().enumerate()
                    {
                        std::ptr::write(
                            values.add(idx), 
                            cstr.as_ptr()
                        );
                    }
                }
            }

            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetString failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_double>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_double` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_double>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetReal(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const Fmi2Real,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetReal(
            fmi2_messages::Fmi2SetReal {
                references,
                values,
            }
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2SetReal failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetInteger(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const Fmi2Integer,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetInteger(
            fmi2_messages::Fmi2SetInteger {
                references,
                values,
            }
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2SetInteger failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

/// set boolean variables of FMU
///
/// Note: fmi2 uses C-int to represent booleans and NOT the boolean type defined by C99 in stdbool.h, _Bool.
/// Rust's bool type is defined to have the same size as _Bool, as the values passed through the C-API must be converted.
/// 
/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `references` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_int>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `references` and `values` the entire memory range of that
///       slice must be contained within a single allocated object! Slices can
///       never span across multiple allocated objects.
///     * `references` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `references` or
///       `values` for zero-length slices using \[`NonNull::dangling`\].
/// * `references` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_int` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_int>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetBoolean(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    values: *const Fmi2Boolean,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let values: Vec<bool> = unsafe { from_raw_parts(values, nvr) }
        .iter()
        .map(|v| *v != 0)
        .collect();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetBoolean(
            fmi2_messages::Fmi2SetBoolean {
                references,
                values,
            }
        )),
    };
    
    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2SetBoolean failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `vr` and `values` must be non-null, \[valid\] for reads for 
///   `nvr * mem::size_of::<c_uint>()` and `nvr * mem::size_of::<c_char>()`
///   many bytes respectively, and they must be properly aligned. This means
///   in particular:
///     * For each of `vr` and `values` the entire memory range of that slice
///       must be contained within a single allocated object! Slices can never
///       span across multiple allocated objects.
///     * `vr` and `values` must each be non-null and aligned even for
///       zero-length slices or slices of ZSTs. One reason for this is that
///       enum layout optimizations may rely on references (including slices of
///       any length) being aligned and non-null to distinguish them from other
///       data. You can obtain a pointer that is usable as `vr` or `values` for
///       zero-length slices using \[`NonNull::dangling`\].
/// * `vr` and `values` must each point to `nvr` consecutive properly
///   initialized values of type `c_uint` and `c_char` respectively.
/// * The total size `nvr * mem::size_of::<c_uint>()` or 
///   `nvr * mem::size_of::<c_char>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `vr` and `values` respectively
///   must not "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetString(
    slave: &mut Fmi2Slave,
    vr: *const c_uint,
    nvr: size_t,
    values: *const Fmi2String,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(vr, nvr) }.to_owned();

    let conversion_result: Result<Vec<String>, Utf8Error> = unsafe {
        from_raw_parts(values, nvr)
            .iter()
            .map(|v| {
                CStr::from_ptr(*v)
                    .to_str()
                    .map(|str| str.to_owned())
                })
            .collect()
    };

    match conversion_result {
        Ok(values) => {
            let cmd = Fmi2Command {
                command: Some(Command::Fmi2SetString(
                    fmi2_messages::Fmi2SetString {
                        references,
                        values,
                    }
                )),
            };
        
            slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
                .map(|status| status.into())
                .unwrap_or_else(|error| {
                    slave.logger.error(&format!(
                        "fmi2SetString failed with error: {}.", error
                    ));
                    Fmi2Status::Error
                })
        },
        Err(conversion_error) => {
            slave.logger.error(&format!(
                "The String values could not be converted to Utf-8; {}.", conversion_error
            ));
            Fmi2Status::Error
        }
    }

    
}

// ------------------------------------- FMI FUNCTIONS (Derivatives) --------------------------------

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`unknown_refs`, `known_refs`, `direction_known`, `direction_unknown`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_uint`, `c_double`, `c_double`\]
/// 
/// and `SIZE_PARAMETERS` \[`nvr_unknown`, `nvr_known`, `nvr_known`, `nvr_unknown`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `SIZE_PARAMETER * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `SIZE_PARAMETER` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `SIZE_PARAMETER * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetDirectionalDerivative(
    slave: &mut Fmi2Slave,
    unknown_refs: *const c_uint,
    nvr_unknown: size_t,
    known_refs: *const c_uint,
    nvr_known: size_t,
    direction_known: *const Fmi2Real,
    direction_unknown: *mut Fmi2Real,
) -> Fmi2Status {
    let references_unknown = unsafe {
        from_raw_parts(unknown_refs, nvr_known)
    }
        .to_owned();

    let references_known = unsafe {
        from_raw_parts(known_refs, nvr_known)
    }
        .to_owned();

    let direction_known = unsafe {
        from_raw_parts(direction_known, nvr_known)
    }
        .to_owned();

    let direction_unknown = unsafe {
        from_raw_parts_mut(direction_unknown, nvr_known)
    };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetDirectionalDerivatives(
            fmi2_messages::Fmi2GetDirectionalDerivatives {
                references_unknown,
                references_known,
                direction_known,
            },
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetDirectionalDerivativesReturn>(
        &cmd
    ) {
        Ok(result) => {
            if !result.values.is_empty() {
                direction_unknown.copy_from_slice(&result.values);
                result.status().into()
            } else {
                slave.logger.error(
                    "fmi2GetDirectionalDerivative returned empty values"
                );
                Fmi2Status::Error
            }
        },
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetDirectionalDerivative failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`references`, `orders`, `values`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_int`, `c_double`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `nvr` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `nvr * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2SetRealInputDerivatives(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const Fmi2Integer,
    values: *const Fmi2Real,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let orders = unsafe { from_raw_parts(orders, nvr) }.to_owned();
    let values = unsafe { from_raw_parts(values, nvr) }.to_owned();

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SetRealInputDerivatives(
            fmi2_messages::Fmi2SetRealInputDerivatives {
                references,
                orders,
                values
            },
        )),
    };

    slave.dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            slave.logger.error(&format!(
                "fmi2SetRealInputDerivatives failed with error: {}.", error
            ));
            Fmi2Status::Error
        })
}

/// # Safety
/// Behavior is undefined if any of the following are violated for each of the
/// 
/// `PARAMETERS` \[`references`, `orders`, `values`\]
/// 
/// with the `TYPES` \[`c_uint`, `c_int`, `c_double`\]:
/// 
/// * `PARAMETER` must be non-null, \[valid\] for reads for
///   `nvr * mem::size_of::<TYPE>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `PARAMETER` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `PARAMETER` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `PARAMETER` must point to `nvr` consecutive properly initialized values
///   of type `TYPE`.
/// * The total size `nvr * mem::size_of::<TYPE>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `PARAMETER` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2GetRealOutputDerivatives(
    slave: &mut Fmi2Slave,
    references: *const c_uint,
    nvr: size_t,
    orders: *const Fmi2Integer,
    values: *mut Fmi2Real,
) -> Fmi2Status {
    let references = unsafe { from_raw_parts(references, nvr) }.to_owned();
    let orders = unsafe { from_raw_parts(orders, nvr) }.to_owned();
    let values_out = unsafe { from_raw_parts_mut(values, nvr) };

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2GetRealOutputDerivatives(
            fmi2_messages::Fmi2GetRealOutputDerivatives {
                references,
                orders,
            },
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2GetRealOutputDerivativesReturn>(
        &cmd
    ) {
        Ok(result) => {
            if !result.values.is_empty() {
                values_out.copy_from_slice(&result.values)
            }
            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetRealOutputDerivatives failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

// ------------------------------------- FMI FUNCTIONS (Serialization) --------------------------------
/// Saves the state of the FMU in the state pointer 
/// 
/// # Parameters
/// - `slave`: a raw pointer to the FMU slave instance
/// - `state`: a raw pointer to the state, that will save the FMU's state
///
/// # Returns
/// - `Fmi2Status::Ok`: If the operation succeeds.
/// - `Fmi2Status::Error`: If an error occurs during the process (e.g., invalid pointers or failed serialization).
///
#[no_mangle]
pub extern "C" fn fmi2SetFMUstate(
    slave: *mut Fmi2Slave, 
    state: *const SlaveState
) -> Fmi2Status {

    if slave.is_null() {
        // slave contains the logger - without it we can't log errors.
        return Fmi2Status::Error;
    }
    if state.is_null() {
        unsafe { (*slave).logger.error(
            "fmi2SetFMUstate called with state pointing to null!"
        );}
        return Fmi2Status::Error;
    }

    let state_ref = unsafe { &*state };

    let state_bytes = state_ref.bytes.to_owned();
    
    let cmd = Fmi2Command {
        command: Some(Command::Fmi2DeserializeFmuState(
            fmi2_messages::Fmi2DeserializeFmuState {
                state: state_bytes,
            }
        )),
    };
    
    unsafe {
        (*slave).dispatch::<fmi2_messages::Fmi2StatusReturn>(&cmd)
    }
        .map(|status| status.into())
        .unwrap_or_else(|error| {
            unsafe { (*slave).logger.error(&format!(
                "fmi2SetFMUstate failed with error: {}.", error
            )); }
            Fmi2Status::Error
        })
}

/// Store a copy of the FMU's state in a buffer for later retrival, see. p25
///
/// # Parameters
/// - `slave`: A raw pointer to the FMU slave instance, which is responsible for managing the state of the FMU.
/// - `state`: A raw pointer to the location where the FMU's state will be stored. After the call, this pointer will hold a copy of the FMU state.
/// 
/// # Returns
/// - `Fmi2Status::Ok`: If the operation succeeds and the FMU state is successfully serialized and stored.
/// - `Fmi2Status::Error`: If an error occurs during the serialization or if invalid pointers are provided.
/// - `Fmi2Status::Fatal`: If an unknown status is returned from the backend or there is an issue with the result status.
///
#[no_mangle]
pub extern "C" fn fmi2GetFMUstate(
    slave: *mut Fmi2Slave,
    state: *mut *mut SlaveState, 
) -> Fmi2Status {

    if slave.is_null() {
        // slave contains the logger - without it we can't log errors.
        return Fmi2Status::Error;
    }

    let slave = unsafe { &mut *slave };

    if state.is_null() {
        slave.logger.error("fmi2GetFMUstate called with state pointing to null!");
        return Fmi2Status::Error;
    }

    let cmd = Fmi2Command {
        command: Some(Command::Fmi2SerializeFmuState(
            fmi2_messages::Fmi2SerializeFmuState {}
        )),
    };

    match slave.dispatch::<fmi2_messages::Fmi2SerializeFmuStateReturn>(&cmd) {
        Ok(result) => {
            unsafe {
                match (*state).as_mut() {
                    Some(state_ptr) => {
                        let state = &mut *state_ptr;
                        state.bytes = result.state.clone();
                    }
                    None => {
                        let new_state = Box::new(SlaveState::new(&result.state));
                        *state = Box::into_raw(new_state);
                    }
                }
            }
            result.status().into()
        }
        Err(error) => {
            slave.logger.error(&format!(
                "fmi2GetFMUstate failed with error: {}.", error
            ));
            Fmi2Status::Error
        }
    }
}

/// Free previously recorded state of slave
/// If state points to null the call is ignored as defined by the specification
///
/// # Parameters
/// - `slave`: A raw pointer to the FMU slave instance.
/// - `state`: A raw pointer to the state that should be freed. If the pointer is null, no action is taken.
///
/// # Returns
/// - `Fmi2Status::Ok`: Indicates that the state was successfully freed (or the state was null, and no action was required).
///
#[no_mangle]
pub extern "C" fn fmi2FreeFMUstate(
    slave: *mut Fmi2Slave,
    state: *mut *mut SlaveState,
) -> Fmi2Status {

    if slave.is_null() {
        return Fmi2Status::Ok;
    }

    if state.is_null() {
        unsafe { (*slave).logger.warning(
            "fmi2FreeFMUstate called with state pointing to null!"
        ); }
        return Fmi2Status::Warning;
    }

    unsafe {
        let state_ptr = *state;

        if state_ptr.is_null() {
            unsafe { (*slave).logger.warning(
                "fmi2FreeFMUstate called with state pointing to null!"
            ); }
            return Fmi2Status::Warning;
        }

        drop(Box::from_raw(state_ptr)); 
        *state = std::ptr::null_mut(); // Setting the state to null

    }

    Fmi2Status::Ok
}

/// Copies the state of a slave into a buffer provided by the environment
///
/// # Parameters 
/// - `slave`: A reference to the FMU slave instance
/// - `state`: A reference to the state of the FMU
/// - `data`: A pointer to the buffer where the state will be copied
/// - `size`: The size of the buffer
///
/// # Returns
/// - `Fmi2Status::Ok`: If the operation succeeds and the FMU state is successfully serialized and stored.
/// - `Fmi2Status::Error`: If an error occurs during the serialization or if invalid pointers are provided.
#[no_mangle]
/// We assume that the buffer is sufficiently large
pub extern "C" fn fmi2SerializeFMUstate(
    slave: &Fmi2Slave,
    state: &SlaveState,
    data: *mut u8,
    size: size_t,
) -> Fmi2Status {
    let serialized_state_len = state.bytes.len();

    if serialized_state_len > size {
        slave.logger.error(
            "Error while calling fmi2SerializeFMUstate: FMUstate too big to be contained in given byte vector."
        );
        return Fmi2Status::Error;
    }

    unsafe { std::ptr::copy(
        state.bytes.as_ptr(),
        data.cast(),
        serialized_state_len
    ) };

    Fmi2Status::Ok
}

/// # Safety
/// Behavior is undefined if any of the following are violated:
/// * `serialized_state` must be non-null, \[valid\] for reads for
///   `size * mem::size_of::<u8>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `refernces` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `serialized_state` for zero-length
///       slices using \[`NonNull::dangling`\].
/// * `serialized_state` must point to `size` consecutive properly initialized values
///   of type `u8`.
/// * The total size `size * mem::size_of::<u8>()` of the slice must be no
///   larger than `isize::MAX`, and adding that size to `serialized_state` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi2DeSerializeFMUstate(
    slave: &mut Fmi2Slave,
    serialized_state: *const u8,
    size: size_t,
    state: *mut *mut SlaveState,
) -> Fmi2Status {
    let serialized_state = unsafe { from_raw_parts(serialized_state, size) };

    if state.is_null() {
        slave.logger.error(
            "fmi2DeSerializeFMUstate called with state pointing to null!"
        );
        return Fmi2Status::Error;
    }

    unsafe {
        if (*state).is_null() {
            // If null allocate the new state and set the pointer to it
            let new_state = Box::new(SlaveState::new(serialized_state));
            *state = Box::into_raw(new_state);
        } else {
            // If not null overwrite the state
            let state_ptr = *state;
            let state = &mut *state_ptr;
            state.bytes = serialized_state.to_owned();
        }
    }
    Fmi2Status::Ok
}

/// Retrieves the size of the serialized state of the FMU
///
/// # Parameters
/// - `slave`: A reference to the FMU slave instance
/// - `state`: A reference to the state of the FMU
/// - `size`: A reference to the size of the serialized state
///
/// # Returns
/// - `Fmi2Status::Ok`: If the operation succeeds and the size of the serialized state is successfully retrieved.
#[no_mangle]
pub extern "C" fn fmi2SerializedFMUstateSize(
    slave: &Fmi2Slave,
    state: &SlaveState,
    size: &mut size_t,
) -> Fmi2Status {
    *size = state.bytes.len();
    Fmi2Status::Ok
}

// ------------------------------------- FMI FUNCTIONS (Status) --------------------------------
#[no_mangle]
pub extern "C" fn fmi2GetStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Status,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2DoStepStatus => match slave.dostep_status {
            Some(status) => status,
            None => {
                slave.logger.error(
                    "'fmi2GetStatus' called with fmi2StatusKind 'Fmi2DoStepStatus' before 'fmi2DoStep' has returned pending."
                );
                Fmi2Status::Error
            }
        },
        _ => {
            slave.logger.error(&format!(
                "'fmi2GetStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            ));
            Fmi2Status::Error
        }
    }
}

/// # Safety
/// Behavior is undefined if `value` points outside of address space and if it
/// is dereferenced after function call.
#[no_mangle]
pub unsafe extern "C" fn fmi2GetRealStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Real,
) -> Fmi2Status {
    match status_kind {
        Fmi2StatusKind::Fmi2LastSuccessfulTime => match slave.last_successful_time {
            Some(last_time) => {
                unsafe {
                    *value = last_time;
                };
                Fmi2Status::Ok
            }
            None => {
                slave.logger.error(
                    "'fmi2GetRealStatus' can not be called before 'Fmi2DoStep'"
                );
                Fmi2Status::Error
            }
        },
        _ => {
            slave.logger.error(&format!(
                "'fmi2GetRealStatus' only accepts the status kind '{:?}'",
                Fmi2StatusKind::Fmi2DoStepStatus
            ));
            Fmi2Status::Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi2GetIntegerStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Integer,
) -> Fmi2Status {
    slave.logger.error(
        "fmi2GetIntegerStatus is not implemented by UniFMU."
    );
    Fmi2Status::Error
}

#[no_mangle]
pub extern "C" fn fmi2GetBooleanStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2Boolean,
) -> Fmi2Status {
    slave.logger.error(
        "fmi2GetBooleanStatus is not implemented by UniFMU."
    );
    Fmi2Status::Error
}

#[no_mangle]
pub extern "C" fn fmi2GetStringStatus(
    slave: &mut Fmi2Slave,
    status_kind: Fmi2StatusKind,
    value: *mut Fmi2String,
) -> Fmi2Status {
    slave.logger.error(
        "fmi2GetStringStatus is not implemented by UniFMU."
    );
    Fmi2Status::Error
}

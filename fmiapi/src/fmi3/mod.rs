#![allow(non_snake_case)]
#![allow(unused_variables)]

mod fmi3_logger;
mod fmi3_messages;
mod fmi3_slave;
mod fmi3_types;

use fmi3_logger::Fmi3Logger;
use fmi3_messages::{
    fmi3_command::Command,
    Fmi3Command
};
use fmi3_slave::{
    Fmi3Slave,
    Fmi3SlaveType,
    SlaveState
};
use fmi3_types::{
    Fmi3Float32,
    Fmi3Float64,
    Fmi3Int8,
    Fmi3UInt8,
    Fmi3Int16,
    Fmi3UInt16,
    Fmi3Int32,
    Fmi3UInt32,
    Fmi3Int64,
    Fmi3UInt64,
    Fmi3Boolean,
    Fmi3String,
    Fmi3Byte,
    Fmi3Binary,
    Fmi3Clock,
    Fmi3Status,
    Fmi3IntervalQualifier,
    Fmi3InstanceEnvironment,
    Fmi3ValueReference,
    Fmi3DependencyKind,
    Fmi3LogCategory,
    Fmi3LogMessageCallback,
    Fmi3IntermediateUpdateCallback,
    UnsupportedCallback,
};

use crate::common::{
    logger::Logger,
    spawn::spawn_slave,
    string_conversion::{c2s, c2non_empty_s}
};

use std::{
    ffi::{c_void, CStr, CString, NulError},
    path::{Path, PathBuf},
    slice::{from_raw_parts, from_raw_parts_mut},
    str::Utf8Error
};

use libc::{c_char, size_t};
use url::Url;

// ------------------------------------- FMI FUNCTIONS --------------------------------
static VERSION: &str = "3.0\0";

#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn fmi3SetDebugLogging(
    instance: &mut Fmi3Slave,
    logging_on: Fmi3Boolean,
    n_categories: size_t,
    categories: *const Fmi3String
) -> Fmi3Status {
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
                        Ok(category_str) => Ok(Fmi3LogCategory::from(category_str))
                    }
                }
            })
            .collect::<Result<Vec<Fmi3LogCategory>, &str>>()
        {
            Err(error) => {
                instance.logger.error(&format!(
                    "Couldn't parse categories; {}", error
                ));
                return Fmi3Status::Fmi3Error;
            }
            Ok(categories) => {
                string_categories = categories.iter()
                    .map(|category| category.to_string())
                    .collect();

                if logging_on {
                    instance.logger.enable_categories(categories);
                } else {
                    instance.logger.disable_categories(categories);
                }
            }
        }
    } else if logging_on {
        instance.logger.enable_all_categories();
    } else {
        instance.logger.disable_all_categories();
    }

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetDebugLogging(
            fmi3_messages::Fmi3SetDebugLogging {
                categories: string_categories,
                logging_on
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetDebugLogging")
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateModelExchange(
    instance_name: Fmi3String,
    instantiation_token: Fmi3String,
    resource_path: Fmi3String,
    visible: Fmi3Boolean,
    logging_on: Fmi3Boolean,
    instance_environment: *const Fmi3InstanceEnvironment,
    log_message: Fmi3LogMessageCallback,
) -> Option<Fmi3SlaveType> {
    let logger = Fmi3Logger::new(
        log_message,
        instance_environment,
        logging_on
    );
    logger.error("fmi3InstantiateModelExchange is not implemented by UNIFMU.");
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
}

/// # Safety
/// Behavior is undefined if any of the following are violated:
/// * `resource_path` must be either null or convertable to a reference.
/// * `required_intermediate_variables` must be non-null, \[valid\] for reads
///   for `n_required_intermediate_variables * mem::size_of::<u32>()` many
///   bytes, and it must be properly aligned. This means in particular:
///     * The entire memory range of this slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `refernces` must be non-null and aligned even for zero-length slices
///       or slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as
///       `required_intermediate_variables` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `required_intermediate_variables` must point to
///   `n_required_intermediate_variables` consecutive properly initialized
///   values of type `u32`.
/// * The total size
///   `n_required_intermediate_variables * mem::size_of::<u32>()` of the slice
///   must be no larger than `isize::MAX`, and adding that size to
///   `required_intermediate_variables` must not "wrap around" the address
///   space. See the safety documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3InstantiateCoSimulation(
    instance_name: Fmi3String,
    instantiation_token: Fmi3String,
    resource_path: Fmi3String,
    visible: Fmi3Boolean,
    logging_on: Fmi3Boolean,
    event_mode_used: Fmi3Boolean,
    early_return_allowed: Fmi3Boolean,
    required_intermediate_variables: *const Fmi3ValueReference,
    n_required_intermediate_variables: size_t,
    instance_environment: *const Fmi3InstanceEnvironment,
    log_message: Fmi3LogMessageCallback,
    intermediate_update: Fmi3IntermediateUpdateCallback,
) -> Option<Fmi3SlaveType> {
    let logger = Fmi3Logger::new(
        log_message,
        instance_environment,
        logging_on
    );

    let instance_name = match c2non_empty_s(instance_name) {
        Ok(name) => name,
        Err(error) => {
            logger.error(&format!(
                "Could not parse instance_name; {}", error
            ));
            return None
        }
    };

    let instantiation_token = match c2s(instantiation_token) {
        Ok(string) => string,
        Err(error) => {
            logger.error(&format!(
                "Could not convert instantiation_token to String; {}", error
            ));
            return None
        }
    };

    let required_intermediate_variables = unsafe {
        from_raw_parts(
            required_intermediate_variables,
            n_required_intermediate_variables,
        )
    }
    .to_owned();

    let resource_path_str = match c2non_empty_s(resource_path) {
        Ok(path_string) => path_string,
        Err(error) => {
            logger.error(&format!(
                "could not parse resource_path; {}", error
            ));
            return None;
        }
    };

    // NOTE: In version 3 of the FMI standard, resourcePath should be a path, e.g., "C:\...". At
    // least one tool seems to still follow version 2 in that it passes a URI, e.g., starting with
    // "file:///C://..." instead. The current implementation maintains this "backwards
    // compatibility" with this incorrect implementation of version 3 of the standard.

    // Check for supported URI schemes or treat as a direct file path
    let resources_dir = if resource_path_str.starts_with("file:")
        || resource_path_str.starts_with("http:")
        || resource_path_str.starts_with("https:")
        || resource_path_str.starts_with("ftp:")
        || resource_path_str.starts_with("fmi2:")
    {
        // Parse as a URI
        let resource_uri = match Url::parse(&resource_path_str) {
            Ok(uri) => uri,
            Err(error) => {
                logger.error(&format!(
                    "Unable to parse uri: {}", error
                ));
                return None;
            }
        };

        if resource_uri.scheme() == "file" {
            match resource_uri.to_file_path() {
                Ok(path) => path,
                Err(_) => {
                    logger.error(&format!(
                        "URI was parsed but could not be converted into a file path, got: '{:?}'.",
                        resource_uri
                    ));
                    return None;
                }
            }
        } else {
            logger.error(&format!(
                "Unsupported URI scheme: '{}'", resource_uri.scheme()
            ));
            return None;
        }
    } else {
        // Treat it as a direct file path
        PathBuf::from(resource_path_str)
    };

    let dispatcher = match spawn_slave(
        Path::new(&resources_dir),
        |port| logger.communicate_port_connection_action(port)
    ) {
        Ok(dispatcher) => dispatcher,
        Err(error) => {
            logger.error(&format!("Spawning fmi3 slave failed; {}.", error));
            return None;
        }
    };

    let resource_path = match resources_dir.into_os_string().into_string() {
        Ok(string_path) => string_path,
        Err(error) => {
            logger.error(&format!("Couldn't convert resource directory path into String; {:?}", error));
            return None;
        }
    };

    let mut slave = Fmi3Slave::new(dispatcher, logger);

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3InstantiateCoSimulation(
            fmi3_messages::Fmi3InstantiateCoSimulation {
                instance_name: instance_name.clone(),
                instantiation_token,
                resource_path,
                visible,
                logging_on,
                event_mode_used,
                early_return_allowed,
                required_intermediate_variables,
            },
        )),
    };

    match slave.dispatch::<fmi3_messages::Fmi3EmptyReturn>(&cmd) {
        Err(error) => {
            slave.logger.error(&format!(
                "Instantiation of fmi3 slave '{}' failed with error [{}].",
                instance_name,
                error
            ));
            None
        },
        Ok(_) => Some(Box::new(slave))
    }
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateScheduledExecution(
    instance_name: Fmi3String,
    instantiation_token: Fmi3String,
    resource_path: Fmi3String,
    visible: Fmi3Boolean,
    logging_on: Fmi3Boolean,
    instance_environment: *const Fmi3InstanceEnvironment,
    log_message: Fmi3LogMessageCallback,
    clock_update: UnsupportedCallback,
    lock_preemption: UnsupportedCallback,
    unlock_preemption: UnsupportedCallback,
) -> Option<Fmi3SlaveType> {
    let logger = Fmi3Logger::new(
        log_message,
        instance_environment,
        logging_on
    );

    logger.error("fmi3InstantiateScheduledExecution is not implemented by UNIFMU.");
    None // Currently, we only support CoSimulation, return null pointer as per the FMI standard
}

/// # Safety
/// Behavior is undefined if any of `last_successful_time`,
/// `event_handling_needed`, `terminate_simulation` and `early_return` points
/// outside of address space and if they are dereferenced after function call.
#[no_mangle]
pub unsafe extern "C" fn fmi3DoStep(
    instance: &mut Fmi3Slave,
    current_communication_point: Fmi3Float64,
    communication_step_size: Fmi3Float64,
    no_set_fmu_state_prior_to_current_point: Fmi3Boolean,
    event_handling_needed: *mut Fmi3Boolean,
    terminate_simulation: *mut Fmi3Boolean,
    early_return: *mut Fmi3Boolean,
    last_successful_time: *mut Fmi3Float64,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3DoStep(
            fmi3_messages::Fmi3DoStep {
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point,
            }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3DoStepReturn>(&cmd) {
        Ok(result) => {
            let mut status = parse_status(result.status, &instance.logger);

            if status.output_is_defined() {
                if !last_successful_time.is_null() {
                    unsafe {
                        *last_successful_time = result.last_successful_time;
                    }
                } else {
                    instance.logger.warning(
                        "The parameter last_successful_time was a null pointer and consequently wasn't set as part of the step."
                    );
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !event_handling_needed.is_null() {
                    unsafe {
                        *event_handling_needed = result.event_handling_needed;
                    }
                } else {
                    instance.logger.warning(
                        "The parameter event_handling_needed was a null pointer and consequently wasn't set as part of the step."
                    );
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !terminate_simulation.is_null() {
                    unsafe {
                        *terminate_simulation = result.terminate_simulation;
                    }
                } else {
                    instance.logger.warning(
                        "The parameter terminate_simulation was a null pointer and consequently wasn't set as part of the step."
                    );
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !early_return.is_null() {
                    unsafe {
                        *early_return = result.early_return;
                    }
                } else {
                    instance.logger.warning(
                        "The parameter early_return was a null pointer and consequently wasn't set as part of the step."
                    );
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3DoStep failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3EnterInitializationMode(
    instance: &mut Fmi3Slave,
    tolerance_defined: Fmi3Boolean,
    tolerance: Fmi3Float64,
    start_time: Fmi3Float64,
    stop_time_defined: Fmi3Boolean,
    stop_time: Fmi3Float64,
) -> Fmi3Status {
    let tolerance = {
        if tolerance_defined {
            Some(tolerance)
        } else {
            None
        }
    };

    let stop_time = {
        if stop_time_defined {
            Some(stop_time)
        } else {
            None
        }
    };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterInitializationMode(
            fmi3_messages::Fmi3EnterInitializationMode {
                tolerance_defined,
                tolerance,
                start_time,
                stop_time_defined,
                stop_time,
            },
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3EnterInitializationMode")
}

#[no_mangle]
pub extern "C" fn fmi3ExitInitializationMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3ExitInitializationMode(
            fmi3_messages::Fmi3ExitInitializationMode {},
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3ExitInitializationMode")
}

#[no_mangle]
pub extern "C" fn fmi3EnterEventMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterEventMode(
            fmi3_messages::Fmi3EnterEventMode {},
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3EnterEventMode")
}

#[no_mangle]
pub extern "C" fn fmi3EnterStepMode(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterStepMode(
            fmi3_messages::Fmi3EnterStepMode {},
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3EnterStepMode")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<f32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `f32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<f32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Float32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetFloat32(
            fmi3_messages::Fmi3GetFloat32 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetFloat32Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetFloat32 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetFloat32 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetFloat32 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<f64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `f64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<f64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Float64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetFloat64(
            fmi3_messages::Fmi3GetFloat64 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetFloat64Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetFloat64 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetFloat64 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(
                &format!(
                "fmi3GetFloat64 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i8>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i8` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i8>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Int8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt8(
            fmi3_messages::Fmi3GetInt8 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetInt8Return>(&cmd) {
        Ok(reply) => {
            let mut status =parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let reply_values: Vec<i8> = reply.values
                        .iter()
                        .map(|v| *v as i8)
                        .collect();

                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };

                    if values_out.len() == reply_values.len() {
                        values_out.copy_from_slice(&reply_values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetInt8 returned {} values, but {} was expected",
                            reply_values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetInt8 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetInt8 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u8>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u8` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u8>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetUInt8(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3UInt8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt8(
            fmi3_messages::Fmi3GetUInt8 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetUInt8Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let reply_values: Vec<u8> = reply.values
                        .iter()
                        .map(|v| *v as u8)
                        .collect();

                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };

                    if values_out.len() == reply_values.len() {
                        values_out.copy_from_slice(&reply_values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetUInt8 returned {} values, but {} was expected",
                            reply_values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetUInt8 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetUInt8 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i16>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i16` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i16>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Int16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt16(
            fmi3_messages::Fmi3GetInt16 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetInt16Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let reply_values: Vec<i16> = reply.values
                        .iter()
                        .map(|v| *v as i16)
                        .collect();

                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };

                    if values_out.len() == reply_values.len() {
                        values_out.copy_from_slice(&reply_values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetInt16 returned {} values, but {} was expected",
                            reply_values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetInt16 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetInt16 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u16>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u16` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u16>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetUInt16(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3UInt16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt16(
            fmi3_messages::Fmi3GetUInt16 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetUInt16Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let reply_values: Vec<u16> = reply.values
                        .iter()
                        .map(|v| *v as u16)
                        .collect();

                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };

                    if values_out.len() == reply_values.len() {
                        values_out.copy_from_slice(&reply_values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetUInt16 returned {} values, but {} was expected",
                            reply_values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetUInt16 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetUInt16 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Int32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt32(
            fmi3_messages::Fmi3GetInt32 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetInt32Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetInt32 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetInt32 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetInt32 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetUInt32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3UInt32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt32(
            fmi3_messages::Fmi3GetUInt32 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetUInt32Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetUInt32 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetUInt32 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetUInt32 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Int64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetInt64(
            fmi3_messages::Fmi3GetInt64 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetInt64Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetInt64 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetInt64 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetInt64 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetUInt64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3UInt64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetUInt64(
            fmi3_messages::Fmi3GetUInt64 { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetUInt64Return>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetUInt64 returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetUInt64 returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetUInt64 failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<bool>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `bool` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<bool>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Boolean,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetBoolean(
            fmi3_messages::Fmi3GetBoolean { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetBooleanReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_values)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetBoolean returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetBoolean returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetBoolean failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<c_char>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `c_char` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<c_char>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetString(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3String,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetString(
            fmi3_messages::Fmi3GetString { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetStringReturn>(&cmd) {
        Ok(result) => {
            let mut status = parse_status(result.status, &instance.logger);

            if status.output_is_defined() {
                if !result.values.is_empty() {
                    let conversion_result: Result<Vec<CString>, NulError> = result
                        .values
                        .iter()
                        .map(|string| CString::new(string.as_bytes()))
                        .collect();

                    match conversion_result {
                        Ok(converted_values) => {
                            instance.string_buffer = converted_values
                        },
                        Err(e) =>  {
                            instance.logger.error(
                                "Backend replied to fmi3GetString with strings containing interior nul bytes. These cannot be converted into CStrings."
                            );
                            return Fmi3Status::Fmi3Error;
                        }
                    }

                    unsafe {
                        for (idx, cstr)
                        in instance.string_buffer.iter().enumerate() {
                            std::ptr::write(
                                values.add(idx),
                                cstr.as_ptr()
                            );
                        }
                    }
                } else {
                    instance.logger.warning("fmi3GetString returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetString failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn fmi3GetBinary(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    value_sizes: *mut size_t,
    values: *mut Fmi3Binary,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetBinary(
            fmi3_messages::Fmi3GetBinary { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetBinaryReturn>(&cmd) {
        Ok(result) => {
            let mut status = parse_status(result.status, &instance.logger);

            if status.output_is_defined() {
                if !result.values.is_empty() {
                    let value_sizes_out = unsafe {
                        from_raw_parts_mut(value_sizes, n_values)
                    };

                    let compatible_value_sizes: Vec<size_t> = result.values
                        .iter()
                        .map(|byte_vec| {byte_vec.len() as size_t})
                        .collect();

                    value_sizes_out.copy_from_slice(&compatible_value_sizes);

                    instance.byte_buffer = result.values;

                    for (idx, byte_vec)
                    in instance.byte_buffer.iter().enumerate() {
                        unsafe {
                            std::ptr::write(
                                values.add(idx),
                                byte_vec.as_ptr()
                            );
                        }
                    }

                } else {
                    instance.logger.warning("fmi3GetBinary returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "Fmi3GetBinary failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }    
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_value_references * mem::size_of::<bool>()` many bytes respectively,
///   and they must be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   consecutive properly initialized values of type `u32` and
///   `bool` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_value_references * mem::size_of::<bool>()` of the slices must be no
///   larger than `isize::MAX`, and adding those sizes to `value_references`
///   and `values` respectively must not "wrap around" the address space. See
///   the safety documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetClock(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *mut Fmi3Clock,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetClock(
            fmi3_messages::Fmi3GetClock { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetClockReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.values.is_empty() {
                    let values_out = unsafe {
                        from_raw_parts_mut(values, n_value_references)
                    };
                    if values_out.len() == reply.values.len() {
                        values_out.copy_from_slice(&reply.values);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetClock returned {} values, but {} was expected",
                            reply.values.len(),
                            values_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetClock returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }
            
            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetClock failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references`, `intervals` and `qualifiers` must be non-null,
///   \[valid\] for reads for `n_value_references * mem::size_of::<u32>()`,
///   `n_value_references * mem::size_of::<f64>()` and
///   `n_value_references * mem::size_of::<i32>()` many bytes respectively,
///   and they must be properly aligned. This means in particular:
///     * For each of `value_references`, `intervals` and `qualifiers` the
///       entire memory range of that slice must be contained within a single
///       allocated object! Slices can never span across multiple allocated
///       objects.
///     * `value_references`, `intervals` and `qualifiers` must each be
///       non-null and aligned even for zero-length slices or slices of ZSTs.
///       One reason for this is that enum layout optimizations may rely on
///       references (including slices of any length) being aligned and
///       non-null to distinguish them from other data. You can obtain a
///       pointer that is usable as `value_references`, `intervals` or
///       `qualifiers` for zero-length slices using \[`NonNull::dangling`\].
/// * `value_references`, `intervals` and `qualifiers` must each point to
///   `n_value_references` consecutive properly initialized values of type
///   `u32`, `f64` and `i32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()`,
///   `n_value_references * mem::size_of::<f64>()` or 
///   `n_value_references * mem::size_of::<booi32l>()` of the slices must be no
///   larger than `isize::MAX`, and adding those sizes to `value_references`,
///   `intervals` and `qualifiers` respectively must not "wrap around" the
///   address space. See the safety documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3GetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    intervals: *mut Fmi3Float64,
	qualifiers: *mut Fmi3IntervalQualifier,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetIntervalDecimal(
            fmi3_messages::Fmi3GetIntervalDecimal { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetIntervalDecimalReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.intervals.is_empty() {
                    let intervals_out = unsafe {
                        from_raw_parts_mut(intervals, n_value_references)
                    };
                    if intervals_out.len() == reply.intervals.len() {
                        intervals_out.copy_from_slice(&reply.intervals);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalDecimal returned {} intervals, but {} was expected",
                            reply.intervals.len(),
                            intervals_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetIntervalDecimal returned no intervals.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !reply.qualifiers.is_empty() {
                    let Ok(reply_qualifiers) = reply.qualifiers.into_iter()
                        .map(Fmi3IntervalQualifier::try_from)
                        .collect::<Result<Vec<Fmi3IntervalQualifier>, _>>()
                    else {
                        instance.logger.error(
                            "fmi3GetIntervalDecimal got unknown interval qualifiers from backend."
                        );
                        return Fmi3Status::Fmi3Error
                    };
                    let qualifiers_out = unsafe {
                        from_raw_parts_mut(qualifiers, n_value_references)
                    };
                    if qualifiers_out.len() == reply_qualifiers.len() {
                        qualifiers_out.copy_from_slice(reply_qualifiers.as_slice());
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalDecimal returned {} qualifiers, but {} was expected",
                            reply_qualifiers.len(),
                            qualifiers_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetIntervalDecimal returned no qualifiers.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetIntervalDecimal failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    counters: *mut Fmi3UInt64,
	resolutions: *mut Fmi3UInt64,
    qualifiers: *mut Fmi3IntervalQualifier,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetIntervalFraction(
            fmi3_messages::Fmi3GetIntervalFraction { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetIntervalFractionReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.counters.is_empty() {
                    let counters_out = unsafe {
                        from_raw_parts_mut(counters, n_value_references)
                    };
                    if counters_out.len() == reply.counters.len() {
                        counters_out.copy_from_slice(&reply.counters);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalFraction returned {} counters, but {} was expected",
                            reply.counters.len(),
                            counters_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetIntervalFraction returned no counters.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !reply.resolutions.is_empty() {
                    let resolutions_out = unsafe {
                        from_raw_parts_mut(resolutions, n_value_references)
                    };
                    if resolutions_out.len() == reply.resolutions.len() {
                        resolutions_out.copy_from_slice(&reply.resolutions);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalFraction returned {} resolutions, but {} was expected",
                            reply.resolutions.len(),
                            resolutions_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetIntervalFraction returned no resolutions.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !reply.qualifiers.is_empty() {
                    let Ok(reply_qualifiers) = reply.qualifiers.into_iter()
                        .map(Fmi3IntervalQualifier::try_from)
                        .collect::<Result<Vec<Fmi3IntervalQualifier>, _>>()
                    else {
                        instance.logger.error(
                            "fmi3GetIntervalFraction got unknown interval qualifiers from backend."
                        );
                        return Fmi3Status::Fmi3Error
                    };
                    let qualifiers_out = unsafe {
                        from_raw_parts_mut(qualifiers, n_value_references)
                    };
                    if qualifiers_out.len() == reply_qualifiers.len() {
                        qualifiers_out.copy_from_slice(reply_qualifiers.as_slice());
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalFraction returned {} qualifiers, but {} was expected",
                            reply_qualifiers.len(),
                            qualifiers_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetIntervalFraction returned no qualifiers.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetIntervalFraction failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    shifts: *mut Fmi3Float64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetShiftDecimal(
            fmi3_messages::Fmi3GetShiftDecimal { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetShiftDecimalReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.shifts.is_empty() {
                    let shifts_out = unsafe {
                        from_raw_parts_mut(shifts, n_value_references)
                    };
                    if shifts_out.len() == reply.shifts.len() {
                        shifts_out.copy_from_slice(&reply.shifts);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetIntervalDecimal returned {} shifts, but {} was expected",
                            reply.shifts.len(),
                            shifts_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetShiftDecimal returned no values.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetShiftDecimal failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3GetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    counters: *mut Fmi3UInt64,
	resolutions: *mut Fmi3UInt64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3GetShiftFraction(
            fmi3_messages::Fmi3GetShiftFraction { value_references }
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3GetShiftFractionReturn>(&cmd) {
        Ok(reply) => {
            let mut status = parse_status(reply.status, &instance.logger);

            if status.output_is_defined() {
                if !reply.counters.is_empty() {
                    let counters_out = unsafe {
                        from_raw_parts_mut(counters, n_value_references)
                    };
                    if counters_out.len() == reply.counters.len() {
                        counters_out.copy_from_slice(&reply.counters);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetShiftFraction returned {} counters, but {} was expected",
                            reply.counters.len(),
                            counters_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetShiftFraction returned no counters.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }

                if !reply.resolutions.is_empty() {
                    let resolutions_out = unsafe {
                        from_raw_parts_mut(resolutions, n_value_references)
                    };
                    if resolutions_out.len() == reply.resolutions.len() {
                        resolutions_out.copy_from_slice(&reply.resolutions);
                    } else {
                        instance.logger.error(&format!(
                            "fmi3GetShiftFraction returned {} resolutions, but {} was expected",
                            reply.resolutions.len(),
                            resolutions_out.len()
                        ));
                        status = status.escalate_status(Fmi3Status::Fmi3Error);
                    }
                } else {
                    instance.logger.warning("fmi3GetShiftFraction returned no resolutions.");
                    status = status.escalate_status(Fmi3Status::Fmi3Warning);
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetShiftFraction failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3SetIntervalDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    intervals: *const Fmi3Float64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let intervals = unsafe {
        from_raw_parts(intervals, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetIntervalDecimal(
            fmi3_messages::Fmi3SetIntervalDecimal {
                value_references,
                intervals,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetIntervalDecimal")
}

#[no_mangle]
pub extern "C" fn fmi3SetIntervalFraction(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    counters: *const Fmi3UInt64,
	resolutions: *const Fmi3UInt64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let counters = unsafe {
        from_raw_parts(counters, n_value_references)
    }.to_owned();

    let resolutions = unsafe {
        from_raw_parts(resolutions, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetIntervalFraction(
            fmi3_messages::Fmi3SetIntervalFraction {
                value_references,
                counters,
                resolutions,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetIntervalFraction")
}

#[no_mangle]
pub extern "C" fn fmi3SetShiftDecimal(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    shifts: *const Fmi3Float64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let shifts = unsafe {
        from_raw_parts(shifts, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetShiftDecimal(
            fmi3_messages::Fmi3SetShiftDecimal {
                value_references,
                shifts,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetShiftDecimal")
}

#[no_mangle]
pub extern "C" fn fmi3SetShiftFraction(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    counters: *const Fmi3UInt64,
	resolutions: *const Fmi3UInt64,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let counters = unsafe {
        from_raw_parts(counters, n_value_references)
    }.to_owned();

    let resolutions = unsafe {
        from_raw_parts(resolutions, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetShiftFraction(
            fmi3_messages::Fmi3SetShiftFraction {
                value_references,
                counters,
                resolutions,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetShiftFraction")
}

#[no_mangle]
pub extern "C" fn fmi3EvaluateDiscreteStates(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    instance.logger.error("fmi3EvaluateDiscreteStates is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

/// # Safety
/// Behavior is undefined if any of `discrete_states_need_update`,
/// `terminate_simulation`, `nominals_continuous_states_changed`,
/// `values_continuous_states_changed`, `next_event_time_defined` and
/// `next_event_time` points outside of address space and if they are
/// dereferenced after function call.
#[no_mangle]
pub unsafe extern "C" fn fmi3UpdateDiscreteStates(
    instance: &mut Fmi3Slave,
	discrete_states_need_update: *mut Fmi3Boolean,
	terminate_simulation: *mut Fmi3Boolean,
	nominals_continuous_states_changed: *mut Fmi3Boolean,
	values_continuous_states_changed: *mut Fmi3Boolean,
	next_event_time_defined: *mut Fmi3Boolean,
	next_event_time: *mut Fmi3Float64,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3UpdateDiscreteStates(
            fmi3_messages::Fmi3UpdateDiscreteStates {}
        )),
    };

    match instance.dispatch::<fmi3_messages::Fmi3UpdateDiscreteStatesReturn>(&cmd){
        Ok(result) => {
            let status = parse_status(result.status, &instance.logger);

            if status.output_is_defined() {
                if !discrete_states_need_update.is_null() {
                    unsafe {
                        *discrete_states_need_update = result
                            .discrete_states_need_update;
                    }
                }
                if !terminate_simulation.is_null() {
                    unsafe {
                        *terminate_simulation = result.terminate_simulation;
                    }
                }
                if !nominals_continuous_states_changed.is_null() {
                    unsafe {
                        *nominals_continuous_states_changed = result
                            .nominals_continuous_states_changed;
                    }
                }
                if !values_continuous_states_changed.is_null() {
                    unsafe {
                        *values_continuous_states_changed = result
                            .values_continuous_states_changed;
                    }
                }
                if !next_event_time_defined.is_null() {
                    unsafe {
                        *next_event_time_defined = result
                            .next_event_time_defined;
                    }
                }
                if !next_event_time.is_null() {
                    unsafe {
                        *next_event_time = result.next_event_time;
                    }
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3UpdateDiscreteStates failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3EnterContinuousTimeMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    instance.logger.error("fmi3EnterContinuousTimeMode is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3CompletedIntegratorStep(
    instance: &mut Fmi3Slave,
	no_set_fmu_state_prior_to_current_point: Fmi3Boolean,
	enter_event_mode: *const Fmi3Boolean,
	terminate_simulation: *const Fmi3Boolean,
) -> Fmi3Status {
    instance.logger.error("fmi3CompletedIntegratorStep is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3SetTime(
    instance: &mut Fmi3Slave,
	time: Fmi3Float64,
) -> Fmi3Status {
    instance.logger.error("fmi3SetTime is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3SetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *const Fmi3Float64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3SetContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetContinuousStateDerivatives(
    instance: &mut Fmi3Slave,
	derivatives: *mut Fmi3Float64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetContinuousStateDerivatives is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetEventIndicators(
    instance: &mut Fmi3Slave,
	event_indicators: *mut Fmi3Float64,
	n_event_indicators: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetEventIndicators is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetContinuousStates(
    instance: &mut Fmi3Slave,
	continuous_states: *mut Fmi3Float64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNominalsOfContinuousStates(
    instance: &mut Fmi3Slave,
	nominals: *mut Fmi3Float64,
	n_continuous_states: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetNominalsOfContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfEventIndicators(
    instance: &mut Fmi3Slave,
	n_event_indicators: *const size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetNumberOfEventIndicators is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfContinuousStates(
    instance: &mut Fmi3Slave,
	n_continuous_states: *const size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetNumberOfContinuousStates is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetOutputDerivatives(
    instance: &mut Fmi3Slave,
	value_references: *const Fmi3ValueReference,
	n_value_references: size_t,
	orders: *const Fmi3Int32,
	values: *mut Fmi3Float64,
	n_values: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetOutputDerivatives is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3ActivateModelPartition(
    instance: &mut Fmi3Slave,
	value_reference: Fmi3ValueReference,
	activation_time: Fmi3Float64,
) -> Fmi3Status {
    instance.logger.error("fmi3ActivateModelPartition is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<f32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `f32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<f32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetFloat32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Float32,
    n_values: size_t,
) -> Fmi3Status{
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references) 
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetFloat32(
            fmi3_messages::Fmi3SetFloat32 {
                value_references,
                values,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetFloat32")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<f64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `f64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<f64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetFloat64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Float64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }
        .to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetFloat64(
            fmi3_messages::Fmi3SetFloat64 {
                value_references,
                values
            }
        )),
    };
    
    send_cmd_recv_status(instance, cmd, "fmi3SetFloat64")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i8>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i8` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i8>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetInt8(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Int8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<i32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as i32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt8(
            fmi3_messages::Fmi3SetInt8 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetInt8")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u8>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u8` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u8>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetUInt8(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3UInt8,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<u32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as u32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt8(
            fmi3_messages::Fmi3SetUInt8 {
                value_references,
                values
            }
        )),
    };
    
    send_cmd_recv_status(instance, cmd, "fmi3SetUInt8")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i16>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i16` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i16>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetInt16(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Int16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<i32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as i32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt16(
            fmi3_messages::Fmi3SetInt16 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetInt16")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u16>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u16` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u16>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetUInt16(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3UInt16,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let values: Vec<u32> = unsafe {
        from_raw_parts(values, n_values)
    }
        .iter()
        .map(|&v| v as u32)
        .collect();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt16(
            fmi3_messages::Fmi3SetUInt16 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetUInt16")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetInt32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Int32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt32(
            fmi3_messages::Fmi3SetInt32 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetInt32")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u32>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u32` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u32>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetUInt32(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3UInt32,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt32(
            fmi3_messages::Fmi3SetUInt32 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetUInt32")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<i64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `i64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<i64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetInt64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Int64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetInt64(
            fmi3_messages::Fmi3SetInt64 {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetInt64")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<u64>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `u64` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<u64>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetUInt64(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3UInt64,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetUInt64(
            fmi3_messages::Fmi3SetUInt64 {
                value_references,
                values,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetUInt64")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<bool>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `bool` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<bool>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetBoolean(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Boolean,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

    let values = unsafe {
        from_raw_parts(values, n_values)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetBoolean(
            fmi3_messages::Fmi3SetBoolean {
                value_references,
                values
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetBoolean")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<*const c_char>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `*const c_char` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<*const c_char>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetString(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3String,
    n_values: size_t,
) -> Fmi3Status {
    let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let conversion_result: Result<Vec<String>, Utf8Error> = unsafe {
        from_raw_parts(values, n_values)
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
            let cmd = Fmi3Command {
                command: Some(Command::Fmi3SetString(
                    fmi3_messages::Fmi3SetString {
                        value_references,
                        values,
                    }
                )),
            };
        
            send_cmd_recv_status(instance, cmd, "fmi3SetString")
        },

        Err(conversion_error) => {
            instance.logger.error(&format!(
                "The String values could not be converted to Utf-8: {}.", conversion_error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `value_sizes` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<size_t>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `value_sizes` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `value_sizes` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `value_sizes` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `value_sizes` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `size_t` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<size_t>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `value_sizes`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
/// 
/// Furthermore, for each `VALUE` pointed to in `values` and its expected size `SIZE`
/// pointed to in `value_sizes`, behavior will be undefined if any of the
/// following conditions are violated:
/// * `VALUE` must be non-null, \[valid\] for reads for
///   `SIZE * mem::size_of::<u8>()` many bytes, and it must be properly
///   aligned. This means in particular:
///     * The entire memory range of that slice must be contained within a
///       single allocated object! Slices can never span across multiple
///       allocated objects.
///     * `VALUE` must be non-null and aligned even for zero-length slices or
///       slices of ZSTs. One reason for this is that enum layout
///       optimizations may rely on references (including slices of any length)
///       being aligned and non-null to distinguish them from other data. You
///       can obtain a pointer that is usable as `VALUE` for zero-length slices
///       using \[`NonNull::dangling`\].
/// * `VALUE` must point to `SIZE` consecutive properly initialized values of
///   type `u8`.
/// * The total size `SIZE * mem::size_of::<u8>()` of the slice must be no
///   larger than`isize::MAX`, and adding that size to `VALUE` must not
///   "wrap around" the address space. See the safety documentation of
///   [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetBinary(
    instance: *mut c_void,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    value_sizes: *const size_t,
    values: *const Fmi3Binary, // Updated
    n_values: size_t,
) -> Fmi3Status {

    let value_references = unsafe {
        std::slice::from_raw_parts(value_references, n_value_references)
    }
        .to_owned();

    let sizes = unsafe {
        std::slice::from_raw_parts(value_sizes, n_value_references)
    };

    // Create a vector of slices for the binary values (with enough space for the amount of internal values)
    let mut binary_values: Vec<&[u8]> = Vec::with_capacity(n_values);

    for i in 0..n_value_references {
        let data_ptr = values.add(i).read();
        let size = value_sizes.add(i).read();
        // Ensure the pointer is not null
        if !data_ptr.is_null() && size > 0 {
            // Read the binary data into a slice
            let value_slice = std::slice::from_raw_parts(data_ptr, size);
            binary_values.push(value_slice);
        }
    }

    let value_sizes = sizes
        .iter()
        .map(|&size| {size as u64})
        .collect();

    let values = binary_values
        .iter()
        .map(|&v| {v.to_vec()})
        .collect();

    // Ensure the instance pointer is valid
    let instance = unsafe { &mut *(instance as *mut Fmi3Slave) };

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetBinary(
            fmi3_messages::Fmi3SetBinary {
                value_references,
                value_sizes,
                values,
            }
        )),
    };

    // Call the dispatcher with references, binary values, and their sizes
    send_cmd_recv_status(instance, cmd, "fmi3SetBinary")
}

/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
/// * `value_references` and `values` must be non-null, \[valid\] for reads for
///   `n_value_references * mem::size_of::<u32>()` and
///   `n_values * mem::size_of::<bool>()` many bytes respectively, and they must
///   be properly aligned. This means in particular:
///     * For each of `value_references` and `values` the entire memory range
///       of that slice must be contained within a single allocated object!
///       Slices can never span across multiple allocated objects.
///     * `value_references` and `values` must each be non-null and aligned
///       even for zero-length slices or slices of ZSTs. One reason for this is
///       that enum layout optimizations may rely on references (including
///       slices of any length) being aligned and non-null to distinguish them
///       from other data. You can obtain a pointer that is usable as
///       `value_references` or `values` for zero-length slices using
///       \[`NonNull::dangling`\].
/// * `value_references` and `values` must each point to `n_value_references`
///   and `n_values` consecutive properly initialized values of type `u32` and
///   `bool` respectively.
/// * The total size `n_value_references * mem::size_of::<u32>()` or 
///   `n_values * mem::size_of::<bool>()` of the slices must be no larger than
///   `isize::MAX`, and adding those sizes to `value_references` and `values`
///   respectively must not "wrap around" the address space. See the safety
///   documentation of [`pointer::offset`].
#[no_mangle]
pub unsafe extern "C" fn fmi3SetClock(
    instance: &mut Fmi3Slave,
    value_references: *const Fmi3ValueReference,
    n_value_references: size_t,
    values: *const Fmi3Clock,
) -> Fmi3Status {
	let value_references = unsafe {
        from_raw_parts(value_references, n_value_references)
    }.to_owned();

	let values = unsafe {
        from_raw_parts(values, n_value_references)
    }.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SetClock(
            fmi3_messages::Fmi3SetClock {
                value_references,
                values,
            }
        )),
    };
	
	send_cmd_recv_status(instance, cmd, "fmi3SetClock")
}

#[no_mangle]
pub extern "C" fn fmi3GetNumberOfVariableDependencies(
    instance: &mut Fmi3Slave,
    value_reference: *const Fmi3ValueReference,
    n_dependencies: *const size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetNumberOfVariableDependencies is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetVariableDependencies(
    instance: &mut Fmi3Slave,
    dependent: *const Fmi3ValueReference,
    element_indices_of_dependent: *const size_t,
	independents: *const Fmi3ValueReference,
	element_indices_of_independents: *const size_t,
	dependency_kinds: *mut Fmi3DependencyKind,
	n_dependencies: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetVariableDependencies is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetFMUState(
    instance: *mut Fmi3Slave,
    state: *mut *mut SlaveState,
) -> Fmi3Status {
    if instance.is_null() {
        // Note that this error message can never reach the importer as the
        // slave contains the logging callback. This is only visible if the
        // api has been compiled with the 'fmt_logging' feature, and then
        // only on the stderr of the process containing the FMU.
        Fmi3Logger::fmt_log(
            "fmi3GetFMUstate called with instance pointing to null!",
            &Fmi3Status::Fmi3Error
        );
        return Fmi3Status::Fmi3Error;
    }

    let instance = unsafe { &mut *instance };

    if state.is_null() {
        instance.logger.error("fmi3GetFMUstate called with state pointing to null!");
        return Fmi3Status::Fmi3Error;
    }

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3SerializeFmuState(
            fmi3_messages::Fmi3SerializeFmuState {}
        )),
    };    

    match instance.dispatch::<fmi3_messages::Fmi3SerializeFmuStateReturn>(&cmd) {
        Ok(result) => {
            let status = parse_status(result.status, &instance.logger);

            if status.output_is_defined() {
                unsafe {
                    match (*state).as_mut() {
                        Some(state_ptr) => {
                            let state = &mut *state_ptr;
                            state.bytes = result.state;
                        }
                        None => {
                            let new_state = Box::new(SlaveState::new(&result.state));
                            *state = Box::into_raw(new_state);
                        }
                    }
                }
            }

            status
        }
        Err(error) => {
            instance.logger.error(&format!(
                "fmi3GetFMUstate failed with error: {}.", error
            ));
            Fmi3Status::Fmi3Error
        }
    }
}

#[no_mangle]
pub extern "C" fn fmi3SetFMUState(
	instance: *mut Fmi3Slave,
	state: *const SlaveState,
) -> Fmi3Status {
    if instance.is_null() {
        // Note that this error message can never reach the importer as the
        // slave includes the logging callback. This is only visible if the
        // api has been compiled with the 'fmt_logging' feature, and then
        // only on the stderr of the process containing the FMU.
        Fmi3Logger::fmt_log(
            "fmi3SetFMUstate called with instance pointing to null!",
            &Fmi3Status::Fmi3Error
        );
        return Fmi3Status::Fmi3Error;
    }

    let instance = unsafe { &mut *instance };

    if state.is_null() {
        instance.logger.error("fmi3SetFMUstate called with state pointing to null!");
        return Fmi3Status::Fmi3Error;
    }
    let state_ref = unsafe { &*state };
    let state_bytes = state_ref.bytes.to_owned();

    let cmd = Fmi3Command {
        command: Some(Command::Fmi3DeserializeFmuState(
            fmi3_messages::Fmi3DeserializeFmuState {
                state: state_bytes,
            }
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3SetFMUState")
}

#[no_mangle]
pub extern "C" fn fmi3FreeFMUState(
    instance: *mut Fmi3Slave,
    state: *mut *mut SlaveState,
) -> Fmi3Status {
    if instance.is_null(){
        // Note that this error message can never reach the importer as the
        // slave includes the logging callback. This log event is thus only
        // visible if the api has been compiled with the 'fmt_logging'
        // feature, and then only on the stderr of the process containing
        // the FMU.
        Fmi3Logger::fmt_log(
            "fmi3FreeFMUstate called with instance pointing to null.",
            &Fmi3Status::Fmi3OK
        );
        return Fmi3Status::Fmi3OK;
    }

    let instance = unsafe { &mut *instance };

    if state.is_null(){
        instance.logger.ok("fmi3FreeFMUstate called with state pointing to null!");
        return Fmi3Status::Fmi3OK;
    }

    unsafe {
        let state_ptr = *state;

        if state_ptr.is_null() {
            instance.logger.ok("fmi3FreeFMUstate called with state pointing to null!");
            return Fmi3Status::Fmi3OK;
        }

        drop(Box::from_raw(state_ptr)); 
        *state = std::ptr::null_mut(); // Setting the state to null
    }

    Fmi3Status::Fmi3OK
}

#[no_mangle]
pub extern "C" fn fmi3SerializedFMUStateSize(
    instance: &Fmi3Slave,
    state: &SlaveState,
	size: &mut size_t,
) -> Fmi3Status {
    *size = state.bytes.len();
    Fmi3Status::Fmi3OK
}

#[no_mangle]
pub extern "C" fn fmi3SerializeFMUState(
    instance: &Fmi3Slave,
    state: &SlaveState,
	serialized_state: *mut Fmi3Byte,
	size: size_t,
) -> Fmi3Status {
    let serialized_state_len = state.bytes.len();

    if serialized_state_len > size {
        instance.logger.error("Error while calling fmi3SerializeFMUstate: FMUstate too big to be contained in given byte vector.");
        return Fmi3Status::Fmi3Error;
    }

    unsafe { std::ptr::copy(
        state.bytes.as_ptr(),
        serialized_state.cast(),
        serialized_state_len
    ) };

    Fmi3Status::Fmi3OK
}

#[no_mangle]
pub unsafe extern "C" fn fmi3DeserializeFMUState(
    instance: &mut Fmi3Slave,
	serialized_state: *const Fmi3Byte,
	size: size_t,
	state: *mut *mut SlaveState,
) -> Fmi3Status {
    let serialized_state = unsafe { from_raw_parts(serialized_state, size) };

    if state.is_null() {
        instance.logger.error("fmi3DeSerializeFMUstate called with state pointing to null!");
        return Fmi3Status::Fmi3Error;
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

    Fmi3Status::Fmi3OK
}

#[no_mangle]
pub extern "C" fn fmi3GetDirectionalDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const Fmi3ValueReference,
	n_unknowns: size_t,
	knowns: *const Fmi3ValueReference,
	n_knowns: size_t,
	delta_knowns: *const Fmi3Float64,
	n_delta_knowns: size_t,
	delta_unknowns: *const Fmi3Float64,
	n_delta_unknowns: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetDirectionalDerivative is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3GetAdjointDerivative(
    instance: &mut Fmi3Slave,
	unknowns: *const Fmi3ValueReference,
	n_unknowns: size_t,
	knowns: *const Fmi3ValueReference,
	n_knowns: size_t,
	delta_unknowns: *const Fmi3Float64,
	n_delta_unknowns: size_t,
	delta_knowns: *const Fmi3Float64,
	n_delta_knowns: size_t,
) -> Fmi3Status {
    instance.logger.error("fmi3GetAdjointDerivative is not implemented by UNIFMU.");
    Fmi3Status::Fmi3Error
}

#[no_mangle]
pub extern "C" fn fmi3EnterConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3EnterConfigurationMode(
            fmi3_messages::Fmi3EnterConfigurationMode {}
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3EnterConfigurationMode")
}

#[no_mangle]
pub extern "C" fn fmi3ExitConfigurationMode(
    instance: &mut Fmi3Slave,
) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3ExitConfigurationMode(
            fmi3_messages::Fmi3ExitConfigurationMode {}
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3ExitConfigurationMode")
}

#[no_mangle]
pub extern "C" fn fmi3Terminate(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3Terminate(
            fmi3_messages::Fmi3Terminate {}
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3Terminate")
}

#[no_mangle]
pub extern "C" fn fmi3FreeInstance(instance: Option<Box<Fmi3Slave>>) {
    let mut instance = instance;

    if instance.as_mut().is_some() {
        drop(instance)
    } else {
        // Note that this error message can never reach the importer as the
        // slave includes the logging callback. This is only visible if the
        // api has been compiled with the 'fmt_logging' feature, and then
        // only on the stderr of the process containing the FMU.
        Fmi3Logger::fmt_log(
            "fmi3FreeInstance called with instance pointing to null.",
            &Fmi3Status::Fmi3Warning
        );
    }
}

#[no_mangle]
pub extern "C" fn fmi3Reset(instance: &mut Fmi3Slave) -> Fmi3Status {
    let cmd = Fmi3Command {
        command: Some(Command::Fmi3Reset(
            fmi3_messages::Fmi3Reset {}
        )),
    };

    send_cmd_recv_status(instance, cmd, "fmi3Reset")
}

/// Send a Fmi3Command to the backend and parse and return the status that it
/// responds with.
/// 
/// If the correspondance with the backend fails, this returns a
/// Fmi3Status::Fmi3Error and emits an error message through the given
/// instance's logger.
/// 
/// If the status fails to parse, this returns an Fmi3Status::Fmi3Fatal.
fn send_cmd_recv_status(
    instance: &mut Fmi3Slave,
    cmd: Fmi3Command,
    function_name: &str
) -> Fmi3Status {
    instance.dispatch::<fmi3_messages::Fmi3StatusReturn>(&cmd)
        .map(|reply| parse_status(reply.status, &instance.logger))
        .unwrap_or_else(|error| {
            instance.logger.error(&format!(
                "{function_name} failed with error: {error}."
            ));
            Fmi3Status::Fmi3Error
        })
}

/// Parses the given status_int as a Fmi3Status, defaulting to
/// Fmi3Status::Fmi3Fatal, if no Fmi3Status corresponds to the
/// given status_int.
/// 
/// If the status fails to parse, an error message will be emitted through the
/// given logger.
fn parse_status(status_int: i32, logger: &Fmi3Logger) -> Fmi3Status {
    Fmi3Status::try_from(status_int)
        .unwrap_or_else(|_| {
            logger.fatal(&format!(
                "Unknown status [{status_int}] returned from backend."
            ));
            Fmi3Status::Fmi3Fatal
    })
}

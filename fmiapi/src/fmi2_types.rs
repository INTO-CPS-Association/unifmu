// Type definitions for parameters in functions crossing the ABI boundary
// betweeen C and Rust.

use std::ffi::{c_char, c_double, c_int};

use core::marker::{PhantomData, PhantomPinned};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub type Fmi2Real = c_double;
pub type Fmi2Integer = c_int;
/// Should be checked for out of range value and converted to bool before use
/// if given as a function argument.
/// 
/// # Example
/// ```
/// use fmiapi::fmi2_types::Fmi2Boolean;
/// 
/// pub extern "C" fn some_ffi_function(boolean_from_c: Fmi2Boolean) {
///     let converted_boolean = match boolean_from_c {
///         0 => false,
///         1 => true,
///         _ => {
///             // Handle error
///             // ...
///             return // Potential error return value
///         }
///     };
/// }
/// ```
pub type Fmi2Boolean = c_int;
pub type Fmi2Char = c_char;
/// Must be checked for null-ness and converted to Rust str before use if given
/// as a function argument.
/// 
/// # Example
/// ```
/// use std::ffi::CStr;
/// use fmiapi::fmi2_types::Fmi2String;
/// 
/// pub unsafe extern "C" fn some_ffi_function(string_from_c: Fmi2String) {
///     let converted_string = unsafe {
///         match string_from_c.as_ref() {
///             Some(reference) => match CStr::from_ptr(reference).to_str() {
///                 Ok(converted_string) => converted_string,
///                 Err(e) => {
///                     // Handle error
///                     // ...
///                     return // Potential error return value
///                 }
///             },
///             None => {
///                 // Handle error
///                 // ...
///                 return // Potential error return value
///             }
///         }
///     };
///     // ...
/// }
/// ```
pub type Fmi2String = *const Fmi2Char;
pub type Fmi2Byte = c_char;

#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Fmi2Status {
    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
    Pending = 5,
}

/// From specification:
///
/// `This is a pointer to a data structure in the simulation environment that
/// calls the FMU.
/// Using this pointer, data from the modelDescription.xml file [(for example,
/// mapping of valueReferences to variable names)] can be transferred between
/// the simulation environment and the logger function.`
///
/// Recommended way to represent opaque pointer, i.e the c type 'void*'
/// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
/// 
/// Representing it this way lets us have type safety without knowing the
/// structure of the type.
/// 
/// The _marker is there to ensure that the rust compiler knows that this
/// type isn't thread safe (as it is essentially a gussied up pointer).
#[repr(C)]
pub struct ComponentEnvironment {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>
}

///
/// Represents the function signature of the logging callback function passsed
/// from the envrionment to the slave during instantiation.
pub type Fmi2CallbackLogger = unsafe extern "C" fn(
    component_environment: *const ComponentEnvironment,
    instance_name: Fmi2String,
    status: Fmi2Status,
    category: Fmi2String,
    message: Fmi2String,
    ...
);

//pub type Fmi2CallbackAllocateMemory = extern "C" fn(nobj: c_ulonglong, size: c_ulonglong);
//pub type Fmi2CallbackFreeMemory = extern "C" fn(obj: *const c_void);
pub type Fmi2StepFinished = unsafe extern "C" fn(
    component_environment: *const ComponentEnvironment,
    status: Fmi2Status
);

/// A set of callback functions provided by the environment
/// Note that all but the 'logger' function are optional and may only be used if the appropriate
/// flags are set in the 'modelDescription.xml' file
#[repr(C)]
pub struct Fmi2CallbackFunctions {
    pub logger: Fmi2CallbackLogger,

    // Memory Management functions aren't feasible in Rust.
    pub _allocate_memory: Option<unsafe extern "C" fn(...)>,
    pub _free_memory: Option<unsafe extern "C" fn(...)>,

    pub step_finished: Option<Fmi2StepFinished>,
    pub component_environment: ComponentEnvironment,
}
// Type definitions for parameters in functions crossing the ABI boundary
// betweeen C and Rust.

use std::ffi::{c_char, c_double, c_int};

pub type Fmi2Real = c_double;
pub type Fmi2Integer = c_int;
/// Should be checked for out of range value and converted to bool.
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
/// Must be checked for null-ness and converted to Rust str.
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

use std::{
    error::Error,
    ffi::{CStr, c_char},
    fmt::Display,
    str::Utf8Error
};

/// Converts a raw C string to a Rust String.
pub fn c2s(c: *const c_char) -> Result<String, StringConversionError> {
    unsafe {
        c.as_ref()
            .ok_or(StringConversionError::NullError)
            .map(|c_pointer| CStr::from_ptr(c_pointer))
            .and_then(|c_str| 
                c_str.to_str()
                    .map_err(StringConversionError::from)
            )
            .map(|r_str| r_str.to_string())
    }
}

/// Converts a raw C string to Rust String if the C string is non empty.
pub fn c2non_empty_s(c: *const c_char) -> Result<String, StringConversionError> {
    c2s(c).and_then(|s|
        if !s.is_empty() {
            Ok(s)
        } else {
            Err(StringConversionError::EmptyError)
        }
    )
}

#[derive(Debug)]
pub enum StringConversionError {
    Utf8ConversionError(Utf8Error),
    NullError,
    EmptyError
}

impl Display for StringConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8ConversionError(utf8_error) => {
                write!(f, "utf-8 error: {}", utf8_error)
            }
            Self::NullError => {
                write!(f, "string pointer was null pointer")
            }
            Self::EmptyError => {
                write!(f, "string is empty")
            }
        }
    }
}

impl Error for StringConversionError {}

impl From<Utf8Error> for StringConversionError {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8ConversionError(value)
    }
}
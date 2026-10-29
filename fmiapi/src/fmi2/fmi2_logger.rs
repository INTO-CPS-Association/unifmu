//! Contains the Fmi2Logger struct, used to send log messages to the importer
//! and the environment if build with the `fmt_logging` feature.

use super::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2LogCategory,
    Fmi2Status,
    Fmi2String
};

use crate::common::logger::{
    Logger,
    category_filter::CategoryFilter
};

use std::ffi::CStr;

/// Contains functionality for filtering and emitting FMI2 log events.
/// 
/// Primarily implements the `common::logger::Logger` trait for FMI2 types,
/// sending log events to the implementer through the contained
/// `Fmi2CallbackLogger` function pointer.
pub struct Fmi2Logger {
    callback: Fmi2CallbackLogger,
    environment: *const ComponentEnvironment,
    filter: CategoryFilter<Fmi2LogCategory>,
    /// This is stored as a Fmi2String instead of a normal Rust type, as it
    /// is only ever used as a parameter of the call to `self.callback`,
    /// which effectively contains a C function pointer.
    instance_name: Fmi2String
}

impl Fmi2Logger {
    pub fn new(
        callback: Fmi2CallbackLogger,
        instance_name: Fmi2String,
        environment: *const ComponentEnvironment,
        enabled: bool
    ) -> Self {
        let filter = if enabled {
            CategoryFilter::new_blacklist()
        } else {
            CategoryFilter::new_whitelist()
        };

        Self {
            callback,
            environment,
            filter,
            instance_name
        }
    }
}

impl Logger for Fmi2Logger {
    type Category = Fmi2LogCategory;
    type Status = Fmi2Status;

    fn log(
        &self,
        status: Fmi2Status,
        category: Fmi2LogCategory,
        message: &str
    ) {
        // When enabled, the call to `fmt_log()` is - among other things -
        // intended as way to gather logging when the importer is possibly
        // failing.
        // As such, it is called before checking if the importer is interested
        // in the category of the log event.
        Self::fmt_log(message, &status);

        if !self.filter.enabled(&category) {
            return
        }

        let mut category_bytes: Vec<u8> = category.to_string().into_bytes();
        category_bytes.push(0);
        let c_category = CStr::from_bytes_until_nul(&category_bytes)
            .unwrap_or_default();

        let mut message_bytes: Vec<u8> = message.to_string().into_bytes();
        message_bytes.push(0);
        let c_message = CStr::from_bytes_until_nul(&message_bytes)
            .unwrap_or_default();

        unsafe { (self.callback)(
            self.environment,
            self.instance_name,
            status,
            c_category.as_ptr(),
            // The FMI2 callback takes a printf format, not a literal message.
            b"%s\0".as_ptr().cast(),
            c_message.as_ptr()
        ); }
    }

    fn filter(&mut self) -> &mut CategoryFilter<Self::Category> {
        &mut self.filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmi2_logger_test as _;
    use std::ffi::{c_char, c_int};

    #[repr(C)]
    struct LogCapture {
        message: [c_char; 512],
        calls: usize,
        valid_format: c_int,
    }

    extern "C" {
        fn fmi2_test_logger(
            environment: *const ComponentEnvironment,
            instance_name: Fmi2String,
            status: Fmi2Status,
            category: Fmi2String,
            format: Fmi2String,
            ...
        );
    }

    #[test]
    fn callback_preserves_literal_messages() {
        let mut capture = LogCapture {
            message: [0; 512],
            calls: 0,
            valid_format: 0,
        };
        let logger = Fmi2Logger::new(
            fmi2_test_logger,
            b"test\0".as_ptr().cast(),
            (&mut capture as *mut LogCapture).cast(),
            true,
        );
        for (index, message) in [
            "ordinary log message",
            "done 50%s of %d",
            "https://example.com/file%3Bname%22s",
            "100% complete",
            "literal %% and %n",
            "Traceback:\n    print('%s')",
            "",
        ].iter().enumerate() {
            logger.log(Fmi2Status::Ok, Fmi2LogCategory::LogAll, message);
            assert_eq!(capture.calls, index + 1);
            assert_eq!(capture.valid_format, 1, "message used as format: {message:?}");
            let actual = unsafe { CStr::from_ptr(capture.message.as_ptr()) };
            assert_eq!(actual.to_str().unwrap(), *message);
        }
    }

    #[test]
    fn disabled_logging_does_not_call_importer() {
        let mut capture = LogCapture {
            message: [0; 512],
            calls: 0,
            valid_format: 0,
        };
        let logger = Fmi2Logger::new(
            fmi2_test_logger,
            b"test\0".as_ptr().cast(),
            (&mut capture as *mut LogCapture).cast(),
            false,
        );
        logger.log(Fmi2Status::Ok, Fmi2LogCategory::LogAll, "%s %d %n");
        assert_eq!(capture.calls, 0);
    }
}
//! Contains the Fmi3Logger struct, used to send log messages to the importer
//! and the environment if build with the `fmt_logging` feature.

use super::fmi3_types::{
    Fmi3InstanceEnvironment,
    Fmi3LogMessageCallback,
    Fmi3LogCategory,
    Fmi3Status
};

use crate::common::logger::{
    Logger,
    category_filter::CategoryFilter
};

use std::ffi::CStr;

/// Contains functionality for filtering and emitting FMI3 log events.
/// 
/// Primarily implements the `common::logger::Logger` trait for FMI3 types,
/// sending log events to the implementer through the contained
/// `Fmi3LogMessageCallback` function pointer.
pub struct Fmi3Logger {
    callback: Fmi3LogMessageCallback,
    environment: *const Fmi3InstanceEnvironment,
    filter: CategoryFilter<Fmi3LogCategory>
}

impl Fmi3Logger {
    pub fn new(
        callback: Fmi3LogMessageCallback,
        environment: *const Fmi3InstanceEnvironment,
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
            filter
        }
    }
}

impl Logger for Fmi3Logger {
    type Category = Fmi3LogCategory;
    type Status = Fmi3Status;

    fn log(
        &self,
        status: Fmi3Status,
        category: Fmi3LogCategory,
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
            status,
            c_category.as_ptr(),
            c_message.as_ptr()
        ); }
    }

    fn filter(&mut self) -> &mut CategoryFilter<Self::Category> {
        &mut self.filter
    }
}
use super::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2LogCategory,
    Fmi2Status,
    Fmi2String
};

use crate::common::{
    category_filter::CategoryFilter,
    logger::Logger
};

use std::ffi::CStr;

pub struct Fmi2Logger {
    callback: Fmi2CallbackLogger,
    environment: *const ComponentEnvironment,
    filter: CategoryFilter<Fmi2LogCategory>,
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
        self.fmt_log(&format!(
            "{}{}", status.fmt_log_prefix(), message
        ));

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
            c_message.as_ptr()
        ); }
    }

    fn ok(&self, message: &str) {
        self.log(
            Fmi2Status::Ok,
            Fmi2LogCategory::LogAll,
            message
        )
    }

    fn warning(&self, message: &str) {
        self.log(
            Fmi2Status::Warning,
            Fmi2LogCategory::LogStatusWarning,
            message
        )
    }

    fn error(&self, message: &str) {
        self.log(
            Fmi2Status::Error,
            Fmi2LogCategory::LogStatusError,
            message
        );
    }

    fn fatal(&self, message: &str) {
        self.log(
            Fmi2Status::Fatal,
            Fmi2LogCategory::LogStatusFatal,
            message
        );
    }

    fn filter(&mut self) -> &mut CategoryFilter<Self::Category> {
        &mut self.filter
    }
}

impl Fmi2Status {
    pub fn fmt_log_prefix(&self) -> String {
        match self {
            Fmi2Status::Ok => String::from("[OK] "),
            Fmi2Status::Warning => String::from("[WARN] "),
            Fmi2Status::Error => String::from("[ERROR] "),
            Fmi2Status::Fatal => String::from("[FATAL] "),
            Fmi2Status::Pending => String::from("[PENDING] "),
            Fmi2Status::Discard => String::from("[DISCARD] ")
        }
    }
}
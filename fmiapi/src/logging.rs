use libc::c_char;
use std::{
    ffi::c_void,
    sync::LazyLock
};

use tracing::{Dispatch, Subscriber};
use tracing_subscriber::{layer::Filter, registry, reload::{self, Handle}, Layer};

use crate::fmi3::Fmi3Status;

{
    todo!("Implement Static Subscriber layered stack with reloadable FMI3Logging layer.");
}
pub enum Fmi3LoggingCategories {
    LogEvents,
    LogSingularLinearSystems,
    LogNonlinearSystems,
    LogDynamicStateSelection,
    LogStatusWarning,
    LogStatusDiscard,
    LogStatusError,
    LogStatusFatal,
    ModelDefinedLog(String)
}

type Fmi3LogMessageCallback = Option<unsafe extern "C" fn (
    *const c_void,
    Fmi3Status,
    *const c_char,
    Option<*const c_char>,
)>;

pub struct Fmi3Logger {
    layer_handle: Handle<>
}

impl Fmi3Logger {
    pub fn create(
        instance_name: &String,
        logging_on: bool,
        log_message: Option<Fmi3LogMessageCallback>
    ) -> Result<Fmi3Logger, Fmi3Status> {
        todo!("Replace hotswap layer over registry with new one containing new layer.");
    }

    pub fn new_callback(log_message: Fmi3LogMessageCallback) {
        todo!("Update whole layer stack so that layer has a new callback and filter is notified of change")
    }

    pub fn set_categories(categories: Vec<Fmi3LoggingCategories>) {
        todo!("Update filter with new categories.");
    }

    pub fn set_logging_on(logging_on: bool) {
        todo!("Update filter with logging bool.");
    }
}

impl Drop for Fmi3Logger {
    fn drop(&mut self) {
        todo!("Remove represented layer from hotswap layer by updating it with a new one, not including this.")
    }
}

struct Fmi3LogLayer {
    environment_pointer: *const c_void,
    log_message_pointer: Fmi3LogMessageCallback
}

impl Layer for Fmi3LogLayer {
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        todo!("Call callback with event info and environment if callback exists.");
    }
}

struct Fmi3LogFilter {
    instance_name: &String,
    logging_on: bool,
    categories: Vec<Fmi3LoggingCategories>
}

impl Filter for Fmi3LogFilter {
    fn enabled(&self,meta: &tracing::Metadata<'_>,cx: &tracing_subscriber::layer::Context<'_,S>) -> bool {
        if !self.logging_on {
            return false;
        }

        todo!("Check instance name of span and presence of categories.");
    }
}
use crate::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2Status
};

use std::{
    ffi::CStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        LazyLock
    }
};

use tracing::{Subscriber};
use tracing_subscriber::{
    fmt, layer::Layered, prelude::*, registry, reload::{self, Handle}, Layer, Registry
};

pub fn enable() -> LoggerResult<()> {
    match &*FMU_LOGGER_RELOAD_HANDLE {
        Err(_) => Err(LoggerError::SubscriberAlreadySet),
        Ok(_) => Ok(())
    }
}

pub fn add_callback(
    callback: Fmi2CallbackLogger,
    component_environment: &ComponentEnvironment,
    enabled: bool
) -> LoggerResult<u64> {
    let mut fmu_layers = match &*FMU_LOGGER_RELOAD_HANDLE {
        Err(_) => return Err(LoggerError::SubscriberAlreadySet),
        Ok(reload_handle) => {
            match reload_handle.clone_current() {
                None => return Err(LoggerError::FmuLayerVectorMissing),
                Some(fmu_layers) => fmu_layers 
            }
        }
    };

    let logger_layer_id = new_logger_id()?;

    fmu_layers.push(FmuLayer::new(logger_layer_id, callback, component_environment, enabled));

    tracing::callsite::rebuild_interest_cache();

    Ok(logger_layer_id)
}

pub type LoggerResult<T> = Result<T, LoggerError>;

pub enum LoggerError {
    SubscriberAlreadySet,
    FmuLayerVectorMissing,
    MaxLoggersExceeded
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn new_logger_id() -> LoggerResult<u64> {
    let new_id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    if new_id == 0 {
        return Err(LoggerError::MaxLoggersExceeded)
    }
    Ok(new_id)
}

type FmuLayerReloadHandle<'b> = Handle<Vec<FmuLayer<'b>>, Layered<Option<fmt::Layer<Registry>>, Registry>>;

static FMU_LOGGER_RELOAD_HANDLE: LazyLock<LoggerResult<FmuLayerReloadHandle>> = LazyLock::new(|| {
    let fmu_layers: Vec<FmuLayer> = Vec::new();

    let (reloadable, reload_handle) = reload::Layer::new(fmu_layers);

    match registry()
        .with(Some(fmt::layer()))
        .with(reloadable)
        .try_init() {
            Ok(_subscriber) => Ok(reload_handle),
            Err(_) => Err(LoggerError::SubscriberAlreadySet)
    }
});

struct FmuLayer<'a>{
    logger_uid: u64,
    callback: Fmi2CallbackLogger,
    component_environment: &'a ComponentEnvironment,
    enabled: bool
}

impl <'a> FmuLayer<'a>{
    pub fn new(
        logger_uid: u64,
        callback: Fmi2CallbackLogger,
        component_environment: &'a ComponentEnvironment,
        enabled: bool
    ) -> Self {
        Self {logger_uid, callback, component_environment, enabled}
    }
}

impl Clone for FmuLayer<'_>{
    fn clone(&self) -> Self {
        Self {
            logger_uid: self.logger_uid,
            callback: self.callback,
            component_environment: self.component_environment,
            enabled: self.enabled
        }
    }
}

impl <S: Subscriber> Layer<S> for FmuLayer<'static>{
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = FmuEventVisitor::new();
        _event.record(&mut visitor);

        if let Some(message) = visitor.message {
            let message = CStr::from_bytes_until_nul((message + "\0").as_bytes()).unwrap().as_ptr();
            let test_name = CStr::from_bytes_until_nul("TEST\0".as_bytes()).unwrap().as_ptr();
            let test_category = CStr::from_bytes_until_nul("logAll\0".as_bytes()).unwrap().as_ptr();
            unsafe { (self.callback)(
                self.component_environment,
                test_name,
                Fmi2Status::Error,
                test_category,
                message
            ) }
        }
    }
}

struct FmuEventVisitor{
    message: Option<String>
}

impl FmuEventVisitor{
    pub fn new() -> Self {
        Self { message: None }
    }
}

impl tracing::field::Visit for FmuEventVisitor{
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let message = format!("{:?}", value);
            self.message = Some(message);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(String::from(value));
        }
    }
}

/*

let test_environment: *const ComponentEnvironment = &functions.component_environment;
    let test_name = CStr::from_bytes_until_nul("TEST\0".as_bytes()).unwrap().as_ptr();
    let test_category = CStr::from_bytes_until_nul("logAll\0".as_bytes()).unwrap().as_ptr();
    let test_message = CStr::from_bytes_until_nul("This is a test\0".as_bytes()).unwrap().as_ptr();
    unsafe { (functions.logger)(
        test_environment,
        test_name,
        Fmi2Status::Error,
        test_category,
        test_message
    ) }

*/
use crate::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2Status,
    SyncComponentEnvironment
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
    component_environment: *const ComponentEnvironment,
    enabled: bool
) -> LoggerResult<u64> {
    let reload_handle = (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?;

    let mut fmu_layers = reload_handle.clone_current()
        .ok_or(LoggerError::FmuLayerVectorMissing)?;

    let logger_layer_id = new_logger_id()?;

    fmu_layers.push(FmuLayer::new(
        logger_layer_id,
        callback,
        SyncComponentEnvironment(component_environment),
        enabled
    ));

    reload_handle.reload(fmu_layers).map_err(|_| LoggerError::FmuLayerVectorMissing)?;

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

type FmuLayerReloadHandle<'b> = Handle<Vec<FmuLayer>, Layered<Option<fmt::Layer<Registry>>, Registry>>;

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

struct FmuLayer{
    logger_uid: u64,
    callback: Fmi2CallbackLogger,
    component_environment: SyncComponentEnvironment,
    enabled: bool
}

impl FmuLayer{
    pub fn new(
        logger_uid: u64,
        callback: Fmi2CallbackLogger,
        component_environment: SyncComponentEnvironment,
        enabled: bool
    ) -> Self {
        Self {logger_uid, callback, component_environment, enabled}
    }
}

impl Clone for FmuLayer{
    fn clone(&self) -> Self {
        Self {
            logger_uid: self.logger_uid,
            callback: self.callback,
            component_environment: self.component_environment,
            enabled: self.enabled
        }
    }
}

impl <S: Subscriber> Layer<S> for FmuLayer{
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = FmuEventVisitor::new();
        _event.record(&mut visitor);

        if let Some(message) = visitor.message {
            let mut message_bytes = message.into_bytes();
            message_bytes.push(0);
            let message = CStr::from_bytes_until_nul(&message_bytes).unwrap().as_ptr();
            let test_name = CStr::from_bytes_until_nul("TEST\0".as_bytes()).unwrap().as_ptr();
            let test_category = CStr::from_bytes_until_nul("logAll\0".as_bytes()).unwrap().as_ptr();
            unsafe { (self.callback)(
                self.component_environment.0,
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
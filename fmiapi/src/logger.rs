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

use tracing::Subscriber;
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

pub fn add_name_to_callback(
    logger_uid: u64,
    instance_name: &str
) -> LoggerResult<()> {
    let reload_handle = (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?;

    let fmu_layers: Vec<FmuLayer> = reload_handle
        .clone_current()
        .ok_or(LoggerError::FmuLayerVectorMissing)?
        .into_iter()
        .map(|mut layer| {
            if layer.logger_uid == logger_uid {
                layer.set_instance_name(instance_name);
            }
            layer
        })
        .collect();

    reload_handle.reload(fmu_layers).map_err(|_| LoggerError::FmuLayerVectorMissing)?;

    Ok(())
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
    enabled: bool,
    instance_name_bytes: Option<Vec<u8>>
}

impl FmuLayer{
    pub fn new(
        logger_uid: u64,
        callback: Fmi2CallbackLogger,
        component_environment: SyncComponentEnvironment,
        enabled: bool
    ) -> Self {
        Self {logger_uid, callback, component_environment, enabled, instance_name_bytes: None}
    }

    pub fn set_instance_name(&mut self, instance_name: &str) {
        let mut instance_name_bytes = String::from(instance_name).into_bytes();
        instance_name_bytes.push(0);
        self.instance_name_bytes = Some(instance_name_bytes);
    }
}

impl Clone for FmuLayer{
    fn clone(&self) -> Self {
        Self {
            logger_uid: self.logger_uid,
            callback: self.callback,
            component_environment: self.component_environment,
            enabled: self.enabled,
            instance_name_bytes: self.instance_name_bytes.clone()
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

            let instance_name_bytes = self.instance_name_bytes
                .clone()
                .unwrap_or("UNKNOWN INSTANCE\0".as_bytes().to_vec());
            
            let message = CStr::from_bytes_until_nul(&message_bytes).unwrap().as_ptr();
            let instance_name = CStr::from_bytes_until_nul(&instance_name_bytes).unwrap().as_ptr();
            let test_category = CStr::from_bytes_until_nul("logAll\0".as_bytes()).unwrap().as_ptr();
            unsafe { (self.callback)(
                self.component_environment.0,
                instance_name,
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
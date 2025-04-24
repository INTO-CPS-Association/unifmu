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

use tracing::{debug, error, span, warn, Level, Subscriber};
use tracing_subscriber::{
    fmt, layer::Layered, prelude::*, registry::{self, SpanRef}, reload::{self, Handle}, Layer, Registry
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
    let logger_layer_id = new_logger_id()?;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?
        .modify(|layer_vector| {
            layer_vector.push(
                FmuLayer::new(
                    logger_layer_id,
                    callback,
                    SyncComponentEnvironment(component_environment),
                    enabled
                )
            );
        })
        .map_err(|_| LoggerError::FmuLayerVectorMissing)?;

    Ok(logger_layer_id)
}

pub fn add_name_to_callback(
    logger_uid: u64,
    instance_name: &str
) -> LoggerResult<()> {
    let mut layer_found = false;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?
        .modify(|layer_vector| {
            for layer in layer_vector {
                if layer.logger_uid == logger_uid {
                    layer_found = true;
                    layer.set_instance_name(instance_name);
                }
            }
        })
        .map_err(|_| LoggerError::FmuLayerVectorMissing)?;

    if !layer_found {
        Err(LoggerError::LoggerLayerNotFound)
    } else {
        Ok(())
    }
}

pub fn remove_callback(logger_uid: u64) -> LoggerResult<()> {
    let mut layer_found = false;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?
        .modify(|layer_vector| {
            if let Some(index) = layer_vector.iter().position(
                |layer| layer.logger_uid == logger_uid
            ) {
                layer_found = true;
                layer_vector.swap_remove(index);
            };
        })
        .map_err(|_| LoggerError::FmuLayerVectorMissing)?;

    if !layer_found {
        Err(LoggerError::LoggerLayerNotFound)
    } else {
        Ok(())
    }
}

pub type LoggerResult<T> = Result<T, LoggerError>;

#[derive(Debug)]
pub enum LoggerError {
    SubscriberAlreadySet,
    FmuLayerVectorMissing,
    MaxLoggersExceeded,
    LoggerLayerNotFound
}

/// ID is just a simple counter under the assumption that users won't run more
/// than 2^64 instances of one FMU in one simulation. 
static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn new_logger_id() -> LoggerResult<u64> {
    let new_id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    if new_id == 0 {
        return Err(LoggerError::MaxLoggersExceeded)
    }
    Ok(new_id)
}

cfg_if::cfg_if! {
    if #[cfg(feature = "fmt_logging")] {
        const ENABLE_FMT_LOGGER: bool = true;
    } else {
        const ENABLE_FMT_LOGGER: bool = false;
    }
}

type FmuLayerVector = Vec<FmuLayer>;
type FmuLayerReloadHandle = Handle<FmuLayerVector, Layered<Option<fmt::Layer<Registry>>, Registry>>;

static FMU_LOGGER_RELOAD_HANDLE: LazyLock<LoggerResult<FmuLayerReloadHandle>> = LazyLock::new(|| {
    let fmu_layers: FmuLayerVector = Vec::new();

    let (reloadable, reload_handle) = reload::Layer::new(fmu_layers);

    match tracing_subscriber::registry()
        .with(if ENABLE_FMT_LOGGER {Some(fmt::layer())} else {None})
        .with(reloadable)
        .try_init() {
            Ok(_subscriber) => Ok(reload_handle),
            Err(_) => Err(LoggerError::SubscriberAlreadySet)
    }
});

struct EnabledForLayer { 
    logger_uid: u64
}

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
    
    fn logger_uid_in_attributes(&self, attrs: &span::Attributes<'_>) -> bool {
        let mut visitor = FmuSpanVisitor::new();
        attrs.values().record(&mut visitor);
        visitor.luid.is_some_and(|luid| luid == self.logger_uid)
    }

    fn interested_in_parent_of_span<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>> (
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.parent()
            .is_some_and(|parent_span|
                self.interested_in_span(&parent_span)
            )
    }

    fn interested_in_span<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>> (
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.extensions()
            .get::<EnabledForLayer>()
            .is_some_and(|enabled_for_layer|
                enabled_for_layer.logger_uid == self.logger_uid
            )
    }

    fn interested_in_event<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>> (
        &self,
        event: &tracing::Event<'_>,
        ctx: &tracing_subscriber::layer::Context<'_, S>
    ) -> bool {
        ctx.event_span(event)
            .is_some_and(|span| self.interested_in_span(&span))
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

impl <S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>> Layer<S> for FmuLayer{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>
    ) {
        if let Some(span) = ctx.span(id) {
            if self.interested_in_parent_of_span(&span)
            || self.logger_uid_in_attributes(attrs)
            {
                span.extensions_mut().insert(EnabledForLayer{ logger_uid: self.logger_uid });
            }
        }
      
    }

    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>
    ) {
        if !(self.enabled && self.interested_in_event(_event, &_ctx)) {
            return
        }

        let mut visitor = FmuEventVisitor::new();
        _event.record(&mut visitor);

        if let Some(message) = visitor.message {
            let mut message_bytes = message.into_bytes();
            message_bytes.push(0);

            let instance_name_bytes = self.instance_name_bytes
                .clone()
                .unwrap_or("UNKNOWN INSTANCE\0".as_bytes().to_vec());

            let fmi_status = match *_event.metadata().level() {
                Level::ERROR => Fmi2Status::Error,
                Level::WARN => Fmi2Status::Warning,
                _ => Fmi2Status::Ok
            };
            
            let message = CStr::from_bytes_until_nul(&message_bytes).unwrap().as_ptr();
            let instance_name = CStr::from_bytes_until_nul(&instance_name_bytes).unwrap().as_ptr();
            let test_category = CStr::from_bytes_until_nul("logAll\0".as_bytes()).unwrap().as_ptr();
            unsafe { (self.callback)(
                self.component_environment.0,
                instance_name,
                fmi_status,
                test_category,
                message
            ) }
        }
    }
}

struct FmuSpanVisitor{
    luid: Option<u64>
}

impl FmuSpanVisitor{
    fn new() -> Self{
        Self {luid: None}
    }
}

impl tracing::field::Visit for FmuSpanVisitor{
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug
    ) {
        if field.name() == "luid" {
            warn!("luid recorded as Debug with value: {value:?}");
            if let Ok(luid) = format!("{value:?}").parse::<u64>() {
                warn!("debug luid coerced into u64, using as actual luid");
                self.luid = Some(luid);
            } else {
                error!("debug luid could not be coerced into u64");
            }
        }
    }

    fn record_u64(
        &mut self,
        field: &tracing::field::Field,
        value: u64
    ) {
        if field.name() == "luid" {
            self.luid = Some(value);
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
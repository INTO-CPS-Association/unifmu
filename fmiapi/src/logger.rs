use crate::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2LogCategory,
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

use tracing::{error, span, warn, Level, Subscriber};
use tracing_subscriber::{
    fmt, layer::Layered, prelude::*, registry::{self, SpanRef}, reload::{self, Handle}, Layer, Registry
};

pub fn initialize() -> LoggerResult<()> {
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
    modify_callback(
        logger_uid,
        |layer| layer.set_instance_name(instance_name)
    )
}

pub fn update_enabled_for_callback(
    logger_uid: u64,
    enabled: bool
) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.enabled = enabled
    )
}

pub fn set_categories_for_callback(
    logger_uid: u64,
    mut categories: Vec<Fmi2LogCategory>
) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.set_categories(&mut categories)
    )
}

pub fn clear_categories_for_callback(logger_uid: u64) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.clear_categories()
    )
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

fn modify_callback(
    logger_uid: u64,
    modification_closure: impl FnOnce(&mut FmuLayer)
) -> LoggerResult<()> {
    let mut layer_found = false;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|_| LoggerError::SubscriberAlreadySet)?
        .modify(|layer_vector| {
            for layer in layer_vector {
                if layer.logger_uid == logger_uid {
                    layer_found = true;
                    modification_closure(layer);
                    break
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
    categories: Vec<Fmi2LogCategory>,
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
        Self {
            logger_uid,
            callback,
            categories: Vec::with_capacity(16), //There are 10 predefined logCategories, so a capacity of 16 will allow the user to implement a handful of their own without this having to reallocate for size
            component_environment,
            enabled,
            instance_name_bytes: None
        }
    }

    pub fn set_instance_name(&mut self, instance_name: &str) {
        let mut instance_name_bytes = String::from(instance_name).into_bytes();
        instance_name_bytes.push(0);
        self.instance_name_bytes = Some(instance_name_bytes);
    }

    pub fn set_categories(&mut self, categories: &mut Vec<Fmi2LogCategory>) {
        self.clear_categories();
        self.categories.append(categories);
    }

    pub fn clear_categories(&mut self) {
        self.categories.clear();
    }
    
    fn logger_uid_in_attributes(&self, attrs: &span::Attributes<'_>) -> bool {
        let mut visitor = FmuSpanVisitor::new();
        attrs.values().record(&mut visitor);
        visitor.luid.is_some_and(|luid| luid == self.logger_uid)
    }

    fn interested_in_parent_of_span<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>(
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.parent()
            .is_some_and(|parent_span|
                self.interested_in_span(&parent_span)
            )
    }

    fn interested_in_span<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>(
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.extensions()
            .get::<EnabledForLayer>()
            .is_some_and(|enabled_for_layer|
                enabled_for_layer.logger_uid == self.logger_uid
            )
    }

    fn interested_in_event<S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>>(
        &self,
        event: &tracing::Event<'_>,
        ctx: &tracing_subscriber::layer::Context<'_, S>
    ) -> bool {
        ctx.event_span(event)
            .is_some_and(|span| self.interested_in_span(&span))
    }

    fn interested_in_category(&self, category: &Fmi2LogCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(category)
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

        let category = visitor.category.unwrap_or_default();

        if !(self.interested_in_category(&category)) {
            return
        }

        if let Some(message) = visitor.message {
            let instance_name_bytes = self.instance_name_bytes
                .clone()
                .unwrap_or("UNKNOWN INSTANCE\0".as_bytes().to_vec());

            let mut category_bytes = category.str_name()
                .to_owned()
                .into_bytes();
            category_bytes.push(0);

            let mut message_bytes = message.into_bytes();
            message_bytes.push(0);

            let instance_name = CStr::from_bytes_until_nul(&instance_name_bytes)
                .unwrap_or_default()
                .as_ptr();

            let fmi_status = visitor.status
                .unwrap_or(
                    match *_event.metadata().level() {
                        Level::ERROR => Fmi2Status::Error,
                        Level::WARN => Fmi2Status::Warning,
                        _ => Fmi2Status::Ok
                    }
                );

            let category = CStr::from_bytes_until_nul(&category_bytes)
                .unwrap_or_default()
                .as_ptr();

            let message = CStr::from_bytes_until_nul(&message_bytes)
                .unwrap_or_default()
                .as_ptr();

            unsafe { (self.callback)(
                self.component_environment.0,
                instance_name,
                fmi_status,
                category,
                message
            ); }
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
    #[allow(unused_variables)]
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug
    ) {
        // Disable debug (and all methods not explicitly defined).
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
    category: Option<Fmi2LogCategory>,
    message: Option<String>,
    status: Option<Fmi2Status>
}

impl FmuEventVisitor{
    pub fn new() -> Self {
        Self { message: None, category: None, status: None }
    }
}

impl tracing::field::Visit for FmuEventVisitor{
    fn record_debug(
        &mut self,
        field: &tracing::field::Field,
        value: &dyn std::fmt::Debug
    ) {
        match field.name() {
            "category" => {
                // The value recorded here is NOT the debug value of a
                // Fmi2LogCategory, despite what the function name might imply.
                // Instead, it is the debug value of the value passed to the
                // event, which - assuming we implemented the design properly -
                // will be a String. This String should - again assuming proper
                // implementation - be created at the event! macro with the
                // Fmi2LogCategory's display() function (or it might just be a 
                // String passed from the backend). Thus what is happening here
                // is a Fmi2LogCategory Displayed into a String, Debugged into
                // the same String, converted into a &str and then remade into
                // a Fmi2LogCategory. We can't pass custom types through the
                // eye of the needle that is a tracing event.
                // And there is no record_display() function for the 
                // tracing::field::Visit trait.
                //
                // P.S.: (If the String passed to the category field is 
                // created from a Fmi2LogCategory with debug(), the resulting
                // Fmi2LogCategory from this recording will be a 
                // Fmi2LogCategory::LogUserDefined("<result of the debug() formatting>")
                // regardless of the Fmi2LogCategory's actual type.
                // You know, just, keep this in mind.)
                let category_string = format!("{:?}", value);
                self.category = Some(
                    Fmi2LogCategory::from(
                        category_string.as_str()
                    )
                );
            }
            "message" => {
                let message = format!("{:?}", value);
                self.message = Some(message);
            }
            _ => {}
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "status" {
            if let Ok(status) = Fmi2Status::try_from(value as i32) {
                self.status = Some(status);
            }
        }
    }
}
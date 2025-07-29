use crate::fmi2_types::{
    ComponentEnvironment,
    Fmi2CallbackLogger,
    Fmi2LogCategory,
    Fmi2Status,
    SyncComponentEnvironment
};

use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    error::Error,
    sync::{
        atomic::{AtomicU64, Ordering},
        LazyLock
    }
};

use tracing::{field::{Field, Visit}, span, Event, Level, Subscriber};
use tracing_subscriber::{
    fmt,
    layer::{Context, Layered},
    prelude::*,
    registry::{LookupSpan, SpanRef},
    reload::{self, Handle},
    Layer,
    Registry
};

/// Initializes the tracing subscriber that forms the basis of the
/// logger module.
/// 
/// When this idempotetnt function is called the lazy static containing the
/// global subscriber for the logger module is evalueted and set as the
/// tracing subscriber. This also happens as part of all other public functions
/// of the logger module, but this one is special insofar as it is the only
/// thing it does. While the base global subscriber doesn't have any of the FMU
/// callbacks (and therefore won't be able to send logging to the importer), it
/// does have a `fmt` layer if the fmiapi is compiled with the `fmt_logging`
/// flag. Thus, this should be called as early as possible in the rust code to
/// ensure that any tracing event that can't be send to the importer through a
/// callback for whatever reason, can atleast be send to the terminal for
/// debugging.
/// 
/// Returns `Err(LoggerError::SubscriberAlreadySet)` if a global subscriber for
/// tracing was set elsewhere. This does not mean that this function was
/// already called, but instead that rust code elsewhere already initialized
/// a subscriber and set it as the global subscriber. This could happen if
/// other rust code uses this API directly (instead of through the C ABI
/// boundary) and implements and sets its own tracing subscriber.
pub fn initialize() -> LoggerResult<()> {
    match &*FMU_LOGGER_RELOAD_HANDLE {
        Err(_) => Err(LoggerError::SubscriberAlreadySet),
        Ok(_) => Ok(())
    }
}

/// Adds a FMI2 logging callback function to the logger module.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// # Parameters
/// - `callback`: A C function pointer to the FMUs logging callback function.
/// - `component_environment`: A raw pointer to the FMI2 ComponentEnvironment
///   for the FMU.
/// - `enabled`: A boolean designating whether or not the resulting FMU logging
///   layer should emit any logging through the callback. This can later be
///   changed with calls to [update_enabled_for_callback].
/// 
/// # Returns
/// - `Ok(u64)`: An Ok result containing the uid of the resulting FMU logging
///   layer. This is needed to modify or remove the layer through the other
///   functions of this module.
/// - `Err(LoggerError)`: If for whatever reason the callback can't be added, 
///   an Err result containing the LoggerError describing the fault
///   is returned.
pub fn add_callback(
    callback: Fmi2CallbackLogger,
    component_environment: *const ComponentEnvironment,
    enabled: bool
) -> LoggerResult<u64> {
    let logger_layer_id = new_logger_id()?;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|e| e.clone())?
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

fn modify_callback(
    logger_uid: u64,
    modification_closure: impl FnOnce(&mut FmuLayer)
) -> LoggerResult<()> {
    let mut layer_found = false;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|e| e.clone())?
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
        Err(LoggerError::LoggerLayerNotFound(logger_uid))
    } else {
        Ok(())
    }
}

fn modify_callback_with_result(
    logger_uid: u64,
    modification_closure: impl FnOnce(&mut FmuLayer) -> LoggerResult<()>
) -> LoggerResult<()> {
    let mut layer_found = false;
    let mut modification_result = Ok(());

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|e| e.clone())?
        .modify(|layer_vector| {
            for layer in layer_vector {
                if layer.logger_uid == logger_uid {
                    layer_found = true;
                    modification_result = modification_closure(layer);
                    break
                }
            }
        })
        .map_err(|_| LoggerError::FmuLayerVectorMissing)?;

    if !layer_found {
        Err(LoggerError::LoggerLayerNotFound(logger_uid))
    } else {
        modification_result
    }
}

/// Removes a FMI2 logging callback function from the logger module.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// Returns an Err if no FMU logging layer with the given `logger_uid` is
/// found, or there is a problem with the structure of the global tracing
/// subscriber.
/// 
/// # Parameters
/// - `logger_uid`: The uid of the FMU logging layer containing the callback
///   to be removed.
pub fn remove_callback(logger_uid: u64) -> LoggerResult<()> {
    let mut layer_found = false;

    (*FMU_LOGGER_RELOAD_HANDLE)
        .as_ref()
        .map_err(|e| e.clone())?
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
        Err(LoggerError::LoggerLayerNotFound(logger_uid))
    } else {
        Ok(())
    }
}

/// Adds an FMU instance name to a FMU logging layer containing a callback.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// Returns an Err if no FMU logging layer with the given `logger_uid` is
/// found, or there is a problem with the structure of the global tracing
/// subscriber.
/// 
/// # Parameters
/// - `logger_uid`: The uid of the FMU logging layer to set the name for.
pub fn add_name_to_callback(
    logger_uid: u64,
    instance_name: &str
) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.set_instance_name(instance_name)
    )
}

pub fn allow_all_categories_for_callback(logger_uid: u64) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| {
            layer.category_filter = CategoryFilter::new_blacklist()
        }
    )
}

pub fn refuse_all_categories_for_callback(logger_uid: u64) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| {
            layer.category_filter = CategoryFilter::new_whitelist()
        }
    )
}

pub fn allow_categories_for_callback(
    logger_uid: u64,
    categories: Vec<Fmi2LogCategory>
) -> LoggerResult<()> {
    modify_callback_with_result(
        logger_uid,
        |layer| {
            let already_listed_categories: Vec<Fmi2LogCategory> = categories.into_iter()
                .filter_map(|category| {
                    if let Err(category) = layer.category_filter
                        .whitelist_category(category)
                    {
                        Some(category)
                    } else {
                        None
                    }
                })
                .collect();

            if already_listed_categories.is_empty() {
                Ok(())
            } else {
                Err(LoggerError::CategoriesAlreadyListed(already_listed_categories))
            }
        }
    )
}

pub fn refuse_categories_for_callback(
    logger_uid: u64,
    categories: Vec<Fmi2LogCategory>
) -> LoggerResult<()> {
    modify_callback_with_result(
        logger_uid,
        |layer| {
            let already_listed_categories: Vec<Fmi2LogCategory> = categories.into_iter()
                .filter_map(|category| {
                    if let Err(category) = layer.category_filter
                        .blacklist_category(category)
                    {
                        Some(category)
                    } else {
                        None
                    }
                })
                .collect();
            
            if already_listed_categories.is_empty() {
                Ok(())
            } else {
                Err(LoggerError::CategoriesAlreadyListed(already_listed_categories))
            }
        }
    )
}

/*
/// Sets the `enabled` parameter of a FMU logging layer containing a callback,
/// determining whether or not that layer should emit any logging through the
/// callback.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// Returns an Err if no FMU logging layer with the given `logger_uid` is
/// found, or there is a problem with the structure of the global tracing
/// subscriber.
/// 
/// # Parameters
/// - `logger_uid`: The uid of the FMU logging layer containing the callback.
/// - `enabled`: A boolean designating whether or not the layer should emit
///   any logging.
pub fn update_enabled_for_callback(
    logger_uid: u64,
    enabled: bool
) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.enabled = enabled
    )
}

/// Sets the logCategories for a FMU logging layer containing a callback.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// Returns an Err if no FMU logging layer with the given `logger_uid` is
/// found, or there is a problem with the structure of the global tracing
/// subscriber.
/// 
/// # Parameters
/// - `logger_uid`: The uid of the FMU logging layer to set categories for.
pub fn set_categories_for_callback(
    logger_uid: u64,
    mut categories: Vec<Fmi2LogCategory>
) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.set_categories(&mut categories)
    )
}

/// Sets the logCategories of a FMU logging layer containing a callback.
/// 
/// Calling this function also initializes the global tracing subscriber if
/// it hasn't already been initialized (see [initialize] for further
/// explanation).
/// 
/// Returns an Err if no FMU logging layer with the given `logger_uid` is
/// found, or there is a problem with the structure of the global tracing
/// subscriber.
/// 
/// # Parameters
/// - `logger_uid`: The uid of the FMU logging layer to clear the categories of.
pub fn clear_categories_for_callback(logger_uid: u64) -> LoggerResult<()> {
    modify_callback(
        logger_uid,
        |layer| layer.clear_categories()
    )
}
*/

pub type LoggerResult<T> = Result<T, LoggerError>;

#[derive(Clone, Debug)]
pub enum LoggerError {
    SubscriberAlreadySet,
    FmuLayerVectorMissing,
    MaxLoggersExceeded,
    LoggerLayerNotFound(u64),
    CategoriesAlreadyListed(Vec<Fmi2LogCategory>)
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::SubscriberAlreadySet => write!(
                f, "the global tracing subscriber was already set"
            ),
            LoggerError::FmuLayerVectorMissing => write!(
                f, "the vector containing the FMU logger layers is missing"
            ),
            LoggerError::MaxLoggersExceeded => write!(
                f, "the maximum number of FMU logger layers (2^64-1) was exceeded"
            ),
            LoggerError::LoggerLayerNotFound(luid) => write!(
                f, "no FMU logger layer with logger_uid {} was found", luid
            ),
            LoggerError::CategoriesAlreadyListed(categories) => {
                write!(
                    f,
                    "the categories {}were already listed in the logging filter",
                    categories.into_iter()
                        .map(|category| format!("{category}"))
                        .reduce(
                            |folded, next_string| {
                                format!("{folded}, {next_string}")
                            }
                        ).map_or(
                            String::from(""),
                            |categories_string| {
                                format!("[{categories_string}] ")
                            }
                        )
                )
            }
        }
    }
}

impl Error for LoggerError {}

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
        pub const ENABLE_FMT_LOGGER: bool = true;
    } else {
        pub const ENABLE_FMT_LOGGER: bool = false;
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

enum CategoryFilter {
    Blacklist(Vec<Fmi2LogCategory>),
    Whitelist(Vec<Fmi2LogCategory>)
}

impl CategoryFilter {
    pub fn new_blacklist() -> Self {
        CategoryFilter::Blacklist(Vec::<Fmi2LogCategory>::with_capacity(16))
    }

    pub fn new_whitelist() -> Self {
        CategoryFilter::Whitelist(Vec::<Fmi2LogCategory>::with_capacity(16))
    }

    pub fn blacklist_category(&mut self, category: Fmi2LogCategory) -> Result<(), Fmi2LogCategory> {
        if let CategoryFilter::Blacklist(categories) = self {
            if !categories.contains(&category) {
                categories.push(category);
                return Ok(()) 
            }
        }
        return Err(category)
    }

    pub fn whitelist_category(&mut self, category: Fmi2LogCategory) -> Result<(), Fmi2LogCategory> {
        if let CategoryFilter::Whitelist(categories) = self {
            if !categories.contains(&category) {
                categories.push(category);
                return Ok(())
            }
        }
        return Err(category)
    }

    pub fn allows(&self, category: &Fmi2LogCategory) -> bool {
        match self {
            CategoryFilter::Blacklist(categories) => {
                !categories.contains(category)
            }
            CategoryFilter::Whitelist(categories) => {
                categories.contains(category)
            }
        }
    }

    pub fn all_categories_refused(&self) -> bool {
        match self {
            CategoryFilter::Blacklist(_) => {
                false
            }
            CategoryFilter::Whitelist(categories) => {
                categories.is_empty()
            }
        }
    }
}

struct FmuLayer{
    logger_uid: u64,
    callback: Fmi2CallbackLogger,
    categories: Vec<Fmi2LogCategory>,
    component_environment: SyncComponentEnvironment,
    category_filter: CategoryFilter,
    instance_name_bytes: Option<Vec<u8>>
}

impl FmuLayer{
    pub fn new(
        logger_uid: u64,
        callback: Fmi2CallbackLogger,
        component_environment: SyncComponentEnvironment,
        enabled: bool
    ) -> Self {
        let category_filter = if enabled {
            CategoryFilter::new_blacklist()
        } else {
            CategoryFilter::new_whitelist()
        };

        Self {
            logger_uid,
            callback,
            categories: Vec::with_capacity(16), // There are 10 predefined logCategories, so a capacity of 16 will allow the user to implement a handful of their own without this having to reallocate for size
            component_environment,
            category_filter,
            instance_name_bytes: None
        }
    }

    /// Sets the instance name that is to be passed to the callback on logging
    /// any event.
    pub fn set_instance_name(&mut self, instance_name: &str) {
        let mut instance_name_bytes = String::from(instance_name).into_bytes();
        instance_name_bytes.push(0);
        self.instance_name_bytes = Some(instance_name_bytes);
    }

    /// Sets the logging categories that the FmuLayer will log events for.
    /// 
    /// If any categories are set, the FmuLayer will only emit events for
    /// which the value of the `category` field is equal to one of these set
    /// categories. (Events without an explicit `category` field defaults to
    /// `LogAll`).
    pub fn set_categories(&mut self, categories: &mut Vec<Fmi2LogCategory>) {
        self.clear_categories();
        self.categories.append(categories);
    }

    /// Clears the logging categories for the FmuLayer, indicating that any
    /// event no matter the presence or value of its `category` field will be
    /// logged (assuming that the event is meant for this FmuLayer and the
    /// FmuLayer is enabled).
    pub fn clear_categories(&mut self) {
        self.categories.clear();
    }
    
    fn logger_uid_in_attributes(&self, attrs: &span::Attributes<'_>) -> bool {
        let mut visitor = FmuSpanVisitor::new();
        attrs.values().record(&mut visitor);
        visitor.luid.is_some_and(|luid| luid == self.logger_uid)
    }

    fn interested_in_parent_of_span<S: Subscriber + for<'lookup> LookupSpan<'lookup>>(
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.parent()
            .is_some_and(|parent_span|
                self.interested_in_span(&parent_span)
            )
    }

    fn interested_in_span<S: Subscriber + for<'lookup> LookupSpan<'lookup>>(
        &self,
        span: &SpanRef<'_, S>
    ) -> bool {
        span.extensions()
            .get::<EnabledForLayer>()
            .is_some_and(|enabled_for_layer|
                enabled_for_layer.logger_uid == self.logger_uid
            )
    }

    fn interested_in_event<S: Subscriber + for<'lookup> LookupSpan<'lookup>>(
        &self,
        event: &Event<'_>,
        ctx: &Context<'_, S>
    ) -> bool {
        ctx.event_span(event)
            .is_some_and(|span| self.interested_in_span(&span))
    }

    /// The FmuLayer is interested in a category if it is contained in the
    /// FmuLayer's `categories` vector OR if the `categories` vector is empty.
    fn interested_in_category(&self, category: &Fmi2LogCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(category)
    }
}

impl <S: Subscriber + for<'lookup> LookupSpan<'lookup>> Layer<S> for FmuLayer{
    /// Notifies this layer that a new span was constructed with the given
    /// `Attributes` and `Id`.
    /// 
    /// If the FmuLayer would be interested in the contents of a span it will
    /// mark it, so that it will be recognised as interesting for the spans
    /// lifetime.
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: Context<'_, S>
    ) {
        if let Some(span) = ctx.span(id) {
            if self.interested_in_parent_of_span(&span)
            || self.logger_uid_in_attributes(attrs)
            {
                span.extensions_mut().insert(
                    EnabledForLayer{ logger_uid: self.logger_uid }
                );
            }
        }
    }

    fn on_event(
        &self,
        _event: &Event<'_>,
        _ctx: Context<'_, S>
    ) {
        if self.category_filter.all_categories_refused()
        || !self.interested_in_event(_event, &_ctx)
        {
            return
        }

        let mut visitor = FmuEventVisitor::new();
        _event.record(&mut visitor);

        let category = visitor.category.unwrap_or_default();

        if !(self.category_filter.allows(&category)) {
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

            let instance_name = CStr::from_bytes_until_nul(
                &instance_name_bytes
            )
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

impl Visit for FmuSpanVisitor{
    #[allow(unused_variables)]
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn Debug
    ) {
        // Disables debug (and all "record" methods not explicitly defined).
    }

    fn record_u64(
        &mut self,
        field: &Field,
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

impl Visit for FmuEventVisitor{
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn Debug
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

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "status" {
            if let Ok(status) = Fmi2Status::try_from(value as i32) {
                self.status = Some(status);
            }
        }
    }
}
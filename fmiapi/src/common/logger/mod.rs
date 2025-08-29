//! Contains the `Logger` trait, along with logging related submodules.

pub mod category_filter;
pub mod log_category;
pub mod log_status;

use category_filter::CategoryFilter;
use log_category::LogCategory;
use log_status::LogStatus;

use colored::Colorize;

/// A `Logger` can send log events with LogCategories and LogStatuses to the
/// importer, and to the environment if the module is compiled with the 
/// "fmt_logging" feature flag set.
pub trait Logger 
where 
    Self::Category: LogCategory,
    Self::Status: LogStatus
{
    type Category;
    type Status;

    fn log(
        &self,
        status: Self::Status,
        category: Self::Category,
        message: &str
    );

    fn filter(&mut self) -> &mut CategoryFilter<Self::Category>;

    /// Log a nominal event.
    fn ok(&self, message: &str) {
        self.log(Self::Status::ok(), Self::Category::ok(), message);
    }

    /// Log an unexpected but noncritical event.
    fn warning(&self, message: &str) {
        self.log(Self::Status::warning(), Self::Category::warning(), message);
    }

    /// Log a critical event related to a unmitigatable error in a single FMU.
    fn error(&self, message: &str) {
        self.log(Self::Status::error(), Self::Category::error(), message);
    }

    /// Log a fatal event with possible consequnces beyond the single
    /// FMU instance.
    fn fatal(&self, message: &str) {
        self.log(Self::Status::fatal(), Self::Category::fatal(), message);
    }

    /// Instruct the user of the distributed FMU to connect the remote part to
    /// the local part through the given port.
    fn communicate_port_connection_action(&self, port_str: &str) {
        self.log(
            Self::Status::ok(),
            Self::Category::unifmu_message(),
            &format!(
                "{} Connect remote backend to dispatcher through port {}",
                "ACTION REQUIRED ~".yellow().bold(),
                port_str.green().bold()
            )
        );
    }

    /// Enable the given categories so that any log event with any of those
    /// categories are emitted to the importer.
    fn enable_categories(
        &mut self,
        categories: Vec<Self::Category>
    ) {
        for category in categories {
            let _ = self.filter().enable_category(category);
        }
    }

    /// Disable the given categories so that any log event with any of those
    /// categories are NOT emitted to the importer.
    fn disable_categories(
        &mut self,
        categories: Vec<Self::Category>
    ) {
        for category in categories {
            let _ = self.filter().disable_category(category);
        }
    }

    /// Emit all log events regardless of their categories..
    fn enable_all_categories(&mut self) {
        *self.filter() = CategoryFilter::new_blacklist();
    }

    /// Supress all log events regardless of their categories.
    fn disable_all_categories(&mut self) {
        *self.filter() = CategoryFilter::new_blacklist();
    }

    /// Prints the message with a prefix based on the status to stderr if the
    /// api was build with the 'fmt_logging' feature.
    /// 
    /// If not build with the 'fmt_logging' feature, this function does nothing
    #[cfg(feature = "fmt_logging")]
    fn fmt_log(message: &str, status: &Self::Status) {
        if status.is_ok() {
            println!("{}", message);
        } else {
            eprintln!("{}{}", status.fmt_log_prefix(), message);
        }
    }

    /// Prints the message with a prefix based on the status to stderr if the
    /// api was build with the 'fmt_logging' feature.
    /// 
    /// If not build with the 'fmt_logging' feature, this function does nothing
    #[cfg(not(feature = "fmt_logging"))]
    fn fmt_log(_message: &str, _status: &Self::Status) {}
    
}
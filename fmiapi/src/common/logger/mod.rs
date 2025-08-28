pub mod category_filter;
pub mod log_category;
pub mod log_status;

use category_filter::CategoryFilter;
use log_category::LogCategory;
use log_status::LogStatus;

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

    fn ok(&self, message: &str) {
        self.log(Self::Status::ok(), Self::Category::ok(), message);
    }

    fn warning(&self, message: &str) {
        self.log(Self::Status::warning(), Self::Category::warning(), message);
    }

    fn error(&self, message: &str) {
        self.log(Self::Status::error(), Self::Category::error(), message);
    }

    fn fatal(&self, message: &str) {
        self.log(Self::Status::fatal(), Self::Category::fatal(), message);
    }

    fn enable_categories(
        &mut self,
        categories: Vec<Self::Category>
    ) {
        for category in categories {
            let _ = self.filter().enable_category(category);
        }
    }

    fn disable_categories(
        &mut self,
        categories: Vec<Self::Category>
    ) {
        for category in categories {
            let _ = self.filter().disable_category(category);
        }
    }

    fn enable_all_categories(&mut self) {
        *self.filter() = CategoryFilter::new_blacklist();
    }

    fn disable_all_categories(&mut self) {
        *self.filter() = CategoryFilter::new_blacklist();
    }

    /// Prints the message with a prefix based on the status to stderr if the
    /// api was build with the 'fmt_logging' feature.
    /// 
    /// If not build with the 'fmt_logging' feature, this function does nothing
    #[cfg(feature = "fmt_logging")]
    fn fmt_log(message: &str, status: &Self::Status) {
        eprintln!("{}{}", status.fmt_log_prefix(), message);
    }

    /// Prints the message with a prefix based on the status to stderr if the
    /// api was build with the 'fmt_logging' feature.
    /// 
    /// If not build with the 'fmt_logging' feature, this function does nothing
    #[cfg(not(feature = "fmt_logging"))]
    fn fmt_log(_message: &str, _status: &Self::Status) {}
    
}
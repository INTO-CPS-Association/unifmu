use crate::category_filter::{CategoryFilter, LogCategory};

pub trait Logger 
where Self::Category: LogCategory
{
    type Category;
    type Status;

    fn log(
        &self,
        status: Self::Status,
        category: Self::Category,
        message: &str
    );

    fn ok(&self, message: &str);

    fn warning(&self, message: &str);

    fn error(&self, message: &str);

    fn fatal(&self, message: &str);

    fn filter(&mut self) -> &mut CategoryFilter<Self::Category>;

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

    
    #[cfg(feature = "fmt_logging")]
    fn fmt_log(&self, message: &str) {
        println!("{message}");
    }

    #[allow(unused_variables)]
    #[cfg(not(feature = "fmt_logging"))]
    fn fmt_log(&self, message: &str) {}
    
}
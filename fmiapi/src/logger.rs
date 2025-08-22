pub trait Logger {
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

    
    #[cfg(feature = "fmt_logging")]
    fn fmt_log(&self, message: &str) {
        println!("{message}");
    }

    #[cfg(not(feature = "fmt_logging"))]
    fn fmt_log(&self, message: &str) {}
    
}
pub trait LogStatus {
    fn ok() -> Self;
    fn warning() -> Self;
    fn error() -> Self;
    fn fatal() -> Self;

    #[cfg(feature = "fmt_logging")]
    fn fmt_log_prefix(&self) -> String;

    #[cfg(feature = "fmt_logging")]
    fn is_ok(&self) -> bool;
}
pub trait LogStatus {
    #[allow(dead_code)]
    fn fmt_log_prefix(&self) -> String;

    fn ok() -> Self;
    fn warning() -> Self;
    fn error() -> Self;
    fn fatal() -> Self;
}
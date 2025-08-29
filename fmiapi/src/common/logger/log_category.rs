use std::{fmt::{Debug, Display}, hash::Hash};

/// Extended logging related functionality for FMIX logCategories.
pub trait LogCategory: for <'a> From<&'a str> + Default + Debug + Display + Eq + Hash {
    fn str_name(&self) -> &str;

    fn ok() -> Self;
    fn warning() -> Self;
    fn error() -> Self;
    fn fatal() -> Self;

    fn unifmu_message() -> Self;
}
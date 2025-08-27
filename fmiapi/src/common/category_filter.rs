use std::{collections::HashSet, fmt::{Debug, Display}, hash::Hash};

pub trait LogCategory: for <'a> From<&'a str> + Default + Debug + Display + Eq + Hash {
    fn str_name(&self) -> &str;
}

/// A filter for logging categories.
/// 
/// If the filter is the Blacklist variant, any category in the list is
/// refused.
/// 
/// If the filter is the Whitelist variant, any category in the list is
/// allowed.
pub enum CategoryFilter<T> 
where
    T: LogCategory
{
    Blacklist(HashSet<T>),
    Whitelist(HashSet<T>)
}

// TODO: Make once initialized static based on read of XML instead.
/// There are 10 predefined logCategories, so a capacity of 16 will allow the
/// user to implement a handful of their own without this having to reallocate
/// for size.
const LIST_CAPACITY: usize = 16;

impl<T: LogCategory> CategoryFilter<T> {
    /// Create a new Blacklist variant of the CategoryFilter.
    pub fn new_blacklist() -> Self {
        CategoryFilter::Blacklist(
            HashSet::<T>::with_capacity(LIST_CAPACITY)
        )
    }

    /// Create a new Whitelist variant of the CategoryFilter.
    pub fn new_whitelist() -> Self {
        CategoryFilter::Whitelist(
            HashSet::<T>::with_capacity(LIST_CAPACITY)
        )
    }

    pub fn enable_category(
        &mut self,
        category: T
    ) -> Result<(), T> {
        match self {
            CategoryFilter::Blacklist(categories) => {
                if categories.contains(&category) {
                    categories.remove(&category);
                    return Ok(()) 
                }
            }
            CategoryFilter::Whitelist(categories) => {
                if !categories.contains(&category) {
                    categories.insert(category);
                    return Ok(())
                }
            }
        }
        Err(category)
    }

    pub fn disable_category(
        &mut self,
        category: T
    ) -> Result<(), T> {
        match self {
            CategoryFilter::Blacklist(categories) => {
                if !categories.contains(&category) {
                    categories.insert(category);
                    return Ok(()) 
                }
            }
            CategoryFilter::Whitelist(categories) => {
                if categories.contains(&category) {
                    categories.remove(&category);
                    return Ok(())
                }
            }
        }
        Err(category)
    }

    /// Is the given category enabled by the filter.
    pub fn enabled(&self, category: &T) -> bool {
        match self {
            CategoryFilter::Blacklist(categories) => {
                !categories.contains(category)
            }
            CategoryFilter::Whitelist(categories) => {
                categories.contains(category)
            }
        }
    }
}
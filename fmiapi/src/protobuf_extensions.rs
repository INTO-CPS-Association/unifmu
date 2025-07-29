/// A protobuf return message wrapped as a oneof in another super message.
pub trait ExpectableReturn<O> {
    /// Extracts the return message if the given message enum variant wraps it.
    fn extract_from(return_variant: O) -> Option<Self> where Self: Sized;
}

/// Implements the [ExpectableReturn] trait for the type given as the first
/// parameter.
/// 
/// The second and third parameter should respectively contain the identities
/// of the type and variant of the enum that wraps the type given as the
/// first parameter. (Make sure that the enum type is in scope and has the
/// exact identity given.)
#[macro_export]
macro_rules! implement_expectable_return {
    ($return_type:ty, $enum_type:ident, $enum_variant:ident) => {
        impl ExpectableReturn<$enum_type> for $return_type {
            fn extract_from(return_variant: $enum_type) -> Option<Self> where Self: Sized {
                match return_variant {
                    $enum_type::$enum_variant(message) => Some(message),
                    _ => None,
                }
            }
        }
    };
}
pub(crate) use implement_expectable_return;
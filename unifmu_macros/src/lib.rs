// ------------------------------ Test Macros ---------------------------------

#[proc_macro_attribute]
pub fn for_each_fmu(
    _attr: ::proc_macro::TokenStream,
    item: ::proc_macro::TokenStream
) -> ::proc_macro::TokenStream {
    item
}
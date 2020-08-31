mod method_info;
mod type_info;
mod utils;

extern crate api_doc;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(TypeInfo)]
pub fn type_info(input: TokenStream) -> TokenStream {
    crate::type_info::impl_type_info(input)
}

#[proc_macro_attribute]
pub fn method_info(attr: TokenStream, input: TokenStream) -> TokenStream {
    crate::method_info::impl_method_info(attr, input)
}

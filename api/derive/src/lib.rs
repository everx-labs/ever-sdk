mod api_function;
mod api_type;
mod utils;
mod api_module;

extern crate api_info;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(ApiType)]
pub fn api_type(input: TokenStream) -> TokenStream {
    crate::api_type::impl_api_type(input)
}

#[proc_macro_derive(ApiModule, attributes(api_module))]
pub fn api_module(input: TokenStream) -> TokenStream {
    crate::api_module::impl_api_module(input)
}

#[proc_macro_attribute]
pub fn api_function(attr: TokenStream, input: TokenStream) -> TokenStream {
    crate::api_function::impl_api_function(attr, input)
}

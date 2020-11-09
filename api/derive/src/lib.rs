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

#[macro_use]
extern crate quote;

#[proc_macro]
pub fn include_build_info(_input: TokenStream) -> TokenStream {
    let content = match std::fs::read_to_string(std::env::current_dir().unwrap().join("ton_client/client/src/build_info.json")) {
        Err(_e) => return quote!("").into(),
        Ok(content) => content,
    };
    return quote!(#content).into();
}

mod derive_function;
mod derive_module;
mod derive_type;
mod utils;

extern crate api_info;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(ApiType)]
pub fn api_type(input: TokenStream) -> TokenStream {
    crate::derive_type::impl_api_type(input)
}

#[proc_macro_derive(ApiModule, attributes(api_module))]
pub fn api_module(input: TokenStream) -> TokenStream {
    crate::derive_module::impl_api_module(input)
}

#[proc_macro_attribute]
pub fn api_function(attr: TokenStream, input: TokenStream) -> TokenStream {
    crate::derive_function::impl_api_function(attr, input)
}

#[proc_macro_derive(ZeroizeOnDrop)]
pub fn zeroize_on_drop(input: TokenStream) -> TokenStream {
    crate::derive_type::impl_zeroize_on_drop(input)
}

#[macro_use]
extern crate quote;

#[proc_macro]
pub fn include_build_info(_input: TokenStream) -> TokenStream {
    let content = match std::fs::read_to_string(
        std::env::current_dir()
            .unwrap()
            .join("ton_client/src/build_info.json"),
    ) {
        Err(_e) => return quote!("").into(),
        Ok(content) => content,
    };
    return quote!(#content).into();
}

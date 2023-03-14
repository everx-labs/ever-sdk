use proc_macro::TokenStream;
use crate::utils::{module_from, module_to_tokens};
use quote::quote;
use syn::DeriveInput;

pub fn impl_api_module(input: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(input).expect("Derive input");
    let module = module_from(Some(&input.ident), &input.attrs);
    let type_ident = &input.ident;
    let module_tokens = module_to_tokens(&module);
    let tokens = quote! {
        impl api_info::ApiModule for #type_ident {
            fn api() -> api_info::Module {
                #module_tokens
            }
        }
    };
    tokens.into()
}

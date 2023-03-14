use proc_macro::{Span, TokenStream};
use crate::utils::{doc_from, field_from, function_to_tokens, get_value_of, type_from};
use api_info;
use quote::quote;
use syn::{FnArg, Ident, Item, ItemFn, Meta, Pat, ReturnType};

pub fn impl_api_function(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let syn_func = match syn::parse::<Item>(item.clone()).unwrap() {
        Item::Fn(ref f) => f.clone(),
        _ => panic!("api_function can only be applied to functions"),
    };
    let name = syn_func.sig.ident.to_string();
    let syn_meta = syn::parse::<Meta>(attr).ok();
    let fn_impl_tokens = function_to_tokens(&function_from(syn_meta, syn_func));

    let fn_ident = Ident::new(&format!("{}_api", name), Span::call_site().into());
    let fn_tokens = quote! {
        pub fn #fn_ident () -> api_info::Function {
            #fn_impl_tokens
        }
    };
    let fn_tokens: TokenStream = fn_tokens.into();
    let mut output = TokenStream::new();
    output.extend(item);
    output.extend(fn_tokens);
    output
}

fn function_from(meta: Option<Meta>, func: ItemFn) -> api_info::Function {
    let name = func.sig.ident.to_string();
    let api_name = match meta {
        Some(Meta::NameValue(nv)) => get_value_of("name", &nv).unwrap_or(name.clone()),
        _ => name.clone(),
    };
    let (summary, description) = doc_from(&func.attrs);
    let params = func.sig.inputs.iter().map(field_from_fn_arg).collect();
    let result = type_from_return_type(&func.sig.output);
    api_info::Function {
        name: api_name,
        params,
        result,
        errors: None,
        summary,
        description,
    }
}

fn field_from_fn_arg(a: &FnArg) -> api_info::Field {
    if let FnArg::Typed(ref a) = a {
        if let Pat::Ident(i) = a.pat.as_ref() {
            return field_from(Some(&i.ident), &a.attrs, type_from(&a.ty));
        }
    }
    panic!("Function can't be struct member");
}

fn type_from_return_type(return_type: &ReturnType) -> api_info::Type {
    match return_type {
        ReturnType::Type(_, ref ty) => type_from(ty),
        _ => api_info::Type::None {},
    }
}

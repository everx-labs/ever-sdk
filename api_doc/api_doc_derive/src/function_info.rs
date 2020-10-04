use crate::utils::{doc_from, field_from, function_to_tokens, get_value_of, type_from};
use api_doc::api;
use quote::__private::{Ident, Span};
use quote::quote;
use syn::{FnArg, Item, ItemFn, Meta, Pat, ReturnType};

pub fn impl_function_info(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let syn_func = match syn::parse::<Item>(item.clone()).unwrap() {
        Item::Fn(ref f) => f.clone(),
        _ => panic!("function_info can only be applied to functions"),
    };
    let name = syn_func.sig.ident.to_string();
    let function_info_fn = Ident::new(&format!("{}_info", name), Span::call_site());
    let syn_meta = syn::parse::<Meta>(attr).ok();
    let function_tokens = function_to_tokens(&function_from(syn_meta, syn_func));

    let function_fn = quote! {
        pub fn #function_info_fn () -> api_doc::api::Function {
            #function_tokens
        }
    };
    let function_fn: proc_macro::TokenStream = function_fn.into();
    let mut output = proc_macro::TokenStream::new();
    output.extend(item);
    output.extend(function_fn);
    output
}

fn function_from(meta: Option<Meta>, func: ItemFn) -> api::Function {
    let name = func.sig.ident.to_string();
    let api_name = match meta {
        Some(Meta::NameValue(nv)) => get_value_of("name", &nv).unwrap_or(name.clone()),
        _ => name.clone(),
    };
    let (summary, description) = doc_from(&func.attrs);
    let params = func.sig.inputs.iter().map(field_from_fn_arg).collect();
    let result = type_from_return_type(&func.sig.output);
    api::Function {
        name: api_name,
        params,
        result,
        errors: None,
        summary,
        description,
    }
}

fn field_from_fn_arg(a: &FnArg) -> api::Field {
    if let FnArg::Typed(ref a) = a {
        if let Pat::Ident(i) = a.pat.as_ref() {
            return field_from(Some(&i.ident), &a.attrs, type_from(&a.ty));
        }
    }
    panic!("Function can't be struct member");
}

fn type_from_return_type(return_type: &ReturnType) -> api::Type {
    match return_type {
        ReturnType::Type(_, ref ty) => type_from(ty),
        _ => api::Type::None {},
    }
}

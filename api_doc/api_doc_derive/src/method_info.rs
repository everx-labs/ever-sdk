use quote::{quote};
use syn::{Item, ItemFn, Meta, ReturnType, FnArg, Pat};
use quote::__private::{Ident, Span};
use api_doc::api;
use crate::utils::{method_to_tokens, doc_from, field_from, type_from, get_value_of};

pub fn impl_method_info(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn_meta = syn::parse::<Meta>(attr).unwrap();
    let syn_func = match syn::parse::<Item>(item.clone()).unwrap() {
        Item::Fn(ref f) => f.clone(),
        _ => panic!("method_info can only be applied to functions"),
    };

    let name = syn_func.sig.ident.to_string();
    let method_info_fn = Ident::new(&format!("{}_method", name), Span::call_site());
    let method_tokens = method_to_tokens(&method_from(&syn_meta, &syn_func));

    let method_fn = quote! {
        fn #method_info_fn () -> Method {
            #method_tokens
        }
    };
    let method_fn: proc_macro::TokenStream = method_fn.into();
    let mut output = proc_macro::TokenStream::new();
    output.extend(item);
    output.extend(method_fn);
    output
}

fn method_from(meta: &Meta, func: &ItemFn) -> api::Method {
    let name = func.sig.ident.to_string();
    let api_name = match meta {
        Meta::NameValue(nv) => {
            get_value_of("name", nv).unwrap_or(name.clone())
        },
        _ => name.clone(),
    };
    let (summary, description) = doc_from(&func.attrs);
    let params = func.sig.inputs.iter().map(field_from_fn_arg).collect();
    let result = type_from_return_type(&func.sig.output);

    api::Method {
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
        _ => api::Type::None {}
    }
}



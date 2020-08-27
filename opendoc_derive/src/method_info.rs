use quote::{quote};

pub fn impl_method_info(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO: implementation doesn't finished yet
    let method_fn = quote! {
        fn baz_method() -> Method {
            Method {
                name: "module.baz".into(),
                params: vec![],
                result: Type::None,
                errors: None,
                summary: None,
                description: None,
            }
        }
    };
    let method_fn: proc_macro::TokenStream = method_fn.into();
    let mut output = proc_macro::TokenStream::new();
    output.extend(input);
    output.extend(method_fn);
    output
}

use quote::{quote};
use syn::{DeriveInput, Data};
use api_doc::api;
use crate::utils::{field_from, type_from, field_to_tokens};

pub fn impl_type_info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn_type = syn::parse::<DeriveInput>(input).unwrap();
    let fields = match syn_type.data {
        Data::Struct(ref s) => &s.fields,
        _ => panic!("TypeInfo can only be derived for structures"),
    };
    let fields = fields.iter().map(|f| {
        field_from(f.ident.as_ref().unwrap(), &f.attrs, type_from(&f.ty))
    });
    let type_info = field_from(&syn_type.ident, &syn_type.attrs, api::Type::Struct(fields.collect()));
    let type_name = syn_type.ident;
    let type_info_tokens = field_to_tokens(&type_info);
    let gen = quote! {
        impl api_doc::reflect::TypeInfo for #type_name {
            fn type_info() -> api_doc::api::Field {
                #type_info_tokens
            }
        }
    };
    gen.into()
}

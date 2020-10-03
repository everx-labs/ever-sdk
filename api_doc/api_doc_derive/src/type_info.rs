use crate::utils::{doc_from, field_from, field_to_tokens, find_attr_value, type_from};
use api_doc::api;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Expr, Fields, Lit, Variant};

pub fn impl_type_info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse::<DeriveInput>(input).expect("Derive input");
    let ty = match input.data {
        Data::Struct(ref data) => api::Type::Struct(fields_from(&data.fields)),
        Data::Enum(ref data) => enum_type(data),
        _ => panic!("TypeInfo can only be derived for structures"),
    };
    let mut info = field_from(Some(&input.ident), &input.attrs, ty);
    if let Some(name) = find_attr_value("type_info", "name", &input.attrs) {
        info.name = name;
    }
    let type_name = &input.ident;
    let type_info_tokens = field_to_tokens(&info);
    let gen = quote! {
        impl api_doc::reflect::TypeInfo for #type_name {
            fn type_info() -> api_doc::api::Field {
                #type_info_tokens
            }
        }
    };
    gen.into()
}

fn enum_type(data: &DataEnum) -> api::Type {
    if data
        .variants
        .iter()
        .find(|v| !v.fields.is_empty())
        .is_some()
    {
        enum_of_types(data)
    } else {
        enum_of_consts(data)
    }
}

fn enum_of_types(data: &DataEnum) -> api::Type {
    let types = data.variants.iter().map(|v| {
        let fields = fields_from(&v.fields);
        field_from(Some(&v.ident), &v.attrs, api::Type::Struct(fields))
    });
    api::Type::EnumOfTypes(types.collect())
}

fn enum_of_consts(data: &DataEnum) -> api::Type {
    let consts = data.variants.iter().map(|v| const_from(v));
    api::Type::EnumOfConsts(consts.collect())
}

fn const_from(v: &Variant) -> api::Const {
    let name = v.ident.to_string();
    let (summary, description) = doc_from(&v.attrs);
    let value = match v.discriminant.as_ref().map(|(_, e)| e) {
        Some(expr) => {
            let lit = match expr {
                Expr::Lit(expr_lit) => &expr_lit.lit,
                _ => panic!("Invalid enum const."),
            };
            value_from_lit(lit)
        }
        None => api::ConstValue::None {},
    };
    api::Const {
        name,
        value,
        summary,
        description,
    }
}

fn value_from_lit(lit: &Lit) -> api::ConstValue {
    match lit {
        Lit::Bool(v) => api::ConstValue::Bool(if v.value { "true" } else { "false" }.into()),
        Lit::Str(v) => api::ConstValue::String(v.value()),
        Lit::Byte(v) => api::ConstValue::Number(v.value().to_string()),
        Lit::Int(v) => api::ConstValue::Number(v.base10_digits().into()),
        Lit::Float(v) => api::ConstValue::Number(v.base10_digits().into()),
        _ => panic!("Invalid enum const."),
    }
}

fn fields_from(fields: &Fields) -> Vec<api::Field> {
    fields
        .iter()
        .map(|f| field_from(f.ident.as_ref(), &f.attrs, type_from(&f.ty)))
        .collect()
}

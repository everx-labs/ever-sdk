extern crate opendoc;
extern crate proc_macro;

//use proc_macro::{TokenStream};

use quote::{quote};
use syn::{Type, Path, Meta, Lit, DeriveInput, Data, PathArguments, GenericArgument, AngleBracketedGenericArguments, Attribute, NestedMeta, MetaNameValue};
use opendoc::api;
use quote::__private::TokenStream;

#[proc_macro_derive(TypeInfo)]
pub fn reflect_type_info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn_type = syn::parse::<DeriveInput>(input).unwrap();
    let fields = match syn_type.data {
        Data::Struct(ref s) => &s.fields,
        _ => panic!("TypeInfo can only be derived for structures"),
    };
    let fields = fields.iter().map(|f| {
        parse_field(f.ident.as_ref().unwrap(), &f.attrs, get_type(&f.ty))
    });
    let type_info = parse_field(&syn_type.ident, &syn_type.attrs, api::Type::Struct(fields.collect()));
    let type_name = syn_type.ident;
    let type_info_tokens = field_to_tokens(&type_info);
    let gen = quote! {
        impl opendoc::reflect::TypeInfo for #type_name {
            fn type_info() -> opendoc::api::Field {
                #type_info_tokens
            }
        }
    };
    gen.into()
}

fn parse_field(name: &syn::Ident, attrs: &Vec<Attribute>, value: api::Type) -> api::Field {
    let (summary, description) = get_doc(attrs);
    api::Field {
        name: name.to_string(),
        summary,
        description,
        value,
    }
}

fn field_to_tokens(f: &api::Field) -> TokenStream {
    let name = &f.name;
    let value = api_type_to_tokens(&f.value);
    let summary = match &f.summary {
        Some(s) => quote! { Some(#s.into()) },
        None => quote! { None }
    };
    let description = match &f.description {
        Some(s) => quote! { Some(#s.into()) },
        None => quote! { None }
    };
    quote! {
        opendoc::api::Field {
            name: #name.into(),
            summary: #summary,
            description: #description,
            value: #value,
        }
    }
}

fn api_type_to_tokens(t: &api::Type) -> TokenStream {
    match t {
        api::Type::None => quote! { opendoc::api::Type::None },
        api::Type::Any => quote! { opendoc::api::Type::Any },
        api::Type::Boolean => quote! { opendoc::api::Type::Boolean },
        api::Type::Number => quote! { opendoc::api::Type::Number },
        api::Type::String => quote! { opendoc::api::Type::String },
        api::Type::Ref(type_name) => {
            quote! { opendoc::api::Type::Ref(#type_name.into()) }
        }
        api::Type::Optional(inner) => {
            let inner_type = api_type_to_tokens(inner);
            quote! { opendoc::api::Type::Optional(#inner_type.into()) }
        }
        api::Type::Array(items) => {
            let items_type = api_type_to_tokens(items);
            quote! { opendoc::api::Type::Array(#items_type.into()) }
        }
        api::Type::Struct(fields) => {
            let field_types = fields.iter().map(|x| field_to_tokens(x));
            quote! { opendoc::api::Type::Struct([#(#field_types),*].into()) }
        }
    }
}

fn get_type(ty: &Type) -> api::Type {
    match ty {
        Type::Array(_a) => panic!("array is unsupported"),
        Type::BareFn(_f) => panic!("function is unsupported"),
        Type::Group(_g) => panic!("group is unsupported"),
        Type::ImplTrait(_t) => panic!("impl_trait is unsupported"),
        Type::Infer(_t) => panic!("infer is unsupported"),
        Type::Macro(_t) => panic!("macro is unsupported"),
        Type::Never(_n) => panic!("never is unsupported"),
        Type::Paren(_p) => panic!("paren is unsupported"),
        Type::Path(p) => {
            get_path_type(&p.path)
        }
        Type::Ptr(_p) => panic!("ptr is unsupported"),
        Type::Reference(_r) => panic!("reference is unsupported"),
        Type::Slice(_s) => panic!("slice is unsupported"),
        Type::TraitObject(_t) => panic!("trait_object is unsupported"),
        Type::Tuple(_t) => panic!("tuple is unsupported"),
        Type::Verbatim(_t) => panic!("verbatim is unsupported"),
        _ => panic!("Unsupported type")
    }
}

fn get_path_type(path: &Path) -> api::Type {
    if let Some(segment) = path.segments.last() {
        let name = unqualified_type_name(segment.ident.to_string());
        if let Some(result) = match &segment.arguments {
            PathArguments::None =>
                Some(resolve_type_name(name)),
            PathArguments::AngleBracketed(args) =>
                get_generic_type(name, &args),
            _ => None
        } {
            return result;
        }
    }
    panic!(format!("Unsupported type {:?}",
        path.segments.last().map(|x| x.ident.to_string()).unwrap_or(String::new())
    ))
}

fn resolve_type_name(name: String) -> api::Type {
    match name.as_ref() {
        "String" => api::Type::String,
        "bool" => api::Type::Boolean,
        "u8" | "u16" | "u32" => api::Type::Number,
        _ => api::Type::Ref(name),
    }
}

fn get_generic_type(name: String, args: &AngleBracketedGenericArguments) -> Option<api::Type> {
    let get_inner_type = || match (args.args.len(), args.args.first()) {
        (1, Some(GenericArgument::Type(t))) => Some(get_type(t)),
        _ => None
    };
    match name.as_ref() {
        "Option" => get_inner_type().map(|x| api::Type::Optional(x.into())),
        "Vec" => get_inner_type().map(|x| api::Type::Array(x.into())),
        _ => None
    }
}

fn get_doc(attrs: &Vec<Attribute>) -> (Option<String>, Option<String>) {
    let mut summary = String::new();
    let mut description = String::new();

    fn try_add(doc: &mut String, s: &str) {
        if !doc.is_empty() {
            doc.push_str("\n");
        }
        doc.push_str(s);
    }

    for attr in attrs.iter() {
        match parse_doc(&attr) {
            Some(("doc", text)) => try_add(&mut description, &text),
            Some(("summary", text)) => try_add(&mut summary, &text),
            _ => ()
        }
    }

    if summary.is_empty() && !description.is_empty() {
        if let Some(line) = description.lines().next() {
            summary.push_str(line);
        }
    }
    fn non_empty(s: String) -> Option<String> {
        if s.is_empty() { None } else { Some(s) }
    }
    (non_empty(summary), non_empty(description))
}

fn parse_doc(attr: &Attribute) -> Option<(&'static str, String)> {
    match attr.parse_meta() {
        Ok(Meta::NameValue(ref meta)) => {
            return meta_value("doc", meta)
        },
        Ok(Meta::List(ref list)) => {
            if path_is(&list.path, "doc") {
                if let Some(NestedMeta::Meta(Meta::NameValue(meta))) = list.nested.first() {
                    return meta_value("summary", &meta);
                }
            }
        },
        _ => ()
    };
    None
}

fn meta_value(name: &'static str, meta: &MetaNameValue) -> Option<(&'static str, String)> {
    if path_is(&meta.path, name) {
        if let Lit::Str(lit) = &meta.lit {
            return Some((name, lit.value()));
        }
    }
    None
}

fn unqualified_type_name(qualified_name: String) -> String {
    match qualified_name.rfind("::") {
        Some(pos) => qualified_name[(pos + 2)..].into(),
        None => qualified_name
    }
}

fn path_is(path: &Path, expected: &str) -> bool {
    if let Some(ident) = path.get_ident() {
        ident.to_string() == expected
    } else {
        false
    }
}

use quote::{quote};
use syn::{
    Type, Path, Meta, Lit, PathArguments, GenericArgument, AngleBracketedGenericArguments,
    Attribute, NestedMeta, MetaNameValue, TypeArray,
};
use api_doc::api;
use quote::__private::TokenStream;


pub(crate) fn field_from(name: Option<&syn::Ident>, attrs: &Vec<Attribute>, value: api::Type) -> api::Field {
    let (summary, description) = doc_from(attrs);
    api::Field {
        name: name.map(|x|x.to_string()).unwrap_or("".into()),
        summary,
        description,
        value,
    }
}

pub(crate) fn doc_to_tokens(
    summary: &Option<String>,
    description: &Option<String>
) -> (TokenStream, TokenStream) {
    let summary = match summary {
        Some(s) => quote! { Some(#s.into()) },
        None => quote! { None }
    };
    let description = match description {
        Some(s) => quote! { Some(#s.into()) },
        None => quote! { None }
    };
    (summary, description)
}

pub(crate) fn field_to_tokens(f: &api::Field) -> TokenStream {
    let name = &f.name;
    let value = type_to_tokens(&f.value);
    let (summary, description) = doc_to_tokens(&f.summary, &f.description);
    quote! {
        api_doc::api::Field {
            name: #name.into(),
            summary: #summary,
            description: #description,
            value: #value,
        }
    }
}

pub(crate) fn const_value_to_tokens(v: &api::ConstValue) -> TokenStream {
    match v {
        api::ConstValue::None => quote! { api_doc::api::ConstValue::None },
        api::ConstValue::Bool(repr) => quote! { api_doc::api::ConstValue::Bool(#repr.into()) },
        api::ConstValue::String(repr) => quote! { api_doc::api::ConstValue::String(#repr.into()) },
        api::ConstValue::Number(repr) => quote! { api_doc::api::ConstValue::Number(#repr.into()) },
    }
}

pub(crate) fn const_to_tokens(c: &api::Const) -> TokenStream {
    let name = &c.name;
    let value = const_value_to_tokens(&c.value);
    let (summary, description) = doc_to_tokens(&c.summary, &c.description);
    quote! {
        api_doc::api::Const {
            name: #name.into(),
            summary: #summary,
            description: #description,
            value: #value,
        }
    }
}

pub(crate) fn method_to_tokens(m: &api::Method) -> TokenStream {
    let name = &m.name;
    let params = m.params.iter().map(|x|field_to_tokens(x));
    let result = type_to_tokens(&m.result);
    let (summary, description) = doc_to_tokens(&m.summary, &m.description);
    quote! {
        api_doc::api::Method {
            name: #name.into(),
            summary: #summary,
            description: #description,
            params: [#(#params), *].into(),
            result: #result,
            errors: None,
        }
    }
}

fn type_to_tokens(t: &api::Type) -> TokenStream {
    match t {
        api::Type::None => quote! { api_doc::api::Type::None },
        api::Type::Any => quote! { api_doc::api::Type::Any },
        api::Type::Boolean => quote! { api_doc::api::Type::Boolean },
        api::Type::Number => quote! { api_doc::api::Type::Number },
        api::Type::BigInt => quote! { api_doc::api::Type::BigInt },
        api::Type::String => quote! { api_doc::api::Type::String },
        api::Type::Ref(type_name) => {
            quote! { api_doc::api::Type::Ref(#type_name.into()) }
        }
        api::Type::Optional(inner) => {
            let inner_type = type_to_tokens(inner);
            quote! { api_doc::api::Type::Optional(#inner_type.into()) }
        }
        api::Type::Array(items) => {
            let items_type = type_to_tokens(items);
            quote! { api_doc::api::Type::Array(#items_type.into()) }
        }
        api::Type::Struct(fields) => {
            let field_types = fields.iter().map(|x| field_to_tokens(x));
            quote! { api_doc::api::Type::Struct([#(#field_types),*].into()) }
        }
        api::Type::EnumOfConsts(consts) => {
            let consts = consts.iter().map(|x| const_to_tokens(x));
            quote! { api_doc::api::Type::EnumOfConsts([#(#consts),*].into()) }
        }
        api::Type::EnumOfTypes(types) => {
            let types = types.iter().map(|x| field_to_tokens(x));
            quote! { api_doc::api::Type::EnumOfTypes([#(#types),*].into()) }
        }
    }
}

pub(crate) fn type_from(ty: &Type) -> api::Type {
    match ty {
        Type::Array(a) => array_type_from(a),
        Type::BareFn(_f) => panic!("function is unsupported"),
        Type::Group(_g) => panic!("group is unsupported"),
        Type::ImplTrait(_t) => panic!("impl_trait is unsupported"),
        Type::Infer(_t) => panic!("infer is unsupported"),
        Type::Macro(_t) => panic!("macro is unsupported"),
        Type::Never(_n) => panic!("never is unsupported"),
        Type::Paren(_p) => panic!("paren is unsupported"),
        Type::Path(p) => {
            type_from_path(&p.path)
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

fn array_type_from(ty: &TypeArray) -> api::Type {
    api::Type::Array(Box::new(type_from(ty.elem.as_ref())))
}

fn type_from_path(path: &Path) -> api::Type {
    if let Some(segment) = path.segments.last() {
        let name = unqualified_type_name(segment.ident.to_string());
        if let Some(result) = match &segment.arguments {
            PathArguments::None =>
                Some(resolve_type_name(name)),
            PathArguments::AngleBracketed(args) =>
                generic_type_from(name, &args),
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
        "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "usize" => api::Type::Number,
        "u64" | "i64" | "u128" | "i128" => api::Type::BigInt,
        _ => api::Type::Ref(name),
    }
}

fn generic_type_from(name: String, args: &AngleBracketedGenericArguments) -> Option<api::Type> {
    let get_inner_type = || match (args.args.len(), args.args.first()) {
        (1, Some(GenericArgument::Type(t))) => Some(type_from(t)),
        _ => None
    };
    match name.as_ref() {
        "Option" => get_inner_type().map(|x| api::Type::Optional(x.into())),
        "Vec" => get_inner_type().map(|x| api::Type::Array(x.into())),
        _ => None
    }
}

pub(crate) fn doc_from(attrs: &Vec<Attribute>) -> (Option<String>, Option<String>) {
    let mut summary = String::new();
    let mut description = String::new();

    fn try_add(doc: &mut String, s: &str) {
        if !doc.is_empty() {
            doc.push_str("\n");
        }
        doc.push_str(s);
    }

    for attr in attrs.iter() {
        match DocAttr::from(&attr) {
            DocAttr::Doc(text) => try_add(&mut description, &text),
            DocAttr::Summary(text) => try_add(&mut summary, &text),
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

enum DocAttr {
    None,
    Summary(String),
    Doc(String),
}

impl DocAttr {
    fn from(attr: &Attribute) -> DocAttr {
        match attr.parse_meta() {
            Ok(Meta::NameValue(ref meta)) => {
                return get_value_of("doc", meta).map(|x| DocAttr::Doc(x)).unwrap_or(DocAttr::None);
            }
            Ok(Meta::List(ref list)) => {
                if path_is(&list.path, "doc") {
                    if let Some(NestedMeta::Meta(Meta::NameValue(meta))) = list.nested.first() {
                        return get_value_of("summary", &meta).map(|x| DocAttr::Summary(x)).unwrap_or(DocAttr::None);
                    }
                }
            }
            _ => ()
        };
        DocAttr::None
    }
}

pub(crate) fn get_value_of(name: &'static str, meta: &MetaNameValue) -> Option<String> {
    if path_is(&meta.path, name) {
        if let Lit::Str(lit) = &meta.lit {
            return Some(lit.value());
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

pub(crate) fn path_is(path: &Path, expected: &str) -> bool {
    if let Some(ident) = path.get_ident() {
        ident.to_string() == expected
    } else {
        false
    }
}

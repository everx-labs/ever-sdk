extern crate serde_derive;

use serde_derive::{Serialize, Deserialize};
use crate::reflect::TypeInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct API {
    pub version: String,
    pub methods: Vec<Method>,
    pub types: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub params: Vec<Field>,
    pub result: Type,
    pub errors: Option<Vec<Error>>,
}

impl Method {
    pub fn from_types<P, R>(name: &str) -> Method where P: TypeInfo, R: TypeInfo {
        let p = P::type_info();
        let r = R::type_info();
        Method {
            name: name.into(),
            summary: p.summary.or(r.summary),
            description: p.description.or(r.description),
            params: if let Type::Struct(fields) = p.value {
                fields
            } else {
                vec![Field {
                    name: "param".into(),
                    summary: None,
                    description: None,
                    value: p.value,
                }]
            },
            result: r.value,
            errors: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    None,
    Any,
    Boolean,
    String,
    Number,
    BigInt,
    Ref(String),
    Optional(Box<Type>),
    Array(Box<Type>),
    Struct(Vec<Field>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Type,
}



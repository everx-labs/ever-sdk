extern crate serde_derive;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    #[serde(flatten)]
    pub value: Type,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConstValue {
    None {},
    Bool(String),
    String(String),
    Number(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Const {
    pub name: String,
    pub value: ConstValue,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    None {},
    Any {},
    Boolean {},
    String {},
    Number {},
    BigInt {},
    Ref(String),
    Optional(Box<Type>),
    Array(Box<Type>),
    Struct(Vec<Field>),
    EnumOfConsts(Vec<Const>),
    EnumOfTypes(Vec<Field>),
    Generic { name: String, args: Vec<Type> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Type,
}

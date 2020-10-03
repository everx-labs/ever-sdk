extern crate serde_derive;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct API {
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub types: Vec<Field>,
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub params: Vec<Field>,
    pub result: Type,
    pub errors: Option<Vec<Error>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
pub struct Const {
    pub name: String,
    pub value: ConstValue,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Type,
}

extern crate serde_derive;

use crate::reflect::TypeInfo;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct API {
    pub version: String,
    pub modules: Vec<Module>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Module {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub types: Vec<Field>,
    pub functions: Vec<Function>,
}

impl Module {
    pub fn new<I: TypeInfo>() -> Self {
        let info = I::type_info();
        Module {
            name: info.name,
            summary: info.summary,
            description: info.description,
            functions: vec![],
            types: vec![],
        }
    }
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
#[serde(rename_all="snake_case")]
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
#[serde(rename_all="snake_case")]
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

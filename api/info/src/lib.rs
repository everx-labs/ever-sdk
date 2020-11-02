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
#[serde(tag = "type", content="value")]
pub enum ConstValue {
    None,
    Bool(String),
    String(String),
    Number(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Const {
    pub name: String,
    #[serde(flatten)]
    pub value: ConstValue,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Type {
    None,
    Any,
    Boolean,
    String,
    Number,
    BigInt,
    Ref {
        #[serde(rename="ref_name")]
        name: String,
    },
    Optional {
        #[serde(rename="optional_inner")]
        inner: Box<Type>,
    },
    Array {
        #[serde(rename="array_item")]
        item: Box<Type>,
    },
    Struct {
        #[serde(rename="struct_fields")]
        fields: Vec<Field>,
    },
    EnumOfConsts {
        #[serde(rename="enum_consts")]
        consts: Vec<Const>,
    },
    EnumOfTypes {
        #[serde(rename="enum_types")]
        types: Vec<Field>,
    },
    Generic {
        #[serde(rename="generic_name")]
        name: String,
        #[serde(rename="generic_args")]
        args: Vec<Type>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Type,
}

pub trait ApiType {
    fn api() -> Field;
}

pub trait ApiModule {
    fn api() -> Module;
}

impl ApiType for String {
    fn api() -> Field {
        Field {
            name: "string".into(),
            summary: None,
            description: None,
            value: Type::String {},
        }
    }
}

impl ApiType for &str {
    fn api() -> Field {
        Field {
            name: "string".into(),
            summary: None,
            description: None,
            value: Type::String {},
        }
    }
}

impl ApiType for u16 {
    fn api() -> Field {
        Field {
            name: "u16".into(),
            summary: None,
            description: None,
            value: Type::Number {},
        }
    }
}

impl ApiType for u32 {
    fn api() -> Field {
        Field {
            name: "u32".into(),
            summary: None,
            description: None,
            value: Type::Number {},
        }
    }
}

impl ApiType for bool {
    fn api() -> Field {
        Field {
            name: "boolean".into(),
            summary: None,
            description: None,
            value: Type::Boolean {},
        }
    }
}

impl ApiType for () {
    fn api() -> Field {
        Field {
            name: "unit".into(),
            summary: None,
            description: None,
            value: Type::None {},
        }
    }
}

impl ApiType for API {
    fn api() -> Field {
        Field {
            name: "API".into(),
            summary: None,
            description: None,
            value: Type::None {},
        }
    }
}

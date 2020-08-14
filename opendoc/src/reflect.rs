use crate::api::{Field, Type};

pub trait TypeInfo {
    fn type_info() -> Field;
}

impl TypeInfo for String {
    fn type_info() -> Field {
        Field {
            name: "string".into(),
            summary: None,
            description: None,
            value: Type::String,
        }
    }
}

impl TypeInfo for u16 {
    fn type_info() -> Field {
        Field {
            name: "u16".into(),
            summary: None,
            description: None,
            value: Type::Number,
        }
    }
}

impl TypeInfo for bool {
    fn type_info() -> Field {
        Field {
            name: "boolean".into(),
            summary: None,
            description: None,
            value: Type::Boolean,
        }
    }
}

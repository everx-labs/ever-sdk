use crate::api::{Field, Type, API};

pub trait TypeInfo {
    fn type_info() -> Field;
}

impl TypeInfo for String {
    fn type_info() -> Field {
        Field {
            name: "string".into(),
            summary: None,
            description: None,
            value: Type::String {},
        }
    }
}

impl TypeInfo for &str {
    fn type_info() -> Field {
        Field {
            name: "string".into(),
            summary: None,
            description: None,
            value: Type::String {},
        }
    }
}

impl TypeInfo for u16 {
    fn type_info() -> Field {
        Field {
            name: "u16".into(),
            summary: None,
            description: None,
            value: Type::Number {},
        }
    }
}

impl TypeInfo for u32 {
    fn type_info() -> Field {
        Field {
            name: "u32".into(),
            summary: None,
            description: None,
            value: Type::Number {},
        }
    }
}

impl TypeInfo for bool {
    fn type_info() -> Field {
        Field {
            name: "boolean".into(),
            summary: None,
            description: None,
            value: Type::Boolean {},
        }
    }
}

impl TypeInfo for () {
    fn type_info() -> Field {
        Field {
            name: "unit".into(),
            summary: None,
            description: None,
            value: Type::None {},
        }
    }
}

impl TypeInfo for API {
    fn type_info() -> Field {
        Field {
            name: "API".into(),
            summary: None,
            description: None,
            value: Type::None {},
        }
    }
}

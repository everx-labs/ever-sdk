mod enums;
mod others;

use api_info;

use api_info::{ApiModule, ApiType, Field, Type};

fn reflect<T: ApiType>() {
    let info = serde_json::to_string_pretty(&T::api()).unwrap();
    println!("{}", info);
}

fn reflect_module<T: ApiModule>() {
    let info = serde_json::to_string_pretty(&T::api()).unwrap();
    println!("{}", info);
}

enum ExpectType {
    Enum(Vec<(&'static str, ExpectType)>),
    Struct(Vec<(&'static str, ExpectType)>),
    String,
    Number,
}

impl ExpectType {
    fn check_fields(
        actual: &Vec<Field>,
        expected: &Vec<(&'static str, ExpectType)>,
        parent_hint: &str,
    ) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "Unexpected field count for {}",
            parent_hint
        );
        for i in 0..actual.len() {
            assert_eq!(
                actual[i].name, expected[i].0,
                "Unexpected field for {}",
                parent_hint
            );
            expected[i].1.check(
                &actual[i].value,
                &format!("{}.{}", parent_hint, actual[i].name),
            );
        }
    }

    fn unexpected(actual: &Type, expected: &str, parent_hint: &str) {
        panic!(
            "Expected {} but {} found for {}",
            expected,
            type_name(actual),
            parent_hint
        )
    }

    fn check(&self, ty: &Type, parent_hint: &str) {
        match self {
            Self::Enum(expected_variants) => match ty {
                Type::EnumOfTypes { types: variants } => {
                    Self::check_fields(variants, expected_variants, parent_hint);
                }
                ty => Self::unexpected(ty, "Enum", parent_hint),
            },
            Self::Struct(expected_fields) => match ty {
                Type::Struct { fields } => {
                    Self::check_fields(fields, expected_fields, parent_hint);
                }
                ty => Self::unexpected(ty, "Struct", parent_hint),
            },
            Self::String => match ty {
                Type::String { .. } => {}
                ty => Self::unexpected(ty, "String", parent_hint),
            },
            Self::Number => match ty {
                Type::Number { .. } => {}
                ty => Self::unexpected(ty, "Number", parent_hint),
            },
        }
    }
}

fn _struct<T: IntoIterator<Item = (&'static str, ExpectType)>>(fields: T) -> ExpectType {
    ExpectType::Struct(fields.into_iter().collect())
}

fn _enum<T: IntoIterator<Item = (&'static str, ExpectType)>>(fields: T) -> ExpectType {
    ExpectType::Enum(fields.into_iter().collect())
}

fn _string() -> ExpectType {
    ExpectType::String
}

fn _number() -> ExpectType {
    ExpectType::Number
}

fn type_name(ty: &Type) -> String {
    match ty {
        Type::None => "None".to_string(),
        Type::Any => "Any".to_string(),
        Type::Boolean => "Boolean".to_string(),
        Type::String => "String".to_string(),
        Type::Number {
            number_type,
            number_size,
        } => format!("Number({:?},{})", number_type, number_size),
        Type::BigInt {
            number_type,
            number_size,
        } => format!("BigInt({:?},{})", number_type, number_size),
        Type::Ref { name } => format!("Ref({})", name),
        Type::Optional { inner } => format!("Optional({})", type_name(inner)),
        Type::Array { item } => format!("Array({})", type_name(item)),
        Type::Struct { fields } => format!("Struct({})", fields.len()),
        Type::EnumOfConsts { consts } => format!("EnumOfConsts({})", consts.len()),
        Type::EnumOfTypes { types } => format!("EnumOfTypes({})", types.len()),
        Type::Generic { name, args } => format!("Generic({},{})", name, args.len()),
    }
}

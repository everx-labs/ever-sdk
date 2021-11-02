use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CompareValuesResult {
    /// Values `a` and `b` are different
    Different(String),

    /// Value `a` has fields which are not exist in value `b`
    PartiallyMatches,

    /// Value `a` is subset of value `b`
    Subset,

    /// Value `a` equals to value `b`
    Equal,
}

pub(crate) enum JsonPath<'a, 'b> {
    InitialEntity(&'static str),
    Field { parent: &'a JsonPath<'a, 'b>, field_name: &'b str },
    Index { parent: &'a JsonPath<'a, 'b>, index: usize },
}

impl<'a, 'b> JsonPath<'a, 'b> {
    pub fn new(initial_entity: &'static str) -> Self {
        Self::InitialEntity(initial_entity)
    }

    fn join_field(&'a self, field_name: &'b str) -> Self {
        JsonPath::Field { parent: self, field_name }
    }

    fn join_index(&'a self, index: usize) -> Self {
        JsonPath::Index { parent: self, index }
    }

    fn gen_flat_str(&self) -> String {
        match self {
            JsonPath::InitialEntity(_) => String::new(),
            JsonPath::Field { parent, field_name} => {
                if let JsonPath::InitialEntity(_) = parent {
                    field_name.to_string()
                } else {
                    format!("{}.{}", parent.gen_flat_str(), field_name)
                }
            }
            JsonPath::Index { parent, .. } => {
                parent.gen_flat_str()
            }
        }
    }

    fn gen_display_str(&self) -> String {
        match self {
            JsonPath::InitialEntity(name) => name.to_string(),
            JsonPath::Field { parent, field_name } => format!("{}.{}", parent, field_name),
            JsonPath::Index { parent, index} => format!("{}[{}]", parent, index),
        }
    }
}

impl Display for JsonPath<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.gen_display_str())
    }
}

impl CompareValuesResult {
    fn apply_child_result(self, child: CompareValuesResult) -> Self {
        std::cmp::min(self, child)
    }
}

pub(crate) fn compare_values(
    a: &Value,
    b: &Value,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    match (a, b) {
        (Value::Null, Value::Null) => CompareValuesResult::Equal,
        (Value::Null, _) => CompareValuesResult::Subset,

        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::Number(_), Value::String(_))
            | (Value::String(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
        => {
            let is_numeric = numeric_fields.contains(path.gen_flat_str().as_str());
            if get_string(a, is_numeric) == get_string(b, is_numeric) {
                CompareValuesResult::Equal
            } else {
                CompareValuesResult::Different(format!("field `{}`: {:?} != {:?}", path, a, b))
            }
        }

        (Value::Array(vec_a), Value::Array(vec_b)) =>
            compare_vectors(vec_a, vec_b, path, ignore_fields, numeric_fields),

        (Value::Object(map_a), Value::Object(map_b)) =>
            compare_maps(map_a, map_b, path, ignore_fields, numeric_fields),

        _ => CompareValuesResult::Different(format!("field `{}`: {:?} != {:?}", path, a, b))
    }
}

fn compare_maps(
    map_a: &Map<String, Value>,
    map_b: &Map<String, Value>,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    let mut result = if count_fields(map_b, ignore_fields) > count_fields(map_a, ignore_fields) {
        CompareValuesResult::Subset
    } else {
        CompareValuesResult::Equal
    };

    for key in map_a.keys().filter(|key| !ignore_fields.contains(key.as_str())) {
        if let Some(b) = map_b.get(key) {
            result = result.apply_child_result(
                compare_values(
                    &map_a[key],
                    b,
                    path.join_field(key),
                    ignore_fields,
                    numeric_fields,
                ),
            );
            if let CompareValuesResult::Different(_) = result {
                return result;
            }
        } else {
            result = result.apply_child_result(CompareValuesResult::PartiallyMatches);
        }
    }

    result
}

fn compare_vectors(
    vec_a: &Vec<Value>,
    vec_b: &Vec<Value>,
    path: JsonPath<'_, '_>,
    ignore_fields: &HashSet<&'static str>,
    numeric_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    if vec_a.len() != vec_b.len() {
        return CompareValuesResult::Different(
            format!("Field `{}`: arrays has different lengths ({} != {})", path, vec_a.len(), vec_b.len()),
        );
    }

    let mut result = CompareValuesResult::Equal;
    for i in 0..vec_a.len() {
        result = result.apply_child_result(
            compare_values(
                &vec_a[i],
                &vec_b[i],
                path.join_index(i),
                ignore_fields,
                numeric_fields,
            ),
        );
        if let CompareValuesResult::Different(_) = result {
            break;
        }
    }

    result
}

fn count_fields(map: &Map<String, Value>, ignore_fields: &HashSet<&'static str>) -> usize {
    map.keys()
        .filter(|key| !ignore_fields.contains(key.as_str()))
        .count()
}

fn get_string(value: &Value, is_numeric: bool) -> String {
    let result = match value {
        Value::String(v) => v.to_string().to_ascii_lowercase(),
        _ => value.to_string(),
    };

    if is_numeric {
        if let Ok(value) = i128::from_str(&result) {
            if value < 0 {
                return format!("-0x{:x}", value.abs());
            }
            return format!("0x{:x}", value);
        }
        if let Ok(value) = i128::from_str_radix(&result, 16) {
            if value < 0 {
                return format!("-0x{:x}", value.abs());
            }
            return format!("0x{:x}", value);
        }
    }

    result
}

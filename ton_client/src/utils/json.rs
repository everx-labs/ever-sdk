use std::collections::HashSet;

use failure::err_msg;
use serde_json::{Map, Value};
use ton_types::Result;

pub trait JsonHelper {
    fn get_u64(&self, field: &str) -> Result<u64>;
    fn get_i64(&self, field: &str) -> Result<i64>;
    fn get_str(&self, field: &str) -> Result<&str>;
    fn get_array(&self, field: &str) -> Result<&Vec<Value>>;

    fn get_u32(&self, field: &str) -> Result<u32> {
        self.get_u64(field).map(|value| value as u32)
    }

    fn get_i32(&self, field: &str) -> Result<i32> {
        self.get_i64(field).map(|value| value as i32)
    }
}

impl JsonHelper for Value {
    fn get_u64(&self, field: &str) -> Result<u64> {
        self[field].as_u64()
            .ok_or_else(|| err_msg(format!("`{}` field must be an unsigned integer", field)))
    }

    fn get_i64(&self, field: &str) -> Result<i64> {
        self[field].as_i64()
            .ok_or_else(|| err_msg(format!("`{}` field must be an integer", field)))
    }

    fn get_str(&self, field: &str) -> Result<&str> {
        self[field].as_str()
            .ok_or_else(|| err_msg(format!("`{}` field must be a string", field)))
    }

    fn get_array(&self, field: &str) -> Result<&Vec<Value>> {
        self[field].as_array()
            .ok_or_else(|| err_msg(format!("`{}` field must be an array", field)))
    }
}

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

impl CompareValuesResult {
    fn apply_child_result(self, child: CompareValuesResult) -> Self {
        std::cmp::min(self, child)
    }
}

pub(crate) fn compare_values(
    a: &Value,
    b: &Value,
    path: &str,
    ignore_fields: &HashSet<&'static str>,
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
            if get_string(a) == get_string(b) {
                CompareValuesResult::Equal
            } else {
                CompareValuesResult::Different(format!("field `{}`: {:?} != {:?}", path, a, b))
            }
        }

        (Value::Array(vec_a), Value::Array(vec_b)) =>
            compare_vectors(vec_a, vec_b, path, ignore_fields),

        (Value::Object(map_a), Value::Object(map_b)) =>
            compare_maps(map_a, map_b, path, ignore_fields),

        _ => CompareValuesResult::Different(format!("field `{}`: {:?} != {:?}", path, a, b))
    }
}

fn compare_maps(
    map_a: &Map<String, Value>,
    map_b: &Map<String, Value>,
    path: &str,
    ignore_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    let mut result = if count_fields(map_b, ignore_fields) > count_fields(map_a, ignore_fields) {
        CompareValuesResult::Subset
    } else {
        CompareValuesResult::Equal
    };

    for key in map_a.keys().filter(|key| !ignore_fields.contains(key.as_str())) {
        if let Some(b) = map_b.get(key) {
            result = result.apply_child_result(
                compare_values(&map_a[key], b, &format!("{}.{}", path, key), ignore_fields),
            );
        } else {
            result = result.apply_child_result(CompareValuesResult::PartiallyMatches);
        }
    }

    result
}

fn compare_vectors(
    vec_a: &Vec<Value>,
    vec_b: &Vec<Value>,
    path: &str,
    ignore_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    if vec_a.len() != vec_b.len() {
        return CompareValuesResult::Different(
            format!("Field `{}`: arrays has different lengths ({} != {})", path, vec_a.len(), vec_b.len()),
        );
    }

    let mut vec_a = vec_a.clone();
    vec_a.sort_by_key(|v| v.to_string());
    let mut vec_b = vec_b.clone();
    vec_b.sort_by_key(|v| v.to_string());

    let mut result = CompareValuesResult::Equal;
    for i in 0..vec_a.len() {
        result = result.apply_child_result(
            compare_values(&vec_a[i], &vec_b[i], &format!("{}[{}]", path, i), ignore_fields),
        );
    }

    result
}

fn count_fields(map: &Map<String, Value>, ignore_fields: &HashSet<&'static str>) -> usize {
    map.keys()
        .filter(|key| !ignore_fields.contains(key.as_str()))
        .count()
}

fn get_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.to_string().to_ascii_lowercase(),
        _ => value.to_string(),
    }
}

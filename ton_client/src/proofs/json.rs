use std::collections::HashSet;

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

        (Value::String(a_str), Value::Number(b_num)) => {
            let b_i128 = if let Some(b) = b_num.as_i64() {
                Some(b as i128)
            } else if let Some(b) = b_num.as_u64() {
                Some(b as i128)
            } else {
                None
            };

            let hex_equal = if let Some(b_i128) = b_i128 {
                *a_str == format!("0x{:x}", b_i128)
            } else {
                false
            };

            if hex_equal || get_string(a) == b.to_string() {
                CompareValuesResult::Equal
            } else {
                CompareValuesResult::Different(format!("field `{}`: {:?} != {:?}", path, a, b_num))
            }
        }

        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::Number(_), Value::String(_))
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
    path: &str,
    ignore_fields: &HashSet<&'static str>,
) -> CompareValuesResult {
    if vec_a.len() != vec_b.len() {
        return CompareValuesResult::Different(
            format!("Field `{}`: arrays has different lengths ({} != {})", path, vec_a.len(), vec_b.len()),
        );
    }

    let mut result = CompareValuesResult::Equal;
    for i in 0..vec_a.len() {
        result = result.apply_child_result(
            compare_values(&vec_a[i], &vec_b[i], &format!("{}[{}]", path, i), ignore_fields),
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

fn get_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.to_string().to_ascii_lowercase(),
        _ => value.to_string(),
    }
}

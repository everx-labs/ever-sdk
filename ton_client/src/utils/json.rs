use failure::err_msg;
use serde_json::Value;
use ton_types::Result;

pub trait JsonHelper {
    fn get_u64(&self, field: &str) -> Result<u64>;
    fn get_i64(&self, field: &str) -> Result<i64>;
    fn get_str(&self, field: &str) -> Result<&str>;
    fn get_array(&self, field: &str) -> Result<&Vec<Value>>;
    fn take_string(&mut self) -> Option<String>;

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

    fn take_string(&mut self) -> Option<String> {
        match self.take() {
            Value::String(string) => Some(string),
            _ => None,
        }
    }
}

use super::context::{from_hex_to_utf8_str, from_0x_hex};
use serde::{de, Deserialize, Deserializer};
use std::convert::From;

#[derive(Clone)]
pub enum AcType {
    Empty = 0,
    RunAction = 1,
    RunMethod = 2,
    SendMsg = 3,
    Invoke = 4,
    Print = 5,
    Goto = 6,
    CallEngine = 10,
    Unknown = 255,
}

impl From<u8> for AcType {
    fn from(ac_type: u8) -> Self {
        match ac_type {
            0 => AcType::Empty,
            1 => AcType::RunAction,
            2 => AcType::RunMethod,
            3 => AcType::SendMsg,
            4 => AcType::Invoke,
            5 => AcType::Print,
            6 => AcType::Goto,
            10 => AcType::CallEngine,
            _ => AcType::Unknown,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DAction {
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub desc: String,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub name: String,
    #[serde(deserialize_with = "str_to_actype")]
    pub action_type: AcType,
    #[serde(deserialize_with = "from_0x_hex")]
    pub to: u8,
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub attrs: String,
    pub misc: String,
}

impl DAction {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        DAction {
            desc: String::new(),
            name: String::new(),
            action_type: AcType::Empty,
            to: 0,
            attrs: String::new(),
            misc: String::new(),
        }
    }
    
    #[allow(dead_code)]
    pub fn new(desc: String, name: String, action_type: u8, to: u8) -> Self {
        DAction {
            desc,
            name,
            action_type: action_type.into(),
            to,
            attrs: String::new(),
            misc: String::new(),
        }
    }

    pub fn is_engine_call(&self) -> bool {
        match self.action_type {
            AcType::CallEngine => true,
            _ => false,
        }
    }

    pub fn is_instant(&self) -> bool {
        self.attrs.split(',')
            .find(|val| val.to_owned() == "instant")
            .map(|_| true)
            .unwrap_or(false)
    }

    pub fn func_attr(&self) -> Option<String> {
        self.attr_value("func")
    }

    pub fn args_attr(&self) -> Option<String> {
        self.attr_value("args")
    }

    pub fn sign_by_user(&self) -> bool {
        self.attr_value("sign")
            .map(|s| s == "by_user")
            .unwrap_or(false)
    }

    pub fn format_args(&self) -> Option<String> {
        self.attr_value("fargs")
    }

    fn attr_value(&self, name: &str) -> Option<String> {
        let name = name.to_owned() + "=";
        self.attrs.split(',')
            .find(|val| val.starts_with(&name))
            .map(|val| { 
                let vec: Vec<&str> = val.split('=').collect();
                vec[1].to_owned()
            })
    }
}

fn str_to_actype<'de, D>(des: D) -> Result<AcType, D::Error> 
where 
    D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(des)?;
    u8::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(de::Error::custom)
        .map(|t| t.into())
}


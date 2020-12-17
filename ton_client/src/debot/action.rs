use crate::encoding::decode_abi_number;
use super::context::{from_abi_num, from_hex_to_utf8_str};
use serde::{de, Deserialize, Deserializer, Serializer};
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

impl Default for AcType {
    fn default() -> Self { AcType::Empty }
}

/// Describes a debot action in a Debot Context.
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DAction {
    /// A short action description. Should be used by Debot Browser as name of
    /// menu item.
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub desc: String,
    /// Depends on action type. Can be a debot function name or a print string 
    /// (for Print Action).
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub name: String,
    /// Action type.
    #[serde(deserialize_with = "str_to_actype")]
    #[serde(serialize_with = "actype_to_str")]
    pub action_type: AcType,
    /// ID of debot context to switch after action execution. 
    #[serde(deserialize_with = "from_abi_num")]
    pub to: u8,
    /// Action attributes. In the form of "param=value,flag".
    /// attribute example: instant, args, fargs, sign.
    #[serde(deserialize_with = "from_hex_to_utf8_str")]
    pub attrs: String,
    /// Some internal action data. Used by debot only.
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

    pub fn is_invoke(&self) -> bool {
        match self.action_type {
            AcType::Invoke => true,
            _ => false,
        }
    }

    pub fn is_instant(&self) -> bool {
        self.attrs
            .split(',')
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
        self.attrs
            .split(',')
            .find(|val| val.starts_with(&name))
            .map(|val| {
                let vec: Vec<&str> = val.split('=').collect();
                vec[1].to_owned()
            })
    }
}

fn str_to_actype<'de, D>(des: D) -> Result<AcType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(des)?;
    decode_abi_number::<u8>(&s)
        .map_err(de::Error::custom)
        .map(|t| t.into())
}

fn actype_to_str<S>(a: &AcType, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let num: u8 = match a {
        AcType::Empty => 0,
        AcType::RunAction => 1,
        AcType::RunMethod => 2,
        AcType::SendMsg => 3,
        AcType::Invoke => 4,
        AcType::Print => 5,
        AcType::Goto => 6,
        AcType::CallEngine => 10,
        AcType::Unknown => 255,
    };
    
    s.serialize_str(&format!("{:x}", num))
}

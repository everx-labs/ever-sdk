use std::convert::TryInto;
use std::sync::Arc;

use ever_abi::{Param, ParamType, Token, TokenValue};
use ever_abi::contract::AbiVersion;
use ever_abi::token::{Cursor, Detokenizer};
use ever_block::{HashmapE, HashmapType, Serializable, SliceData, fail, BuilderData};
use ever_block::Result;
use ever_vm::int;
use ever_vm::stack::integer::IntegerData;
use ever_vm::stack::integer::serialization::{UnsignedIntegerBigEndianEncoding, SignedIntegerBigEndianEncoding};
use ever_vm::stack::StackItem;
use num_bigint::{BigInt};
use ever_block::IBitstring;

use crate::abi::Error;
use crate::ClientContext;
use crate::error::{ClientError, ClientResult};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiHandle(u32);

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag = "type", content = "value")]
pub enum Abi {
    Contract(AbiContract),
    Json(String),
    Handle(AbiHandle),

    Serialized(AbiContract),
}

impl Default for Abi {
    fn default() -> Self {
        Abi::Json(Default::default())
    }
}

impl Abi {
    pub fn json_string(&self) -> ClientResult<String> {
        match self {
            Self::Contract(abi) | Self::Serialized(abi) => {
                Ok(serde_json::to_string(abi).map_err(|err| Error::invalid_abi(err))?)
            }
            Self::Json(abi) => Ok(abi.clone()),
            _ => Err(crate::client::Error::not_implemented(
                "ABI handles are not supported yet",
            )),
        }
    }

    pub fn abi(&self) -> ClientResult<ever_abi::Contract> {
        ever_abi::Contract::load(self.json_string()?.as_bytes())
            .map_err(|x| Error::invalid_json(x))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiContract {
    #[serde(rename = "ABI version", default = "default_abi_version")]
    pub obsolete_abi_version: u32,
    #[serde(default = "default_abi_version")]
    pub abi_version: u32,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub header: Vec<String>,
    #[serde(default)]
    pub functions: Vec<AbiFunction>,
    #[serde(default)]
    pub events: Vec<AbiEvent>,
    #[serde(default)]
    pub data: Vec<AbiData>,
    #[serde(default)]
    pub fields: Vec<AbiParam>,
}

fn default_abi_version() -> u32 {
    2
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<AbiParam>,
    pub outputs: Vec<AbiParam>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiEvent {
    pub name: String,
    pub inputs: Vec<AbiParam>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiData {
    pub key: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub components: Vec<AbiParam>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct AbiParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub components: Vec<AbiParam>,
    #[serde(default)]
    pub init: bool,
}

impl TryInto<ever_abi::Param> for AbiParam {
    type Error = ClientError;

    fn try_into(self) -> ClientResult<ever_abi::Param> {
        serde_json::from_value(
            serde_json::to_value(&self)
                .map_err(|err| Error::invalid_json(err))?
        ).map_err(|err| Error::invalid_json(err))
    }
}

/// The ABI function header.
///
/// Includes several hidden function parameters that contract
/// uses for security, message delivery monitoring and replay protection reasons.
///
/// The actual set of header fields depends on the contract's ABI.
/// If a contract's ABI does not include some headers, then they are not filled.
#[derive(Serialize, Deserialize, ApiType, PartialEq, Debug, Clone, Default)]
pub struct FunctionHeader {
    /// Message expiration timestamp (UNIX time) in seconds.
    ///
    /// If not specified - calculated automatically from message_expiration_timeout(),
    /// try_index and message_expiration_timeout_grow_factor() (if ABI includes `expire` header).
    pub expire: Option<u32>,

    /// Message creation time in milliseconds.
    ///
    /// If not specified, `now` is used (if ABI includes `time` header).
    pub time: Option<u64>,

    /// Public key is used by the contract to check the signature.
    ///
    /// Encoded in `hex`. If not specified, method fails with exception (if ABI includes `pubkey` header).
    pub pubkey: Option<String>,
}

fn required_time(token: &Token) -> ClientResult<u64> {
    match &token.value {
        TokenValue::Time(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`time` header has invalid format",
        )),
    }
}

fn required_expire(token: &Token) -> ClientResult<u32> {
    match &token.value {
        TokenValue::Expire(v) => Ok(v.clone()),
        _ => Err(Error::invalid_message_for_decode(
            "`expire` header has invalid format",
        )),
    }
}

fn required_pubkey(token: &Token) -> ClientResult<Option<String>> {
    match token.value {
        TokenValue::PublicKey(key) => Ok(key.as_ref().map(|x| hex::encode(&x))),
        _ => Err(Error::invalid_message_for_decode(
            "`pubkey` header has invalid format",
        )),
    }
}

impl FunctionHeader {
    pub fn from(tokens: &Vec<Token>) -> ClientResult<Option<Self>> {
        if tokens.len() == 0 {
            return Ok(None);
        }
        let mut header = FunctionHeader::default();
        for token in tokens {
            match token.name.as_str() {
                "time" => header.time = Some(required_time(&token)?),
                "expire" => header.expire = Some(required_expire(&token)?),
                "pubkey" => header.pubkey = required_pubkey(&token)?,
                _ => (),
            }
        }
        Ok(Some(header))
    }
}

pub(crate) async fn resolve_signature_id(
    context: &Arc<ClientContext>,
    provided_signature_id: Option<i32>,
) -> ClientResult<Option<i32>> {
    if let Some(signature_id) = provided_signature_id.or(context.config.network.signature_id) {
        return Ok(Some(signature_id));
    }

    Ok(crate::net::get_signature_id(context.clone()).await?.signature_id)
}

pub(crate) async fn extend_data_to_sign(
    context: &Arc<ClientContext>,
    provided_signature_id: Option<i32>,
    data_to_sign: Option<Vec<u8>>
) -> ClientResult<Option<Vec<u8>>> {
    if let Some(data_to_sign) = data_to_sign {
        if let Some(signature_id) = resolve_signature_id(context, provided_signature_id).await? {
            let mut extended_data = Vec::with_capacity(4 + data_to_sign.len());
            extended_data.extend_from_slice(&signature_id.to_be_bytes());
            extended_data.extend_from_slice(&data_to_sign);
            Ok(Some(extended_data))
        } else {
            Ok(Some(data_to_sign))
        }
    } else {
        Ok(None)
    }
}

pub struct TokenValueToStackItem;

impl TokenValueToStackItem {

    fn hashmap_e_to_stack_item(hashmap_e: HashmapE) -> StackItem {
        if let Some(cell) = hashmap_e.data() {
            return StackItem::Cell(cell.clone());
        }
        return StackItem::None;
    }

    pub fn convert_token_to_vm_type(token_value: TokenValue, abi_version: &AbiVersion) -> Result<StackItem> {
        Ok(match token_value {
            TokenValue::Uint(v) => int!(v.number),
            TokenValue::Int(v) => int!(v.number),
            TokenValue::VarInt(_, v) => int!(v),
            TokenValue::VarUint(_, v) => int!(v),
            TokenValue::Bool(v) => ever_vm::boolean!(v),
            TokenValue::Tuple(values) => {
                let mut res: Vec<StackItem> = vec![];
                for token in values {
                    let item = Self::convert_token_to_vm_type(token.value, abi_version)?;
                    res.push(item);
                }
                StackItem::Tuple(Arc::new(res))
            }
            TokenValue::Array(param_type, token_values) => {
                let hashmap_e = TokenValue::put_array_into_dictionary(&param_type, &token_values, abi_version)?;
                let cell = Self::hashmap_e_to_stack_item(hashmap_e);
                let size = int!(token_values.len());
                let res: Vec<StackItem> = vec![size, cell];
                StackItem::Tuple(Arc::new(res))
            }
            TokenValue::FixedArray(_, _) => fail!("Not supported"),
            TokenValue::Cell(v) => StackItem::Cell(v),
            TokenValue::Map(key_type, value_type, values) => {
                let hashmap_e = TokenValue::map_token_to_hashmap_e(&key_type, &value_type, &values, abi_version)?;
                Self::hashmap_e_to_stack_item(hashmap_e)
            }
            TokenValue::Address(addr)
            | TokenValue::AddressStd(addr) => {
                let cell = addr.serialize().unwrap();
                let slice :SliceData = SliceData::load_cell(cell).unwrap();
                StackItem::Slice(slice)
            }
            TokenValue::Bytes(bytes) =>
                StackItem::Cell(TokenValue::bytes_to_cells(bytes.as_ref(), abi_version)?),
            TokenValue::FixedBytes(bytes) => {
                let mut num = BigInt::ZERO;
                for b in bytes {
                    num = (num << 8) + b;
                }
                int!(num)
            }
            TokenValue::String(str) =>
                StackItem::Cell(TokenValue::bytes_to_cells(str.as_bytes(), abi_version)?),
            TokenValue::Token(_) => fail!("Not supported"),
            TokenValue::Time(_) => fail!("Not supported"),
            TokenValue::Expire(_) => fail!("Not supported"),
            TokenValue::PublicKey(_) => fail!("Not supported"),
            TokenValue::Optional(_, token_value) => {
                if let Some(value) = token_value {
                    let is_opt_or_map = match *value {
                        TokenValue::Optional(_, _) => true,
                        TokenValue::Map(_, _, _) => true,
                        _ => false
                    };
                    let res = Self::convert_token_to_vm_type(*value, abi_version)?;
                    if is_opt_or_map {
                        StackItem::Tuple(Arc::new(vec![res]))
                    } else {
                        res
                    }
                } else {
                    StackItem::None
                }
            }
            TokenValue::Ref(_) => fail!("Not supported"),
        })
    }
}

pub struct StackItemToJson;

impl StackItemToJson {

    fn cell_to_slice(stack_item: &StackItem) -> Result<SliceData> {
        let StackItem::Cell(x) = stack_item else { fail!("Unexpected vm item") };
        Ok(SliceData::load_builder(x.write_to_new_cell()?)?)
    }

    fn int_to_slice(stack_item: &StackItem, size: usize, is_sign: bool) -> Result<SliceData> {
        let StackItem::Integer(x) = stack_item else { fail!("Unexpected vm item") };
        Ok(
            if is_sign {
                x.as_slice::<SignedIntegerBigEndianEncoding>(size)?
            } else {
                x.as_slice::<UnsignedIntegerBigEndianEncoding>(size)?
            }
        )
    }

    fn tuple_to_token(items: &[StackItem], params: &[Param], abi_version: &AbiVersion) -> Result<Vec<Token>> {
        let mut tokens = vec![];
        for i in 0..items.len() {
            let x = &Self::stack_item_to_token(&items[i], &params[i], abi_version)?;
            tokens.push(x.clone());
        }
        Ok(tokens)
    }

    fn dict_to_builder(item: &StackItem) -> Result<BuilderData> {
        let mut builder = BuilderData::new();
        if let StackItem::Cell(cell) = item {
            builder.append_bit_one()?;
            builder.checked_append_reference(cell.clone())?;
        } else if let StackItem::None = item {
            builder.append_bit_zero()?;
        } else {
            fail!("Unexpected vm item")
        }
        Ok(builder)
    }

    fn stack_item_to_token(stack_item: &StackItem, param: &Param, abi_version: &AbiVersion) -> Result<Token> {
        let slice = match &param.kind {
            ParamType::Uint(size) => Self::int_to_slice(stack_item, *size, false)?,
            ParamType::Int(size) => Self::int_to_slice(stack_item, *size, true)?,
            ParamType::VarUint(size) => {
                let StackItem::Integer(x) = stack_item else { fail!("Unexpected vm item") };
                let num = x.take_value_of(|num| Some(num.clone()))?
                    .to_biguint()
                    .unwrap();
                SliceData::load_builder(TokenValue::write_varuint(&num, *size)?)?
            }
            ParamType::VarInt(size) => {
                let StackItem::Integer(x) = stack_item else { fail!("Unexpected vm item") };
                let num = x.take_value_of(|num| Some(num.clone()))?;
                SliceData::load_builder(TokenValue::write_varint(&num, *size)?)?
            }
            ParamType::Bool => Self::int_to_slice(stack_item, 1, true)?,
            ParamType::Tuple(params) => {
                let StackItem::Tuple(items) = stack_item else { fail!("Unexpected vm item") };
                let tokens = Self::tuple_to_token(items, params, abi_version)?;
                let token_value = TokenValue::Tuple(tokens);
                return Ok(Token { name: param.name.to_string(), value: token_value });
            },
            ParamType::Array(_) => {
                let StackItem::Tuple(items) = stack_item else { fail!("Unexpected vm item") };
                assert_eq!(items.len(), 2);
                let len = Self::int_to_slice(&items[0], 32, false)?;
                let dict = Self::dict_to_builder(&items[1])?;
                let mut builder = BuilderData::new();
                builder.checked_append_references_and_data(&len)?;
                builder.append_builder(&dict)?;
                SliceData::load_builder(builder)?
            }
            ParamType::FixedArray(_, _) => fail!("Not supported"),
            ParamType::Cell => {
                let StackItem::Cell(cell) = stack_item else { fail!("Unexpected vm item") };
                SliceData::load_builder(cell.write_to_new_cell()?)?
            }
            ParamType::Map(_, _) => {
                SliceData::load_builder(Self::dict_to_builder(stack_item)?)?
            }
            ParamType::Address | ParamType::AddressStd => {
                let StackItem::Slice(slice) = stack_item else { fail!("Unexpected vm item") };
                slice.clone()
            }
            ParamType::Bytes => Self::cell_to_slice(stack_item)?,
            ParamType::FixedBytes(size) => Self::int_to_slice(stack_item, 8 * *size, false)?,
            ParamType::String => Self::cell_to_slice(stack_item)?,
            ParamType::Token => fail!("Not supported"),
            ParamType::Time => fail!("Not supported"),
            ParamType::Expire => fail!("Not supported"),
            ParamType::PublicKey => fail!("Not supported"),
            ParamType::Optional(underlying) => {
                let underlying_token_value = if let StackItem::None = stack_item {
                    TokenValue::Optional(*underlying.clone(), None)
                } else {
                    let mut is_opt  = false;
                    let mut is_map  = false;
                    match **underlying {
                        ParamType::Optional(_) => is_opt = true,
                        ParamType::Map(_, _) => is_map = true,
                        _ => { }
                    };
                    let underlying_param = Param{name: "unnamed".to_string(), kind: *underlying.clone()};
                    if is_opt || is_map {
                        let StackItem::Tuple(tuple) = stack_item else { fail!("Unexpected vm item") };
                        Self::stack_item_to_token(&tuple[0], &underlying_param, abi_version)?
                    } else {
                        Self::stack_item_to_token(stack_item, &underlying_param, abi_version)?
                    }.value
                };
                return Ok(Token { name: param.name.to_string(), value: underlying_token_value });
            }
            ParamType::Ref(_) => fail!("Not supported"),
        };
        let (token_value, _cursor) = TokenValue::read_from(
            &param.kind,
            Cursor{used_bits: 0, used_refs:0, slice},
            true,
            abi_version,
            false
        )?;
        Ok(Token { name: param.name.to_string(), value: token_value })
    }

    pub fn convert_vm_items_to_json(stack_items: &[StackItem], params: &[Param], abi_version: &AbiVersion) -> Result<serde_json::Value> {
        assert_eq!(stack_items.len(), params.len());
        let tokens : Vec<Token> = Self::tuple_to_token(stack_items, params, abi_version)?;
        Ok(Detokenizer::detokenize_to_json_value(tokens.as_slice())?)
    }
}
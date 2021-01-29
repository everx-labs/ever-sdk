use super::dinterface::{decode_answer_id, get_arg, DebotInterface, InterfaceResult};
use super::routines;
use super::TonClient;
use crate::abi::Abi;
use crate::crypto::{chacha20, ParamsOfChaCha20};
use serde_json::Value;

const ABI: &str = r#"

"#;

pub const SDK_ID: &str = "8fc6454f90072c9f1f6d3313ae1608f64f4a0660c6ae9f42c68b6a79e2a1bc4b";

pub struct SdkInterface {
    ton: TonClient,
}

impl SdkInterface {
    pub fn new(ton: TonClient) -> Self {
        Self { ton }
    }

    async fn get_balance(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = routines::get_balance(self.ton.clone(), args).await?;
        Ok((answer_id, json!({ "nanotokens": value })))
    }

    async fn get_account_type(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let value = routines::get_account_state(self.ton.clone(), args).await?;
        Ok((answer_id, json!({ "acc_type": value.acc_type })))
    }

    fn get_random(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let rnd = routines::generate_random(self.ton.clone(), args)?;
        let buf = base64::decode(&rnd)
            .map_err(|e| format!("failed to decode random buffer to byte array: {}", e))?;
        Ok((answer_id, json!({ "buffer": hex::encode(buf) })))
    }

    fn chacha20(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let data = base64::encode(&hex::decode(&get_arg(args, "data")?).unwrap());
        let nonce = get_arg(args, "nonce")?;
        let key = get_arg(args, "key")?;
        let result = chacha20(self.ton.clone(), ParamsOfChaCha20 { data, key, nonce })
            .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({ "decrypted": hex::encode(&base64::decode(&result.data).unwrap()) }),
        ))
    }
}

#[async_trait::async_trait]
impl DebotInterface for SdkInterface {
    fn get_id(&self) -> String {
        SDK_ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "getBalance" => self.get_balance(args).await,
            "getAccountType" => self.get_account_type(args).await,
            //"getAccountHash", boxed(Self::decode)),
            "chacha20" => self.chacha20(args),
            //"signHash", self.signHash(),
            "getRandom" => self.get_random(args),
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

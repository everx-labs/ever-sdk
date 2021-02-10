use super::calltype::send_ext_msg;
use super::dinterface::{get_arg, DebotInterface, InterfaceResult};
use crate::abi::{decode_message, Abi, ParamsOfDecodeMessage};
use crate::crypto::{get_signing_box, KeyPair};
use crate::debot::BrowserCallbacks;
use crate::debot::TonClient;
use crate::encoding::decode_abi_bigint;
use serde_json::Value;
use std::sync::Arc;
use ton_abi::Contract;

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "sendWithKeypair",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"message","type":"cell"},
				{"name":"pub","type":"uint256"},
				{"name":"sec","type":"uint256"}
			],
			"outputs": []
		},
	],
	"data": [
	],
	"events": [
	]
}
"#;

const ID: &str = "475a5d1729acee4601c2a8cb67240e4da5316cc90a116e1b181d905e79401c51";

pub struct MsgInterface {
    ton: TonClient,
    debot_addr: String,
    debot_abi: Abi,
    browser: Arc<dyn BrowserCallbacks + Send + Sync>,
}

impl MsgInterface {
    pub fn new(
        ton: TonClient,
        debot_addr: String,
        debot_abi: Abi,
        browser: Arc<dyn BrowserCallbacks + Send + Sync>,
    ) -> Self {
        Self {
            ton,
            debot_addr,
            debot_abi,
            browser,
        }
    }

    async fn send_with_keypair(&self, args: &Value) -> InterfaceResult {
        let message = get_arg(args, "message")?;
        let public = get_arg(args, "pub")?;
        let public = decode_abi_bigint(&public).map_err(|e| format!("{}", e))?;
        let secret = get_arg(args, "sec")?;
        let secret = decode_abi_bigint(&secret).map_err(|e| format!("{}", e))?;
        let kpair = KeyPair::new(format!("{:064x}", public), format!("{:064x}", secret));
        let signing_box = get_signing_box(self.ton.clone(), kpair)
            .await
            .map_err(|e| format!("{}", e))?
            .handle;
        let msg = send_ext_msg(
            self.browser.clone(),
            self.ton.clone(),
            message,
            signing_box,
            String::new(),
            &self.debot_addr,
        )
        .await
        .map_err(|e| format!("{}", e))?;
        let result = decode_message(
            self.ton.clone(),
            ParamsOfDecodeMessage {
                abi: self.debot_abi.clone(),
                message: msg,
            },
        )
        .await
        .map_err(|e| format!("failed to decode message: {}", e))?;
        let abi_str = self.debot_abi.json_string().unwrap();
        let contract = Contract::load(abi_str.as_bytes()).map_err(|e| format!("{}", e))?;
        let answer_id = contract
            .function(&result.name)
            .map_err(|e| format!("{}", e))?
            .get_input_id();
        Ok((answer_id, result.value.unwrap_or_default()))
    }
}

#[async_trait::async_trait]
impl DebotInterface for MsgInterface {
    fn get_id(&self) -> String {
        ID.to_string()
    }

    fn get_abi(&self) -> Abi {
        Abi::Json(ABI.to_owned())
    }

    async fn call(&self, func: &str, args: &Value) -> InterfaceResult {
        match func {
            "sendWithKeypair" => self.send_with_keypair(args).await,
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

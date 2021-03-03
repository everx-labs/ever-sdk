use super::dinterface::{
    decode_answer_id, get_arg, get_bool_arg, get_num_arg, get_string_arg, DebotInterface,
    InterfaceResult,
};
use super::routines;
use super::TonClient;
use crate::abi::Abi;
use crate::crypto::{
    chacha20, hdkey_derive_from_xprv, hdkey_derive_from_xprv_path, hdkey_public_from_xprv,
    hdkey_secret_from_xprv, hdkey_xprv_from_mnemonic, mnemonic_derive_sign_keys,
    mnemonic_from_random, mnemonic_verify, nacl_sign_keypair_from_secret_key, nacl_box, 
    nacl_box_open, nacl_box_keypair_from_secret_key, ParamsOfChaCha20,
    ParamsOfHDKeyDeriveFromXPrv, ParamsOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeyPublicFromXPrv,
    ParamsOfHDKeySecretFromXPrv, ParamsOfHDKeyXPrvFromMnemonic, ParamsOfMnemonicDeriveSignKeys,
    ParamsOfMnemonicFromRandom, ParamsOfMnemonicVerify, ParamsOfNaclSignKeyPairFromSecret,
    ParamsOfNaclBox, ParamsOfNaclBoxOpen, ParamsOfNaclBoxKeyPairFromSecret, 
};
use crate::encoding::decode_abi_bigint;
use serde_json::Value;

const ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "getBalance",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"addr","type":"address"}
			],
			"outputs": [
				{"name":"nanotokens","type":"uint128"}
			]
		},
		{
			"name": "getAccountType",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"addr","type":"address"}
			],
			"outputs": [
				{"name":"acc_type","type":"int8"}
			]
		},
		{
			"name": "getAccountCodeHash",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"addr","type":"address"}
			],
			"outputs": [
				{"name":"code_hash","type":"uint256"}
			]
		},
		{
			"name": "chacha20",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"data","type":"bytes"},
				{"name":"nonce","type":"bytes"},
				{"name":"key","type":"uint256"}
			],
			"outputs": [
				{"name":"output","type":"bytes"}
			]
		},
		{
			"name": "signHash",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"hash","type":"uint256"}
			],
			"outputs": [
				{"name":"signature","type":"bytes"}
			]
		},
		{
			"name": "genRandom",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"length","type":"uint32"}
			],
			"outputs": [
				{"name":"buffer","type":"bytes"}
			]
		},
		{
			"name": "compress7z",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"uncompressed","type":"bytes"}
			],
			"outputs": [
				{"name":"comp","type":"bytes"}
			]
		},
		{
			"name": "uncompress7z",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"compressed","type":"bytes"}
			],
			"outputs": [
				{"name":"uncomp","type":"bytes"}
			]
		},
		{
			"name": "mnemonicFromRandom",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"dict","type":"uint32"},
				{"name":"wordCount","type":"uint32"}
			],
			"outputs": [
				{"name":"phrase","type":"bytes"}
			]
		},
		{
			"name": "mnemonicVerify",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"bytes"}
			],
			"outputs": [
				{"name":"valid","type":"bool"}
			]
		},
		{
			"name": "mnemonicDeriveSignKeys",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"bytes"},
				{"name":"path","type":"bytes"}
			],
			"outputs": [
				{"name":"pub","type":"uint256"},
				{"name":"sec","type":"uint256"}
			]
		},
		{
			"name": "hdkeyXprvFromMnemonic",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"bytes"}
			],
			"outputs": [
				{"name":"xprv","type":"bytes"}
			]
		},
		{
			"name": "hdkeyDeriveFromXprv",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"inXprv","type":"bytes"},
				{"name":"childIndex","type":"uint32"},
				{"name":"hardened","type":"bool"}
			],
			"outputs": [
				{"name":"xprv","type":"bytes"}
			]
		},
		{
			"name": "hdkeyDeriveFromXprvPath",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"inXprv","type":"bytes"},
				{"name":"path","type":"bytes"}
			],
			"outputs": [
				{"name":"xprv","type":"bytes"}
			]
		},
		{
			"name": "hdkeySecretFromXprv",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"xprv","type":"bytes"}
			],
			"outputs": [
				{"name":"sec","type":"uint256"}
			]
		},
		{
			"name": "hdkeyPublicFromXprv",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"xprv","type":"bytes"}
			],
			"outputs": [
				{"name":"pub","type":"uint256"}
			]
		},
		{
			"name": "naclSignKeypairFromSecretKey",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"secret","type":"uint256"}
			],
			"outputs": [
				{"name":"sec","type":"uint256"},
				{"name":"pub","type":"uint256"}
			]
		},
		{
			"name": "substring",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"str","type":"bytes"},
				{"name":"start","type":"uint32"},
				{"name":"count","type":"uint32"}
			],
			"outputs": [
				{"name":"substr","type":"bytes"}
			]
        },
        {
			"name": "naclBox",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"decrypted","type":"bytes"},
				{"name":"nonce","type":"bytes"},
				{"name":"publicKey","type":"uint256"},
				{"name":"secretKey","type":"uint256"}
			],
			"outputs": [
				{"name":"encrypted","type":"bytes"}
			]
        },
        {
			"name": "naclBoxOpen",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"encrypted","type":"bytes"},
				{"name":"nonce","type":"bytes"},
				{"name":"publicKey","type":"uint256"},
				{"name":"secretKey","type":"uint256"}
			],
			"outputs": [
				{"name":"decrypted","type":"bytes"}
			]
		},
		{
			"name": "naclKeypairFromSecret",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"secret","type":"uint256"}
			],
			"outputs": [
				{"name":"publicKey","type":"uint256"},
				{"name":"secretKey","type":"uint256"}
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

const SDK_ID: &str = "8fc6454f90072c9f1f6d3313ae1608f64f4a0660c6ae9f42c68b6a79e2a1bc4b";

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
            json!({ "output": hex::encode(&base64::decode(&result.data).unwrap()) }),
        ))
    }

    fn mnemonic_from_random(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let dict = get_num_arg::<u8>(args, "dict")?;
        let word_count = get_num_arg::<u8>(args, "wordCount")?;
        let result = mnemonic_from_random(
            self.ton.clone(),
            ParamsOfMnemonicFromRandom {
                dictionary: Some(dict),
                word_count: Some(word_count),
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({ "phrase": hex::encode(result.phrase.as_bytes()) }),
        ))
    }

    fn mnemonic_derive_sign_keys(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let phrase = get_string_arg(args, "phrase")?;
        let path = get_string_arg(args, "path")?;
        let keypair = mnemonic_derive_sign_keys(
            self.ton.clone(),
            ParamsOfMnemonicDeriveSignKeys {
                phrase,
                path: if path == "" { None } else { Some(path) },
                dictionary: None,
                word_count: None,
            },
        )
        .map_err(|e| format!("{}", e))?;

        Ok((
            answer_id,
            json!({
                "pub": format!("0x{}", keypair.public),
                "sec": format!("0x{}", keypair.secret)
            }),
        ))
    }

    fn mnemonic_verify(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let phrase = get_string_arg(args, "phrase")?;
        let result = mnemonic_verify(
            self.ton.clone(),
            ParamsOfMnemonicVerify {
                phrase,
                dictionary: None,
                word_count: None,
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "valid": result.valid })))
    }

    fn hdkey_xprv_from_mnemonic(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let phrase = get_string_arg(args, "phrase")?;
        let result = hdkey_xprv_from_mnemonic(
            self.ton.clone(),
            ParamsOfHDKeyXPrvFromMnemonic {
                phrase,
                dictionary: None,
                word_count: None,
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({ "xprv": hex::encode(result.xprv.as_bytes()) }),
        ))
    }

    fn hdkey_public_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_string_arg(args, "xprv")?;
        let result = hdkey_public_from_xprv(self.ton.clone(), ParamsOfHDKeyPublicFromXPrv { xprv })
            .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "pub": format!("0x{}", result.public) })))
    }

    fn hdkey_derive_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_string_arg(args, "inXprv")?;
        let child_index = get_num_arg::<u32>(args, "childIndex")?;
        let hardened = get_bool_arg(args, "hardened")?;
        let result = hdkey_derive_from_xprv(
            self.ton.clone(),
            ParamsOfHDKeyDeriveFromXPrv {
                xprv,
                child_index,
                hardened,
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({ "xprv": hex::encode(result.xprv.as_bytes()) }),
        ))
    }

    fn hdkey_derive_from_xprv_path(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_string_arg(args, "inXprv")?;
        let path = get_string_arg(args, "path")?;
        let result = hdkey_derive_from_xprv_path(
            self.ton.clone(),
            ParamsOfHDKeyDeriveFromXPrvPath { xprv, path },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({ "xprv": hex::encode(result.xprv.as_bytes()) }),
        ))
    }

    fn hdkey_secret_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_string_arg(args, "xprv")?;
        let result = hdkey_secret_from_xprv(self.ton.clone(), ParamsOfHDKeySecretFromXPrv { xprv })
            .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "sec": format!("0x{}", result.secret) })))
    }

    fn nacl_sign_keypair_from_secret_key(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let secret = decode_abi_bigint(&get_arg(args, "secret")?).map_err(|e| format!("{}", e))?;
        let result = nacl_sign_keypair_from_secret_key(
            self.ton.clone(),
            ParamsOfNaclSignKeyPairFromSecret {
                secret: format!("{:064x}", secret),
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({
                "sec": format!("0x{}", result.secret.get(0..64).ok_or(format!("secret key is invalid"))?),
                "pub": format!("0x{}", result.public)
            }),
        ))
    }

    fn substring(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let src_str = get_string_arg(args, "str")?;
        let start = get_num_arg::<u32>(args, "start")? as usize;
        let count = get_num_arg::<u32>(args, "count")? as usize;
        if start >= src_str.len() {
            return Err(format!("\"start\" is invalid"));
        }
        if count > src_str.len() {
            return Err(format!("\"count\" is invalid"));
        }
        let end = start + count;
        if end > src_str.len() {
            return Err(format!("start + count is out of range"));
        }
        let sub_str = src_str.get(start..end).ok_or(format!("substring failed"))?;
        Ok((
            answer_id,
            json!({ "substr": hex::encode(sub_str.as_bytes()) })
        ))
    }

    fn nacl_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let decrypted = base64::encode(&hex::decode(&get_arg(args, "decrypted")?).map_err(|e| format!("{}", e))?);
        let nonce = get_arg(&args, "nonce")?;
        let public = decode_abi_bigint(&get_arg(&args, "publicKey")?).map_err(|e| e.to_string())?;
        let secret = decode_abi_bigint(&get_arg(&args, "secretKey")?).map_err(|e| e.to_string())?;
        let result = nacl_box(
            self.ton.clone(),
            ParamsOfNaclBox {                
                decrypted,
                nonce,
                their_public: format!("{:064x}", public),
                secret: format!("{:064x}", secret),
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "encrypted": hex::encode(&base64::decode(&result.encrypted).map_err(|e| format!("{}", e))?) })))
    }

    fn nacl_box_open(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let encrypted = base64::encode(&hex::decode(&get_arg(args, "encrypted")?).map_err(|e| format!("{}", e))?);
        let nonce = get_arg(&args, "nonce")?;
        let public = decode_abi_bigint(&get_arg(&args, "publicKey")?).map_err(|e| e.to_string())?;
        let secret = decode_abi_bigint(&get_arg(&args, "secretKey")?).map_err(|e| e.to_string())?;
        let result = nacl_box_open(
            self.ton.clone(),
            ParamsOfNaclBoxOpen {                
                encrypted,
                nonce,
                their_public: format!("{:064x}", public),
                secret: format!("{:064x}", secret),
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "decrypted": hex::encode(&base64::decode(&result.decrypted).map_err(|e| format!("{}", e))?) })))
    }

    fn nacl_box_keypair_from_secret_key(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let secret = decode_abi_bigint(&get_arg(args, "secret")?).map_err(|e| format!("{}", e))?;
        let result = nacl_box_keypair_from_secret_key(
            self.ton.clone(),
            ParamsOfNaclBoxKeyPairFromSecret {
                secret: format!("{:064x}", secret),
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((
            answer_id,
            json!({
                "publicKey": format!("0x{}", result.public),
                "secretKey": format!("0x{}", result.secret.get(0..64).ok_or(format!("secret key is invalid"))?)
            }),
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
            "chacha20" => self.chacha20(args),
            "genRandom" => self.get_random(args),
            "mnemonicFromRandom" => self.mnemonic_from_random(args),
            "mnemonicDeriveSignKeys" => self.mnemonic_derive_sign_keys(args),
            "mnemonicVerify" => self.mnemonic_verify(args),
            "hdkeyXprvFromMnemonic" => self.hdkey_xprv_from_mnemonic(args),
            "hdkeyDeriveFromXprv" => self.hdkey_derive_from_xprv(args),
            "hdkeyDeriveFromXprvPath" => self.hdkey_derive_from_xprv_path(args),
            "hdkeySecretFromXprv" => self.hdkey_secret_from_xprv(args),
            "hdkeyPublicFromXprv" => self.hdkey_public_from_xprv(args),
            "naclSignKeypairFromSecretKey" => self.nacl_sign_keypair_from_secret_key(args),
            "substring" => self.substring(args),
            "naclBox" => self.nacl_box(args),
            "naclBoxOpen" => self.nacl_box_open(args),
            "naclKeypairFromSecret" => self.nacl_box_keypair_from_secret_key(args),   
            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

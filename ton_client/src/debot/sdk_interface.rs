use super::dinterface::{
    decode_answer_id, get_arg, get_bool_arg, get_num_arg, DebotInterface, InterfaceResult,
};
use super::routines;
use super::TonClient;
use crate::abi::Abi;
use crate::crypto::{
    chacha20, encryption_box_decrypt, encryption_box_encrypt, encryption_box_get_info,
    hdkey_derive_from_xprv, hdkey_derive_from_xprv_path, hdkey_public_from_xprv,
    hdkey_secret_from_xprv, hdkey_xprv_from_mnemonic, mnemonic_derive_sign_keys,
    mnemonic_from_random, mnemonic_verify, nacl_box, nacl_box_keypair_from_secret_key,
    nacl_box_open, nacl_sign_keypair_from_secret_key, signing_box_get_public_key, signing_box_sign,
    EncryptionBoxHandle, EncryptionBoxInfo, ParamsOfChaCha20, ParamsOfEncryptionBoxDecrypt,
    ParamsOfEncryptionBoxEncrypt, ParamsOfEncryptionBoxGetInfo, ParamsOfHDKeyDeriveFromXPrv,
    ParamsOfHDKeyDeriveFromXPrvPath, ParamsOfHDKeyPublicFromXPrv, ParamsOfHDKeySecretFromXPrv,
    ParamsOfHDKeyXPrvFromMnemonic, ParamsOfMnemonicDeriveSignKeys, ParamsOfMnemonicFromRandom,
    ParamsOfMnemonicVerify, ParamsOfNaclBox, ParamsOfNaclBoxKeyPairFromSecret, ParamsOfNaclBoxOpen,
    ParamsOfNaclSignKeyPairFromSecret, ParamsOfSigningBoxSign, RegisteredSigningBox,
};
use crate::encoding::decode_abi_bigint;
use crate::net::{query_collection, OrderBy, ParamsOfQueryCollection, SortDirection};
use serde_json::Value;

const ABI: &str = r#"
{
	"ABI version": 2,
	"version": "2.2",
	"header": ["time"],
	"functions": [
		{
			"name": "getBalance",
			"id": "0x0036b4f3",
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
			"id": "0x2b885111",
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
			"id": "0x38b68a99",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"addr","type":"address"}
			],
			"outputs": [
				{"name":"code_hash","type":"uint256"}
			]
		},
		{
			"name": "getAccountsDataByHash",
			"id": "0x2ff074fa",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"codeHash","type":"uint256"},
				{"name":"gt","type":"address"}
			],
			"outputs": [
				{"components":[{"name":"id","type":"address"},{"name":"data","type":"cell"}],"name":"accounts","type":"tuple[]"}
			]
		},
		{
			"name": "encrypt",
			"id": "0x1edf9b42",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"boxHandle","type":"uint32"},
				{"name":"data","type":"bytes"}
			],
			"outputs": [
				{"name":"result","type":"uint32"},
				{"name":"encrypted","type":"bytes"}
			]
		},
		{
			"name": "decrypt",
			"id": "0x6d1ab339",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"boxHandle","type":"uint32"},
				{"name":"data","type":"bytes"}
			],
			"outputs": [
				{"name":"result","type":"uint32"},
				{"name":"decrypted","type":"bytes"}
			]
		},
		{
			"name": "getEncryptionBoxInfo",
			"id": "0x6ce70176",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"boxHandle","type":"uint32"}
			],
			"outputs": [
				{"name":"result","type":"uint32"},
				{"components":[{"name":"hdpath","type":"string"},{"name":"algorithm","type":"string"},{"name":"options","type":"string"},{"name":"publicInfo","type":"string"}],"name":"info","type":"tuple"}
			]
		},
		{
			"name": "signHash",
			"id": "0x422d1a4a",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"boxHandle","type":"uint32"},
				{"name":"hash","type":"uint256"}
			],
			"outputs": [
				{"name":"signature","type":"bytes"}
			]
		},
		{
			"name": "getSigningBoxInfo",
			"id": "0x5e836915",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"boxHandle","type":"uint32"}
			],
			"outputs": [
				{"name":"result","type":"uint32"},
				{"name":"key","type":"uint256"}
			]
		},
		{
			"name": "genRandom",
			"id": "0x05c672c3",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"length","type":"uint32"}
			],
			"outputs": [
				{"name":"buffer","type":"bytes"}
			]
		},
		{
			"name": "substring",
			"id": "0x56c2d6d7",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"str","type":"string"},
				{"name":"start","type":"uint32"},
				{"name":"count","type":"uint32"}
			],
			"outputs": [
				{"name":"substr","type":"string"}
			]
		},
		{
			"name": "mnemonicFromRandom",
			"id": "0x2f22913c",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"dict","type":"uint32"},
				{"name":"wordCount","type":"uint32"}
			],
			"outputs": [
				{"name":"phrase","type":"string"}
			]
		},
		{
			"name": "mnemonicVerify",
			"id": "0x11ae5ae1",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"string"}
			],
			"outputs": [
				{"name":"valid","type":"bool"}
			]
		},
		{
			"name": "mnemonicDeriveSignKeys",
			"id": "0x14f12d13",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"string"},
				{"name":"path","type":"string"}
			],
			"outputs": [
				{"name":"pub","type":"uint256"},
				{"name":"sec","type":"uint256"}
			]
		},
		{
			"name": "hdkeyXprvFromMnemonic",
			"id": "0x3141b013",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"phrase","type":"string"}
			],
			"outputs": [
				{"name":"xprv","type":"string"}
			]
		},
		{
			"name": "hdkeyDeriveFromXprv",
			"id": "0x3df91936",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"inXprv","type":"string"},
				{"name":"childIndex","type":"uint32"},
				{"name":"hardened","type":"bool"}
			],
			"outputs": [
				{"name":"xprv","type":"string"}
			]
		},
		{
			"name": "hdkeyDeriveFromXprvPath",
			"id": "0x612b1f35",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"inXprv","type":"string"},
				{"name":"path","type":"string"}
			],
			"outputs": [
				{"name":"xprv","type":"string"}
			]
		},
		{
			"name": "hdkeySecretFromXprv",
			"id": "0x24b30dc5",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"xprv","type":"string"}
			],
			"outputs": [
				{"name":"sec","type":"uint256"}
			]
		},
		{
			"name": "hdkeyPublicFromXprv",
			"id": "0x0c991027",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"xprv","type":"string"}
			],
			"outputs": [
				{"name":"pub","type":"uint256"}
			]
		},
		{
			"name": "naclSignKeypairFromSecretKey",
			"id": "0x5340824d",
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
			"name": "naclBox",
			"id": "0x7f9ee6c7",
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
			"id": "0x4151b21f",
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
			"id": "0x21159daf",
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
			"name": "chacha20",
			"id": "0x34499e29",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"data","type":"bytes"},
				{"name":"nonce","type":"bytes"},
				{"name":"key","type":"uint256"}
			],
			"outputs": [
				{"name":"output","type":"bytes"}
			]
		}
	]
}
"#;

const SDK_ID: &str = "8fc6454f90072c9f1f6d3313ae1608f64f4a0660c6ae9f42c68b6a79e2a1bc4b";

pub struct SdkInterface {
    ton: TonClient,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EncryptionBoxInfoResult {
    pub hdpath: String,
    pub algorithm: String,
    pub options: String,
    pub public_info: String,
}

use std::convert::From;
impl From<EncryptionBoxInfo> for EncryptionBoxInfoResult {
    fn from(info: EncryptionBoxInfo) -> Self {
        Self {
            algorithm: info.algorithm.unwrap_or_default(),
            hdpath: info.hdpath.unwrap_or_default(),
            options: info.options.map(|v| v.to_string()).unwrap_or_default(),
            public_info: info.public.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
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
        let value = routines::get_account_state(self.ton.clone(), args).await;
        Ok((answer_id, json!({ "acc_type": value.acc_type })))
    }

    async fn get_account_code_hash(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let res = routines::get_account(self.ton.clone(), args).await;
        let code_hash_str = match &res {
            Ok(acc) => acc["code_hash"].as_str().unwrap_or("0"),
            Err(e) => {
                debug!("get_account_code_hash failed: {}", e);
                "0"
            }
        };
        Ok((
            answer_id,
            json!({ "code_hash": format!("0x{}", code_hash_str) }),
        ))
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
        Ok((answer_id, json!({ "phrase": result.phrase })))
    }

    fn mnemonic_derive_sign_keys(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let phrase = get_arg(args, "phrase")?;
        let path = get_arg(args, "path")?;
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
        let phrase = get_arg(args, "phrase")?;
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
        let phrase = get_arg(args, "phrase")?;
        let result = hdkey_xprv_from_mnemonic(
            self.ton.clone(),
            ParamsOfHDKeyXPrvFromMnemonic {
                phrase,
                dictionary: None,
                word_count: None,
            },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "xprv": result.xprv })))
    }

    fn hdkey_public_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_arg(args, "xprv")?;
        let result = hdkey_public_from_xprv(self.ton.clone(), ParamsOfHDKeyPublicFromXPrv { xprv })
            .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "pub": format!("0x{}", result.public) })))
    }

    fn hdkey_derive_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_arg(args, "inXprv")?;
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
        Ok((answer_id, json!({ "xprv": result.xprv })))
    }

    fn hdkey_derive_from_xprv_path(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_arg(args, "inXprv")?;
        let path = get_arg(args, "path")?;
        let result = hdkey_derive_from_xprv_path(
            self.ton.clone(),
            ParamsOfHDKeyDeriveFromXPrvPath { xprv, path },
        )
        .map_err(|e| format!("{}", e))?;
        Ok((answer_id, json!({ "xprv": result.xprv })))
    }

    fn hdkey_secret_from_xprv(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let xprv = get_arg(args, "xprv")?;
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
        let src_str = get_arg(args, "str")?;
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
        Ok((answer_id, json!({ "substr": sub_str })))
    }

    fn nacl_box(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let decrypted = base64::encode(
            &hex::decode(&get_arg(args, "decrypted")?).map_err(|e| format!("{}", e))?,
        );
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
        Ok((
            answer_id,
            json!({ "encrypted": hex::encode(&base64::decode(&result.encrypted).map_err(|e| format!("{}", e))?) }),
        ))
    }

    fn nacl_box_open(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let encrypted = base64::encode(
            &hex::decode(&get_arg(args, "encrypted")?).map_err(|e| format!("{}", e))?,
        );
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
        Ok((
            answer_id,
            json!({ "decrypted": hex::encode(&base64::decode(&result.decrypted).map_err(|e| format!("{}", e))?) }),
        ))
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

    async fn encrypt(&self, args: &Value) -> InterfaceResult {
        self.encrypt_or_decrypt(args, true).await
    }

    async fn decrypt(&self, args: &Value) -> InterfaceResult {
        self.encrypt_or_decrypt(args, false).await
    }

    async fn get_encryption_box_info(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let encryption_box = EncryptionBoxHandle(get_num_arg::<u32>(args, "boxHandle")?);

        let result = encryption_box_get_info(
            self.ton.clone(),
            ParamsOfEncryptionBoxGetInfo { encryption_box },
        )
        .await
        .map_err(|e| e.code as u32)
        .map(|x| x.info);

        let (result, info) = match result {
            Ok(info) => (0, EncryptionBoxInfoResult::from(info)),
            Err(code) => (code, EncryptionBoxInfoResult::default()),
        };

        let return_args = json!({ "result": result, "info": info});
        Ok((answer_id, return_args))
    }

    async fn encrypt_or_decrypt(&self, args: &Value, encrypt: bool) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let encryption_box = EncryptionBoxHandle(get_num_arg::<u32>(args, "boxHandle")?);
        let data =
            base64::encode(&hex::decode(&get_arg(args, "data")?).map_err(|e| format!("{}", e))?);
        let result = if encrypt {
            encryption_box_encrypt(
                self.ton.clone(),
                ParamsOfEncryptionBoxEncrypt {
                    encryption_box,
                    data,
                },
            )
            .await
            .map_err(|e| e.code as u32)
            .map(|x| x.data)
        } else {
            encryption_box_decrypt(
                self.ton.clone(),
                ParamsOfEncryptionBoxDecrypt {
                    encryption_box,
                    data,
                },
            )
            .await
            .map_err(|e| e.code as u32)
            .map(|x| x.data)
        };

        let (result, data) = match result {
            Ok(data) => {
                let data = base64::decode(&data)
                    .map(|x| hex::encode(x))
                    .map_err(|e| format!("failed to decode base64: {}", e))?;
                (0, data)
            }
            Err(code) => (code, "".to_owned()),
        };

        let return_args = if encrypt {
            json!({ "result": result, "encrypted": data })
        } else {
            json!({ "result": result, "decrypted": data })
        };
        Ok((answer_id, return_args))
    }

    async fn query_accounts(&self, args: &Value, result: &str) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let code_hash = get_arg(args, "codeHash")?;
        let gt_addr = get_arg(args, "gt")?;
        let code_hash = decode_abi_bigint(&code_hash)
            .map_err(|e| format!("failed to parse integer \"{}\": {}", code_hash, e))?;

        let accounts = query_collection(
            self.ton.clone(),
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({
                    "code_hash": { "eq": format!("{:064x}", code_hash) },
                    "id": {"gt": gt_addr }
                })),
                result: result.to_owned(),
                order: Some(vec![OrderBy {
                    path: "id".to_owned(),
                    direction: SortDirection::ASC,
                }]),
                limit: None,
            },
        )
        .await
        .map_err(|e| format!("account query failed: {}", e))?
        .result;

        Ok((answer_id, json!({ "accounts": accounts })))
    }

    async fn get_accounts_data_by_hash(&self, args: &Value) -> InterfaceResult {
        let res = self
            .query_accounts(args, "id data")
            .await
            .map_err(|e| format!("query account failed: {}", e))?;
        Ok(res)
    }

    async fn get_signing_box_info(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let box_handle = get_num_arg::<u32>(args, "boxHandle")?;
        let result = signing_box_get_public_key(
            self.ton.clone(),
            RegisteredSigningBox {
                handle: box_handle.into(),
            },
        )
        .await;

        let (result, key) = match result {
            Ok(val) => (0, format!("0x{}", val.pubkey)),
            Err(e) => (e.code as u32, format!("0")),
        };
        Ok((answer_id, json!({ "result": result, "key": key})))
    }

    async fn sign_hash(&self, args: &Value) -> InterfaceResult {
        let answer_id = decode_answer_id(args)?;
        let box_handle = get_num_arg::<u32>(args, "boxHandle")?;
        let hash_to_sign =
            decode_abi_bigint(&get_arg(&args, "hash")?).map_err(|e| e.to_string())?;

        let signature = signing_box_sign(
            self.ton.clone(),
            ParamsOfSigningBoxSign {
                signing_box: box_handle.into(),
                unsigned: base64::encode(&hash_to_sign.to_bytes_be().1),
            },
        )
        .await
        .map_err(|e| format!("{}", e))?
        .signature;

        Ok((answer_id, json!({ "signature": signature })))
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
            "getAccountCodeHash" => self.get_account_code_hash(args).await,

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

            "genRandom" => self.get_random(args),
            "signHash" => self.sign_hash(args).await,
            "getSigningBoxInfo" => self.get_signing_box_info(args).await,
            "naclBox" => self.nacl_box(args),
            "naclBoxOpen" => self.nacl_box_open(args),
            "naclKeypairFromSecret" => self.nacl_box_keypair_from_secret_key(args),
            "chacha20" => self.chacha20(args),

            "encrypt" => self.encrypt(args).await,
            "decrypt" => self.decrypt(args).await,
            "getEncryptionBoxInfo" => self.get_encryption_box_info(args).await,

            "getAccountsDataByHash" => self.get_accounts_data_by_hash(args).await,

            _ => Err(format!("function \"{}\" is not implemented", func)),
        }
    }
}

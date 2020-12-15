use super::dengine::TonClient;
use crate::crypto::{
    generate_random_bytes, nacl_box_keypair_from_secret_key, signing_box_sign, KeyPair,
    ParamsOfGenerateRandomBytes, ParamsOfNaclBox, ParamsOfNaclBoxKeyPairFromSecret,
    ParamsOfSigningBoxSign, SigningBoxHandle,
};
use crate::encoding::{decode_abi_bigint, decode_abi_number};
use crate::net::{query_collection, ParamsOfQueryCollection};
use chrono::{Local, TimeZone};

pub async fn call_routine(
    ton: TonClient,
    name: &str,
    arg: &str,
    signer: Option<SigningBoxHandle>,
) -> Result<serde_json::Value, String> {
    let arg_json: Result<serde_json::Value, String> =
        serde_json::from_str(arg).map_err(|e| format!("argument is invalid json: {}", e));
    match name {
        "convertTokens" => {
            debug!("convertTokens({})", arg);
            let tokens = convert_string_to_tokens(ton, arg)?;
            Ok(json!({ "arg1": tokens }))
        }
        "getBalance" => {
            debug!("getBalance({})", arg);
            let args = if arg_json.is_err() { json!({ "addr": arg }) } else { arg_json? };
            let balance = get_balance(ton, args).await?;
            Ok(json!({ "arg1": balance }))
        }
        "getAccountState" => {
            debug!("getAccountState({})", arg);
            get_account_state(ton, json!({ "addr": arg })).await
        }
        "loadBocFromFile" => {
            debug!("loadBocFromFile({})", arg);
            let loaded_cell = load_boc_from_file(ton, arg)?;
            Ok(json!({ "arg1": loaded_cell }))
        }
        "signHash" => {
            let arg_json = arg_json?;
            debug!("signHash({})", arg_json);
            let sign = sign_hash(
                ton,
                arg_json,
                signer.ok_or("Signing box is needed to sign hash".to_owned())?,
            )
            .await?;
            Ok(json!({ "arg1": sign }))
        }
        "encryptAuth" => {
            let arg_json = arg_json?;
            debug!("encryptAuth({})", arg_json);
            let encrypted = nacl_box(ton, arg_json)?;
            Ok(json!({
                "encrypted": hex::encode(base64::decode(&encrypted).unwrap())
            }))
        }
        "genKeypairFromSecret" => {
            let arg_json = arg_json?;
            debug!("genKeypairFromSecret({})", arg_json);
            nacl_box_gen_keypair(ton, arg_json).map(|keypair| {
                json!({
                    "publicKey" : format!("0x{}", keypair.public),
                    "secretKey": format!("0x{}", keypair.secret)
                })
            })
        }
        "genRandom" => {
            let arg_json = arg_json?;
            debug!("genRandom({})", arg_json);
            let rnd = generate_random(ton, arg_json)?;
            let buf = base64::decode(&rnd)
                .map_err(|e| format!("failed to decode random buffer to byte array: {}", e))?;
            Ok(json!({ "buffer": hex::encode(buf) }))
        }
        _ => Err(format!("unknown engine routine: {}({})", name, arg_json?))?,
    }
}

pub fn convert_string_to_tokens(_ton: TonClient, arg: &str) -> Result<String, String> {
    let parts: Vec<&str> = arg.split(".").collect();
    if parts.len() >= 1 && parts.len() <= 2 {
        let mut result = String::new();
        result += parts[0];
        if parts.len() == 2 {
            let fraction = format!("{:0<9}", parts[1]);
            if fraction.len() != 9 {
                return Err("invalid fractional part".to_string());
            }
            result += &fraction;
        } else {
            result += "000000000";
        }
        u64::from_str_radix(&result, 10).map_err(|e| format!("failed to parse amount: {}", e))?;
        return Ok(result);
    }
    Err("Invalid amout value".to_string())
}

pub async fn get_balance(ton: TonClient, arg_json: serde_json::Value) -> Result<String, String> {
    let acc = get_account_state(ton, arg_json).await?;
    Ok(acc["balance"].as_str().unwrap().to_owned())
}

pub(super) fn format_string(fstr: &str, params: &serde_json::Value) -> String {
    let mut str_builder = String::new();
    for (i, s) in fstr.split("{}").enumerate() {
        str_builder += s;
        str_builder += &format_arg(&params, i);
    }
    str_builder
}

pub(super) fn format_arg(params: &serde_json::Value, i: usize) -> String {
    let idx = i.to_string();
    if let Some(arg) = params["param".to_owned() + &idx].as_str() {
        return arg.to_owned();
    }
    if let Some(arg) = params["str".to_owned() + &idx].as_str() {
        return String::from_utf8(hex::decode(arg).unwrap_or(vec![])).unwrap_or(String::new());
    }
    if let Some(arg) = params["number".to_owned() + &idx].as_str() {
        // TODO: need to use big number instead of u64
        debug!("parsing number{}: {}", idx, arg);
        return format!(
            "{}",
            // TODO: remove unwrap and return error
            decode_abi_number::<u64>(arg).unwrap()
        );
    }
    if let Some(arg) = params["utime".to_owned() + &idx].as_str() {
        let utime = decode_abi_number::<u32>(arg).unwrap();
        return if utime == 0 {
            "undefined".to_owned()
        } else {
            let date = Local.timestamp(utime as i64, 0);
            date.to_rfc2822()
        };
    }
    String::new()
}

pub(super) fn load_boc_from_file(_ton: TonClient, arg: &str) -> Result<String, String> {
    let boc =
        std::fs::read(arg).map_err(|e| format!(r#"failed to read boc file "{}": {}"#, arg, e))?;
    Ok(base64::encode(&boc))
}

pub(super) async fn sign_hash(
    ton: TonClient,
    arg_json: serde_json::Value,
    signer: SigningBoxHandle,
) -> Result<String, String> {
    let hash_str = arg_json["hash"]
        .as_str()
        .ok_or(format!(r#""hash" argument not found"#))?;
    let hash_as_bigint = decode_abi_bigint(hash_str).map_err(|err| err.to_string())?;
    let result = signing_box_sign(
        ton,
        ParamsOfSigningBoxSign {
            unsigned: base64::encode(&hash_as_bigint.to_bytes_be().1),
            signing_box: signer,
        },
    )
    .await
    .map_err(|err| format!("Can not sign hash: {}", err))?;
    Ok(result.signature)
}

pub(super) fn generate_random(ton: TonClient, args: serde_json::Value) -> Result<String, String> {
    let len_str = args["length"]
        .as_str()
        .ok_or(format!(r#""len" not found"#))?;
    let len =
        u32::from_str_radix(len_str, 10).map_err(|e| format!("failed to parse length: {}", e))?;
    let result = generate_random_bytes(ton, ParamsOfGenerateRandomBytes { length: len })
        .map_err(|e| format!(" failed to generate random: {}", e))?;
    Ok(result.bytes)
}

fn get_arg(args: &serde_json::Value, name: &str) -> Result<String, String> {
    args[name]
        .as_str()
        .ok_or(format!("\"{}\" not found", name))
        .map(|v| v.to_string())
}

pub(super) fn nacl_box(ton: TonClient, args: serde_json::Value) -> Result<String, String> {
    let public = decode_abi_bigint(&get_arg(&args, "publicKey")?).map_err(|e| e.to_string())?;
    let secret = decode_abi_bigint(&get_arg(&args, "secretKey")?).map_err(|e| e.to_string())?;
    let result = crate::crypto::nacl_box(
        ton,
        ParamsOfNaclBox {
            decrypted: base64::encode(&get_arg(&args, "decrypted")?),
            nonce: get_arg(&args, "nonce")?,
            their_public: hex::encode(public.to_bytes_be().1),
            secret: hex::encode(secret.to_bytes_be().1),
        },
    )
    .map_err(|e| format!(" failed to encrypt with nacl box: {}", e))?;
    Ok(result.encrypted)
}

pub(super) fn nacl_box_gen_keypair(
    ton: TonClient,
    args: serde_json::Value,
) -> Result<KeyPair, String> {
    let secret = decode_abi_bigint(&get_arg(&args, "secret")?).map_err(|e| e.to_string())?;
    let result = nacl_box_keypair_from_secret_key(
        ton,
        ParamsOfNaclBoxKeyPairFromSecret {
            secret: hex::encode(secret.to_bytes_be().1),
        },
    )
    .map_err(|e| format!(" failed to generate keypair from secret: {}", e))?;
    Ok(result)
}

pub(super) async fn get_account_state(
    ton: TonClient,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let addr = get_arg(&args, "addr")?.to_lowercase();
    let mut accounts = query_collection(
        ton,
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({
                "id": { "eq": addr }
            })),
            result: "balance acc_type last_trans_lt code data library".to_owned(),
            order: None,
            limit: Some(1),
        },
    )
    .await
    .map_err(|e| format!("account query failed: {}", e))?
    .result;
    if accounts.len() == 0 {
        return Err(format!("account not found"));
    }
    let mut acc = accounts.swap_remove(0);
    let library_cell = acc
        .as_object_mut()
        .ok_or("invalid account json".to_string())?
        .remove("library")
        .unwrap();

    acc["lib"] = if library_cell.is_null() {
        json!("")
    } else {
        library_cell
    };
    if acc["code"].is_null() {
        acc["code"] = json!("");
    }
    if acc["data"].is_null() {
        acc["data"] = json!("");
    }
    Ok(acc)
}

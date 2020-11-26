use super::dengine::TonClient;
use crate::crypto::{signing_box_sign, ParamsOfSigningBoxSign, SigningBoxHandle};
use crate::net::{query_collection, ParamsOfQueryCollection};
use crate::encoding::{decode_abi_bigint, decode_abi_number};
use chrono::{Local, TimeZone};

pub async fn call_routine(
    ton: TonClient,
    name: &str,
    arg: &str,
    signer: Option<SigningBoxHandle>,
) -> Result<String, String> {
    match name {
        "convertTokens" => convert_string_to_tokens(ton, arg),
        "getBalance" => get_balance(ton, arg).await,
        "loadBocFromFile" => load_boc_from_file(ton, arg),
        "signHash" => {
            sign_hash(ton, arg, signer.ok_or("Signing box is needed to sign hash".to_owned())?)
                .await
        },
        _ => Err(format!("unknown engine routine: {}", name))?,
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

pub async fn get_balance(ton: TonClient, arg: &str) -> Result<String, String> {
    let arg_json: serde_json::Value =
        serde_json::from_str(arg).map_err(|e| format!("arguments is invalid json: {}", e))?;
    let addr = arg_json["addr"].as_str().ok_or(format!("addr not found"))?;
    let accounts = query_collection(
        ton,
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(json!({
                "id": { "eq": addr }
            })),
            result: "acc_type_name balance".to_owned(),
            order: None,
            limit: Some(1),
        },
    )
    .await
    .map_err(|e| format!("account query failed: {}", e.to_string()))?
    .result;
    let acc = accounts.get(0).ok_or(format!("account not found"))?;
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
    debug!("load boc file {}", arg);
    let boc =
        std::fs::read(arg).map_err(|e| format!(r#"failed to read boc file "{}": {}"#, arg, e))?;
    Ok(base64::encode(&boc))
}

pub(super) async fn sign_hash(ton: TonClient, arg: &str, signer: SigningBoxHandle) -> Result<String, String> {
    debug!("sign hash {}", arg);
    let arg_json: serde_json::Value =
        serde_json::from_str(arg).map_err(|e| format!("argument is invalid json: {}", e))?;
    let hash_str = arg_json["hash"]
        .as_str()
        .ok_or(format!(r#""hash" argument not found"#))?;
    let hash_as_bigint = decode_abi_bigint(hash_str)
        .map_err(|err| err.to_string())?;
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

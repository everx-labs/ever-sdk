use crate::abi::{Abi, MessageSigning};
use crate::crypto::internal::{sign_using_secret, decode_public_key};
use crate::encoding::hex_decode;
use crate::error::{ApiError, ApiResult};
use serde_json::Value;
use ton_sdk::{ContractImage};

pub(crate) fn resolve_abi(abi: &Abi) -> ApiResult<String> {
    if let Abi::Value(value) = abi {
        Ok(value.to_string())
    } else {
        Err(ApiError::contracts_create_deploy_message_failed(
            "Abi handle doesn't supported yet",
        ))
    }
}

/// Combines `hex` encoded `signature` with `base64` encoded `unsigned_message`.
/// Returns signed message encoded with `base64`.
pub(crate) fn add_sign_to_message(
    abi: &str,
    signature: &[u8],
    public_key: Option<&[u8]>,
    unsigned_message: &[u8],
) -> ApiResult<Vec<u8>> {
    let signed = ton_sdk::Contract::add_sign_to_message(
        abi.to_string(),
        signature,
        public_key,
        unsigned_message,
    )
    .map_err(|err| ApiError::contracts_encode_message_with_sign_failed(err))?;
    Ok(signed.serialized_message)
}

pub(crate) fn result_of_encode_message(
    abi: &str,
    message: &[u8],
    data_to_sign: &[u8],
    signing: &MessageSigning,
) -> ApiResult<(String, Option<String>)> {
    let (message, data_to_sign) = if let Some(keys) = signing.resolve_keys()? {
        let secret = hex_decode(&format!("{}{}", &keys.secret, &keys.public))?;
        let (_, signature) = sign_using_secret(&data_to_sign, &secret)?;
        let message =
            add_sign_to_message(abi, &signature, Some(&hex_decode(&keys.public)?), &message)?;
        (message, None)
    } else {
        (message.to_vec(), Some(data_to_sign))
    };
    Ok((
        base64::encode(&message),
        data_to_sign.map(|x| base64::encode(&x)),
    ))
}

pub(crate) fn create_tvc_image(
    abi: &str,
    init_params: Option<&Value>,
    tvc: &String,
    public_key: &String,
) -> ApiResult<ContractImage> {
    let tvc = base64::decode(tvc).map_err(|err| ApiError::contracts_invalid_image(err))?;
    let public = decode_public_key(&public_key)?;
    let mut image = ContractImage::from_state_init_and_key(&mut tvc.as_slice(), &public)
        .map_err(|err| ApiError::contracts_image_creation_failed(err))?;

    if let Some(params) = init_params {
        image
            .update_data(&params.to_string(), abi)
            .map_err(|err| {
                ApiError::contracts_image_creation_failed(format!(
                    "Failed to set initial data: {}",
                    err
                ))
            })?;
    }

    Ok(image)
}

use crate::abi::{Abi, Error, Signer};
use crate::client;
use crate::crypto::internal::{decode_public_key, sign_using_secret};
use crate::encoding::hex_decode;
use crate::error::ApiResult;
use serde_json::Value;
use ton_sdk::ContractImage;

pub(crate) fn resolve_abi(abi: &Abi) -> ApiResult<String> {
    if let Abi::Serialized(value) = abi {
        Ok(value.to_string())
    } else {
        Err(client::Error::not_implemented(
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
    .map_err(|err| Error::attach_signature_failed(err))?;
    Ok(signed.serialized_message)
}

pub(crate) fn result_of_encode_message(
    abi: &str,
    message: Vec<u8>,
    data_to_sign: Option<Vec<u8>>,
    signer: &Signer,
) -> ApiResult<(Vec<u8>, Option<String>)> {
    if let Some(keys) = signer.resolve_keys()? {
        if let Some(data_to_sign) = data_to_sign {
            let secret = hex_decode(&format!("{}{}", &keys.secret, &keys.public))?;
            let (_, signature) = sign_using_secret(&data_to_sign, &secret)?;
            let message =
                add_sign_to_message(abi, &signature, Some(&hex_decode(&keys.public)?), &message)?;
            return Ok((message, None));
        }
    }
    Ok((message, data_to_sign.map(|x| base64::encode(&x))))
}

pub(crate) fn create_tvc_image(
    abi: &str,
    init_params: Option<&Value>,
    tvc: &String,
    public_key: &String,
) -> ApiResult<ContractImage> {
    let tvc = base64::decode(tvc).map_err(|err| Error::invalid_tvc_image(err))?;
    let public = decode_public_key(&public_key)?;
    let mut image = ContractImage::from_state_init_and_key(&mut tvc.as_slice(), &public)
        .map_err(|err| Error::invalid_tvc_image(err))?;

    if let Some(params) = init_params {
        image.update_data(&params.to_string(), abi).map_err(|err| {
            Error::invalid_tvc_image(format!("Failed to set initial data: {}", err))
        })?;
    }

    Ok(image)
}

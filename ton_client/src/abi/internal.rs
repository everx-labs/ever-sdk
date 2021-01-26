use std::sync::Arc;

use serde_json::Value;

use ton_sdk::ContractImage;

use crate::abi::{DeploySet, Error, Signer};
use crate::ClientContext;
use crate::encoding::hex_decode;
use crate::error::ClientResult;

/// Combines `hex` encoded `signature` with `base64` encoded `unsigned_message`.
/// Returns signed message encoded with `base64`.
pub(crate) fn add_sign_to_message(
    abi: &str,
    signature: &[u8],
    public_key: Option<&[u8]>,
    unsigned_message: &[u8],
) -> ClientResult<Vec<u8>> {
    let signed = ton_sdk::Contract::add_sign_to_message(
        abi.to_string(),
        signature,
        public_key,
        unsigned_message,
    )
    .map_err(|err| Error::attach_signature_failed(err))?;
    Ok(signed.serialized_message)
}

/// Combines `hex` encoded `signature` with `base64` encoded `unsigned_message`.
/// Returns signed message encoded with `base64`.
pub(crate) fn add_sign_to_message_body(
    abi: &str,
    signature: &[u8],
    public_key: Option<&[u8]>,
    unsigned_body: &[u8],
) -> ClientResult<Vec<u8>> {
    let unsigned = ton_sdk::Contract::deserialize_tree_to_slice(unsigned_body)
        .map_err(|err| Error::attach_signature_failed(err))?;
    let body = ton_abi::add_sign_to_function_call(abi.to_string(), signature, public_key, unsigned)
        .map_err(|err| Error::attach_signature_failed(err))?;
    Ok(ton_types::serialize_toc(
        &body
            .into_cell()
            .map_err(|err| Error::attach_signature_failed(err))?,
    )
    .map_err(|err| Error::attach_signature_failed(err))?)
}

pub(crate) async fn result_of_encode_message(
    context: Arc<ClientContext>,
    abi: &str,
    message: Vec<u8>,
    data_to_sign: Option<Vec<u8>>,
    signer: &Signer,
) -> ClientResult<(Vec<u8>, Option<String>)> {
    if let Some(unsigned) = &data_to_sign {
        if let Some(signature) = signer.sign(context.clone(), unsigned).await? {
            let pubkey = signer.resolve_public_key(context)
                .await?
                .map(|string| hex_decode(&string))
                .transpose()?;
            let message = add_sign_to_message(
                abi, &signature, pubkey.as_ref().map(|vec| vec.as_slice()), &message
            )?;
            return Ok((message, None));
        }
    }
    Ok((message, data_to_sign.map(|x| base64::encode(&x))))
}

pub(crate) fn create_tvc_image(
    abi: &str,
    init_params: Option<&Value>,
    tvc: &String,
) -> ClientResult<ContractImage> {
    let tvc = base64::decode(tvc).map_err(|err| Error::invalid_tvc_image(err))?;
    let mut image = ContractImage::from_state_init(&mut tvc.as_slice())
        .map_err(|err| Error::invalid_tvc_image(err))?;

    if let Some(params) = init_params {
        image.update_data(&params.to_string(), abi).map_err(|err| {
            Error::invalid_tvc_image(format!("Failed to set initial data: {}", err))
        })?;
    }

    Ok(image)
}

/// Determines, if public key consists only zeroes, i.e. is empty.
pub(crate) fn is_empty_pubkey(pubkey: &ed25519_dalek::PublicKey) -> bool {
    for b in pubkey.as_bytes() {
        if *b != 0 {
            return false;
        }
    }
    true
}

/// Resolves public key from deploy set, tvc or signer, using this priority:
/// 1. Initial public key from the deploy set
/// 2. Public key from TVC image
/// 3. Signer
/// Returns None, if no public key was resolved.
pub(crate) async fn resolve_pubkey(
    context: &Arc<ClientContext>,
    deploy_set: &Option<(&DeploySet, ContractImage)>,
    signer: &Signer,
) -> ClientResult<Option<String>> {
    if let Some((deploy_set, image)) = deploy_set {
        if deploy_set.initial_pubkey.is_some() {
            return Ok(deploy_set.initial_pubkey.clone());
        }

        let pubkey = match image.get_public_key().map_err(|err| Error::invalid_tvc_image(err))? {
            Some(pub_key) => {
                if is_empty_pubkey(&pub_key) {
                    None
                } else {
                    Some(pub_key)
                }
            },
            None => None,
        };

        if let Some(pubkey) = pubkey {
            return Ok(Some(hex::encode(pubkey.as_ref())));
        }
    }

    signer.resolve_public_key(Arc::clone(context)).await
}

#[cfg(test)]
#[path = "tests/internal.rs"]
mod tests_internal;

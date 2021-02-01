use super::errors::Error;
use super::helpers::build_internal_message;
use super::{BrowserCallbacks, TonClient};
use crate::abi::Signer;
use crate::boc::internal::{deserialize_object_from_base64, serialize_object_to_base64};
use crate::error::ClientResult;
use crate::processing::{
    send_message, wait_for_transaction, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent,
};
use crate::tvm::{run_tvm, ParamsOfRunTvm};
use std::sync::Arc;
use ton_abi::Contract;
use ton_block::Message;
use ton_types::{BuilderData, IBitstring, Cell};
use crate::crypto::{SigningBoxHandle};

pub(super) enum DebotCallType {
    Interface { msg: String, id: String },
    GetMethod { msg: String, dest: String },
    External { msg: String, dest: String },
    // TODO: support later
    // Invoke { msg: String },
}

pub async fn run_get_method(
    ton: TonClient,
    msg: String,
    target_state: String,
    debot_abi: &String,
    debot_addr: &String,
    dest_addr: &String,
) -> ClientResult<String> {
    let (answer_id, _onerror_id, func_id, fixed_msg) = 
        decode_and_fix_ext_msg(ton.clone(), msg, debot_abi, Signer::None).await
            .map_err(|e| Error::get_method_failed(e))?;

    let mut result = run_tvm(
        ton.clone(),
        ParamsOfRunTvm {
            account: target_state,
            message: fixed_msg,
            abi: None,
            execution_options: None,
        },
    )
    .await
    .map_err(|e| Error::get_method_failed(e))?;

    if result.out_messages.len() != 1 {
        return Err(Error::get_method_failed(
            "get-metod returns more than 1 message",
        ));
    }
    let out_msg = result.out_messages.pop().unwrap();
    build_answer_msg(&out_msg, answer_id, func_id, dest_addr, debot_addr)
}

pub async fn send_ext_msg<'a>(
    browser: Arc<dyn BrowserCallbacks + Send + Sync>,
    ton: TonClient,
    msg: String,
    signing_box: SigningBoxHandle,
    _target_state: String,
    debot_abi: &'a String,
    debot_addr: &'a String,
    dest_addr: &'a String,
) -> ClientResult<String> {
    let signer = Signer::SigningBox{handle: signing_box};
    
    let (answer_id, _onerror_id, func_id, fixed_msg) = 
        decode_and_fix_ext_msg(ton.clone(), msg, debot_abi, signer).await
            .map_err(|e| Error::external_call_failed(e))?;

    let browser = browser.clone();
    let callback = move |event| {
        debug!("{:?}", event);
        let browser = browser.clone();
        async move {
            match event {
                ProcessingEvent::WillSend {
                    shard_block_id: _,
                    message_id,
                    message: _,
                } => {
                    browser.log(format!("Sending message {}", message_id)).await;
                }
                _ => (),
            };
        }
    };
    let result = send_message(
        ton.clone(),
        ParamsOfSendMessage {
            message: fixed_msg.clone(),
            abi: None,
            send_events: true,
        },
        callback.clone(),
    )
    .await
    .map(|e| {
        error!("{:?}", e);
        e
    })?;
    let result = wait_for_transaction(
        ton.clone(),
        ParamsOfWaitForTransaction {
            abi: None,
            message: fixed_msg,
            shard_block_id: result.shard_block_id,
            send_events: true,
        },
        callback,
    )
    .await?;

    for out_msg in &result.out_messages {
        if let Ok(answer_msg) =
            build_answer_msg(out_msg, answer_id, func_id, dest_addr, debot_addr)
        {
            return Ok(answer_msg);
        }
    }
    // answer message not found, build empty answer.
    let mut new_body = BuilderData::new();
    new_body.append_u32(answer_id).unwrap();
    build_internal_message(dest_addr, debot_addr, new_body.into())
}

async fn decode_and_fix_ext_msg(
    ton: TonClient,
    msg: String,
    debot_abi: &String,
    signer: Signer,
) -> ClientResult<(u32, u32, u32, String)> {
    let debot_abi = Contract::load(debot_abi.as_bytes()).map_err(|e| Error::invalid_msg(e))?;
    let mut message: Message = deserialize_object_from_base64(&msg, "message")
        .map_err(|e| Error::invalid_msg(e))?
        .object;

    let mut in_body_slice = message.body().ok_or(Error::invalid_msg("empty body"))?;
    let mut pubkey_bit_present = false;
    // skip signature bit (must be 0)
    let sign_bit = in_body_slice.get_next_bit().unwrap();
    if let Signer::SigningBox{handle: _} = signer {
        if !sign_bit {
            return Err(Error::invalid_msg("signature bit is zero"));
        }
        in_body_slice.get_next_bits(512).unwrap();
    }
    let slice_clone = in_body_slice.clone();
    // skip timestamp in miliseconds
    in_body_slice.get_next_u64().unwrap();
    // `expire` is a callback id of debot
    let mut answer_id = in_body_slice.get_next_u32().unwrap();
    // remember function id
    let mut func_id = in_body_slice.get_next_u32().unwrap();

    let result = debot_abi.function_by_id(answer_id, true);
    if result.is_err() {
        debug!("function with answer id not found in debot ABI, second try.");
        in_body_slice = slice_clone;
        // skip pubkey bit (must be 0)
        in_body_slice.get_next_bit().unwrap();
        pubkey_bit_present = true;
        in_body_slice.get_next_u64().unwrap();
        answer_id = in_body_slice.get_next_u32().unwrap();
        func_id = in_body_slice.get_next_u32().unwrap();

        debot_abi.function_by_id(answer_id, true).map_err(|e| {
            error!("answer id not found: {}", e);
            Error::invalid_msg(e)
        })?;
    }

    // rebuild msg body - insert correct `expire` header instead of answerId
    let mut new_body = BuilderData::new();
    
    let pubkey = signer.resolve_public_key(ton.clone()).await?;
    if pubkey_bit_present {
        if let Some(key) = pubkey {
            new_body.append_bit_one().unwrap();
            new_body.append_raw(&hex::decode(&key).unwrap(), 256).unwrap();
        } else {
            // pubkey bit = 0
            new_body.append_bit_zero().unwrap();
        }
    }
    let now = ton.env.now_ms();
    let expired_at = ((now / 1000) as u32) + ton.config.abi.message_expiration_timeout;
    new_body
        .append_u64(now)
        .unwrap()
        .append_u32(expired_at)
        .unwrap()
        .append_u32(func_id)
        .unwrap()
        .append_builder(&BuilderData::from_slice(&in_body_slice))
        .unwrap();

    let mut signed_body = BuilderData::new();
    match signer {
        Signer::SigningBox{handle: _} => {
            let hash = Cell::from(&new_body).repr_hash().as_slice().to_vec();
            let signature = signer.sign(ton.clone(), &hash).await?;
            if let Some(signature) = signature {
                signed_body.append_bit_one().unwrap();
                signed_body.append_raw(&signature, signature.len() * 8).unwrap();
            } else {
                signed_body.append_bit_zero().unwrap();
            }
        },
        _ => {
            signed_body.append_bit_zero().unwrap();
        }
    }

    signed_body.append_builder(&new_body).unwrap();

    message.set_body(signed_body.into());
    let msg =
        serialize_object_to_base64(&message, "message").map_err(|e| Error::invalid_msg(e))?;

    Ok((answer_id, 0, func_id, msg))
}

fn build_answer_msg(
    out_msg: &String,
    answer_id: u32,
    func_id: u32,
    dest_addr: &String,
    debot_addr: &String,
) -> ClientResult<String> {
    let out_message: Message = deserialize_object_from_base64(out_msg, "message")?.object;
    let mut out_body = out_message.body();
    let mut new_body = BuilderData::new();
    new_body.append_u32(answer_id).unwrap();

    if let Some(body_slice) = out_body.as_mut() {
        let response_id = body_slice.get_next_u32().unwrap();
        let request_id = response_id & !(1u32 << 31);
        if func_id != request_id {
            return Err(Error::invalid_msg("incorrect response id"));
        }
        new_body
            .append_builder(&BuilderData::from_slice(&body_slice))
            .unwrap();
    }

    build_internal_message(dest_addr, debot_addr, new_body.into())
}
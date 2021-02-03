use super::errors::Error;
use super::helpers::build_internal_message;
use super::{BrowserCallbacks, TonClient};
use crate::abi::Signer;
use crate::boc::internal::{deserialize_object_from_base64, serialize_object_to_base64};
use crate::crypto::SigningBoxHandle;
use crate::error::{ClientError, ClientResult};
use crate::processing::{
    send_message, wait_for_transaction, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent,
};
use crate::tvm::{run_tvm, ParamsOfRunTvm};
use std::fmt::Display;
use std::sync::Arc;
use ton_abi::Contract;
use ton_block::{Message, MsgAddressExt};
use ton_types::{BuilderData, Cell, IBitstring};

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
        decode_and_fix_ext_msg(ton.clone(), msg, debot_abi, Signer::None)
            .await
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
    let signer = Signer::SigningBox {
        handle: signing_box,
    };

    let (answer_id, onerror_id, func_id, fixed_msg) =
        decode_and_fix_ext_msg(ton.clone(), msg, debot_abi, signer)
            .await
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
    .map(|e| { error!("{:?}", e); e })?;
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
    .await;
    match result {
        Ok(res) => {
            for out_msg in &res.out_messages {
                let res = build_answer_msg(out_msg, answer_id, func_id, dest_addr, debot_addr);
                if let Ok(answer_msg) = res {
                    return Ok(answer_msg);
                }
                debug!("Skip outbound message :{}", res.unwrap_err());
            }
            debug!("Build empty body");
            // answer message not found, build empty answer.
            let mut new_body = BuilderData::new();
            new_body.append_u32(answer_id).map_err(msg_err)?;
            build_internal_message(dest_addr, debot_addr, new_body.into())
        },
        Err(e) => {
            debug!("Transaction failed: {}", e);
            let mut new_body = BuilderData::new();
            new_body.append_u32(onerror_id).map_err(msg_err)?;
            new_body.append_u32(e.code).map_err(msg_err)?;
            let error_code = e.data["exit_code"].as_u64().unwrap_or(0) as u32;
            new_body.append_u32(error_code).map_err(msg_err)?;
            build_internal_message(dest_addr, debot_addr, new_body.into())
        },
    }
}

fn msg_err(e: impl Display) -> ClientError {
    Error::invalid_msg(e)
}

async fn decode_and_fix_ext_msg(
    ton: TonClient,
    msg: String,
    debot_abi: &String,
    signer: Signer,
) -> ClientResult<(u32, u32, u32, String)> {
    let debot_abi = Contract::load(debot_abi.as_bytes()).map_err(msg_err)?;
    let mut message: Message = deserialize_object_from_base64(&msg, "message")
        .map_err(msg_err)?
        .object;

    let src = std::mem::replace(
        &mut message.ext_in_header_mut().ok_or(msg_err("not an external inbound message"))?.src,
        MsgAddressExt::AddrNone
    );
    let (answer_id, onerror_id) = match src {
        MsgAddressExt::AddrNone => return Err(msg_err("src address is empty")),
        MsgAddressExt::AddrExtern(extern_addr) => {
            let mut slice = extern_addr.external_address.clone();
            let abi_ver = slice.get_next_byte().map_err(msg_err)?;
            if abi_ver != 2 {
                return Err(msg_err("invalid ABI version in src address"));
            }
            (
                slice.get_next_u32().map_err(msg_err)?,
                slice.get_next_u32().map_err(msg_err)?
            )
        },
    };

    debot_abi.function_by_id(answer_id, true).map_err(msg_err)?;

    // need to rebuild message body:
    // set correct timestamp (now) and expiration time (default value),
    //
    let mut in_body_slice = message.body().ok_or(msg_err("empty body"))?;
    let mut pubkey_bit_present = false;
    // skip signature bit and signature if present
    let sign_bit = in_body_slice.get_next_bit().map_err(msg_err)?;
    if let Signer::SigningBox { handle: _ } = signer {
        if !sign_bit {
            return Err(msg_err("signature bit is zero"));
        }
        in_body_slice
            .get_next_bits(512)
            .map_err(msg_err)?;
    }
    let slice_clone = in_body_slice.clone();
    // skip timestamp in miliseconds
    in_body_slice.get_next_u64().map_err(msg_err)?;
    // `expire` header is an check id.
    // it must be equal to answer_id from src address.
    // It is used to undestand if body contains pubkey bit (and pubkey itself) or not.
    let mut check_id = in_body_slice.get_next_u32().map_err(msg_err)?;
    // remember function id
    let mut func_id = in_body_slice.get_next_u32().map_err(msg_err)?;

    if answer_id != check_id {
        in_body_slice = slice_clone;
        let pubkey_bit = in_body_slice.get_next_bit().map_err(msg_err)?;
        pubkey_bit_present = true;
        if pubkey_bit {
            in_body_slice
                .get_next_bits(256)
                .map_err(msg_err)?;
        }
        in_body_slice.get_next_u64().map_err(msg_err)?;
        check_id = in_body_slice.get_next_u32().map_err(msg_err)?;
        func_id = in_body_slice.get_next_u32().map_err(msg_err)?;

        if answer_id != check_id {
            let err = "answer id not equal to check id";
            error!("{}", err);
            return Err(msg_err(err));
        }
    }

    // rebuild msg body - insert correct `expire` header instead of answerId
    let mut new_body = BuilderData::new();
    let pubkey = signer.resolve_public_key(ton.clone()).await?;
    if pubkey_bit_present {
        if let Some(key) = pubkey {
            new_body.append_bit_one().map_err(msg_err)?;
            new_body
                .append_raw(&hex::decode(&key).unwrap(), 256)
                .map_err(msg_err)?;
        } else {
            // pubkey bit = 0
            new_body.append_bit_zero().map_err(msg_err)?;
        }
    }
    let now = ton.env.now_ms();
    let expired_at = ((now / 1000) as u32) + ton.config.abi.message_expiration_timeout;
    new_body
        .append_u64(now)
        .and_then(|b| b.append_u32(expired_at))
        .and_then(|b| b.append_u32(func_id))
        .and_then(|b| b.append_builder(&BuilderData::from_slice(&in_body_slice)))
        .map_err(msg_err)?;

    let mut signed_body = BuilderData::new();
    match signer {
        Signer::SigningBox { handle: _ } => {
            let hash = Cell::from(&new_body).repr_hash().as_slice().to_vec();
            let signature = signer.sign(ton.clone(), &hash).await?;
            if let Some(signature) = signature {
                signed_body
                    .append_bit_one()
                    .and_then(|b| b.append_raw(&signature, signature.len() * 8))
                    .map_err(msg_err)?;
            } else {
                signed_body.append_bit_zero().map_err(msg_err)?;
            }
        }
        _ => {
            signed_body.append_bit_zero().map_err(msg_err)?;
        }
    }

    signed_body
        .append_builder(&new_body)
        .map_err(msg_err)?;

    message.set_body(signed_body.into());
    let msg = serialize_object_to_base64(&message, "message").map_err(|e| Error::invalid_msg(e))?;

    Ok((answer_id, onerror_id, func_id, msg))
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
    new_body.append_u32(answer_id).map_err(msg_err)?;

    if let Some(body_slice) = out_body.as_mut() {
        let response_id = body_slice.get_next_u32().map_err(msg_err)?;
        let request_id = response_id & !(1u32 << 31);
        if func_id != request_id {
            return Err(msg_err("incorrect response id"));
        }
        new_body
            .append_builder(&BuilderData::from_slice(&body_slice))
            .map_err(msg_err)?;
    }

    build_internal_message(dest_addr, debot_addr, new_body.into())
}

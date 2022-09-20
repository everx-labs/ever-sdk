use super::errors::Error;
use super::helpers::build_internal_message;
use super::{BrowserCallbacks, DebotActivity, Spending, TonClient};
use crate::abi::Signer;
use crate::boc::internal::{deserialize_object_from_base64, serialize_object_to_base64};
use crate::boc::{get_boc_hash, parse_message, ParamsOfParse, ParamsOfGetBocHash};
use crate::crypto::{KeyPair, SigningBoxHandle, get_signing_box};
use crate::encoding::decode_abi_number;
use crate::error::{ClientError, ClientResult};
use crate::processing::{
    send_message, wait_for_transaction, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent,
};
use crate::tvm::{run_executor, run_tvm, AccountForExecutor, ParamsOfRunExecutor, ParamsOfRunTvm};
use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;
use ton_block::{Message, MsgAddressExt, MsgAddressInt};
use ton_types::{BuilderData, IBitstring, SliceData};
use crate::net::{query_transaction_tree, ParamsOfQueryTransactionTree};

const SUPPORTED_ABI_VERSION: u8 = 2;

pub(super) enum DebotCallType {
    Interface { msg: String, id: String },
    GetMethod { msg: String, dest: String },
    External { msg: String, dest: String },
    Invoke { msg: String },
}

fn msg_err(e: impl Display) -> ClientError {
    Error::invalid_msg(e)
}

#[derive(Default)]
struct Metadata {
    answer_id: u32,
    onerror_id: u32,
    is_timestamp: bool,
    is_expire: bool,
    is_pubkey: bool,
    signing_box_handle: Option<SigningBoxHandle>,
    timestamp: Option<u64>,
    expire: Option<u32>,
}

impl TryFrom<MsgAddressExt> for Metadata {
    type Error = ClientError;

    fn try_from(addr: MsgAddressExt) -> Result<Self, Self::Error> {
        match addr {
            MsgAddressExt::AddrNone => return Err(msg_err("src address is empty")),
            MsgAddressExt::AddrExtern(extern_addr) => {
                // src address contains several metafields describing
                // structure of message body.
                let mut slice = extern_addr.external_address;
                let answer_id = slice.get_next_u32().map_err(msg_err)?;
                let onerror_id = slice.get_next_u32().map_err(msg_err)?;
                let abi_version = slice.get_next_byte().map_err(msg_err)?;
                if abi_version != SUPPORTED_ABI_VERSION {
                    return Err(msg_err(format!(
                        "unsupported ABI version in src address (must be {})",
                        SUPPORTED_ABI_VERSION
                    )));
                }
                let is_timestamp = slice.get_next_bit().map_err(msg_err)?;
                let is_expire = slice.get_next_bit().map_err(msg_err)?;
                let is_pubkey = slice.get_next_bit().map_err(msg_err)?;
                let is_sign_box_handle = slice.get_next_bit().unwrap_or(false);
                let signing_box_handle = if is_sign_box_handle {
                    Some(SigningBoxHandle(slice.get_next_u32().map_err(msg_err)?))
                } else {
                    None
                };

                Ok(Self {
                    answer_id,
                    onerror_id,
                    is_timestamp,
                    is_expire,
                    is_pubkey,
                    signing_box_handle,
                    timestamp: None,
                    expire: None,
                })
            }
        }
    }
}

#[cfg(not(feature = "wasm-base"))]
pub fn prepare_ext_in_message(
    msg: &Message,
    now_ms: u64,
    keypair: Option<KeyPair>,
) -> Result<(u32, u32, u32, MsgAddressInt, Message), String> {
    let config = crate::ClientConfig::default();
    let ton_client = Arc::new(crate::ClientContext::new(config).unwrap());

    let signer = if let Some(keypair) = keypair {
        let future = get_signing_box(ton_client.clone(), keypair);
        let signing_box = ton_client.env.block_on(future).unwrap();
        Signer::SigningBox {
            handle: signing_box.handle.clone(),
        }
    } else {
        Signer::default()
    };

    let hdr = msg.ext_in_header().unwrap();
    let dst_addr: MsgAddressInt = hdr.dst.clone();
    let meta = Metadata::try_from(hdr.src.clone()).unwrap();

    let future =
        decode_and_fix_ext_msg(msg, now_ms, &signer, true, &meta, &ton_client);

    let result = ton_client.env.block_on(future);

    let (func_id, msg) = result.map_err(|e| format!("prepare_ext_in_message: {:?}", e))?;

    Ok((func_id, meta.answer_id, meta.onerror_id, dst_addr, msg))
}

async fn decode_and_fix_ext_msg(
    msg: &Message,
    now_ms: u64,
    signer: &Signer,
    allow_no_signature: bool,
    meta: &Metadata,
    ton: &TonClient
) -> ClientResult<(u32, Message)> {
    // find function id in message body: parse signature, pubkey and abi headers
    let mut message = msg.clone();
    let mut in_body_slice = message.body().ok_or(msg_err("empty body"))?;
    // skip signature bit and signature if present
    let sign_bit = in_body_slice.get_next_bit().map_err(msg_err)?;
    if let Signer::SigningBox { handle: _ } = signer {
        if sign_bit {
            in_body_slice.get_next_bits(512).map_err(msg_err)?;
        } else {
            if !allow_no_signature {
                return Err(msg_err("signature bit is zero"));
            }
        }
    }
    if meta.is_pubkey {
        let pubkey_bit = in_body_slice.get_next_bit().map_err(msg_err)?;
        if pubkey_bit {
            in_body_slice.get_next_bits(256).map_err(msg_err)?;
        }
    }
    if meta.is_timestamp {
        // skip `timestamp` header
        in_body_slice.get_next_u64().map_err(msg_err)?;
    }
    if meta.is_expire {
        // skip `expire` header
        in_body_slice.get_next_u32().map_err(msg_err)?;
    }
    // remember function id
    let func_id = in_body_slice.get_next_u32().map_err(msg_err)?;

    // rebuild msg body - insert correct `timestamp` and `expire` headers if they are present,
    // then sign body with signing box

    let mut new_body = BuilderData::new();
    let pubkey = signer.resolve_public_key(ton.clone()).await?;
    if meta.is_pubkey {
        if let Some(ref key) = pubkey {
            new_body
                .append_bit_one()
                .and_then(|b| b.append_raw(&hex::decode(key).unwrap(), 256))
                .map_err(msg_err)?;
        } else {
            // pubkey bit = 0
            new_body.append_bit_zero().map_err(msg_err)?;
        }
    }
    if meta.is_timestamp ||  meta.timestamp.is_some() {
        let timestamp = match meta.timestamp {
            Some(value) => value,
            None => now_ms,
        };
        new_body.append_u64(timestamp).map_err(msg_err)?;
    }
    if meta.is_expire ||  meta.expire.is_some() {
        let expired_at = match meta.expire {
            Some(value) => value,
            None => ((now_ms / 1000) as u32) + ton.config.abi.message_expiration_timeout,
        };
        new_body.append_u32(expired_at).map_err(msg_err)?;
    }
    new_body
        .append_u32(func_id)
        .and_then(|b| b.append_builder(&BuilderData::from_slice(&in_body_slice)))
        .map_err(msg_err)?;

    let mut signed_body = BuilderData::new();
    match signer {
        Signer::SigningBox { handle: _ } => {
            let hash = new_body.clone().into_cell().map_err(msg_err)?.repr_hash().as_slice().to_vec();
            let signature = signer.sign(ton.clone(), &hash).await?;
            if let Some(signature) = signature {
                signed_body
                    .append_bit_one()
                    .and_then(|b| b.append_raw(&signature, signature.len() * 8))
                    .map_err(msg_err)?;
            } else {
                signed_body.append_bit_zero().map_err(msg_err)?;
            }
        },
        _ => {
            signed_body.append_bit_zero().map_err(msg_err)?;
        }
    }
    signed_body.append_builder(&new_body).map_err(msg_err)?;

    message.set_body(signed_body.into_cell().map_err(msg_err)?.into());
    Ok((func_id, message))
}

pub(crate) struct ContractCall {
    browser: Arc<dyn BrowserCallbacks + Send + Sync>,
    ton: TonClient,
    msg: Message,
    signer: Signer,
    target_state: String,
    debot_addr: String,
    dest_addr: String,
    local_run: bool,
    meta: Metadata,
}

impl ContractCall {
    pub async fn new(
        browser: Arc<dyn BrowserCallbacks + Send + Sync>,
        ton: TonClient,
        msg: String,
        signer: Signer,
        target_state: String,
        debot_addr: String,
        local_run: bool,
    ) -> ClientResult<Self> {
        let mut msg: Message = deserialize_object_from_base64(&msg, "message")
            .map_err(msg_err)?
            .object;
        let meta = get_meta(&mut msg)?;
        let signer = resolve_signer(
            !local_run,
            signer,
            meta.signing_box_handle.clone(),
            browser.clone()
        ).await?;
        let dest_addr = msg
            .header()
            .get_dst_address()
            .map(|x| x.to_string())
            .unwrap_or_default();
        Ok(Self { browser, ton, msg, signer, target_state, debot_addr, dest_addr, local_run, meta })
    }

    pub async fn execute(&self, wait_tx: bool) -> ClientResult<String> {
        let result = self.decode_and_fix_ext_msg()
            .await
            .map_err(|e| Error::external_call_failed(e));
        if let Err(e) = result {
            let error_body = build_onerror_body(self.meta.onerror_id, e)?;
            return build_internal_message(&self.dest_addr, &self.debot_addr, error_body);
        }

        let (func_id, fixed_msg) = result.unwrap();

        if self.local_run {
            self.run_get_method(func_id, fixed_msg).await
        } else {
            self.send_ext_msg(func_id, fixed_msg, wait_tx).await
        }
    }

    pub fn override_message_header(&mut self, timestamp: Option<u64>, expire: Option<u32>)  {
        self.meta.timestamp = timestamp;
        self.meta.expire = expire;
    }

    async fn run_get_method(&self, func_id: u32, fixed_msg: String) -> ClientResult<String> {
        let result = run_tvm(
            self.ton.clone(),
            ParamsOfRunTvm {
                account: self.target_state.clone(),
                message: fixed_msg,
                abi: None,
                execution_options: None,
                boc_cache: None,
                return_updated_account: Some(true),
            },
        )
        .await
        .map_err(|e| Error::get_method_failed(e));

        if let Err(e) = result {
            let error_body = build_onerror_body(self.meta.onerror_id, e)?;
            return build_internal_message(&self.dest_addr, &self.debot_addr, error_body);
        }

        let mut messages = result.unwrap().out_messages;

        if messages.len() != 1 {
            return Err(Error::get_method_failed(
                "get-method returns more than 1 message",
            ));
        }
        let out_msg = messages.pop().unwrap();
        build_answer_msg(&out_msg, self.meta.answer_id, func_id, &self.dest_addr, &self.debot_addr)
            .ok_or(Error::get_method_failed("failed to build answer message"))
    }

    async fn send_ext_msg(&self, func_id: u32, fixed_msg: String, wait_tx: bool) -> ClientResult<String> {
        let activity = emulate_transaction(
            self.ton.clone(),
            self.dest_addr.clone(),
            fixed_msg.clone(),
            self.target_state.clone(),
            self.signer.clone(),
        ).await;
        match activity {
            Ok(activity) => {
                if !self.browser.approve(activity).await? {
                    return self.build_error_answer_msg(Error::operation_rejected());
                }
            },
            Err(e) => {
                return self.build_error_answer_msg(e);
            },
        }

        let browser = self.browser.clone();
        let callback = move |event| {
            debug!("{:?}", event);
            let browser = browser.clone();
            async move {
                match event {
                    ProcessingEvent::WillSend {
                        shard_block_id: _,
                        message_id: _,
                        message: _,
                    } => {
                        browser.log("Sending message...".to_owned()).await;
                    }
                    _ => (),
                };
            }
        };

        let result = send_message(
            self.ton.clone(),
            ParamsOfSendMessage {
                message: fixed_msg.clone(),
                abi: None,
                send_events: true,
            },
            callback.clone(),
        )
        .await
        .map(|e| { error!("{:?}", e); e })?;
        let msg_id = get_boc_hash(self.ton.clone(), ParamsOfGetBocHash { boc: fixed_msg.clone() }).await?.hash;

        if wait_tx {
            let result = wait_for_transaction(
                self.ton.clone(),
                ParamsOfWaitForTransaction {
                    abi: None,
                    message: fixed_msg,
                    shard_block_id: result.shard_block_id,
                    send_events: true,
                    sending_endpoints: Some(result.sending_endpoints),
                },
                callback,
            )
            .await;
            match result {
                Ok(res) => {
                    let result = query_transaction_tree(
                        self.ton.clone(),
                        ParamsOfQueryTransactionTree {
                            in_msg: msg_id,
                            ..Default::default()
                        },
                    ).await;
                    if let Err(e) = result {
                        return self.build_error_answer_msg(e);
                    }
                    for out_msg in &res.out_messages {
                        let res = build_answer_msg(
                            out_msg,
                            self.meta.answer_id,
                            func_id,
                            &self.dest_addr,
                            &self.debot_addr
                        );
                        if let Some(answer_msg) = res {
                            return Ok(answer_msg);
                        }
                        debug!("Skip outbound message");
                    }
                    debug!("Build empty body");
                    // answer message not found, build empty answer.
                    let mut new_body = BuilderData::new();
                    new_body.append_u32(self.meta.answer_id).map_err(msg_err)?;
                    build_internal_message(&self.dest_addr, &self.debot_addr, new_body.into_cell().map_err(msg_err)?.into())
                }
                Err(e) => {
                    debug!("Transaction failed: {:?}", e);
                    self.build_error_answer_msg(e)
                }
            }
        } else {
            let msg_id = hex::decode(msg_id).map_err(msg_err)?;
            let mut new_body = BuilderData::new();
            new_body
                .append_u32(self.meta.answer_id)
                .and_then(|b| b.append_raw(&msg_id, 256))
                .map_err(msg_err)?;
            build_internal_message(&self.dest_addr, &self.debot_addr, new_body.into_cell().map_err(msg_err)?.into())
        }
    }

    async fn decode_and_fix_ext_msg(&self) -> ClientResult<(u32, String)> {
        let now_ms = self.ton.env.now_ms();
        let result: (u32, Message) = decode_and_fix_ext_msg(&self.msg, now_ms, &self.signer, false, &self.meta, &self.ton).await?;
        let (func_id, message) = result;
        let msg = serialize_object_to_base64(&message, "message").map_err(|e| Error::invalid_msg(e))?;
        Ok((func_id, msg))
    }

    fn build_error_answer_msg(&self, e: ClientError) -> ClientResult<String> {
        let error_body = build_onerror_body(self.meta.onerror_id, e)?;
        build_internal_message(&self.dest_addr, &self.debot_addr, error_body)
    }
}

fn build_onerror_body(onerror_id: u32, e: ClientError) -> ClientResult<SliceData> {
    let mut new_body = BuilderData::new();
    new_body.append_u32(onerror_id).map_err(msg_err)?;
    new_body.append_u32(e.code).map_err(msg_err)?;
    let error_code = e
        .data
        .pointer("/local_error/data/exit_code")
        .or(e.data.pointer("/exit_code"))
        .or(e.data.pointer("/compute/exit_code"))
        .and_then(|val| val.as_i64())
        .unwrap_or(0);
    new_body.append_u32(error_code as u32).map_err(msg_err)?;
    Ok(new_body.into_cell().map_err(msg_err)?.into())
}

fn build_answer_msg(
    out_msg: &String,
    answer_id: u32,
    func_id: u32,
    dest_addr: &String,
    debot_addr: &String,
) -> Option<String> {
    let out_message: Message = deserialize_object_from_base64(out_msg, "message").ok()?.object;
    if out_message.is_internal() {
        return None;
    }
    let mut new_body = BuilderData::new();
    new_body.append_u32(answer_id).ok()?;

    if let Some(body_slice) = out_message.body().as_mut() {
        let response_id = body_slice.get_next_u32().ok()?;
        let request_id = response_id & !(1u32 << 31);
        if func_id != request_id {
            return None;
        }
        new_body
            .append_builder(&BuilderData::from_slice(&body_slice))
            .ok()?;
    }

    build_internal_message(dest_addr, debot_addr, new_body.into_cell().ok()?.into()).ok()
}

async fn resolve_signer(
    sign: bool,
    signer: Signer,
    msg_signing_box: Option<SigningBoxHandle>,
    browser: Arc<dyn BrowserCallbacks + Send + Sync>
) -> ClientResult<Signer> {
    let new_signer = if sign {
        match signer {
            Signer::SigningBox {handle: _} => signer,
            _ => Signer::SigningBox {
                handle: match msg_signing_box {
                    Some(signing_box_handle) => signing_box_handle,
                    None => browser.get_signing_box().await
                        .map_err(|e| Error::external_call_failed(e))?,
                },
            },
        }
    } else {
        Signer::None
    };
    Ok(new_signer)
}

fn get_meta(message: &mut Message) -> ClientResult<Metadata> {
    let src = std::mem::replace(
        &mut message
            .ext_in_header_mut()
            .ok_or(msg_err("not an external inbound message"))?
            .src,
        MsgAddressExt::AddrNone,
    );
    Metadata::try_from(src)
}

async fn emulate_transaction(
    client: TonClient,
    dst: String,
    msg: String,
    target_state: String,
    signer: Signer,
) -> ClientResult<DebotActivity> {
    let result = run_executor(
        client.clone(),
        ParamsOfRunExecutor {
            message: msg.clone(),
            account: AccountForExecutor::Account {
                boc: target_state,
                unlimited_balance: None,
            },
            ..Default::default()
        },
    )
    .await?;

    let exit_code = result
        .transaction
        .pointer("/compute/exit_code")
        .and_then(|val| val.as_i64())
        .unwrap_or(0);

    if exit_code != 0 {
        let err = ClientError{
            code: 0,
            message: String::from(""),
            data: result.transaction,
        };
        return Err(err);
    }

    let mut out = vec![];
    for out_msg in result.out_messages {
        let parsed = parse_message(client.clone(), ParamsOfParse { boc: out_msg })
            .await?
            .parsed;
        let msg_type = parsed["msg_type"].as_u64().unwrap();
        // if internal message
        if msg_type == 0 {
            let out_dst = parsed["dst"].as_str().unwrap().to_owned();
            let out_amount = decode_abi_number(parsed["value"].as_str().unwrap())?;
            out.push(Spending {
                dst: out_dst,
                amount: out_amount,
            });
        }
    }

    let (signing_box_handle, signkey) = if let Signer::SigningBox { ref handle } = signer {
        (handle.0, signer.resolve_public_key(client.clone()).await?.unwrap_or_default())
    } else {
        (0, String::new())
    };
    Ok(DebotActivity::Transaction {
        msg: msg.clone(),
        dst: dst.clone(),
        out,
        fee: result.fees.total_account_fees,
        setcode: false,
        signkey,
        signing_box_handle,
    })
}

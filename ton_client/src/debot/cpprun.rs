/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use std::sync::Arc;
use std::str::FromStr;
use serde_json::Value;
use futures::executor::block_on;
use ton_block::{CommonMsgInfo, MsgAddressInt};
use ton_types::{UInt256, BuilderData, IBitstring, Cell};
use ton_abi::token::Tokenizer;
use crate::abi::{Abi, Signer, ParamsOfEncodeMessage, CallSet, DeploySet, decode_body, encode_message};
use crate::tvm::{run_executor, AccountForExecutor, ParamsOfRunExecutor,
    run_tvm, ParamsOfRunTvm, ResultOfRunTvm};
use crate::boc::{parse_account, ParamsOfParse, internal::deserialize_object_from_base64};
use crate::debot::console_abi::CONSOLE_ABI;
use crate::debot::cpp_debot_abi::CPP_DEBOT_ABI;
use crate::debot::cpp_console::CppConsole;
use crate::{ClientContext, ClientConfig};
use crate::crypto::generate_random_sign_keys;
use crate::processing::{get_message_id, Error, wait_for_transaction, ParamsOfWaitForTransaction,
    ProcessingEvent, process_message, ParamsOfProcessMessage, ResultOfProcessMessage};
use crate::processing::blocks_walking::find_last_shard_block;
use crate::encoding::hex_decode;
use ton_types::SliceData;
use crate::boc::internal::serialize_object_to_cell;
use std::fs::File;
use std::io::Write;
use crate::error::{ClientError};
use crate::debot::cpprun::encode_message::calc_timeout;
use std::collections::VecDeque;
use crate::net::{query_collection, ParamsOfQueryCollection, wait_for_collection,
    ParamsOfWaitForCollection, MESSAGES_TABLE_NAME, MAX_TIMEOUT};
use crate::debot::multisig_abi::MSIG_ABI;
use num_traits::ToPrimitive;
use ton_abi::contract::Contract as AbiContract;
use crate::debot::cpp_browser::CppBrowserCallbacks;

fn load_abi(abi: &str) -> Result<Abi, String> {
    Ok(Abi::Contract(
        serde_json::from_str(abi)
            .map_err(|e| format!("failed to parse abi: {}", e))?
    ))
}

fn get_answer_id(msg_json: &Value) -> u32 {
    let answer_id_str = msg_json["_answer_id"].as_str().unwrap();
    let answer_id = u32::from_str_radix(answer_id_str, 10).unwrap();
    return answer_id;
}

async fn request_signer_from_user(browser: &Arc<dyn CppBrowserCallbacks + Send + Sync>) -> Signer {
    browser.log("Debot requests a signature for external message".to_string()).await;
    return Signer::SigningBox { handle: browser.get_signing_box().await.unwrap() };
}

async fn send_ext_message<F: futures::Future<Output = ()> + Send>(
    context: Arc<ClientContext>,
    addr_int: MsgAddressInt,
    ext_msg: ton_block::Message,
    callback: impl Fn(ProcessingEvent) -> F + Send + Sync + 'static,
    func_id: u32,
    answer_id: u32,
    msgs_to_debot: &mut VecDeque<String>
) -> Result<(), String> {
    let message_boc = ton_sdk::Contract::serialize_message(&ext_msg).unwrap().0;
    let message_id = get_message_id(&ext_msg).unwrap();

    let address = ext_msg
        .dst()
        .ok_or(Error::message_has_not_destination_address()).unwrap();

    let shard_block_id = match find_last_shard_block(&context, &address).await {
        Ok(block) => block.to_string(),
        Err(err) => {
            let error = Error::fetch_first_block_failed(err, &message_id);
            return Err(error.to_string());
        }
    };
    context.get_server_link().unwrap()
        .send_message(&hex_decode(&message_id).unwrap(), &message_boc).await
        .map_err(|e| format!("failed to send message: {}", e))?;
    let wait_for = wait_for_transaction(
            context.clone(),
            ParamsOfWaitForTransaction {
                message: base64::encode(&message_boc),
                send_events: false,
                abi: None,
                shard_block_id: shard_block_id.clone(),
            },
            &callback,
        ).await.map_err(|e| format!("failed to wait for transaction: {}", e))?;
    wait_for_messages_tree(context.clone(), &wait_for, 100).await?;
    for msg in wait_for.out_messages.iter() {
        let msg = deserialize_object_from_base64::<ton_block::Message>(&msg, "message").unwrap().object;
        let is_ext_ok = match msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => false,
            CommonMsgInfo::ExtInMsgInfo(_) => false,
            CommonMsgInfo::ExtOutMsgInfo(_) => true
        };
        if is_ext_ok {
            let mut body_read = msg.body().unwrap().clone();
            let func_ret_id = body_read.get_next_u32().unwrap();
            if func_ret_id != (func_id | (1u32 << 31)) {
                continue;
            }
            let mut hdr_bldr = BuilderData::new();
            hdr_bldr.append_u32(answer_id).unwrap();

            let mut builder = BuilderData::from_slice(&body_read);
            builder.prepend_builder(&hdr_bldr).unwrap();

            let answer_body = SliceData::from(builder);
            let msg = ton_sdk::Contract::create_internal_message(
                MsgAddressInt::default(), addr_int, answer_body.clone().into(), 1000000000).unwrap();
            let (body, _) = ton_sdk::Contract::serialize_message(&msg).unwrap();
            msgs_to_debot.push_back(base64::encode(&body));
            return Ok(());
        }
    }
    // If no correct external out message found, sending just "true" in body for void methods
    let mut body_bldr = BuilderData::new();
    body_bldr.append_u32(answer_id).unwrap();
    body_bldr.append_bit_one().unwrap();
    let answer_body = SliceData::from(body_bldr);
    let msg = ton_sdk::Contract::create_internal_message(
        MsgAddressInt::default(), addr_int, answer_body.clone().into(), 1000000000).unwrap();
    let (body, _) = ton_sdk::Contract::serialize_message(&msg).unwrap();
    msgs_to_debot.push_back(base64::encode(&body));

    Ok(())
}

fn load_state(context: Arc<ClientContext>, addr: String) -> Result<String, String> {
    let account_request = block_on(query_collection(
        context,
        ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(serde_json::json!({
                "id": { "eq": addr }
            })),
            result: "boc".to_owned(),
            limit: Some(1),
            order: None,
        },
    ));
    let acc = account_request.map_err(|e| format!("failed to query account: {}", e))?;
    if acc.result.is_empty() {
        return Err(format!(
            "Cannot find account with this address {} in blockchain",
            addr
        ));
    }
    let state = acc.result[0]["boc"].as_str().unwrap().to_owned();
    Ok(state)
}

fn process_getter (
    context: Arc<ClientContext>,
    addr_int: MsgAddressInt,
    msg: &ton_block::Message
) -> Result<String, String> {
    let mut body_read = msg.body().unwrap().clone();
    let has_signature = body_read.get_next_bit().unwrap();
    if has_signature {
        return Err("getter shouldn't have signature or pubkey".to_string());
    }
    let null_timestamp = body_read.get_next_bits(64).unwrap();
    if null_timestamp.iter().any(|b| *b != 0 as u8) {
        return Err("Non-null timestamp in external message".to_string());
    }
    // answer_id in place of expire
    let answer_id = body_read.get_next_u32().unwrap();
    let func_id = body_read.clone().get_next_u32().unwrap();

    let now = context.env.now_ms();
    let config = &context.config.abi;
    let timeout = calc_timeout(
        config.message_expiration_timeout,
        config.message_expiration_timeout_grow_factor,
        0,
        );
    let timestamp_val = now;
    let expire_val = ((now + timeout as u64) / 1000) as u32;

    // prepend none signature, none pubkey, timestamp + expire
    let mut hdr_bldr = BuilderData::new();
    hdr_bldr.append_bit_zero().unwrap();
    hdr_bldr.append_bit_zero().unwrap();
    hdr_bldr.append_u64(timestamp_val).unwrap();
    hdr_bldr.append_u32(expire_val).unwrap();

    let mut builder = BuilderData::from_slice(&body_read);
    builder.prepend_builder(&hdr_bldr).unwrap();

    let new_body = SliceData::from(builder);

    let mut fix_msg = msg.clone();
    fix_msg.set_body(new_body);

    let acc_state = load_state(context.clone(), msg.dst().unwrap().to_string())?;

    let (fix_msg_serialized, _) = ton_sdk::Contract::serialize_message(&fix_msg).unwrap();

    let result = block_on(run_tvm(
        context.clone(),
        ParamsOfRunTvm {
            abi: None,
            account: acc_state,
            message: base64::encode(&fix_msg_serialized),
            execution_options: None,
        }))
        .map_err(|err|{
            let exit_code = &err.data["exit_code"];
            let descr = err.data["description"].as_str().unwrap();
            format!("Getter call error {{ exit_code: {}; description: {} }}", exit_code, descr)
        })?;
    for cur_msg in result.out_messages.iter() {
        let msg = deserialize_object_from_base64::<ton_block::Message>(&cur_msg, "message").unwrap().object;
        let is_ext_ok = match msg.header() {
            CommonMsgInfo::IntMsgInfo(_) => false,
            CommonMsgInfo::ExtInMsgInfo(_) => false,
            CommonMsgInfo::ExtOutMsgInfo(_) => true
        };
        if is_ext_ok {
            let mut body_read = msg.body().unwrap().clone();
            let func_ret_id = body_read.get_next_u32().unwrap();
            if func_ret_id != (func_id | (1u32 << 31)) {
                continue;
            }
            let mut hdr_bldr = BuilderData::new();
            hdr_bldr.append_u32(answer_id).unwrap();

            let mut builder = BuilderData::from_slice(&body_read);
            builder.prepend_builder(&hdr_bldr).unwrap();

            let answer_body = SliceData::from(builder);
            let msg = ton_sdk::Contract::create_internal_message(
                MsgAddressInt::default(), addr_int, answer_body.clone().into(), 1000000000).unwrap();
            let (body, _) = ton_sdk::Contract::serialize_message(&msg).unwrap();
            return Ok(base64::encode(&body));
        }
    };
    Err("No output ".to_string())
}

async fn process_ext_messages(
    context: Arc<ClientContext>,
    browser: &Arc<dyn CppBrowserCallbacks + Send + Sync>,
    addr_int: MsgAddressInt,
    ext_msgs: Vec<ton_block::Message>,
    msgs_to_debot: &mut VecDeque<String>
) -> Result<bool, String> {
    for msg in ext_msgs.iter() {
        if msg.body().is_none() {
            continue;
        }
        let mut body_read = msg.body().unwrap().clone();
        let has_signature = body_read.get_next_bit().unwrap();
        if !has_signature {
            msgs_to_debot.push_back(
                process_getter(context.clone(), addr_int.clone(), msg)?
                );
            continue;
        }
        browser.log("Debot wants to send external message".to_string()).await;
        // we expecting [has_sign:1, signature:512, pubkey:256] in body
        let null_sign = body_read.get_next_bits(512).unwrap();
        if null_sign.iter().any(|b| *b != 0 as u8) {
            continue;
        }
        let has_pubkey = body_read.get_next_bit().unwrap();
        if !has_pubkey {
            continue;
        }
        let null_pubkey = body_read.get_next_bits(256).unwrap();
        if null_pubkey.iter().any(|b| *b != 0 as u8) {
            continue;
        }
        let null_timestamp = body_read.get_next_bits(64).unwrap();
        if null_timestamp.iter().any(|b| *b != 0 as u8) {
            continue;
        }
        // answer_id in place of expire_at
        let answer_id = body_read.get_next_u32().unwrap();
        let func_id = body_read.clone().get_next_u32().unwrap();
        let signer = request_signer_from_user(browser).await;
        let pubkey = block_on(signer.resolve_public_key(context.clone())).unwrap().unwrap();

        let now = context.env.now_ms();
        let config = &context.config.abi;
        let timeout = calc_timeout(
            config.message_expiration_timeout,
            config.message_expiration_timeout_grow_factor,
            0,
            );
        let timestamp_val = now;
        let expire_val = ((now + timeout as u64) / 1000) as u32;

        // prepend pubkey + timestamp + expire
        let mut hdr_bldr = BuilderData::new();
        hdr_bldr.append_bit_one().unwrap();
        hdr_bldr.append_raw(&hex_decode(&pubkey).unwrap(), 256).unwrap();
        hdr_bldr.append_u64(timestamp_val).unwrap();
        hdr_bldr.append_u32(expire_val).unwrap();

        let mut builder = BuilderData::from_slice(&body_read);
        builder.prepend_builder(&hdr_bldr).unwrap();

        let hash = Cell::from(&builder).repr_hash().as_slice().to_vec();

        let signature = block_on(signer.sign(context.clone(), &hash)).unwrap().unwrap();

        let mut sign_builder = BuilderData::new();
        sign_builder.append_bit_one().unwrap();
        sign_builder.append_raw(&signature, signature.len() * 8).unwrap();

        builder.prepend_builder(&sign_builder).unwrap();

        let new_body = SliceData::from(builder);

        let mut fix_msg = msg.clone();
        fix_msg.set_body(new_body);
        let callback = move |_: ProcessingEvent| {
            futures::future::ready(())
        };
        send_ext_message(context.clone(), addr_int.clone(),
            fix_msg, callback, func_id, answer_id, msgs_to_debot).await?;
    }
    Ok(true)
}

struct MultisigProxy {
    pub address: String,
    pub signer: Signer
}

pub(crate) fn get_acc_balance(context: Arc<ClientContext>, addr: &String) -> u64  {
    let result = block_on(crate::net::query_collection(
        context,
        crate::net::ParamsOfQueryCollection {
            collection: "accounts".to_owned(),
            filter: Some(serde_json::json!({
                "id": { "eq": addr }
            })),
            limit: None,
            order: None,
            result: "balance".to_string(),
        },
    )).unwrap();
    if result.result.is_empty() {
      return 0;
    }
    let balance_str = result.result[0]["balance"].as_str().unwrap().trim_start_matches("0x");
    let balance = u64::from_str_radix(balance_str, 16).unwrap();
    return balance;
}

async fn request_multisig_proxy(
    context: Arc<ClientContext>,
    browser: &Arc<dyn CppBrowserCallbacks + Send + Sync>,
    proxy: &mut Option<MultisigProxy>
) -> Result<bool, String> {
    if proxy.is_some() {
        let balance = get_acc_balance(context.clone(), &proxy.as_ref().unwrap().address);
        let use_existing = browser.input_yes_or_no(
            format!("Use selected multisig proxy ({}, balance = {:.2}T), y/n?",
                proxy.as_ref().unwrap().address.clone(), (balance as f64) / 1000000000.0f64)).await;
        if use_existing.is_none() { return Ok(false); }
        if !use_existing.unwrap() {
            *proxy = None;
        }
    }

    if proxy.is_none() {
        let addr = browser.input_address("Enter your multisig proxy address:".to_string()).await;
        if addr.is_none() { return Ok(false); }
        let addr_str = addr.unwrap().to_string();
        let signer = request_signer_from_user(browser).await;
        *proxy = Some(MultisigProxy{address: addr_str, signer: signer});
    }
    Ok(true)
}

async fn wait_for_messages_tree(
    context: Arc<ClientContext>, result: &ResultOfProcessMessage, mut max_count: u32
) -> Result<(), String> {
    let finalized = ton_sdk::json_helper::transaction_status_to_u8(
        ton_block::TransactionProcessingStatus::Finalized);

    let mut out_msgs = result.out_messages.clone();
    while !out_msgs.is_empty() && max_count > 0 {
        max_count = max_count - 1;
        let mut next_out_msgs: Vec<String> = Vec::new();
        for msg in out_msgs.iter() {
            let msg = deserialize_object_from_base64::<ton_block::Message>(&msg, "message").unwrap().object;
            if !msg.is_internal() {
                continue;
            }
            let next_result = wait_for_collection(context.clone(),
                ParamsOfWaitForCollection {
                    collection: "transactions".to_owned(),
                    filter: Some(json!({
                        "in_msg": { "eq": get_message_id(&msg).unwrap() },
                        "status": { "eq": finalized }
                    })),
                    result: "id out_messages { boc }".to_owned(),
                    timeout: Some(context.config.network.wait_for_timeout),
                }).await.map_err(|err| { format!("Wait tree error: {}", err) })?.result;
            next_out_msgs.extend(
                next_result["out_messages"].as_array().unwrap().iter()
                    .map(|x| x["boc"].as_str().unwrap().to_owned() ));
        }
        out_msgs = next_out_msgs;
    }
    Ok(())
}

async fn process_int_messages(
    context: Arc<ClientContext>,
    browser: &Arc<dyn CppBrowserCallbacks + Send + Sync>,
    _addr_int: MsgAddressInt,
    int_msgs: Vec<ton_block::Message>,
    proxy: &mut Option<MultisigProxy>,
    msgs_to_debot: &mut VecDeque<String>
) -> Result<bool, String> {
    for msg in int_msgs.iter() {
        if msg.body().is_none() {
            continue;
        }
        let dest = msg.dst().unwrap();
        let value = msg.get_value().unwrap().grams.value();
        browser.log(format!("Debot wants to send internal message to dest={}, with value={:.2}T",
            dest.to_string(), (value.to_u64().unwrap() as f64) / 1000000000.0f64)).await;

        if !request_multisig_proxy(context.clone(), browser, proxy).await? {
            return Ok(false);
        }
        let address_val = proxy.as_ref().unwrap().address.clone();
        let signer_val = proxy.as_ref().unwrap().signer.clone();
        let mut msg_body_sl = msg.body().unwrap().clone();

        let _func_id = msg_body_sl.get_next_u32().unwrap();
        let _answer_id = msg_body_sl.get_next_u32().unwrap();
        let payload_val = base64::encode(&ton_types::serialize_toc(&msg.body_as_cell()).unwrap());

        let now = ton_sdk::Contract::now();

        let args = json!({
            "dest": dest.to_string(),
            "value": value.to_u64(), // TODO: fix i128->u64 truncation
            "bounce": true,
            "flags": 3,
            "payload": payload_val
        });
        let call_params =
            ParamsOfEncodeMessage {
                abi: load_abi(MSIG_ABI).unwrap(),
                call_set: CallSet::some_with_function_and_input("sendTransaction", args),
                signer: signer_val,
                address: Some(address_val.clone()),
                deploy_set: None,
                processing_try_index: None,
            };
        let callback = move |_: ProcessingEvent| {
            futures::future::ready(())
        };
        let result = process_message(
            context.clone(),
            ParamsOfProcessMessage {
                message_encode_params: call_params,
                send_events: false,
            },
            callback
        ).await.map_err(|err| format!("process internal message error: {}", err))?;
        wait_for_messages_tree(context.clone(), &result, 100).await?;

        // TODO: verify that message func_id is equal to answer_id
        // TODO: maybe override message dst to debot address (addr_int)
        //   (run_tvm message processing currently works ok even with wrong dst)
        let answer_message = wait_for_collection(
            context.clone(),
            ParamsOfWaitForCollection {
                collection: MESSAGES_TABLE_NAME.into(),
                filter: Some(json!({
                    "dst": { "eq": address_val.clone() },
                    "status": { "eq": 5 },
                    "msg_type": { "eq": 0 },
                    "created_at": { "ge": now },
                    "src" : { "eq": dest.to_string() }
                })),
                result: "id dst boc msg_type".into(),
                timeout: Some(MAX_TIMEOUT), // Some(context.config.network.wait_for_timeout)
            },
        ).await.map_err(|err| format!("error while waiting for proxy answer: {}", err))?.result;
        assert_eq!(answer_message["dst"], address_val.clone());
        assert_eq!(answer_message["msg_type"], 0);
        let answer_msg_boc = answer_message["boc"].as_str().unwrap();
        msgs_to_debot.push_back(answer_msg_boc.to_string());
    }
    Ok(true)
}

struct HomeState {
    pub account_state: String,
    pub message: String
}

async fn process_console_messages(
    context: Arc<ClientContext>,
    browser: &Arc<dyn CppBrowserCallbacks + Send + Sync>,
    cabi: &AbiContract,
    addr_int: MsgAddressInt,
    cur_account: &String,
    console_msgs: &Vec<ton_block::Message>,
    msgs_to_debot: &mut VecDeque<String>,
    last_home: &mut HomeState
) -> Result<bool, String> {
    for msg in console_msgs {
        let decoded = decode_body(cabi.clone(), msg.body().unwrap(), true).unwrap();
        let msg_json = decoded.value.unwrap();
        let answer_id = get_answer_id(&msg_json);
        let browser = browser.clone();

        let ret_val = match decoded.name.as_str() {
            "print"              => CppConsole::print(browser, context.clone(), &msg_json).await,
            "printf"             => CppConsole::printf(browser, context.clone(), &msg_json).await,
            "inputStr"           => CppConsole::input(browser, context.clone(), &msg_json).await,
            "inputAddress"       => CppConsole::input_address(browser, context.clone(), &msg_json).await,
            "inputUint256"       => CppConsole::input_uint256(browser, context.clone(), &msg_json).await,
            "inputPubkey"        => CppConsole::input_pubkey(browser, context.clone(), &msg_json).await,
            "inputTONs"          => CppConsole::input_tons(browser, context.clone(), &msg_json).await,
            "inputYesOrNo"       => CppConsole::input_yes_or_no(browser, context.clone(), &msg_json).await,
            "inputDateTime"      => CppConsole::input_datetime(browser, context.clone(), &msg_json).await,
            "inputCell"          => CppConsole::input_cell(browser, context.clone(), &msg_json).await,
            "inputDeployMessage" => CppConsole::input_deploy_message(browser, context.clone(), &msg_json).await,
            "iAmHome"            => CppConsole::i_am_home(cur_account, &mut last_home.account_state).await,
            &_ => unimplemented!()
        };
        if ret_val.is_none() { return Ok(false); }
        let ret_val = ret_val.unwrap();

        let v = serde_json::from_str(&ret_val).unwrap();
        let func_abi = cabi.function(&decoded.name).unwrap();
        let output_tokens = Tokenizer::tokenize_all_params(func_abi.output_params(), &v).unwrap();
        let msg_body: BuilderData = func_abi.encode_internal_output(answer_id, &output_tokens).unwrap();
        let answer_msg = ton_sdk::Contract::create_internal_message(
            MsgAddressInt::default(), addr_int.clone(), msg_body.clone().into(), 1000000000).unwrap();
        let (answer_msg_body, _) = ton_sdk::Contract::serialize_message(&answer_msg).unwrap();

        let encoded_message = base64::encode(&answer_msg_body);
        msgs_to_debot.push_back(encoded_message.clone());
        if decoded.name.as_str() == "iAmHome" {
            last_home.message = encoded_message;
        }
    }
    Ok(true)
}

fn process_run_error(crash_dumps: bool, _err: &ClientError, account_base64: String, message_code: &String) {
    if !crash_dumps {
        return;
    }
    let msg = deserialize_object_from_base64::<ton_block::Message>(message_code, "message").unwrap().object;
    let message_body = msg.body().unwrap();

    let account = deserialize_object_from_base64(&account_base64, "account").unwrap().object;

    let stuff = match account {
        ton_block::Account::AccountNone => Err("Acount is None"),
        ton_block::Account::Account(stuff) => Ok(stuff),
    }.unwrap();
    let addr = format!("{:x}", stuff.addr.address());

    let init = match stuff.storage.state {
        ton_block::AccountState::AccountUninit => Err("unexpected AccountUninit"),
        ton_block::AccountState::AccountActive(state) => Ok(state),
        ton_block::AccountState::AccountFrozen(_) => Err("unexpected AccountFrozen")
    }.unwrap();

    let cell = serialize_object_to_cell::<ton_block::StateInit>(&init, "StateInit").unwrap();
    let bytes = ton_types::cells_serialization::serialize_toc(&cell).unwrap();
    std::fs::create_dir_all("core.dump").unwrap();

    let mut file = File::create(format!("core.dump/{}.tvc", addr)).unwrap();
    file.write_all(&bytes).unwrap();
    let mut msg_file = File::create("core.dump/core.dump.msg").unwrap();
    let msg_str = hex::encode(message_body.get_bytestring(0));
    msg_file.write_all(msg_str.as_bytes()).unwrap();
}

pub async fn cpprun_exec(
    network_client: Arc<ClientContext>,
    browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
    tvc_path: String,
    core_dumps: bool
) -> Result<(), String> {
    let client_impl = ClientContext::new(ClientConfig::default())
        .map_err(|e| format!("failed to create local tonclient: {}", e))?;
    let client = Arc::new(client_impl);
    let abi = load_abi(CPP_DEBOT_ABI).unwrap();

    let mut cur_account: String;
    let addr_int: MsgAddressInt;
    let addr_parsed = MsgAddressInt::from_str(&tvc_path.trim());
    if addr_parsed.is_ok() {
        addr_int = addr_parsed.unwrap();
        cur_account = load_state(network_client.clone(), addr_int.to_string()).unwrap();
    } else {
        let keys = generate_random_sign_keys(client.clone()).unwrap();
        let tvc = base64::encode(
            &std::fs::read(tvc_path.clone()).unwrap(),
            );
        let message = block_on(encode_message(
            client.clone(),
            ParamsOfEncodeMessage {
                abi: abi.clone(),
                address: None,
                call_set: Some(CallSet {
                    function_name: "constructor".to_owned(),
                    header: None,
                    input: None,
                }),
                deploy_set: Some(DeploySet {
                    initial_data: None,
                    tvc,
                    workchain_id: None,
                    initial_pubkey: None
                }),
                processing_try_index: None,
                signer: Signer::Keys { keys: keys.clone() },
            }))
            .unwrap();

        let result = block_on(run_executor(
            client.clone(),
            ParamsOfRunExecutor {
                abi: None,
                account: AccountForExecutor::Uninit,
                execution_options: None,
                message: message.message.to_owned(),
                skip_transaction_check: None,
            },
        )).unwrap();
        let acc = parse_account(
            client.clone(),
            ParamsOfParse {
                boc: result.account.clone(),
            }
        ).unwrap().parsed;

        assert_eq!(acc["id"], message.address);
        assert_eq!(acc["acc_type_name"], "Active");

        addr_int = MsgAddressInt::from_str(&message.address).unwrap();
        cur_account = result.account;
    }

    let mut proxy: Option<MultisigProxy> = None;

    let filter_messages = |res: &ResultOfRunTvm| {
        let mut ext_msgs = Vec::new();
        let mut int_msgs = Vec::new();
        let mut console_msgs = Vec::new();
        for msg in res.out_messages.iter() {
            let msg = deserialize_object_from_base64::<ton_block::Message>(&msg, "message").unwrap().object;
            match msg.header() {
                CommonMsgInfo::IntMsgInfo(hdr) => match hdr.dst {
                    MsgAddressInt::AddrStd(ref std) => 
                        if UInt256::default() == std.address && std.workchain_id == 0 { console_msgs.push(msg); }
                        else { int_msgs.push(msg); },
                    MsgAddressInt::AddrVar(ref _var) => ()
                },
                CommonMsgInfo::ExtInMsgInfo(_) => ext_msgs.push(msg),
                CommonMsgInfo::ExtOutMsgInfo(_) => ()
            };
        };
        return (ext_msgs, int_msgs, console_msgs);
    };

    let cabi = load_abi(CONSOLE_ABI).unwrap();
    let cabi = cabi.json_string().unwrap();
    let cabi: AbiContract = AbiContract::load(cabi.as_bytes()).unwrap();

    // Creating incoming messages queue with initial "start" message
    let start_message = block_on(encode_message(
        client.clone(),
        ParamsOfEncodeMessage {
            abi: abi.clone(),
            call_set: CallSet::some_with_function("start"),
            signer: Signer::None,
            address: Some(addr_int.to_string()),
            deploy_set: None,
            processing_try_index: None,
        }))
        .unwrap();
    let mut msgs_to_debot = VecDeque::new();
    let mut last_home = HomeState { account_state: cur_account.clone(), message: start_message.message.clone() };
    msgs_to_debot.push_back(start_message.message.clone());
    while !msgs_to_debot.is_empty() {
        let debot_msg = msgs_to_debot.pop_front().unwrap();
        let result = block_on(run_tvm(
            client.clone(),
            ParamsOfRunTvm {
                abi: Some(abi.clone()),
                account: cur_account.clone(),
                message: debot_msg.clone(),
                execution_options: None,
            }))
            .map_err(|err| {
                process_run_error(core_dumps, &err, cur_account, &debot_msg);
                format!("Error executing debot: {}", err) })?;
        cur_account = result.account.clone();

        let (ext_msgs, int_msgs, console_msgs) = filter_messages(&result);
        if !process_ext_messages(network_client.clone(), &browser, addr_int.clone(),
            ext_msgs, &mut msgs_to_debot).await? {
            cur_account = last_home.account_state.clone();
            msgs_to_debot.clear();
            msgs_to_debot.push_back(last_home.message.clone());
            browser.log("".to_string()).await;
        }
        if !process_console_messages(network_client.clone(), &browser, &cabi, addr_int.clone(),
            &cur_account, &console_msgs, &mut msgs_to_debot, &mut last_home).await? {
            cur_account = last_home.account_state.clone();
            msgs_to_debot.clear();
            msgs_to_debot.push_back(last_home.message.clone());
            browser.log("".to_string()).await;
        }
        if !process_int_messages(network_client.clone(), &browser, addr_int.clone(),
            int_msgs, &mut proxy, &mut msgs_to_debot).await? {
            cur_account = last_home.account_state.clone();
            msgs_to_debot.clear();
            msgs_to_debot.push_back(last_home.message.clone());
            browser.log("".to_string()).await;
        }
    }
    Ok(())
}

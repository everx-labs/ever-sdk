use ton_sdk::{Contract, Message, MessageType};
use crypto::keys::{KeyPair, u256_encode, account_decode};
use types::{ApiResult, ApiError, base64_decode};

use ton_sdk::Transaction;
use tvm::block::{TransactionProcessingStatus, TransactionId};
use tvm::types::AccountId;
use ed25519_dalek::Keypair;
use futures::Stream;
use tvm::block::TrComputePhase::*;
use contracts::{EncodedMessage, EncodedUnsignedMessage};
use client::ClientContext;


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfRun {
    pub address: String,
    pub abi: serde_json::Value,
    pub functionName: String,
    pub input: serde_json::Value,
    pub keyPair: Option<KeyPair>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfLocalRun {
    pub address: String,
    pub account: Option<serde_json::Value>,
    pub abi: serde_json::Value,
    pub functionName: String,
    pub input: serde_json::Value,
    pub keyPair: Option<KeyPair>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeUnsignedRunMessage {
    pub address: String,
    pub abi: serde_json::Value,
    pub functionName: String,
    pub input: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfDecodeRunOutput {
    pub abi: serde_json::Value,
    pub functionName: String,
    pub bodyBase64: String,
}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfRun {
    pub output: serde_json::Value
}

pub(crate) fn run(_context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<ResultOfRun> {
    debug!("-> contracts.run({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(ref keys) = params.keyPair { Some(keys.decode()?) } else { None };

    debug!("load contract");
    let contract = load_contract(&address)?;

    debug!("run contract");
    let tr_id = call_contract(&contract.id(), &params, key_pair.as_ref())?;
    let tr_id_hex = tr_id.to_hex_string();

    debug!("load transaction {}", tr_id_hex);
    let tr = load_transaction(&tr_id);

    if tr.out_messages_id().len() == 0 {
        debug!("out messages missing");
        let block_transaction = tr.tr();
        debug!("block transaction: {}", serde_json::to_string(block_transaction).unwrap());
        get_result_from_block_transaction(&block_transaction)
    } else {
        debug!("load out messages");
        let out_msg = load_out_message(&tr);
        let response = out_msg.body().expect("error unwrap out message body").into();

        debug!("decode output");
        let result = Contract::decode_function_response_json(
            params.abi.to_string().to_owned(),
            params.functionName.to_owned(),
            response)
            .expect("Error decoding result");

        debug!("<-");
        Ok(ResultOfRun {
            output: serde_json::from_str(result.as_str())
                .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
        })
    }
}

pub(crate) fn local_run(_context: &mut ClientContext, params: ParamsOfLocalRun) -> ApiResult<ResultOfRun> {
    debug!("-> contracts.run.local({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let _address = account_decode(&params.address)?;

    let key_pair = match params.keyPair {
        None => None,
        Some(pair) => Some(pair.decode()?)
    };

    let contract = match params.account {
        // load contract data from node manually
        #[cfg(feature = "node_interaction")]
        None => {
            debug!("load contract");
            load_contract(&address)?
        }
        // can't load
        #[cfg(not(feature = "node_interaction"))]
        None => {
            debug!("no account provided");
            return Err(ApiError::contracts_run_contract_not_found());
        }

        Some(account) => {
            Contract::from_json(&account.to_string())
                .map_err(|err| ApiError::invalid_params(&account.to_string(), err))?
        }
    };

    let messages = contract.local_call_json(
        params.functionName.clone(),
        params.input.to_string(),
        params.abi.to_string(),
        key_pair.as_ref())
        .expect("Error calling locally");

    for msg in messages {
        let msg = Message::with_msg(msg);
        if msg.msg_type() == MessageType::ExternalOutbound {
            let output = Contract::decode_function_response_json(
                params.abi.to_string(), params.functionName, msg.body().expect("Message has no body"))
                .expect("Error decoding result");

            let output: serde_json::Value = serde_json::from_str(&output)
                .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;

            return Ok(ResultOfRun { output });
        }
    }

    return Ok(ResultOfRun { output: serde_json::Value::default() });
}

pub(crate) fn encode_message(_context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<EncodedMessage> {
    debug!("-> contracts.run.message({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(keys) = params.keyPair { Some(keys.decode()?) } else { None };

    let (body, id) = Contract::construct_call_message_json(
        ton_sdk::AccountAddress::Short(address),
        params.functionName.to_owned(),
        params.input.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        key_pair.as_ref())
        .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;

    debug!("<-");
    Ok(EncodedMessage {
        messageId: u256_encode(&id),
        messageIdBase64: base64::encode(id.as_slice()),
        messageBodyBase64: base64::encode(&body),
    })
}

pub(crate) fn encode_unsigned_message(_context: &mut ClientContext, params: ParamsOfEncodeUnsignedRunMessage) -> ApiResult<EncodedUnsignedMessage> {
    let encoded = ton_sdk::Contract::get_call_message_bytes_for_signing(
        ton_sdk::AccountAddress::Short(account_decode(&params.address)?),
        params.functionName,
        params.input.to_string(),
        params.abi.to_string(),
    ).map_err(|err| ApiError::contracts_create_run_message_failed(err))?;
    Ok(EncodedUnsignedMessage {
        unsignedBytesBase64: base64::encode(&encoded.message),
        bytesToSignBase64: base64::encode(&encoded.data_to_sign),
    })
}

pub(crate) fn decode_output(_context: &mut ClientContext, params: ParamsOfDecodeRunOutput) -> ApiResult<ResultOfRun> {
    let body = base64_decode(&params.bodyBase64)?;
    let result = Contract::decode_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        params.functionName.to_owned(),
        &body).map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfRun {
        output: serde_json::from_str(result.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
    })
}

// Internals

fn ok_null() -> ApiResult<ResultOfRun> {
    Ok(ResultOfRun {
        output: serde_json::Value::Null
    })
}


fn get_result_from_block_transaction(transaction: &tvm::block::Transaction) -> ApiResult<ResultOfRun> {
    match transaction.compute_phase_ref() {
        Some(compute_phase) => {
            match compute_phase {
                Skipped(skipped) => {
                    debug!("VM compute phase was skipped");
                    let reason: u8 = skipped.reason.clone() as u8;
                    Err(ApiError::tvm_execution_skipped(reason))
                }
                Vm(vm) => {
                    if vm.success {
                        debug!("VM compute phase was succeeded");
                        ok_null()
                    } else {
                        debug!("VM compute phase was not succeeded");
                        Err(ApiError::tvm_execution_failed(vm.exit_code))
                    }
                }
            }
        }
        None => {
            debug!("VM compute phase have missing!");
            ok_null()
        }
    }
}

fn load_transaction(id: &TransactionId) -> Transaction {
    Transaction::load(id.clone())
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction")
}

fn load_out_message(tr: &Transaction) -> Message {
    tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                .expect("error unwrap out message 2")
                .msg_type() == MessageType::ExternalOutbound
        })
        .expect("error unwrap out message 2")
        .expect("error unwrap out message 3")
        .expect("error unwrap out message 4")
}

fn load_contract(address: &AccountId) -> ApiResult<Contract> {
    Contract::load(address.clone().into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .ok_or(ApiError::contracts_run_contract_not_found())
}

fn call_contract(
    address: &AccountId,
    params: &ParamsOfRun,
    key_pair: Option<&Keypair>,
) -> ApiResult<TransactionId> {
    let changes_stream = Contract::call_json(
        address.clone().into(),
        params.functionName.to_owned(),
        params.input.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        key_pair)
        .expect("Error calling contract method");

    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(state) = state {
            debug!("run: {:?}", state.status);
            if state.status == TransactionProcessingStatus::Preliminary ||
                state.status == TransactionProcessingStatus::Proposed ||
                state.status == TransactionProcessingStatus::Finalized
            {
                tr_id = Some(state.id.clone());
                break;
            }
        }
    }
    tr_id.ok_or(ApiError::contracts_run_transaction_missing())
}

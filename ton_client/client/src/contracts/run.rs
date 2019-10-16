use ton_sdk::{Contract, Message, MessageType, AbiContract};
use tvm::block::MsgAddressInt;
use crypto::keys::{KeyPair, u256_encode, account_decode};
use types::{ApiResult, ApiError, base64_decode};

use contracts::{EncodedMessage, EncodedUnsignedMessage};
use client::ClientContext;

#[cfg(feature = "node_interaction")]
use ton_sdk::{Transaction, AbiFunction};
#[cfg(feature = "node_interaction")]
use tvm::block::{TransactionProcessingStatus, TransactionId};
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use futures::Stream;
#[cfg(feature = "node_interaction")]
use tvm::block::TrComputePhase::*;

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

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ParamsOfDecodeUnknownRun {
    pub abi: serde_json::Value,
    pub bodyBase64: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfRun {
    pub output: serde_json::Value
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ResultOfDecodeUnknownRun {
    pub function: String,
    pub output: serde_json::Value
}

#[cfg(feature = "node_interaction")]
pub(crate) fn run(_context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<ResultOfRun> {
    debug!("-> contracts.run({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(ref keys) = params.keyPair { Some(keys.decode()?) } else { None };

    debug!("run contract");
    let tr_id = call_contract(address.get_msg_address(), &params, key_pair.as_ref())?;
    let tr_id_hex = tr_id.to_hex_string();

    debug!("load transaction {}", tr_id_hex);
    let tr = load_transaction(&tr_id);

    let abi_contract = AbiContract::load(params.abi.to_string().as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(&params.functionName).expect("Couldn't find function");

    if  tr.out_messages_id().len() == 0 ||
        !abi_function.has_output()
    {
        debug!("out messages missing");
        let block_transaction = tr.tr();
        debug!("block transaction: {}", serde_json::to_string(block_transaction).unwrap());
        get_result_from_block_transaction(&block_transaction)
    } else {
        debug!("load out messages");
        let out_msg = load_out_message(&tr, abi_function);
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

    let address = account_decode(&params.address)?;

    let key_pair = match params.keyPair {
        None => None,
        Some(pair) => Some(pair.decode()?)
    };

    let contract = match params.account {
        // load contract data from node manually
        #[cfg(feature = "node_interaction")]
        None => {
            debug!("load contract");
            load_contract(&address.get_msg_address())?
        }
        // can't load
        #[cfg(not(feature = "node_interaction"))]
        None => {
            debug!("no account provided");
            let _address = address;
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

    let abi_contract = AbiContract::load(params.abi.to_string().as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(&params.functionName).expect("Couldn't find function");

    for msg in messages {
        let msg = Message::with_msg(msg);
        if msg.msg_type() == MessageType::ExternalOutbound &&
            abi_function.is_my_message(msg.body().expect("Message has no body")).expect("Error is_my_message")
        {
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
        address,
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
        account_decode(&params.address)?,
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

pub(crate) fn decode_unknown_input(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.bodyBase64)?;
    let result = Contract::decode_unknown_function_call_from_bytes_json(
        params.abi.to_string().to_owned(),
        &body).map_err(|err|ApiError::contracts_decode_run_input_failed(err))?;
    Ok(ResultOfDecodeUnknownRun {
        function: result.function_name,
        output: serde_json::from_str(result.params.as_str())
            .map_err(|err| ApiError::contracts_decode_run_input_failed(err))?
    })
}

pub(crate) fn decode_unknown_output(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.bodyBase64)?;
    let result = Contract::decode_unknown_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        &body).map_err(|err|ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfDecodeUnknownRun {
        function: result.function_name,
        output: serde_json::from_str(result.params.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
    })
}

// Internals
#[cfg(feature = "node_interaction")]
fn ok_null() -> ApiResult<ResultOfRun> {
    Ok(ResultOfRun {
        output: serde_json::Value::Null
    })
}

#[cfg(feature = "node_interaction")]
pub(crate) fn get_result_from_block_transaction(transaction: &tvm::block::Transaction) -> ApiResult<ResultOfRun> {
    match transaction.compute_phase_ref() {
        Some(compute_phase) => {
            match compute_phase {
                Skipped(skipped) => {
                    debug!("VM compute phase was skipped");
                    let reason = format!("{:?}", skipped.reason.clone());
                    Err(ApiError::tvm_execution_skipped(&reason))
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

#[cfg(feature = "node_interaction")]
pub(crate) fn load_transaction(id: &TransactionId) -> Transaction {
    Transaction::load(id.clone())
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction")
}

#[cfg(feature = "node_interaction")]
fn load_out_message(tr: &Transaction, abi_function: &AbiFunction) -> Message {
    tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            let msg = msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                    .expect("error unwrap out message 2");
            msg.msg_type() == MessageType::ExternalOutbound
            && msg.body().is_some()
            && abi_function.is_my_message(msg.body().expect("No body")).expect("error is_my_message")
        })
        .expect("error unwrap out message 3")
        .expect("error unwrap out message 4")
        .expect("error unwrap out message 5")
}

#[cfg(feature = "node_interaction")]
fn load_contract(address: &MsgAddressInt) -> ApiResult<Contract> {
    Contract::load(address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .ok_or(ApiError::contracts_run_contract_not_found())
}

#[cfg(feature = "node_interaction")]
fn call_contract(
    address: MsgAddressInt,
    params: &ParamsOfRun,
    key_pair: Option<&Keypair>,
) -> ApiResult<TransactionId> {
    let changes_stream = Contract::call_json(
        address,
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

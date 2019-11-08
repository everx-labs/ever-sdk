use ton_sdk::{Contract, MessageType, AbiContract};
use ton_sdk::json_abi::encode_function_call;
use crypto::keys::{KeyPair, account_decode};
use types::{ApiResult, ApiError, base64_decode};
use tvm::cells_serialization::BagOfCells;

use contracts::{EncodedMessage, EncodedUnsignedMessage};
use client::ClientContext;

#[cfg(feature = "node_interaction")]
use ton_sdk::{Transaction, AbiFunction, Message};
#[cfg(feature = "node_interaction")]
use tvm::block::{TransactionProcessingStatus, MsgAddressInt, AccStatusChange};
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use futures::Stream;

fn bool_false() -> bool { false }

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
    #[serde(default = "bool_false")]
    pub internal: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ParamsOfDecodeUnknownRun {
    pub abi: serde_json::Value,
    pub bodyBase64: String,
    #[serde(default = "bool_false")]
    pub internal: bool,
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

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetRunBody {
    pub abi: serde_json::Value,
    pub function: String,
    pub params: serde_json::Value,
    #[serde(default = "bool_false")]
    pub internal: bool,
    pub keyPair: Option<KeyPair>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfGetRunBody {
    pub bodyBase64: String,
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
    let tr = call_contract(address, &params, key_pair.as_ref())?;

    let abi_contract = AbiContract::load(params.abi.to_string().as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(&params.functionName).expect("Couldn't find function");

    if  tr.out_messages_id().len() == 0 ||
        !abi_function.has_output()
    {
        debug!("out messages missing");
        debug!("transaction: {:?}", tr);
        check_transaction_status(&tr)?;
        ok_null()
    } else {
        debug!("load out messages");
        let out_msg = load_out_message(&tr, abi_function);
        let response = out_msg.body().expect("error unwrap out message body").into();

        debug!("decode output");
        let result = Contract::decode_function_response_json(
            params.abi.to_string().to_owned(),
            params.functionName.to_owned(),
            response,
            false)
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
            load_contract(&address)?
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
       .map_err(|err| ApiError::contracts_local_run_failed(err))?;

    let abi_contract = AbiContract::load(params.abi.to_string().as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(&params.functionName).expect("Couldn't find function");

    for msg in messages {
        if  msg.msg_type() == MessageType::ExternalOutbound &&
            abi_function.is_my_message(
                msg.body().ok_or(ApiError::contracts_decode_run_output_failed("Message has no body"))?,
                false)
                    .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
        {
            let output = Contract::decode_function_response_json(
                params.abi.to_string(), params.functionName, msg.body().expect("Message has no body"), false)
                     .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;

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
        false,
        key_pair.as_ref())
        .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;

    debug!("<-");
    Ok(EncodedMessage {
        messageId: id.to_string(),
        messageIdBase64: id.to_base64().map_err(|err| ApiError::contracts_create_run_message_failed(err))?,
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
        &body,
        params.internal)
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfRun {
        output: serde_json::from_str(result.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
    })
}

pub(crate) fn decode_unknown_input(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.bodyBase64)?;
    let result = Contract::decode_unknown_function_call_from_bytes_json(
        params.abi.to_string().to_owned(),
        &body,
        params.internal)
            .map_err(|err|ApiError::contracts_decode_run_input_failed(err))?;
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
        &body,
        params.internal)
            .map_err(|err|ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfDecodeUnknownRun {
        function: result.function_name,
        output: serde_json::from_str(result.params.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
    })
}

pub(crate) fn get_run_body(_context: &mut ClientContext, params: ParamsOfGetRunBody) -> ApiResult<ResultOfGetRunBody> {
    debug!("-> contracts.run.body({})", params.params.to_string());

    let keys = match params.keyPair {
        Some(str_pair) => Some(str_pair.decode()?),
        None => None
    };

    let body = encode_function_call(
        params.abi.to_string(),
        params.function,
        params.params.to_string(),
        params.internal,
        keys.as_ref())
            .map_err(|err| ApiError::contracts_run_body_creation_failed(err))?;

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(&body.into());
    bag.write_to(&mut data, false)
        .map_err(|err| ApiError::contracts_run_body_creation_failed(err))?;

    debug!("<-");
    Ok(ResultOfGetRunBody {
        bodyBase64: base64::encode(&data)
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
pub(crate) fn check_transaction_status(transaction: &Transaction) -> ApiResult<()> {
    if !transaction.is_aborted() {
        return Ok(());
    }

    let id = transaction.id().to_string();

    if let Some(storage) = &transaction.storage {
        if storage.status_change != AccStatusChange::Unchanged {
            Err(ApiError::storage_phase_failed(id.clone(), &storage.status_change))?;
        }
    }
   
    
    if let Some(reason) = &transaction.compute.skipped_reason {
        Err(ApiError::tvm_execution_skipped(id.clone(), &reason))?;
    }

    if !transaction.compute.success {
        Err(ApiError::tvm_execution_failed(id.clone(), transaction.compute.exit_code))?;
    }

    if !transaction.action.success {
        Err(ApiError::action_phase_failed(
                id.clone(), 
                transaction.action.result_code,
                transaction.action.valid,
                transaction.action.no_funds,
            ))?;
    }

    Err(ApiError::transaction_aborted(id))
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
            && abi_function.is_my_message(msg.body().expect("No body"), false).expect("error is_my_message")
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
) -> ApiResult<Transaction> {
    let changes_stream = Contract::call_json(
        address,
        params.functionName.to_owned(),
        params.input.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        key_pair)
        .expect("Error calling contract method");

    let mut tr = None;
    for transaction in changes_stream.wait() {
        if let Err(e) = transaction {
            panic!("error next state getting: {}", e);
        }
        if let Ok(transaction) = transaction {
            debug!("run: {:?}", transaction.status);
            if transaction.status == TransactionProcessingStatus::Preliminary ||
                transaction.status == TransactionProcessingStatus::Proposed ||
                transaction.status == TransactionProcessingStatus::Finalized
            {
                tr = Some(transaction);
                break;
            }
        }
    }
    tr.ok_or(ApiError::contracts_run_transaction_missing())
}

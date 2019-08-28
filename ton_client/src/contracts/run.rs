use ton_sdk::Contract;
use crypto::keys::{KeyPair, u256_encode, account_decode};
use types::{ApiResult, ApiError, base64_decode};

#[cfg(feature = "node_interaction")]
use ton_sdk::{Transaction, MessageType, Message};
#[cfg(feature = "node_interaction")]
use tvm::block::{MessageProcessingStatus, TransactionId};
#[cfg(feature = "node_interaction")]
use tvm::types::AccountId;
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use futures::Stream;
#[cfg(feature = "node_interaction")]
use tvm::block::TrComputePhase::*;
use contracts::{EncodedMessage, EncodedUnsignedMessage};
use client::Context;


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfRun {
    pub address: String,
    pub abi: serde_json::Value,
    pub functionName: String,
    pub input: serde_json::Value,
    pub keyPair: KeyPair,
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

#[cfg(feature = "node_interaction")]
pub(crate) fn run(context: &mut Context, params: ParamsOfRun) -> ApiResult<ResultOfRun> {
    verbose!("-> contracts.run({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let address = account_decode(&params.address)?;
    let key_pair = params.keyPair.decode()?;

    verbose!("load contract");
    let contract = load_contract(&address)?;

    verbose!("run contract");
    let tr_id = call_contract(&contract.id(), &params, &key_pair)?;
    let tr_id_hex = tr_id.to_hex_string();

    verbose!("load transaction {}", tr_id_hex);
    let tr = load_transaction(&tr_id);

    if tr.out_messages_id().len() == 0 {
        verbose!("out messages missing");
        let block_transaction = tr.tr();
        verbose!("block transaction: {}", serde_json::to_string(block_transaction).unwrap());
        get_result_from_block_transaction(&block_transaction)
    } else {
        verbose!("load out messages");
        let out_msg = load_out_message(&tr);
        let response = out_msg.body().expect("error unwrap out message body").into();

        verbose!("decode output");
        let result = Contract::decode_function_response_json(
            params.abi.to_string().to_owned(),
            params.functionName.to_owned(),
            response)
            .expect("Error decoding result");

        verbose!("<-");
        Ok(ResultOfRun {
            output: serde_json::from_str(result.as_str())
                .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
        })
    }
}

pub(crate) fn encode_message(context: &mut Context, params: ParamsOfRun) -> ApiResult<EncodedMessage> {
    debug!("-> contracts.run.message({}, {}, {})",
        params.address.clone(),
        params.functionName.clone(),
        params.input.to_string()
    );

    let address = account_decode(&params.address)?;
    let key_pair = params.keyPair.decode()?;

    let (body, id) = Contract::construct_call_message_json(
        ton_sdk::AccountAddress::Short(address),
        params.functionName.to_owned(),
        params.input.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        Some(&key_pair))
        .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;

    debug!("<-");
    Ok(EncodedMessage {
        messageId: u256_encode(&id),
        messageIdBase64: base64::encode(id.as_slice()),
        messageBodyBase64: base64::encode(&body),
    })
}

pub(crate) fn encode_unsigned_message(context: &mut Context, params: ParamsOfEncodeUnsignedRunMessage) -> ApiResult<EncodedUnsignedMessage> {
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

pub(crate) fn decode_output(context: &mut Context, params: ParamsOfDecodeRunOutput) -> ApiResult<ResultOfRun> {
    let body = base64_decode(&params.bodyBase64)?;
    let result = Contract::decode_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        params.functionName.to_owned(),
        &body).map_err(|err|ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfRun {
        output: serde_json::from_str(result.as_str())
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
fn get_result_from_block_transaction(transaction: &tvm::block::Transaction) -> ApiResult<ResultOfRun> {
    match transaction.compute_phase_ref() {
        Some(compute_phase) => {
            match compute_phase {
                Skipped(skipped) => {
                    verbose!("VM compute phase was skipped");
                    let reason: u8 = skipped.reason.clone() as u8;
                    Err(ApiError::tvm_execution_skipped(reason))
                }
                Vm(vm) => {
                    if vm.success {
                        verbose!("VM compute phase was succeeded");
                        ok_null()
                    } else {
                        verbose!("VM compute phase was not succeeded");
                        Err(ApiError::tvm_execution_failed(vm.exit_code))
                    }
                }
            }
        }
        None => {
            verbose!("VM compute phase have missing!");
            ok_null()
        }
    }
}

#[cfg(feature = "node_interaction")]
fn load_transaction(id: &TransactionId) -> Transaction {
    Transaction::load(id.clone())
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction")
}

#[cfg(feature = "node_interaction")]
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

#[cfg(feature = "node_interaction")]
fn load_contract(address: &AccountId) -> ApiResult<Contract> {
    Contract::load(address.clone().into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .ok_or(ApiError::contracts_run_contract_not_found())
}

#[cfg(feature = "node_interaction")]
fn call_contract(
    address: &AccountId,
    params: &ParamsOfRun,
    key_pair: &Keypair,
) -> ApiResult<TransactionId> {
    let changes_stream = Contract::call_json(
        address.clone().into(),
        params.functionName.to_owned(),
        params.input.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        Some(&key_pair))
        .expect("Error calling contract method");

    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(state) = state {
            verbose!("run: {:?}", state.message_state);
            if state.message_state == MessageProcessingStatus::Preliminary ||
                state.message_state == MessageProcessingStatus::Proposed ||
                state.message_state == MessageProcessingStatus::Finalized
            {
                tr_id = Some(state.message_id.clone());
                break;
            }
        }
    }
    tr_id.ok_or(ApiError::contracts_run_transaction_missing())
}

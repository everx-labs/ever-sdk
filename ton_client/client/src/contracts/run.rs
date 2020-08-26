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

use ton_sdk::{Contract, MessageType, AbiContract, FunctionCallSet, Transaction, Message,
    TransactionFees, LocalRunContext};
use ton_sdk::json_abi::encode_function_call;
use ton_types::cells_serialization::BagOfCells;
use ton_block::{AccStatusChange, Message as TvmMessage, MsgAddressInt};

use crate::contracts::{EncodedMessage, EncodedUnsignedMessage};
use crate::client::ClientContext;
use crate::crypto::keys::{KeyPair, account_decode};
use crate::types::{
    ApiResult,
    ApiError,
    base64_decode,
    long_num_to_json_string};

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClient, ReceivedTransaction, SdkError};
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use crate::types::{apierror_from_sdkerror, ApiErrorCode, ApiSdkErrorCode, StdContractError};


fn bool_false() -> bool { false }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunFunctionCallSet {
    // contract ABI
    pub abi: serde_json::Value,
    // function name
    pub function_name: String,
    // header parameters
    pub header: Option<serde_json::Value>,
    // input parameters
    pub input: serde_json::Value,
}

impl Into<FunctionCallSet> for RunFunctionCallSet {
    fn into(self) -> FunctionCallSet {
        FunctionCallSet {
            func: self.function_name.clone(),
            header: self.header.map(|value| value.to_string().to_owned()),
            input: self.input.to_string(),
            abi: self.abi.to_string(),
        }
    }
}

#[doc(summary="Method that calls a contract's function")]
/// Method creates a run message signed with key_pair, sends it to the targer workchain,
/// waits for the result transaction and outbound messages and decodes the result parameters using ABI.
/// If the contract implements Pragma Expire, the method repeats the algorithm
/// for message_retries_count times 
/// if the message was not delivered during message_expiration_timeout (see setup.SetupParams).
/// If the contract does not implement Pragra Expire - the method waits for the result
/// transaction for message_processing_timeout (defined in the Client Config),
/// and exits with 1012 original error.
/// Before retrying or exiting with 1012, the message is processed on the local transaction executor to check the possible reason
/// why the transaction was not finalized
/// and  if such error is found - stops retrying and returns it,
/// if not - continues retrying or returns disclainmer that the local execution was successful.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfRun {
    /// account address
    pub address: String,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    /// key pair to sign the message
    pub key_pair: Option<KeyPair>,
    // [1.0.0] will be deprecated 
    pub try_index: Option<u8>,
}

#[doc(summary="Method that calls a contract on a local TVM")]
/// Method calls a contract's function on a local TVM
/// Method can work in 2 modes: TVM mode and Transaction Executor mode.
/// Mode is defined by full_run parameter. If true - Transaction Executor mode is on.
///
/// TVM mode:
/// used for running get methods. Works same as 'run' but the message is processed on the local TVM:
/// creates a message, runs it on the local TVM and decodes the return result.
/// can be used for methods without ACCEPT.
///
/// Transaction executor mode:
/// used to fully emulate message processing to calculate fees and check if all the phases are passed successfully 
/// If not- returns the error with the exit code and phase.

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfLocalRun {
    /// account address (used to load boc from graphql API if boc is not defined)
    pub address: String,
    /// [1.0.0] account boc
    pub account: Option<serde_json::Value>,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    /// key pair to sign the message
    pub key_pair: Option<KeyPair>,
    #[serde(default)]
    /// flag that enables/disables full run with transaction executor.
    pub full_run: bool,
    /// will be deprecated
    pub time: Option<u32>,
    #[serde(flatten)]
    pub context: LocalRunContext,
}

#[doc(summary="Method that processes a specified message on a local TVM")]
/// Method works as LocalRun, but takes an already prepared message
/// as a parameter. 
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfLocalRunWithMsg {
    /// account address (used to load boc from graphql API if boc is not defined)
    pub address: String,
    // [1.0.0] account boc
    pub account: Option<serde_json::Value>,
    /// contract ABI
    pub abi: Option<serde_json::Value>,
    /// function name
    pub function_name: Option<String>,
    /// message boc
    pub message_base64: String,
    #[serde(default)]
    /// flag that enables/disables full run with transaction executor (see localRun documentation).
    pub full_run: bool,
    /// will be deprecated
    pub time: Option<u32>,
    #[serde(flatten)]
    pub context: LocalRunContext,
}

#[doc(summary="Method that creates an unsigned message")]
/// Method prepares an unsigned message that can be signed and sent later. 
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeUnsignedRunMessage {
    // account address
    pub address: String,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    /// Parameter is used only for contracts that support Expire Pragma
    /// to calculate expire header for subsequent retries. 
    /// Specify 1 for the first retry, 2 - for the second and so on.
    /// The formula is: expire = now + message_expiration_timeout * message_expiration_timeout_grow_factor * try_index
    pub try_index: Option<u8>,
}


#[doc(summary="??? Method that decodes")]
/// ??? 
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfDecodeRunOutput {
    /// contract ABI
    pub abi: serde_json::Value,
    /// contract's function name
    pub function_name: String,
    /// boc of message body in base64
    pub body_base64: String,
    #[serde(default = "bool_false")]
    /// flag that specifies the message type: internal or external
    pub internal: bool,
}

#[doc(summary="??? Method that decodes")]
/// ??? 
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParamsOfDecodeUnknownRun {
    /// contract ABI
    pub abi: serde_json::Value,
    /// boc of message body in base64
    pub body_base64: String,
    #[serde(default = "bool_false")]
    /// flag that specifies the message type: internal or external
    pub internal: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfRun {
    /// list of decoded parameters returned by the contract's function
    pub output: serde_json::Value,
    /// fees spent on transaction
    pub fees: RunFees,
    /// transaction object in json format according to graphQL schema
    pub transaction: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfLocalRun {
    /// list of decoded parameters returned by the contract's function
    pub output: serde_json::Value,
    /// fees spent on transaction
    pub fees: Option<RunFees>,
    /// [1.0.0] account boc after the local execution
    pub account: Option<Contract>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ResultOfDecode {
    /// list of decoded parameters
    pub output: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunFees {
    /// fee paid for internal message delivery
    pub in_msg_fwd_fee: String,
    /// fee paid for storage
    pub storage_fee: String,
    /// fee paid for transaction execution
    pub gas_fee: String,
    /// fee paid for external message delivery
    pub out_msgs_fwd_fee: String,
    /// total fees paid
    pub total_account_fees: String,
    /// total value for all internal outbound messaged
    pub total_output: String,
}

impl From<TransactionFees> for RunFees {
    fn from(value: TransactionFees) -> Self {
        RunFees {
            in_msg_fwd_fee: long_num_to_json_string(value.in_msg_fwd_fee),
            storage_fee: long_num_to_json_string(value.storage_fee),
            gas_fee: long_num_to_json_string(value.gas_fee),
            out_msgs_fwd_fee: long_num_to_json_string(value.out_msgs_fwd_fee),
            total_account_fees: long_num_to_json_string(value.total_account_fees),
            total_output: long_num_to_json_string(value.total_output),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResultOfDecodeUnknownRun {
    /// function name
    pub function: String,
    /// list of decoded parameters returned by the contract's function
    pub output: serde_json::Value,
}

#[doc(summary="Method that generates a message and extracts the body from it")]
/// Method generates a message of internal (internal==true) or external (internal==false) type
/// according to ABI and specified header, extracts the body boc in base64 from it
/// and returns it.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetRunBody {
    /// contract ABI
    pub abi: serde_json::Value,
    /// function name
    pub function: String,
    /// message header
    pub header: Option<serde_json::Value>,
    /// function parameters
    pub params: serde_json::Value,
    #[serde(default = "bool_false")]
    /// internal (true) or external (false) message type 
    pub internal: bool,
    /// key pair for signature
    pub key_pair: Option<KeyPair>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetRunBody {
    /// message body boc in base64
    pub body_base64: String,
}

#[doc(summary="Method that processes the message on local Transaction Executor to investigate why the transaction was not finalized in blockchain.")]
/// Method investigates why the specified external message was not delivered
/// It is used in the following methods: run, deploy, process_message, wait_for_transactoin:
/// in case of not finding the transaction within the specified timeout, the message is processed locally 
/// on Transaction Executor and if there is an exception, it returns it as 3025 error code and places
/// exit_code and phase inside of data field of error object. If local processing was successful it returns 
/// a disclamer that it was successfull. Which means that you need to wait more.
///
/// Let's imagine a transaction was not found within the specified timeout.
/// This may happen for several reasons, including network lag 
/// or real TVM exception that happened before ACCEPT in the contract on the real network. 
/// If TVM exception happens before ACCEPT for external inbound messages, such transactions 
/// are not finalized and will never appear on blockchain. 
/// So, ResolveError method diagnoses such problems locally so that user can get a hint of what 
/// the reason may be.  
/// Although, this method does not give 100% guarantee that the transaction will never be 
/// finalized because of the locally received exception . 
/// For instance, if it was not finalized for the reason of not enough balance, and 
/// value was send to the account before or after the method execution even after some period of time - if it was, there is a probability that
/// the value can finally be delivered later and the message can eventually succeed, 
/// because it is in fact broadcasted between validators for some (not defined) period of time and circulates in the network, 
/// and while it failed on 1/2 of validators before the value was delivered it can succeed on the other 1/2
/// of validators after the value is delivered.
/// If in such situation a developer just blindly performs a retry - she may get a double spend. 
/// Only developer knows which operations and in what
/// sequence she performs to diagnoze the result of this method and make a decision.
/// To avoid double spending we recommend using Pragma Expire that at least helps to ensure that the transaction will 
/// not happen 100% after message is expired. 
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfResolveError {
    /// account address 
    pub address: String,
    /// [1.0.0] account boc
    pub account: Contract,
    /// message boc (header+body)
    pub message_base64: String,
    /// ???
    pub time: u32,
    /// original error received. For instance - 1012 - the transaction was not delivered during the specified timeout
    pub main_error: ApiError,
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn run(context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<ResultOfRun> {
    trace!("-> contracts.run({}, {:?})",
        params.address.clone(),
        params.call_set.clone(),
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(ref keys) = params.key_pair { Some(keys.decode()?) } else { None };

    let client = context.get_client()?;
    trace!("run contract");
    let tr = call_contract(client, address.clone(), &params, key_pair.as_ref()).await
        .map_err(|err| err
            .add_function(Some(&params.call_set.function_name))
            .add_network_url(client)
        )?;

    process_transaction(
        tr.parsed,
        tr.value,
        Some(params.call_set.abi),
        Some(params.call_set.function_name),
        &address,
        None,
        true)
        .map_err(|err| err
            .add_network_url(client)
        )
}

pub(crate) fn process_out_messages(
    messages: &Vec<Message>,
    abi: Option<serde_json::Value>,
    function: Option<String>,
) -> ApiResult<serde_json::Value> {
    if let Some(abi) = abi {
        let function = function.ok_or(ApiError::contracts_decode_run_output_failed("No function name provided", None))?;

        let abi_contract = AbiContract::load(abi.to_string().as_bytes()).expect("Couldn't parse ABI");
        let abi_function = abi_contract.function(&function).expect("Couldn't find function");

        if messages.len() == 0 || !abi_function.has_output() {
            trace!("out messages missing");
            Ok(serde_json::Value::Null)
        } else {
            trace!("processing out messages");

            for msg in messages {
                if msg.msg_type() == MessageType::ExternalOutbound &&
                    abi_function.is_my_output_message(
                        msg.body().ok_or(ApiError::contracts_decode_run_output_failed(
                            "Message has no body", Some(&function)))?,
                        false)
                        .map_err(|err|
                            ApiError::contracts_decode_run_output_failed(err, Some(&function)))?
                {
                    let output = Contract::decode_function_response_json(
                        abi.to_string(), function.clone(), msg.body().expect("Message has no body"), false)
                        .map_err(|err| ApiError::contracts_decode_run_output_failed(err, Some(&function)))?;

                    let output: serde_json::Value = serde_json::from_str(&output)
                        .map_err(|err|
                            ApiError::contracts_decode_run_output_failed(err, Some(&function)))?;

                    return Ok(output);
                }
            }
            return Err(ApiError::contracts_decode_run_output_failed(
                "No external output messages", Some(&function)));
        }
    } else {
        trace!("No abi provided");
        Ok(serde_json::Value::Null)
    }
}

pub(crate) fn process_transaction(
    transaction: Transaction,
    json: serde_json::Value,
    abi: Option<serde_json::Value>,
    function: Option<String>,
    address: &MsgAddressInt,
    balance: Option<u64>,
    real_tr: bool,
) -> ApiResult<ResultOfRun> {
    check_transaction_status(&transaction, real_tr, address, balance)
        .map_err(|err| err.add_function(function.as_ref().map(|string| string.as_str())))?;
    let fees = transaction.calc_fees().into();
    let output = process_out_messages(&transaction.out_messages, abi, function.clone())
        .map_err(|err| err
            .add_function(function.as_ref().map(|string| string.as_str()))
            .add_address(address)
        )?;

    Ok(ResultOfRun { output, fees: fees, transaction: json })
}

pub(crate) fn local_run(context: &mut ClientContext, params: ParamsOfLocalRun) -> ApiResult<ResultOfLocalRun> {
    trace!("-> contracts.run.local({}, {:?})",
        params.address.clone(),
        params.call_set.clone()
    );

    let address = account_decode(&params.address)?;

    let key_pair = params.key_pair.map(|pair| pair.decode()).transpose()?;

    let account = params.account
        .as_ref()
        .map(|acc|
            Contract::from_json(&acc.to_string())
                .map_err(|err| ApiError::invalid_params(&acc.to_string(), err))
        )
        .transpose()?;

    do_local_run(
        Some(context),
        params.call_set,
        key_pair.as_ref(),
        address,
        account,
        params.full_run,
        params.context,
    ).map_err(|err| match context.get_client() {
        Ok(client) => err.add_network_url(client),
        Err(_) => err
    })
}

pub(crate) fn local_run_msg(context: &mut ClientContext, params: ParamsOfLocalRunWithMsg) -> ApiResult<ResultOfLocalRun> {
    trace!("-> contracts.run.local.msg({}, {}, {})",
        params.address.clone(),
        params.function_name.clone().unwrap_or_default(),
        params.message_base64
    );

    let address = account_decode(&params.address)?;

    let account = params.account
        .as_ref()
        .map(|acc|
            Contract::from_json(&acc.to_string())
                .map_err(|err| ApiError::invalid_params(&acc.to_string(), err))
        )
        .transpose()?;

    let msg = Contract::deserialize_message(&base64::decode(&params.message_base64)
        .map_err(|err| ApiError::crypto_invalid_base64(&params.message_base64, err))?)
        .map_err(|err| ApiError::invalid_params(&params.message_base64, err))?;

    do_local_run_msg(
        Some(context),
        address,
        account,
        params.abi,
        params.function_name,
        msg,
        params.full_run,
        params.context,
    ).map_err(|err| match context.get_client() {
        Ok(client) => err.add_network_url(client),
        Err(_) => err
    })
}

pub(crate) fn encode_message(context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<EncodedMessage> {
    trace!("-> contracts.run.message({}, {:?})",
        params.address.clone(),
        params.call_set.clone()
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(keys) = params.key_pair { Some(keys.decode()?) } else { None };
    let function = params.call_set.function_name.clone();

    let msg = Contract::construct_call_message_json(
        address,
        params.call_set.into(),
        false,
        key_pair.as_ref(),
        Some(context.get_client()?.timeouts()),
        None)
        .map_err(|err| ApiError::contracts_create_run_message_failed(err, &function))?;

    trace!("<-");
    Ok(EncodedMessage::from_sdk_msg(msg))
}

pub(crate) fn encode_unsigned_message(context: &mut ClientContext, params: ParamsOfEncodeUnsignedRunMessage) -> ApiResult<EncodedUnsignedMessage> {
    let function = params.call_set.function_name.clone();
    let encoded = ton_sdk::Contract::get_call_message_bytes_for_signing(
        account_decode(&params.address)?,
        params.call_set.into(),
        Some(context.get_client()?.timeouts()),
        None,
    ).map_err(|err| ApiError::contracts_create_run_message_failed(err, &function))?;
    Ok(EncodedUnsignedMessage {
        unsigned_bytes_base64: base64::encode(&encoded.message),
        bytes_to_sign_base64: base64::encode(&encoded.data_to_sign),
        expire: encoded.expire,
    })
}

pub(crate) fn decode_output(_context: &mut ClientContext, params: ParamsOfDecodeRunOutput) -> ApiResult<ResultOfDecode> {
    let body = base64_decode(&params.body_base64)?;
    let result = Contract::decode_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        params.function_name.to_owned(),
        &body,
        params.internal)
        .map_err(|err| ApiError::contracts_decode_run_output_failed(err, Some(&params.function_name)))?;
    Ok(ResultOfDecode {
        output: serde_json::from_str(result.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err, Some(&params.function_name)))?
    })
}

pub(crate) fn decode_unknown_input(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.body_base64)?;
    let result = Contract::decode_unknown_function_call_from_bytes_json(
        params.abi.to_string().to_owned(),
        &body,
        params.internal)
        .map_err(|err| ApiError::contracts_decode_run_input_failed(err, None))?;
    Ok(ResultOfDecodeUnknownRun {
        output: serde_json::from_str(result.params.as_str())
            .map_err(|err| ApiError::contracts_decode_run_input_failed(err, Some(&result.function_name)))?,
        function: result.function_name,
    })
}

pub(crate) fn decode_unknown_output(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.body_base64)?;
    let result = Contract::decode_unknown_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        &body,
        params.internal)
        .map_err(|err| ApiError::contracts_decode_run_output_failed(err, None))?;
    Ok(ResultOfDecodeUnknownRun {
        output: serde_json::from_str(result.params.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err, Some(&result.function_name)))?,
        function: result.function_name,
    })
}

pub(crate) fn get_run_body(_context: &mut ClientContext, params: ParamsOfGetRunBody) -> ApiResult<ResultOfGetRunBody> {
    trace!("-> contracts.run.body({})", params.params.to_string());

    let keys = match params.key_pair {
        Some(str_pair) => Some(str_pair.decode()?),
        None => None
    };

    let body = encode_function_call(
        params.abi.to_string(),
        params.function,
        params.header.map(|value| value.to_string().to_owned()),
        params.params.to_string(),
        params.internal,
        keys.as_ref())
        .map_err(|err| ApiError::contracts_run_body_creation_failed(err))?;

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(&body.into());
    bag.write_to(&mut data, false)
        .map_err(|err| ApiError::contracts_run_body_creation_failed(err))?;

    trace!("<-");
    Ok(ResultOfGetRunBody {
        body_base64: base64::encode(&data)
    })
}


pub(crate) fn resolve_error(_context: &mut ClientContext, params: ParamsOfResolveError) -> ApiResult<()> {
    let address = account_decode(&params.address)?;
    let msg = base64_decode(&params.message_base64)?;
    Err(resolve_msg_error(address, params.account, &msg, params.time, params.main_error))
}

pub(crate) fn check_transaction_status(
    transaction: &Transaction,
    real_tr: bool,
    address: &MsgAddressInt,
    balance: Option<u64>,
) -> ApiResult<()> {
    if !transaction.is_aborted() {
        return Ok(());
    }

    let id = if real_tr {
        Some(transaction.id().to_string())
    } else {
        None
    };

    if let Some(storage) = &transaction.storage {
        if storage.status_change != AccStatusChange::Unchanged {
            Err(ApiError::storage_phase_failed(id.clone(), &storage.status_change, address, balance))?;
        }
    }


    if let Some(reason) = &transaction.compute.skipped_reason {
        Err(ApiError::tvm_execution_skipped(id.clone(), &reason, address, balance))?;
    }

    if transaction.compute.success.is_none() || !transaction.compute.success.unwrap() {
        Err(ApiError::tvm_execution_failed(
            id.clone(), transaction.compute.exit_code.unwrap_or(-1), address))?;
    }

    if let Some(action) = &transaction.action {
        if !action.success {
            Err(ApiError::action_phase_failed(
                id.clone(),
                action.result_code,
                action.valid,
                action.no_funds,
                address,
                balance,
            ))?;
        }
    }


    Err(ApiError::transaction_aborted(id))
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn load_contract(context: &ClientContext, address: &MsgAddressInt, deployed: bool) -> ApiResult<Contract> {
    let client = context.get_client()?;
    let result = Contract::load_wait(client, address, deployed, None)
        .await
        .map_err(|err| crate::types::apierror_from_sdkerror(
            &err, ApiError::contracts_run_contract_load_failed, Some(client)));
    if let Err(err) = result {
        if err.code == crate::types::ApiSdkErrorCode::WaitForTimeout.as_number() {
            let result = Contract::load(context.get_client()?, address).await
                .map_err(|err| crate::types::apierror_from_sdkerror(
                    &err, ApiError::contracts_run_contract_load_failed, Some(client)))?;
            if let Some(contract) = result {
                if contract.acc_type == ton_block::AccountStatus::AccStateActive {
                    Ok(contract)
                } else {
                    Err(ApiError::account_code_missing(&address))
                }
            } else {
                Err(ApiError::account_missing(&address))
            }
        } else {
            Err(err)
        }
    } else {
        result
    }
}

#[cfg(feature = "node_interaction")]
pub async fn retry_call<F, Fut>(retries_count: u8, func: F) -> ApiResult<ReceivedTransaction>
    where
        F: Fn(u8) -> Fut,
        Fut: futures::Future<Output=ApiResult<ReceivedTransaction>>
{
    let mut result = Err(ApiError::contracts_send_message_failed("Unreacheable"));
    for i in 0..(retries_count + 1) {
        if i != 0 {
            warn!("Message expired. Retry#{}", i);
        }
        result = func(i).await;
        match &result {
            Err(error) => {
                // retry if message expired or if resolving returned that message expired/replay
                // protection error or if transaction with message expired/replay protection error
                // returned
                let retry = error.code == ApiSdkErrorCode::MessageExpired as isize ||
                    ((error.data["exit_code"] == StdContractError::ReplayProtection as i32 ||
                        error.data["exit_code"] == StdContractError::ExtMessageExpired as i32) &&
                        (error.data["original_error"].is_null() ||
                            error.data["original_error"]["code"] == ApiSdkErrorCode::MessageExpired as isize));
                if retry {
                    continue;
                } else {
                    return result;
                }
            }
            _ => return result
        }
    }
    result
}

#[cfg(feature = "node_interaction")]
async fn call_contract(
    client: &NodeClient,
    address: MsgAddressInt,
    params: &ParamsOfRun,
    key_pair: Option<&Keypair>,
) -> ApiResult<ReceivedTransaction> {
    retry_call(client.timeouts().message_retries_count, |try_index: u8| {
        let address = address.clone();
        let call_set = params.call_set.clone();
        async move {
            let msg = Contract::construct_call_message_json(
                address,
                call_set.into(),
                false,
                key_pair,
                Some(client.timeouts()),
                Some(try_index))
                .map_err(|err| ApiError::contracts_create_run_message_failed(
                    err, &params.call_set.function_name))?;

            let result = Contract::process_message(client, &msg, true).await;

            match result {
                Err(err) =>
                    Err(resolve_msg_sdk_error(
                        client, err, &msg, Some(&params.call_set.function_name), ApiError::contracts_run_failed
                    ).await?),
                Ok(tr) => Ok(tr)
            }
        }
    }).await
}

pub(crate) fn do_local_run(
    context: Option<&mut ClientContext>,
    call_set: RunFunctionCallSet,
    keys: Option<&ed25519_dalek::Keypair>,
    address: MsgAddressInt,
    account: Option<Contract>,
    full_run: bool,
    run_context: LocalRunContext,
) -> ApiResult<ResultOfLocalRun> {
    let msg = Contract::construct_call_message_json(
        address.clone(), call_set.clone().into(), false, keys, None, None)
        .map_err(|err| ApiError::contracts_create_run_message_failed(err, &call_set.function_name))?;

    do_local_run_msg(
        context,
        address,
        account,
        Some(call_set.abi),
        Some(call_set.function_name),
        msg.message,
        full_run,
        run_context)
}

pub(crate) fn do_local_run_msg(
    context: Option<&mut ClientContext>,
    address: MsgAddressInt,
    account: Option<Contract>,
    abi: Option<serde_json::Value>,
    function_name: Option<String>,
    msg: TvmMessage,
    full_run: bool,
    run_context: LocalRunContext,
) -> ApiResult<ResultOfLocalRun> {
    let contract = match account {
        // load contract data from node manually
        #[cfg(feature = "node_interaction")]
        None => {
            trace!("load contract");
            if let Some(context) = context {
                let mut runtime = context.take_runtime()?;
                let result = runtime.block_on(load_contract(context, &address, !full_run));
                context.runtime = Some(runtime);
                result?
            } else {
                return Err(ApiError::sdk_not_init());
            }
        }
        // can't load
        #[cfg(not(feature = "node_interaction"))]
        None => {
            trace!("no account provided");
            let _address = address;
            let _context = context;
            return Err(ApiError::invalid_params("account", "No account provided"));
        }

        Some(account) => account
    };

    if full_run {
        let result = contract.local_call(msg, run_context)
            .map_err(|err|
                match err.downcast_ref::<ton_sdk::SdkError>() {
                    Some(ton_sdk::SdkError::ContractError(exit_code)) =>
                        ApiError::tvm_execution_failed(None, *exit_code, &address),
                    Some(ton_sdk::SdkError::NoFundsError) =>
                        ApiError::low_balance(&address, Some(contract.balance_grams())),
                    _ => ApiError::contracts_local_run_failed(err)
                }
                    .add_function(function_name.as_ref().map(|string| string.as_str()))
            )?;
        let run_result = process_transaction(
            result.transaction,
            json!(null),
            abi,
            function_name,
            &address,
            Some(contract.balance_grams()),
            false)?;
        Ok(ResultOfLocalRun {
            output: run_result.output,
            fees: Some(run_result.fees),
            account: Some(result.updated_account),
        })
    } else {
        let messages = contract.local_call_tvm(msg)
            .map_err(|err| {
                ApiError::contracts_local_run_failed(err)
                    .add_function(function_name.as_ref().map(|string| string.as_str()))
            })?;

        let output = process_out_messages(&messages, abi, function_name.clone())
            .map_err(|err| err.add_function(function_name.as_ref().map(|string| string.as_str())))?;
        Ok(ResultOfLocalRun {
            output: output,
            fees: None,
            account: None,
        })
    }
}

pub(crate) fn resolve_msg_error(
    address: MsgAddressInt,
    account: Contract,
    msg: &[u8],
    time: u32,
    mut main_error: ApiError,
) -> ApiError {
    let msg = Contract::deserialize_message(&msg)
        .map_err(|err| ApiError::invalid_params("message", format!("cannot parse BOC ({})", err)));
    let msg = match msg {
        Ok(msg) => msg,
        Err(err) => return err
    };

    let mut context = LocalRunContext::default();
    context.time = Some(time);
    let result = do_local_run_msg(None, address.clone(), Some(account), None, None, msg, true, context);

    if let Err(mut err) = result {
        err.data["original_error"] = serde_json::to_value(main_error).unwrap_or_default();
        err
    } else {
        main_error.data["disclaimer"] = "Local contract call succeded. Can not resolve extended error".into();
        main_error.add_address(&address)
    }
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn resolve_msg_sdk_error<F: Fn(String) -> ApiError>(
    client: &NodeClient,
    error: failure::Error,
    msg: &ton_sdk::SdkMessage,
    function: Option<&str>,
    default_error: F,
) -> ApiResult<ApiError> {
    let err = {
        match error.downcast_ref::<SdkError>() {
            Some(SdkError::MessageExpired { msg_id: _, expire: _, sending_time, block_time: _, block_id: _ }) |
            Some(SdkError::TransactionWaitTimeout { msg_id: _, sending_time, timeout: _, state: _ }) => {
                let account = Contract::load(client, &msg.address)
                    .await
                    .map_err(|err| apierror_from_sdkerror(
                            &err, ApiError::contracts_run_contract_load_failed, Some(client),
                        ).add_address(&msg.address))?
                    .ok_or(ApiError::account_missing(&msg.address))?;
                let main_error = apierror_from_sdkerror(&error, default_error, None);
                let resolved = resolve_msg_error(
                    msg.address.clone(), account, &msg.serialized_message, *sending_time, main_error,
                );
                Ok(resolved)
            }
            _ => Err(apierror_from_sdkerror(&error, default_error, Some(client)))
        }
    }?;
    Ok(err
        .add_network_url(client)
        .add_function(function)
        .add_address(&msg.address))
}

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

use ton_sdk::{Contract, MessageType, AbiContract, FunctionCallSet};
use ton_sdk::json_abi::encode_function_call;
use ton_types::cells_serialization::BagOfCells;
use ton_block::{AccStatusChange, Message as TvmMessage, MsgAddressInt};
use ton_sdk::{Transaction, Message, TransactionFees};

use crate::contracts::{EncodedMessage, EncodedUnsignedMessage};
use crate::client::ClientContext;
use crate::crypto::keys::{KeyPair, account_decode};
use crate::types::{
    ApiResult,
    ApiError,
    base64_decode,
    long_num_to_json_string};

#[cfg(feature = "node_interaction")]
use ton_sdk::{NodeClient, RecievedTransaction, SdkError};
#[cfg(feature = "node_interaction")]
use ed25519_dalek::Keypair;
#[cfg(feature = "node_interaction")]
use crate::types::{apierror_from_sdkerror, ApiErrorCode, ApiSdkErrorCode, StdContractError};


fn bool_false() -> bool { false }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunFunctionCallSet {
    pub abi: serde_json::Value,
    pub function_name: String,
    pub header: Option<serde_json::Value>,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfRun {
    pub address: String,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    pub key_pair: Option<KeyPair>,
    pub try_index: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfLocalRun {
    pub address: String,
    pub account: Option<serde_json::Value>,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    pub key_pair: Option<KeyPair>,
    #[serde(default)]
    pub full_run: bool,
    pub time: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfLocalRunWithMsg {
    pub address: String,
    pub account: Option<serde_json::Value>,
    pub abi: Option<serde_json::Value>,
    pub function_name: Option<String>,
    pub message_base64: String,
    #[serde(default)]
    pub full_run: bool,
    pub time: Option<u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfEncodeUnsignedRunMessage {
    pub address: String,
    #[serde(flatten)]
    pub call_set: RunFunctionCallSet,
    pub try_index: Option<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfDecodeRunOutput {
    pub abi: serde_json::Value,
    pub function_name: String,
    pub body_base64: String,
    #[serde(default = "bool_false")]
    pub internal: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamsOfDecodeUnknownRun {
    pub abi: serde_json::Value,
    pub body_base64: String,
    #[serde(default = "bool_false")]
    pub internal: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfRun {
    pub output: serde_json::Value,
    pub fees: RunFees,
    pub transaction: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfLocalRun {
    pub output: serde_json::Value,
    pub fees: Option<RunFees>,
    pub account: Option<Contract>
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfDecode {
    pub output: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunFees {
    pub in_msg_fwd_fee: String,
    pub storage_fee: String,
    pub gas_fee: String,
    pub out_msgs_fwd_fee: String,
    pub total_account_fees: String,
    pub total_output: String
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

#[derive(Serialize, Deserialize)]
pub struct ResultOfDecodeUnknownRun {
    pub function: String,
    pub output: serde_json::Value
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfGetRunBody {
    pub abi: serde_json::Value,
    pub function: String,
    pub header: Option<serde_json::Value>,
    pub params: serde_json::Value,
    #[serde(default = "bool_false")]
    pub internal: bool,
    pub key_pair: Option<KeyPair>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultOfGetRunBody {
    pub body_base64: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParamsOfResolveError {
    pub address: String,
    pub account: Contract,
    pub message_base64: String,
    pub time: u32,
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
    let tr = call_contract(client, address.clone(), &params, key_pair.as_ref()).await?;

    process_transaction(
        tr.parsed,
        tr.value,
        Some(params.call_set.abi),
        Some(params.call_set.function_name),
        &address,
        true)
}

pub(crate) fn process_out_messages(
    messages: &Vec<Message>,
    abi: Option<serde_json::Value>,
    function: Option<String>
) -> ApiResult<serde_json::Value> {
    if let Some(abi) = abi {
        let function = function.ok_or(ApiError::contracts_decode_run_output_failed("No function name provided"))?;

        let abi_contract = AbiContract::load(abi.to_string().as_bytes()).expect("Couldn't parse ABI");
        let abi_function = abi_contract.function(&function).expect("Couldn't find function");

        if  messages.len() == 0 || !abi_function.has_output() {
            trace!("out messages missing");
            Ok(serde_json::Value::Null)
        } else {
            trace!("processing out messages");

            for msg in messages {
                if  msg.msg_type() == MessageType::ExternalOutbound &&
                    abi_function.is_my_output_message(
                        msg.body().ok_or(ApiError::contracts_decode_run_output_failed("Message has no body"))?,
                        false)
                            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
                {
                    let output = Contract::decode_function_response_json(
                        abi.to_string(), function, msg.body().expect("Message has no body"), false)
                            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;
    
                    let output: serde_json::Value = serde_json::from_str(&output)
                        .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;
    
                    return Ok(output);
                }
            }
            return Err(ApiError::contracts_decode_run_output_failed("No external output messages"));
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
    real_tr: bool,
) -> ApiResult<ResultOfRun> {
    check_transaction_status(&transaction, real_tr, address)?;
    let fees = transaction.calc_fees().into();
    let output = process_out_messages(&transaction.out_messages, abi, function)?;
    
    Ok( ResultOfRun { output, fees: fees, transaction: json } )
}

pub(crate) fn local_run(context: &mut ClientContext, params: ParamsOfLocalRun) -> ApiResult<ResultOfLocalRun> {
    trace!("-> contracts.run.local({}, {:?})",
        params.address.clone(),
        params.call_set.clone()
    );

    let address = account_decode(&params.address)?;

    let key_pair = params.key_pair.map(|pair| pair.decode()).transpose()?;

    let account = params.account.as_ref().map(|acc| 
            Contract::from_json(&acc.to_string())
                .map_err(|err| ApiError::invalid_params(&acc.to_string(), err)))
        .transpose()?;

    do_local_run(
        Some(context), params.call_set, key_pair.as_ref(), address, account, params.full_run, params.time)
}

pub(crate) fn local_run_msg(context: &mut ClientContext, params: ParamsOfLocalRunWithMsg) -> ApiResult<ResultOfLocalRun> {
    trace!("-> contracts.run.local.msg({}, {}, {})",
        params.address.clone(),
        params.function_name.clone().unwrap_or_default(),
        params.message_base64
    );

    let address = account_decode(&params.address)?;

    let account = params.account.as_ref().map(|acc| 
            Contract::from_json(&acc.to_string())
                .map_err(|err| ApiError::invalid_params(&acc.to_string(), err)))
        .transpose()?;

    let msg = Contract::deserialize_message(
        &base64::decode(&params.message_base64)
            .map_err(|err| ApiError::crypto_invalid_base64(&params.message_base64, err))?)
        .map_err(|err| ApiError::invalid_params(&params.message_base64, err))?;

    do_local_run_msg(
        Some(context), address, account, params.abi, params.function_name, msg, params.full_run, params.time)
}

pub(crate) fn encode_message(context: &mut ClientContext, params: ParamsOfRun) -> ApiResult<EncodedMessage> {
    trace!("-> contracts.run.message({}, {:?})",
        params.address.clone(),
        params.call_set.clone()
    );

    let address = account_decode(&params.address)?;
    let key_pair = if let Some(keys) = params.key_pair { Some(keys.decode()?) } else { None };

    let msg = Contract::construct_call_message_json(
        address,
        params.call_set.into(),
        false,
        key_pair.as_ref(),
        Some(context.get_client()?.timeouts()),
        None)
        .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;

    trace!("<-");
    Ok(EncodedMessage::from_sdk_msg(msg))
}

pub(crate) fn encode_unsigned_message(context: &mut ClientContext, params: ParamsOfEncodeUnsignedRunMessage) -> ApiResult<EncodedUnsignedMessage> {
    let encoded = ton_sdk::Contract::get_call_message_bytes_for_signing(
        account_decode(&params.address)?,
        params.call_set.into(),
        Some(context.get_client()?.timeouts()),
        None
    ).map_err(|err| ApiError::contracts_create_run_message_failed(err))?;
    Ok(EncodedUnsignedMessage {
        unsigned_bytes_base64: base64::encode(&encoded.message),
        bytes_to_sign_base64: base64::encode(&encoded.data_to_sign),
        expire: encoded.expire
    })
}

pub(crate) fn decode_output(_context: &mut ClientContext, params: ParamsOfDecodeRunOutput) -> ApiResult<ResultOfDecode> {
    let body = base64_decode(&params.body_base64)?;
    let result = Contract::decode_function_response_from_bytes_json(
        params.abi.to_string().to_owned(),
        params.function_name.to_owned(),
        &body,
        params.internal)
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?;
    Ok(ResultOfDecode {
        output: serde_json::from_str(result.as_str())
            .map_err(|err| ApiError::contracts_decode_run_output_failed(err))?
    })
}

pub(crate) fn decode_unknown_input(_context: &mut ClientContext, params: ParamsOfDecodeUnknownRun) -> ApiResult<ResultOfDecodeUnknownRun> {
    let body = base64_decode(&params.body_base64)?;
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
    let body = base64_decode(&params.body_base64)?;
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
    address: &MsgAddressInt
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
            Err(ApiError::storage_phase_failed(id.clone(), &storage.status_change, address))?;
        }
    }


    if let Some(reason) = &transaction.compute.skipped_reason {
        Err(ApiError::tvm_execution_skipped(id.clone(), &reason, address))?;
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
                    address
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
        .map_err(|err| crate::types::apierror_from_sdkerror(&err, ApiError::contracts_run_contract_load_failed));
    if let Err(err) = result {
        if err.code == crate::types::ApiSdkErrorCode::WaitForTimeout.as_number() {
            let result = Contract::load(context.get_client()?, address).await
                .map_err(|err| crate::types::apierror_from_sdkerror(&err, ApiError::contracts_run_contract_load_failed))?;
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
pub async fn retry_call<F, Fut>(retries_count: u8, func: F) -> ApiResult<RecievedTransaction>
    where
        F: Fn(u8) -> Fut,
        Fut: futures::Future<Output=ApiResult<RecievedTransaction>>
{
    let mut result = Err(ApiError::contracts_send_message_failed("Unreacheable"));
    for i in 0..(retries_count + 1) {
        if i != 0 {
            warn!("Message expired. Retry#{}", i);
        }
        result = func(i).await;
        match &result {
            Err(error) => {
                let retry = error.code == ApiSdkErrorCode::MessageExpired as isize ||
                    (error.code == ApiSdkErrorCode::ContractsTvmError as isize && 
                        (error.data["exit_code"] == StdContractError::ReplayProtection as i32 ||
                        error.data["exit_code"] == StdContractError::ExtMessageExpired as i32));
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
) -> ApiResult<RecievedTransaction> {
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
                .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;
    
            let result = Contract::process_message(client, &msg, true).await;
            
            match result {
                Err(err) => 
                    Err(resolve_msg_sdk_error(
                        client, err, &msg.serialized_message, ApiError::contracts_run_failed).await?),
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
    time: Option<u32>,
) -> ApiResult<ResultOfLocalRun> {

    let msg = Contract::construct_call_message_json(
        address.clone(), call_set.clone().into(), false, keys, None, None)
    .map_err(|err| ApiError::contracts_create_run_message_failed(err))?;

    do_local_run_msg(
        context,
        address,
        account,
        Some(call_set.abi),
        Some(call_set.function_name),
        msg.message,
        full_run,
        time)
}

pub(crate) fn do_local_run_msg(
    context: Option<&mut ClientContext>,
    address: MsgAddressInt,
    account: Option<Contract>,
    abi: Option<serde_json::Value>,
    function_name: Option<String>,
    msg: TvmMessage,
    full_run: bool,
    time: Option<u32>,
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
        let result = contract.local_call(msg, time)
            .map_err(|err| 
                match err.downcast_ref::<ton_sdk::SdkError>() {
                    Some(ton_sdk::SdkError::ContractError(exit_code)) =>
                        ApiError::tvm_execution_failed(None, *exit_code, &address),
                    Some(ton_sdk::SdkError::NoFundsError) =>
                        ApiError::low_balance(&address),
                    _ => ApiError::contracts_local_run_failed(err)
                })?;
        let run_result = process_transaction(
            result.transaction, serde_json::Value::Null, abi, function_name, &address, false)?;
        Ok(ResultOfLocalRun {
            output: run_result.output,
            fees: Some(run_result.fees),
            account: Some(result.updated_account),
        })
    } else {
        let messages = contract.local_call_tvm(msg)
            .map_err(|err| ApiError::contracts_local_run_failed(err))?;

        Ok(ResultOfLocalRun {
            output: process_out_messages(&messages, abi, function_name)?,
            fees: None,
            account: None
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

    let result = do_local_run_msg(None, address, Some(account), None, None, msg, true, Some(time));

    if let Err(mut err) = result {
        err.data["original_error"] = serde_json::to_value(main_error).unwrap_or_default();
        err
    } else {
        main_error.data["disclaimer"] = "Local contract call succeded. Can not resolve extended error".into();
        main_error
    }
}

#[cfg(feature = "node_interaction")]
pub(crate) async fn resolve_msg_sdk_error<F: Fn(String) -> ApiError>(
    client: &NodeClient,
    error: failure::Error,
    msg: &[u8],
    default_error: F
) -> ApiResult<ApiError> {
    match error.downcast_ref::<SdkError>() {
        Some(SdkError::MessageExpired{msg_id: _, expire: _, sending_time, block_time: _, block_id: _}) | 
        Some(SdkError::TransactionWaitTimeout{msg_id: _, sending_time, timeout: _, state: _}) => {
            let address = Contract::get_dst_from_msg(msg)
                .map_err(|err| ApiError::invalid_params("message", format!("cannot get target address: {}", err)))?;
            let account = Contract::load(client, &address)
                .await
                .map_err(|err| apierror_from_sdkerror(&err, ApiError::contracts_run_contract_load_failed))?
                .ok_or(ApiError::account_missing(&address))?;
            let main_error = apierror_from_sdkerror(&error, default_error);
            Ok(resolve_msg_error(address, account, msg, *sending_time, main_error))
        }
        _ => Err(apierror_from_sdkerror(&error, default_error))
    }
}

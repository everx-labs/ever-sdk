use crate::abi::{
    CallSet, DecodedMessageBody, DeploySet, FunctionHeader, MessageBodyType, ParamsOfEncodeMessage,
    Signer,
};
use crate::boc::internal::deserialize_object_from_cell;
use crate::boc::tvc::resolve_state_init_cell;
use crate::json_interface::modules::ProcessingModule;
use crate::net::{ParamsOfQuery, ResultOfQuery};
use crate::processing::types::DecodedOutput;
use crate::processing::{
    ErrorCode, ParamsOfProcessMessage, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent, ProcessingResponseType,
};
use crate::tests::GIVER_V2;
use crate::tests::{TestClient, EVENTS_OLD, HELLO};
use crate::tvm::ErrorCode as TvmErrorCode;
use crate::tvm::{AccountForExecutor, ParamsOfRunExecutor, ResultOfRunExecutor};
use crate::utils::conversion::abi_uint;
use api_info::ApiModule;
use ever_struct::scheme::TVC;
use ever_block::{Serializable, StateInit};

fn processing_event_name(e: Option<&ProcessingEvent>) -> &str {
    if let Some(e) = e {
        match e {
            ProcessingEvent::DidSend { .. } => "DidSend",
            ProcessingEvent::FetchFirstBlockFailed { .. } => "FetchFirstBlockFailed",
            ProcessingEvent::FetchNextBlockFailed { .. } => "FetchNextBlockFailed",
            ProcessingEvent::MessageExpired { .. } => "MessageExpired",
            ProcessingEvent::SendFailed { .. } => "SendFailed",
            ProcessingEvent::WillFetchFirstBlock { .. } => "WillFetchFirstBlock",
            ProcessingEvent::WillFetchNextBlock { .. } => "WillFetchNextBlock",
            ProcessingEvent::WillSend { .. } => "WillSend",
            ProcessingEvent::RempSentToValidators { .. } => "RempSentToValidators",
            ProcessingEvent::RempIncludedIntoBlock { .. } => "RempIncludedIntoBlock",
            ProcessingEvent::RempIncludedIntoAcceptedBlock { .. } => {
                "RempIncludedIntoAcceptedBlock"
            }
            ProcessingEvent::RempOther { .. } => "RempOther",
            ProcessingEvent::RempError { .. } => "RempError",
        }
    } else {
        ""
    }
}

fn assert_events(events: &[ProcessingEvent], remp_enabled: bool) {
    let expected = if remp_enabled {
        return;
/*
        &[
            "WillFetchFirstBlock",
            "WillSend",
            "DidSend",
            "RempOther*",
            "RempIncludedIntoBlock",
            "RempOther*",
            "RempIncludedIntoAcceptedBlock",
            "RempOther*",
            "RempIncludedIntoAcceptedBlock*",
            "RempOther*",
        ][..]
*/
    } else {
        &[
            "WillFetchFirstBlock",
            "WillSend",
            "WillSend*",
            "DidSend",
            "WillFetchNextBlock*",
        ][..]
    };
    let mut i = 0;
    for expected in expected {
        if let Some(name) = expected.strip_suffix("*") {
            while i < events.len() && processing_event_name(events.get(i)) == name {
                i += 1;
            }
        } else {
            let event = events.get(i);
            assert_eq!(processing_event_name(event), *expected);
            i += 1;
        }
    }
    assert_eq!(events.len(), i);
}

async fn remp_enabled(client: &TestClient) -> bool {
    let info: ResultOfQuery = client
        .request_async(
            "net.query",
            ParamsOfQuery {
                query: "query{info{rempEnabled}}".to_owned(),
                variables: None,
            },
        )
        .await
        .unwrap_or_default();

    info.result["data"]["info"]["rempEnabled"]
        .as_bool()
        .unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_wait_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS_OLD, Some(2));
    let keys = client.generate_sign_keys();
    let abi = events_abi.clone();

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        println!(
            "{} {:#}",
            chrono::prelude::Utc::now().timestamp_millis(),
            json!(result)
        );
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let send_message = client.wrap_async_callback(
        crate::json_interface::processing::send_message,
        ProcessingModule::api(),
        crate::json_interface::processing::send_message_api(),
    );
    let wait_for_transaction = client.wrap_async_callback(
        crate::json_interface::processing::wait_for_transaction,
        ProcessingModule::api(),
        crate::json_interface::processing::wait_for_transaction_api(),
    );

    let encode_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: DeploySet::some_with_tvc(events_tvc.clone()),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                pubkey: Some(keys.public.clone()),
                ..Default::default()
            }),
            input: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        ..Default::default()
    };

    let encoded = client.encode_message(encode_params.clone()).await.unwrap();

    client
        .get_tokens_from_giver_async(&encoded.address, None)
        .await;

    let encoded = client.encode_message(encode_params).await.unwrap();
    let result = send_message
        .call_with_callback(
            ParamsOfSendMessage {
                message: encoded.message.clone(),
                send_events: true,
                abi: Some(abi.clone()),
            },
            callback.clone(),
        )
        .await
        .unwrap();

    let output = wait_for_transaction
        .call_with_callback(
            ParamsOfWaitForTransaction {
                message: encoded.message.clone(),
                shard_block_id: result.shard_block_id,
                send_events: true,
                abi: Some(abi.clone()),
                sending_endpoints: Some(result.sending_endpoints),
            },
            callback.clone(),
        )
        .await
        .unwrap();

    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![],
            output: None,
        })
    );
    assert_events(&events.lock().await, remp_enabled(&client).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_process_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS_OLD, Some(2));
    let keys = client.generate_sign_keys();
    let abi = events_abi.clone();

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let encode_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: DeploySet::some_with_tvc(events_tvc.clone()),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                expire: None,
                time: None,
                pubkey: Some(keys.public.clone()),
            }),
            input: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        ..Default::default()
    };

    let encoded = client.encode_message(encode_params.clone()).await.unwrap();

    client
        .get_tokens_from_giver_async(&encoded.address, None)
        .await;

    let output = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: encode_params,
                send_events: true,
            },
            callback,
        )
        .await
        .unwrap();

    assert!(output.fees.total_account_fees > 0);
    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![],
            output: None,
        })
    );

    let remp_enabled = remp_enabled(&client).await;
    assert_events(&events.lock().await, remp_enabled);

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let output = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    abi: abi.clone(),
                    address: Some(encoded.address.clone()),
                    call_set: CallSet::some_with_function_and_input(
                        "returnValue",
                        json!({
                            "id": "0x1"
                        }),
                    ),
                    signer: Signer::Keys { keys: keys.clone() },
                    ..Default::default()
                },
                send_events: true,
            },
            callback,
        )
        .await
        .unwrap();
    assert_eq!(output.out_messages.len(), 2);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Event,
                    name: "EventThrown".into(),
                    value: Some(json!({"id": abi_uint(1, 256)})),
                    header: None,
                }),
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Output,
                    name: "returnValue".into(),
                    value: Some(json!({"value0": abi_uint(1, 256)})),
                    header: None,
                })
            ],
            output: Some(json!({
                "value0": abi_uint(1, 256)
            })),
        })
    );

    assert_events(&events.lock().await, remp_enabled);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_resolving() {
    // skip on Evernode SE since it behaves different to real node
    if TestClient::node_se() {
        return;
    }

    let default_client = TestClient::new();
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": TestClient::endpoints(),
            "message_processing_timeout": 10000,
            "message_retries_count": 0,
            "out_of_sync_threshold": 2500,
        },
        "abi": {
            "message_expiration_timeout": 5000,
        }
    }));
    let keys = client.generate_sign_keys();

    let deploy_params = ParamsOfEncodeMessage {
        abi: TestClient::abi(HELLO, None),
        deploy_set: Some(DeploySet {
            tvc: TestClient::tvc(HELLO, None),
            ..Default::default()
        }),
        signer: Signer::Keys { keys: keys.clone() },
        call_set: CallSet::some_with_function("constructor"),
        ..Default::default()
    };

    let address = client
        .encode_message(deploy_params.clone())
        .await
        .unwrap()
        .address;

    let mut run_params = ParamsOfEncodeMessage {
        abi: TestClient::abi(HELLO, None),
        signer: Signer::Keys { keys },
        address: Some(address.clone()),
        call_set: Some(CallSet {
            function_name: "sendAllMoney".to_owned(),
            header: None,
            input: Some(json!({ "dest_addr": client.giver_address().await })),
        }),
        ..Default::default()
    };

    let remp_enabled = remp_enabled(&client).await;
    let original_code = if remp_enabled {
        ErrorCode::MessageRejected
    } else {
        if TestClient::abi_version() == 1 {
            ErrorCode::TransactionWaitTimeout
        } else {
            ErrorCode::MessageExpired
        }
    } as u32;

    // deploy to non-exesting account
    let result = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: deploy_params.clone(),
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap_err();

    log::debug!("{:#}", json!(result));
    if TestClient::node_se() {
        assert_eq!(result.code, TvmErrorCode::AccountMissing as u32);
    } else {
        assert_eq!(result.code, original_code);
        assert_eq!(
            result.data["local_error"]["code"],
            TvmErrorCode::AccountMissing as u32
        );
    }

    // deploy with low balance
    default_client
        .get_tokens_from_giver_async(&address, Some(1000))
        .await;

    let result = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: deploy_params.clone(),
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap_err();

    log::debug!("{:#}", json!(result));
    if TestClient::node_se() {
        assert_eq!(result.code, TvmErrorCode::LowBalance as u32);
    } else {
        assert_eq!(result.code, original_code);
        assert_eq!(
            result.data["local_error"]["code"],
            TvmErrorCode::LowBalance as u32
        );
    }

    // ABI version 1 messages don't expire so previous deploy message can be processed after
    // increasing balance. Need to wait until message will be rejected by all validators
    if TestClient::abi_version() == 1 {
        tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
    }

    // run before deploy
    default_client
        .get_tokens_from_giver_async(&address, None)
        .await;

    let result = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: run_params.clone(),
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap_err();

    log::debug!("{:#}", json!(result));
    if TestClient::node_se() {
        assert_eq!(result.code, TvmErrorCode::AccountCodeMissing as u32);
    } else {
        assert_eq!(result.code, original_code);
        assert_eq!(
            result.data["local_error"]["code"],
            TvmErrorCode::AccountCodeMissing as u32
        );
    }

    // normal deploy
    client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: deploy_params.clone(),
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap();

    // unsigned message
    run_params.signer = Signer::None;
    let result = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: run_params.clone(),
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap_err();

    log::debug!("{:#}", json!(result));
    if TestClient::node_se() {
        assert_eq!(result.code, TvmErrorCode::ContractExecutionError as u32);
        assert_eq!(result.data["exit_code"], 100);
    } else {
        assert_eq!(result.code, original_code);
        assert_eq!(
            result.data["local_error"]["code"],
            TvmErrorCode::ContractExecutionError as u32
        );
        assert_eq!(result.data["local_error"]["data"]["exit_code"], 100)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn test_retries() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": TestClient::endpoints(),
            "message_retries_count": 10,
            "out_of_sync_threshold": 2500,
        },
        "abi": {
            "message_expiration_timeout": 5000,
        }
    }));

    if remp_enabled(&client).await {
        return;
    }

    let client = std::sync::Arc::new(client);
    let (abi, tvc) = TestClient::package(HELLO, Some(2));
    let keys = client.generate_sign_keys();

    let address = client
        .deploy_with_giver_async(
            ParamsOfEncodeMessage {
                abi: abi.clone(),
                deploy_set: DeploySet::some_with_tvc(tvc.clone()),
                call_set: CallSet::some_with_function("constructor"),
                signer: Signer::Keys { keys: keys.clone() },
                ..Default::default()
            },
            None,
        )
        .await;

    let mut tasks = vec![];
    for _ in 0..10 {
        let address = Some(address.clone());
        let abi = abi.clone();
        let keys = keys.clone();
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client
                .net_process_message(
                    ParamsOfProcessMessage {
                        message_encode_params: ParamsOfEncodeMessage {
                            abi,
                            address,
                            call_set: CallSet::some_with_function("touch"),
                            signer: Signer::Keys { keys },
                            ..Default::default()
                        },
                        send_events: false,
                    },
                    TestClient::default_callback,
                )
                .await
        }));
    }
    for result in futures::future::join_all(tasks).await {
        result.unwrap().unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fees() {
    let client = TestClient::new();
    let (abi, tvc) = TestClient::package(GIVER_V2, Some(2));
    let keys = client.generate_sign_keys();

    let address = client
        .deploy_with_giver_async(
            ParamsOfEncodeMessage {
                abi: abi.clone(),
                deploy_set: DeploySet::some_with_tvc(tvc.clone()),
                call_set: CallSet::some_with_function("constructor"),
                signer: Signer::Keys { keys: keys.clone() },
                ..Default::default()
            },
            None,
        )
        .await;

    let params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        address: Some(address.clone()),
        call_set: CallSet::some_with_function_and_input(
            "sendTransaction",
            json!({
                "dest": address.to_string(),
                "value": 100_000_000u64,
                "bounce": false
            }),
        ),
        signer: Signer::Keys { keys },
        ..Default::default()
    };

    let account: String = client.fetch_account(&address).await["boc"]
        .as_str()
        .unwrap()
        .into();

    let message = client.encode_message(params.clone()).await.unwrap();

    let local_result: ResultOfRunExecutor = client
        .request_async(
            "tvm.run_executor",
            ParamsOfRunExecutor {
                account: AccountForExecutor::Account {
                    boc: account,
                    unlimited_balance: None,
                },
                message: message.message,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let run_result = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: params,
                send_events: false,
            },
            TestClient::default_callback,
        )
        .await
        .unwrap();

    assert_eq!(local_result.fees.gas_fee, run_result.fees.gas_fee);
    assert_eq!(
        local_result.fees.out_msgs_fwd_fee,
        run_result.fees.out_msgs_fwd_fee
    );
    assert_eq!(
        local_result.fees.in_msg_fwd_fee,
        run_result.fees.in_msg_fwd_fee
    );
    assert_eq!(local_result.fees.total_output, run_result.fees.total_output);
    assert_eq!(local_result.fees.total_output, 100_000_000u64);
    assert_eq!(
        local_result.fees.total_account_fees - local_result.fees.storage_fee,
        run_result.fees.total_account_fees - run_result.fees.storage_fee
    );
    assert!(run_result.fees.storage_fee >= local_result.fees.storage_fee);

    assert!(local_result.fees.gas_fee > 0);
    assert!(local_result.fees.out_msgs_fwd_fee > 0);
    assert!(local_result.fees.in_msg_fwd_fee > 0);
    assert!(local_result.fees.total_account_fees > 0);
    assert_eq!(
        local_result.fees.total_account_fees,
        local_result.fees.account_fees
    );
    assert_eq!(
        local_result.fees.ext_in_msg_fee,
        local_result.fees.in_msg_fwd_fee
    );
    assert_eq!(
        local_result.fees.total_fwd_fees,
        local_result.fees.out_msgs_fwd_fee
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_deploy_from_tvc_v1() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_state_init) = TestClient::package(EVENTS_OLD, Some(2));
    let state_init_cell =
        resolve_state_init_cell(&client.context(), &events_state_init.clone().unwrap()).unwrap();
    let state_init =
        deserialize_object_from_cell::<StateInit>(state_init_cell.clone(), "state init").unwrap();
    let tvc = base64::encode(
        &TVC::new(
            Some(state_init.code.clone().unwrap()),
            Some("Some Contract".to_string()),
        )
        .write_to_bytes()
        .unwrap(),
    );

    let keys = client.generate_sign_keys();
    let abi = events_abi.clone();

    let encode_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: DeploySet::some_with_tvc(Some(tvc)),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                expire: None,
                time: None,
                pubkey: Some(keys.public.clone()),
            }),
            input: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        ..Default::default()
    };

    let encoded = client.encode_message(encode_params.clone()).await.unwrap();

    client
        .get_tokens_from_giver_async(&encoded.address, None)
        .await;

    let _ = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: encode_params,
                send_events: false,
            },
            |_: ProcessingEvent, _: ProcessingResponseType| async {},
        )
        .await
        .unwrap();

    let output = client
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    abi: abi.clone(),
                    address: Some(encoded.address.clone()),
                    call_set: CallSet::some_with_function_and_input(
                        "returnValue",
                        json!({
                            "id": "0x1"
                        }),
                    ),
                    signer: Signer::Keys { keys: keys.clone() },
                    ..Default::default()
                },
                send_events: false,
            },
            move |_: ProcessingEvent, _: ProcessingResponseType| async {},
        )
        .await
        .unwrap();
    assert_eq!(output.out_messages.len(), 2);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Event,
                    name: "EventThrown".into(),
                    value: Some(json!({"id": abi_uint(1, 256)})),
                    header: None,
                }),
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Output,
                    name: "returnValue".into(),
                    value: Some(json!({"value0": abi_uint(1, 256)})),
                    header: None,
                })
            ],
            output: Some(json!({
                "value0": abi_uint(1, 256)
            })),
        })
    );
}

#[test]
fn test_process_message_sync() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS_OLD, Some(2));
    let keys = client.generate_sign_keys();
    let abi = events_abi.clone();

    let encode_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: DeploySet::some_with_tvc(events_tvc.clone()),
        call_set: Some(CallSet {
            function_name: "constructor".into(),
            header: Some(FunctionHeader {
                expire: None,
                time: None,
                pubkey: Some(keys.public.clone()),
            }),
            input: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        ..Default::default()
    };

    let encoded = client.encode_message_sync(encode_params.clone()).unwrap();

    client.get_tokens_from_giver_sync(&encoded.address, None);

    let output = client
        .process_message_sync(
            ParamsOfProcessMessage {
                message_encode_params: encode_params,
                send_events: true,
            },
        )
        .unwrap();

    assert!(output.fees.total_account_fees > 0);
    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![],
            output: None,
        })
    );

    let output = client
        .process_message_sync(
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    abi: abi.clone(),
                    address: Some(encoded.address.clone()),
                    call_set: CallSet::some_with_function_and_input(
                        "returnValue",
                        json!({
                            "id": "0x1"
                        }),
                    ),
                    signer: Signer::Keys { keys: keys.clone() },
                    ..Default::default()
                },
                send_events: true,
            },
        )
        .unwrap();
    assert_eq!(output.out_messages.len(), 2);
    assert_eq!(
        output.decoded,
        Some(DecodedOutput {
            out_messages: vec![
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Event,
                    name: "EventThrown".into(),
                    value: Some(json!({"id": abi_uint(1, 256)})),
                    header: None,
                }),
                Some(DecodedMessageBody {
                    body_type: MessageBodyType::Output,
                    name: "returnValue".into(),
                    value: Some(json!({"value0": abi_uint(1, 256)})),
                    header: None,
                })
            ],
            output: Some(json!({
                "value0": abi_uint(1, 256)
            })),
        })
    );
}

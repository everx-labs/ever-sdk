use crate::abi::{
    CallSet, DecodedMessageBody, MessageBodyType, DeploySet, FunctionHeader,
    ParamsOfEncodeMessage, Signer,
};
use crate::processing::{
    ErrorCode, ParamsOfProcessMessage, ParamsOfSendMessage, ParamsOfWaitForTransaction,
    ProcessingEvent, ProcessingResponseType,
};
use crate::json_interface::modules::ProcessingModule;
use crate::processing::types::DecodedOutput;
use crate::tests::{TestClient, EVENTS, HELLO};
use crate::tvm::ErrorCode as TvmErrorCode;
use api_info::ApiModule;
use crate::utils::conversion::{abi_uint};

#[tokio::test(core_threads = 2)]
async fn test_wait_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
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

    let encoded = client
        .encode_message(ParamsOfEncodeMessage {
            abi: abi.clone(),
            address: None,
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
            processing_try_index: None,
        })
        .await
        .unwrap();

    client
        .get_grams_from_giver_async(&encoded.address, None)
        .await;

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
    let events = events.lock().await.clone();
    let mut events = events.iter();
    assert!(match events.next() {
        Some(ProcessingEvent::WillFetchFirstBlock {}) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::WillSend { .. }) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::DidSend { .. }) => true,
        _ => false,
    });
    let mut evt = events.next();
    while match evt {
        Some(ProcessingEvent::WillFetchNextBlock { .. }) => true,
        _ => false,
    } {
        evt = events.next();
    }
}

#[tokio::test(core_threads = 2)]
async fn test_process_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
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
        address: None,
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
        processing_try_index: None,
    };

    let encoded = client
        .encode_message(encode_params.clone())
        .await
        .unwrap();

    client
        .get_grams_from_giver_async(&encoded.address, None)
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
    let events = events.lock().await.clone();
    let mut events = events.iter();
    assert!(match events.next() {
        Some(ProcessingEvent::WillFetchFirstBlock {}) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::WillSend { .. }) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::DidSend { .. }) => true,
        _ => false,
    });
    let mut evt = events.next();
    while match evt {
        Some(ProcessingEvent::WillFetchNextBlock { .. }) => true,
        _ => false,
    } {
        evt = events.next();
    }

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
                    deploy_set: None,
                    call_set: CallSet::some_with_function_and_input(
                        "returnValue",
                        json!({
                            "id": "0x1"
                        }),
                    ),
                    signer: Signer::Keys { keys: keys.clone() },
                    processing_try_index: None,
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

    let events = events.lock().await.clone();
    let mut events = events.iter();
    assert!(match events.next() {
        Some(ProcessingEvent::WillFetchFirstBlock {}) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::WillSend { .. }) => true,
        _ => false,
    });
    assert!(match events.next() {
        Some(ProcessingEvent::DidSend { .. }) => true,
        _ => false,
    });
    let mut evt = events.next();
    while match evt {
        Some(ProcessingEvent::WillFetchNextBlock { .. }) => true,
        _ => false,
    } {
        evt = events.next();
    }
}

#[tokio::test(core_threads = 2)]
async fn test_error_resolving() {
    // skip on Node SE since it behaves different to real node
    if TestClient::node_se() {
        return;
    }

    let default_client = TestClient::new();
    let client = TestClient::new_with_config(json!({
        "network": {
            "server_address": TestClient::network_address(),
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
            initial_data: None,
            tvc: TestClient::tvc(HELLO, None),
            workchain_id: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor")
    };

    let address = client.encode_message(deploy_params.clone()).await.unwrap().address;

    let mut run_params = ParamsOfEncodeMessage {
        abi: TestClient::abi(HELLO, None),
        deploy_set: None,
        signer: Signer::Keys { keys },
        processing_try_index: None,
        address: Some(address.clone()),
        call_set: Some(CallSet {
            function_name: "sendAllMoney".to_owned(),
            header: None,
            input: Some(json!({
                "dest_addr": TestClient::giver_address()
            }))
        }),
    };

    
    // deploy to non-exesting account
    let result = client.net_process_message(ParamsOfProcessMessage { 
            message_encode_params: deploy_params.clone(),
            send_events: false,
        },
        TestClient::default_callback
    ).await.unwrap_err();

    assert_eq!(result.code, TvmErrorCode::AccountMissing as u32);


    // deploy with low balance
    default_client.get_grams_from_giver_async(&address, Some(1000)).await;

    let result = client.net_process_message(ParamsOfProcessMessage { 
            message_encode_params: deploy_params.clone(),
            send_events: false,
        },
        TestClient::default_callback
    ).await.unwrap_err();

    let original_code = if TestClient::abi_version() == 1 {
        ErrorCode::TransactionWaitTimeout
    } else {
        ErrorCode::MessageExpired
    } as isize;

    assert_eq!(result.code, TvmErrorCode::LowBalance as u32);
    assert_eq!(result.data["original_error"]["code"], original_code);

    // ABI version 1 messages don't expire so previous deploy message can be processed after
    // increasing balance. Need to wait until message will be rejected by all validators
    if TestClient::abi_version() == 1 {
        tokio::time::delay_for(tokio::time::Duration::from_secs(40)).await;
    }


    // run before deploy
    default_client.get_grams_from_giver_async(&address, None).await;

    let result = client.net_process_message(ParamsOfProcessMessage { 
            message_encode_params: run_params.clone(),
            send_events: false,
        },
        TestClient::default_callback
    ).await.unwrap_err();

    assert_eq!(result.code, TvmErrorCode::AccountCodeMissing as u32);
    assert_eq!(result.data["original_error"]["code"], original_code);


    // normal deploy
    client.net_process_message(ParamsOfProcessMessage { 
            message_encode_params: deploy_params.clone(),
            send_events: false,
        },
        TestClient::default_callback
    ).await.unwrap();


    // unsigned message
    run_params.signer = Signer::None;
    let result = client.net_process_message(ParamsOfProcessMessage { 
            message_encode_params: run_params.clone(),
            send_events: false,
        },
        TestClient::default_callback
    ).await.unwrap_err();

    assert_eq!(result.code, TvmErrorCode::ContractExecutionError as u32);
    assert_eq!(result.data["original_error"]["code"], original_code);
}
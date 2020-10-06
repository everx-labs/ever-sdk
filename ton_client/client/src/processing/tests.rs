use crate::abi::{
    encode_message, encode_message_method, Abi, CallSet, DecodedMessageBody, DecodedMessageType,
    DeploySet, FunctionHeader, ParamsOfEncodeMessage, Signer,
};
use crate::processing::{
    process_message_api, process_message_api_method, send_message_api, send_message_api_method,
    wait_for_transaction_api, wait_for_transaction_api_method, MessageSource,
    ParamsOfProcessMessage, ParamsOfSendMessage, ParamsOfWaitForTransaction, ProcessingEvent,
    ProcessingResponseType
};

use crate::processing::types::AbiDecodedOutput;
use crate::tests::{TestClient, EVENTS};

#[tokio::test(core_threads = 2)]
async fn test_wait_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = client.generate_sign_keys();
    let abi = Abi::Serialized(events_abi.clone());

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let encode_message = client.wrap_async(encode_message, encode_message_method);
    let send_message = client.wrap_async_callback(send_message_api, send_message_api_method);
    let wait_for_transaction = client.wrap_async_callback(wait_for_transaction_api, wait_for_transaction_api_method);

    let encoded = encode_message
        .call(ParamsOfEncodeMessage {
            abi: abi.clone(),
            address: None,
            deploy_set: Some(DeploySet {
                workchain_id: None,
                tvc: events_tvc.clone(),
                initial_data: None,
            }),
            call_set: Some(CallSet {
                function_name: "constructor".into(),
                header: Some(FunctionHeader {
                    expire: None,
                    time: None,
                    pubkey: Some(keys.public.clone()),
                }),
                input: None,
            }),
            signer: Signer::WithKeys(keys.clone()),
            processing_try_index: None,
        })
        .await;

    client
        .get_grams_from_giver_async(&encoded.address, None)
        .await;

    let result = send_message
        .call_with_callback(ParamsOfSendMessage {
                message: encoded.message.clone(),
                send_events: true,
                abi: Some(abi.clone()),
            },
            callback.clone()
        )
        .await;

    let output = wait_for_transaction
        .call_with_callback(ParamsOfWaitForTransaction {
                message: encoded.message.clone(),
                shard_block_id: result.shard_block_id,
                send_events: true,
                abi: Some(abi.clone()),
            },
            callback.clone()
        )
        .await;

    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.abi_decoded,
        Some(AbiDecodedOutput {
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
    assert!(match evt {
        Some(ProcessingEvent::TransactionReceived { .. }) => true,
        _ => false,
    });
}

#[tokio::test(core_threads = 2)]
async fn test_process_message() {
    TestClient::init_log();
    let client = TestClient::new();
    let (events_abi, events_tvc) = TestClient::package(EVENTS, Some(2));
    let keys = client.generate_sign_keys();
    let abi = Abi::Serialized(events_abi.clone());

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let encode_message = client.wrap_async(encode_message, encode_message_method);
    let process_message = client.wrap_async_callback(process_message_api, process_message_api_method);

    let encoded = encode_message
        .call(ParamsOfEncodeMessage {
            abi: abi.clone(),
            address: None,
            deploy_set: Some(DeploySet {
                workchain_id: None,
                tvc: events_tvc.clone(),
                initial_data: None,
            }),
            call_set: Some(CallSet {
                function_name: "constructor".into(),
                header: Some(FunctionHeader {
                    expire: None,
                    time: None,
                    pubkey: Some(keys.public.clone()),
                }),
                input: None,
            }),
            signer: Signer::WithKeys(keys.clone()),
            processing_try_index: None,
        })
        .await;

    client
        .get_grams_from_giver_async(&encoded.address, None)
        .await;

    let output = process_message
        .call_with_callback(ParamsOfProcessMessage {
                message: MessageSource::Encoded {
                    message: encoded.message.clone(),
                    abi: Some(abi.clone()),
                },
                send_events: true,
            },
            callback
        )
        .await;

    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.abi_decoded,
        Some(AbiDecodedOutput {
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
    assert!(match evt {
        Some(ProcessingEvent::TransactionReceived { .. }) => true,
        _ => false,
    });

    let events = std::sync::Arc::new(tokio::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |result: ProcessingEvent, response_type: ProcessingResponseType| {
        assert_eq!(response_type, ProcessingResponseType::ProcessingEvent);
        let events_copy = events_copy.clone();
        async move {
            events_copy.lock().await.push(result);
        }
    };

    let output = process_message
        .call_with_callback(ParamsOfProcessMessage {
                message: MessageSource::AbiEncodingParams(ParamsOfEncodeMessage {
                    abi: abi.clone(),
                    address: Some(encoded.address.clone()),
                    deploy_set: None,
                    call_set: Some(CallSet {
                        function_name: "returnValue".into(),
                        header: None,
                        input: Some(json!({
                            "id": "0x1"
                        })),
                    }),
                    signer: Signer::WithKeys(keys.clone()),
                    processing_try_index: None,
                }),
                send_events: true,
            },
            callback
        )
        .await;
    assert_eq!(output.out_messages.len(), 2);
    assert_eq!(
        output.abi_decoded,
        Some(AbiDecodedOutput {
            out_messages: vec![
                Some(DecodedMessageBody {
                    message_type: DecodedMessageType::Event,
                    name: "EventThrown".into(),
                    value: json!({"id": "0x1"}),
                    header: None,
                }),
                Some(DecodedMessageBody {
                    message_type: DecodedMessageType::FunctionOutput,
                    name: "returnValue".into(),
                    value: json!({"value0": "0x1"}),
                    header: None,
                })
            ],
            output: Some(json!({
                "value0": "0x1"
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
    assert!(match evt {
        Some(ProcessingEvent::TransactionReceived { .. }) => true,
        _ => false,
    });
}

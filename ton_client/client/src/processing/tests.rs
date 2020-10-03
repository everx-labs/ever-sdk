use crate::abi::{
    encode_message, encode_message_info, Abi, CallSet, DecodedMessageBody, DecodedMessageType,
    DeploySet, FunctionHeader, ParamsOfEncodeMessage, Signer,
};
use crate::error::ApiResult;
use crate::processing::{
    process_message, process_message_info, send_message, send_message_info,
    wait_for_transaction, wait_for_transaction_info, CallbackParams, MessageSource,
    ParamsOfProcessMessage, ParamsOfSendMessage, ParamsOfWaitForTransaction, ProcessingEvent,
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

    let events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |event: ApiResult<ProcessingEvent>| {
        if let Ok(event) = event {
            events_copy.lock().unwrap().push(event);
        }
    };

    let callback_id = client.register_callback(callback);

    let encode_message = client.wrap_async(encode_message, encode_message_info);
    let send_message = client.wrap_async(send_message, send_message_info);
    let wait_for_transaction = client.wrap_async(wait_for_transaction, wait_for_transaction_info);

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
        .call(ParamsOfSendMessage {
            message: encoded.message.clone(),
            events_handler: Some(CallbackParams::with_id(callback_id)),
            abi: Some(abi.clone()),
        })
        .await;

    let output = wait_for_transaction
        .call(ParamsOfWaitForTransaction {
            message: encoded.message.clone(),
            shard_block_id: result.shard_block_id,
            events_handler: Some(CallbackParams::with_id(callback_id)),
            abi: Some(abi.clone()),
        })
        .await;

    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.abi_decoded,
        Some(AbiDecodedOutput {
            out_messages: vec![],
            output: None,
        })
    );
    client.unregister_callback(callback_id);
    let events = events.lock().unwrap().clone();
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

    let events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |event: ApiResult<ProcessingEvent>| {
        if let Ok(event) = event {
            events_copy.lock().unwrap().push(event);
        }
    };

    let callback_id = client.register_callback(callback);

    let encode_message = client.wrap_async(encode_message, encode_message_info);
    let process_message = client.wrap_async(process_message, process_message_info);

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
        .call(ParamsOfProcessMessage {
            message: MessageSource::Encoded {
                message: encoded.message.clone(),
                abi: Some(abi.clone()),
            },
            events_handler: Some(CallbackParams::with_id(callback_id)),
        })
        .await;

    assert_eq!(output.out_messages.len(), 0);
    assert_eq!(
        output.abi_decoded,
        Some(AbiDecodedOutput {
            out_messages: vec![],
            output: None,
        })
    );
    client.unregister_callback(callback_id);
    let events = events.lock().unwrap().clone();
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

    let events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let events_copy = events.clone();
    let callback = move |event: ApiResult<ProcessingEvent>| {
        if let Ok(event) = event {
            events_copy.lock().unwrap().push(event);
        }
    };

    let callback_id = client.register_callback(callback);

    let output = process_message
        .call(ParamsOfProcessMessage {
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
            events_handler: Some(CallbackParams::with_id(callback_id)),
        })
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
    client.unregister_callback(callback_id);

    let events = events.lock().unwrap().clone();
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

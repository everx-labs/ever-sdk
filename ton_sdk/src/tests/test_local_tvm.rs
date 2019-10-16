use super::*;
use tvm::assembler::compile_code;
use tvm::block::{
    Message,
    ExternalInboundMessageHeader,
    MsgAddressExt,
    MsgAddressInt,
    Grams,
    StateInit,
};
use tvm::types::AccountId;
use crate::test_piggy_bank::PIGGY_BANK_CONTRACT_ABI;

#[test]
#[ignore]
fn test_local_contract_call() {
    // sample contract
    let code = compile_code("
        SETCP0
        THROWIF 100  ; check if message is external
        PLDU 8
        MULCONST 8
        PUSHROOT
        CTOS
        SWAP
        SDSKIPFIRST
        LDSLICE 8
        PLDSLICE 8
        PUSHCONT {
        ; build external outbound message
        ; s0 - body: slice
        ; returns: msg: cell

            NEWC
            TWO
            STONES ; ext_out_msg_info$11

            TWO
            STZEROES ; addr_none$00 - will be changed on action phase

            TWO
            STZEROES ; addr_none$00

            PUSHINT 0
            STUR 64  ; created_lt:uint64
            PUSHINT 0
            STUR 32  ; created_at:uint32

            TWO
            STZEROES ; Maybe StateInit to 0bit and body Either: left$0
            STSLICE
            ENDC
        }
        ROT
        OVER
        CALLX
        PUSHINT 0
        SENDRAWMSG

        CALLX
        PUSHINT 0
        SENDRAWMSG
    ").unwrap();
    let code = code.cell();
    let data = SliceData::from_raw(vec![1, 2, 3, 4], 32);
    let data = data.cell();
    let mut msg = Message::with_ext_in_header(ExternalInboundMessageHeader {
        src: MsgAddressExt::with_extern(SliceData::from_raw(vec![11; 32], 256)).unwrap(),
        dst: MsgAddressInt::AddrNone,
        import_fee: Grams::zero(),
    });
    *msg.body_mut() = Some(SliceData::from_raw(vec![1], 8));

    let msgs = local_contract_call(code.clone(), Some(data.clone()), &msg).unwrap();
    assert_eq!(msgs.len(), 2);

    assert_eq!(msgs[0].body(), Some(SliceData::from_raw(vec![2], 8)));
    assert_eq!(msgs[1].body(), Some(SliceData::from_raw(vec![3], 8)));

    *msg.body_mut() = Some(SliceData::from_raw(vec![2], 8));

    let msgs = local_contract_call(code.clone(), Some(data.clone()), &msg).unwrap();
    assert_eq!(msgs.len(), 2);

    assert_eq!(msgs[0].body(), Some(SliceData::from_raw(vec![3], 8)));
    assert_eq!(msgs[1].body(), Some(SliceData::from_raw(vec![4], 8)));
}

const STATE_INIT: &str = r#"{"code": "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", "data": "te6ccgEBGgEAqwABAcABAgPOYBcCAgOkwAQDAEGiOgiruoA5inQNlubKrobVXYNzL+iNNUxN0LUUoEQD51gCASAGBQARAAAAAAAAAB7gAQEgBwIDzkAJCAAD22QCASAQCgIBIA0LAgEgFQwAAxhgAgEgDw4AAxngAAMIIAIBIBQRAgEgExIAAxlgAAMbYAIBIBYVAAMb4AADFOACAdIZGAAhoAAAFt0xlGvwAAAAAAAOpggAAbw="}"#;

#[test]
fn test_local_piggy_call() {
    let state_init: StateInit = serde_json::from_str(STATE_INIT).expect("Error parsing state init");

    let id = hex::decode("e6392da8a96f648098f818501f0211f27c89675e5f196445d211947b48e7c85b").unwrap();
    let address = MsgAddressInt::with_standart(None, 0, AccountId::from(id)).unwrap();
    let (msg, _) = crate::Contract::construct_call_message_json(
        address.into(),
        "getGoal".to_owned(),
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(), None)
        .expect("Error creating message");
    let msg = crate::Contract::deserialize_message(&msg).unwrap();
    println!("msg {}", msg.body().unwrap());
    let messages = crate::Contract::local_contract_call_by_data(state_init, msg).expect("Error calling contract");

    println!("messages count {}", messages.len());
    assert!(messages.len() == 1);

    let answer = crate::Contract::decode_function_response_json(
        PIGGY_BANK_CONTRACT_ABI.to_owned(), "getGoal".to_owned(), messages[0].body().expect("Message has no body"))
            .expect("Error decoding result");

    println!("answer {}", answer);

    assert_eq!(answer, r#"{"value0":["0x53","0x6f","0x6d","0x65","0x20","0x67","0x6f","0x61","0x6c"]}"#);
}
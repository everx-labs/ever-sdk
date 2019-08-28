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

const STATE_INIT: &str = r#"{"code":"te6ccoECHQEAAmkAABwAIAAlADwAQQBFAEoATwCdAL8AxADMAU4BiAHaAd8B4wHpAfgCNQI5Aj4CQwJKAlECVgJdAmQCaQEy/wCJ9AUhwQGTePSgm3j0DfK0gCD0ofIz4gEBAcACAgEgBAMAKf/gAcPABIXHwAQHTBwHydNMfAfACgIB1g8FAQGsBgIBSAoHAgFiCQgAl7fZDiUW+1E0NQwcQF49A4wyM7J0I40ji7Ics9Bcs9Acs9AcM8LP3DPCx9xz0BczzUB10mkvpRxz0DOmXHPQQHIzsnPFOLJ2HD7ANiAAP7dZ1T/MdY/AXBtePQWcQF49BbIzO1E0Nb/MM8Wye1UgAgFIDgsBCbllhHqQDAH+MYEBAJgBiwrXJgHXGNgw0//R7UdvEG8XbxDtRNDUMHABePQOMNM/MCG7jk8ggA+AZKmFXKEyiwhwWI4+/vkAU25kQmR5SW50Ae1HbxBvGPpCbxLIz4ZAygfL/8nQjhfIz4Ugz4oAQIEBAM9AzgH6AoBrz0DOydhw+wDY3wGLCA0AcHASAwHtR28Qbxj6Qm8SyM+GQMoHy//J0I4XyM+FIM+KAECBAQDPQM4B+gKAa89AzsnYgQCA+wAwAJ+4igfSC32omhqGDgAvHoHGGmfmGRln+ToRxpHF2Q5Z6C5Z6A5Z6A4Z4WfuGeFj7jnoC5nmoDrpNJfSjjnoGdMuOeggORnZOeKcWTsOH2AbEAIBIBwQAQEwEQIDz8ATEgAZNDXKAX6QPpA+gBvBIAFzCDTB9MfMAHydIn0BYAg9A7yqdcLAI4ZIMcC8mjVIMcA8mgh+QEB7UTQ1wv/+RDyqJcgxwKS1DHf4oBQBAcAVAgFIGRYCAWIYFwAJt9kOJRAACbdZ1T/QAgFIGxoACbllhHqYAAm4igfSCAAFNswg","data":"te6ccoEBBAEAQAAjKDVAAUBe+JrhSU+8ZE9gTfXktw9EUCeXHDWDE8mIUYERzD3Y+AECAc8DAgAVIJU29tZSBnb2FsgAEQAAAAAAAAAe4A=="}"#;
const PIGGY_BANK_CONTRACT_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
        "name": "transfer",
        "signed": true,
        "inputs": [{"name": "to", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "getTargetAmount",
        "inputs": [],
        "outputs": [{"name": "amount", "type": "uint64"}]
    }, {
        "name": "getGoal",
        "inputs": [],
        "outputs": [{"name": "goal", "type": "uint8[]"}]
    }, {
        "name": "constructor",
        "inputs": [
				    {"name": "amount","type": "uint64"},
            {"name": "goal","type": "uint8[]"}
        ],
        "outputs": []
    }]
}"#;

#[test]
fn test_local_piggy_call() {
    let state_init: StateInit = serde_json::from_str(STATE_INIT).expect("Error parsing state init");

    let id = hex::decode("db5cd0a25f4daccdb17a7f216474b508a51696ad26a13526dc50cade7ca70256").unwrap();
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

    assert_eq!(answer, r#"{"goal":["0x53","0x6f","0x6d","0x65","0x20","0x67","0x6f","0x61","0x6c"]}"#);
}
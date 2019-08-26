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

const STATE_INIT: &str = r#"{"code":"te6ccoECJQEABCwAABwAIAAlADwAQQBFAEoATwCAAIUBBQENAY8B9gI5Aj4CawJzAvQDYAOZA54DogOoA7cD7APwA/UD+gQBBAYEDQQUBBkEIAQnBCwBMv8AifQFIcEBk3j0oJt49A3ytIAg9KHyM+IBAQHAAgIBIAQDACn/4AHDwASFx8AEB0wcB8nTTHwHwAoCAdYVBQEBrAYCASAPBwIBIAkIAF28E8mNIYwICATADFhWuTAOuMbGjp/+j2omhrpkCAgHot+VTkZnaiaGoY54tk9qpAICdQsKAfm0mwx7GMCAgEwAxYVrkwDrjGxo6f/o9qJoa6ZAgIB6B/lUupC8egd5WemD6IC5kLx6B3lZ6Y/ogLkQvHoHeVnpv+iAuJC8egd5WZjkZ8NAZwDKwYNUhhDLwYNQLGWDgPQY54WDgMrBg1SGEMvBg1AsZYOA9BjnhYPlg+ToQBQBCbUQ68JADAH+MYEBAJgBiwrXJgHXGNgB0//RAYECAJgBiwrXJgHXGNjRISDtRNDXTIEBAPQP8qlwIXj0DvKz0//RE/kQ8qh0IXj0DvKz0x/RAXMhePQO8rPTH9ESoPgjIFi8nnUiePQO8rPTB9FzuvJ838jLH8nQdFh49BaLEDh1WHj0FnDtRA0ByNDXTIEBAPQO8qkBciF49A7yswFxIXj0DvKzASTtRNDXTIEBAPQXyMztRNDUMc8Wye1UghAk9OFVcMjLB8sfz4aAzgHTf9GVgwapDCGXgwagWMsHAegxzwsHydBYMAHT/9FwWXAOAIKOPv75AFNuZEJkeUludAHtR28Qbxj6Qm8SyM+GQMoHy//J0I4XyM+FIM+KAECBAQDPQM4B+gKAa89AzsnYcPsA2AICcBEQAFW2G0V3DGBAQCYAYsK1yYB1xjY0e1E0CDXSnG63AFwbYEBAPQWyMzOye1UgAQm2a9hBoBIB/DGBAQCYAYsK1yYB1xjYgQEAmAGLCtcmAdcY2IEBAJgBiwrXJgHXGNjTANMGWI4VcXcDklUgnNMA0wYDIKYHBVmsoOgx3tMA0wZYjhVxdwOSVSCc0wDTBgMgpgcFWayg6DHe0V4wINdJgQEAuvK2IddJgQEAuvK2ItdJgQEAuhMB0vK2I8EB8nYkwQHydlUwcG149BZxAXj0FiHIy3/J0DJyAXj0FiHIyx/J0DJzAXj0FvgjyMsfydB0WHj0FosQCHVYePQWIPkAAtP/0e1E0NdMgQEA9BfIzO1E0NQxzxbJ7VTIz4aAy//J0BQAbo40ji7Ics9Bcs9Acs9AcM8LP3DPCx9xz0BczzUB10mkvpRxz0DOmXHPQQHIzsnPFOLJ2HD7ANgCASAkFgEBMBcCA8/AGRgAGTQ1ygF+kD6QPoAbwSABYwg0wfTHzAB8nSJ9AWAIPQO8qnXCwCOGSDHAvJo1SDHAPJoIfkBAe1E0NcL//kQ8qjegGgEBwBsCASAhHAIBIB4dAAm8E8mNJgICdSAfAAm0mwx7IAAJtRDrwiACAnAjIgAJthtFdxAACbZr2EGwAAU2zCA=","data":"te6ccoECGAEAAVIAACMAKABMAFEAVgBdAGIAZwBuAIEAhgCqAK8AtAC4AL8AxADJANAA4wDoAQsBLgFSAUBxvUePKT57atg9nEBbFj9t42uJ2m86o+Fl64rJ/O+goQECAWIKAgFBv0RERERERERERERERERERERERERERERERERERERERERFAwIBywYEAgFIBQ4ACRdY3tpgAgEgFAcCASAJCAAJAABUYCAAIQAAAAAAAAAAAAAAAEqBfIAgAgEgFwsBQb8ERERERERERERERERERERERERERERERERERERERERERgwCAcsQDQIBSA8OAAMAIAAJF1je2SACASAUEQIBIBMSAAkAAAByIAAhAAAAAAAAAAAAAAAAAAAAHuACASAWFQBBNTjb/XO0thHRz6c7NkAec/cb0VoXT8GpcDyyoHFXBMzgAEEcb1Hjyk+e2rYPZxAWxY/beNridpvOqPhZeuKyfzvoKGAAQ9+XmH79mE6HwCz+e/M0j9sgHeKrnKhrV3uapI7mBSErZXA="}"#;
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

    let id = hex::decode("149941f340d5f5b03b4d6e92e79baa713dba4b45d28c8f251f5b8855931183fc").unwrap();
    let address = MsgAddressInt::with_standart(None, 0, AccountId::from(id)).unwrap();
    let (msg, _) = crate::Contract::construct_call_message_json(
        address.into(),
        "getGoal".to_owned(),
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(), None)
        .expect("Error creating message");
    let msg = crate::Contract::deserialize_message(&msg).unwrap();
    let messages = crate::Contract::local_contract_call_by_data(state_init, msg).expect("Error calling contract");

    println!("messages count {}", messages.len());
    for out_message in messages {
        println!("{}", serde_json::to_string_pretty(&out_message).expect("Error serializing message"));
    }
}
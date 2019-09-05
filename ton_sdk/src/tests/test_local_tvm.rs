use super::*;
use tvm::assembler::compile_code;
use tvm::block::{
    Message,
    ExternalInboundMessageHeader,
    MsgAddressExt,
    MsgAddressInt,
    Grams,
};

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
    let data = SliceData::from_raw(vec![1, 2, 3, 4], 32);
    let data = data.cell();
    let mut msg = Message::with_ext_in_header(ExternalInboundMessageHeader {
        src: MsgAddressExt::with_extern(SliceData::from_raw(vec![11; 32], 256)).unwrap(),
        dst: MsgAddressInt::AddrNone,
        import_fee: Grams::zero(),
    });
    *msg.body_mut() = Some(SliceData::from_raw(vec![1], 8));

    let msgs = local_contract_call(code.clone(), data.clone(), &msg).unwrap();
    assert_eq!(msgs.len(), 2);

    assert_eq!(msgs[0].body(), Some(SliceData::from_raw(vec![2], 8)));
    assert_eq!(msgs[1].body(), Some(SliceData::from_raw(vec![3], 8)));

    *msg.body_mut() = Some(SliceData::from_raw(vec![2], 8));

    let msgs = local_contract_call(code.clone(), data.clone(), &msg).unwrap();
    assert_eq!(msgs.len(), 2);

    assert_eq!(msgs[0].body(), Some(SliceData::from_raw(vec![3], 8)));
    assert_eq!(msgs[1].body(), Some(SliceData::from_raw(vec![4], 8)));
}

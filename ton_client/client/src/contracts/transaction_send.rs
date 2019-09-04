use crypto::types::{KeyPair, u256_encode, u256_decode, u256_zero};
use ton_sdk::{Contract};
use rand::{thread_rng, Rng};
use tvm::types::AccountId;
use tvm::block::{
    Message,
    MsgAddressExt,
    MsgAddressInt,
    InternalMessageHeader,
    Grams,
    ExternalInboundMessageHeader,
    CurrencyCollection,
    Serializable,
};
use tvm::bitstring::Bitstring;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct SendParams {
    pub fromAddress: String,
    pub toAddress: String,
    pub amount: String,
    pub keyPair: KeyPair,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
pub struct SendResult {
    pub id: String,
}


pub fn api_send(params: SendParams) -> Result<SendResult, String> {
    let src: AccountId = u256_decode(&params.fromAddress)?;
    let dst: AccountId = u256_decode(&params.toAddress)?;
    let value: u128 = params.amount.clone().parse()
        .map_err(|_| format!("Invalid amount string [{}]", params.amount))?;

    let mut rng = thread_rng();
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(&Bitstring::from(rng.gen::<u64>())).unwrap(),
            dst: MsgAddressInt::with_standart(None, 0, src.clone()).unwrap(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams::from(value);

    let int_msg_hdr = InternalMessageHeader::with_addresses(
        MsgAddressInt::with_standart(None, 0, src).unwrap(),
        MsgAddressInt::with_standart(None, 0, dst).unwrap(),
        balance);

    msg.body = Some(int_msg_hdr.write_to_new_cell().unwrap().into());
    let id = msg.clone().transaction_id.unwrap_or(u256_zero());
    Contract::send_message(msg).unwrap();
    Ok(SendResult { id: u256_encode(&id) })
}


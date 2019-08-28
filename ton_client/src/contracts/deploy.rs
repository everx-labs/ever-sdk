use std::sync::Arc;
use crypto::keys::{KeyPair, u256_encode, decode_public_key, account_encode, generic_id_encode, account_decode};
use ton_sdk::{Contract, ContractImage};
use tvm::cells_serialization::BagOfCells;
use tvm::stack::{CellData, SliceData};

use tvm::block::{TransactionId, TransactionProcessingStatus};
use ton_sdk::Transaction;
use futures::Stream;
use contracts::{EncodedUnsignedMessage, EncodedMessage};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfDeploy {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub imageBase64: String,
    pub keyPair: KeyPair,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfEncodeUnsignedDeployMessage {
    pub abi: serde_json::Value,
    pub constructorParams: serde_json::Value,
    pub imageBase64: String,
    pub publicKeyHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ResultOfEncodeUnsignedDeployMessage {
    pub encoded: EncodedUnsignedMessage,
    pub addressHex: String,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ParamsOfGetDeployAddress {
    pub abi: serde_json::Value,
    pub imageBase64: String,
    pub keyPair: KeyPair,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfDeploy {
    pub address: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ResultOfEncodeDeployMessage {
    pub address: String,
    pub messageId: String,
    pub messageIdBase64: String,
    pub messageBodyBase64: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub(crate) struct ParamsOfSendGrams {
    pub fromAccount: String,
    pub toAccount: String,
    pub amount: u128,
}

pub(crate) fn deploy(context: &mut Context, params: ParamsOfDeploy) -> ApiResult<ResultOfDeploy> {
    debug!("-> contracts.deploy({})", params.constructorParams.to_string());

    let key_pair = params.keyPair.decode()?;

    let contract_image = create_image(&params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.account_id();
    debug!("image prepared with address: {}", account_encode(&account_id));

    debug!("send 100 nano grams from zero account");
    let msg = create_external_transfer_funds_message(
        &AccountId::from([0_u8; 32]),
        &account_id,
        100);
    send_message(msg)?;

    debug!("deploy");
    let tr_id = deploy_contract(&params, contract_image, &key_pair)?;
    debug!("deploy transaction: {}", u256_encode(&tr_id.into()));

    debug!("<-");
    Ok(ResultOfDeploy {
        address: account_encode(&account_id)
    })
}

pub(crate) fn get_address(context: &mut Context, params: ParamsOfGetDeployAddress) -> ApiResult<String> {
    let key_pair = params.keyPair.decode()?;
    let contract_image = create_image(&params.imageBase64, &key_pair.public)?;
    let account_id = contract_image.account_id();
    Ok(account_encode(&account_id))
}

pub(crate) fn encode_message(context: &mut Context, params: ParamsOfDeploy) -> ApiResult<ResultOfEncodeDeployMessage> {
    debug!("-> contracts.deploy.message({})", params.constructorParams.to_string());

    let keys = params.keyPair.decode()?;

    let contract_image = create_image(&params.imageBase64, &keys.public)?;
    let account_id = contract_image.account_id();
    debug!("image prepared with address: {}", account_encode(&account_id));
    let account_id = contract_image.account_id();
    let (message_body, message_id) = Contract::construct_deploy_message_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        contract_image,
        Some(&keys)).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;

    debug!("<-");
    Ok(ResultOfEncodeDeployMessage {
        address: account_encode(&account_id),
        messageId: generic_id_encode(&message_id),
        messageIdBase64: base64::encode(message_id.data.as_slice()),
        messageBodyBase64: base64::encode(&message_body),
    })
}

fn serialize_message(msg: tvm::block::Message) -> ApiResult<(Vec<u8>, MessageId)> {
    let cells = msg.write_to_new_cell()
        .map_err(|err| ApiError::contracts_create_send_grams_message_failed(err))?;

    let cells = &Arc::<CellData>::from(cells);
    let id = cells.repr_hash();

    let mut data = Vec::new();
    let bag = BagOfCells::with_root(cells);
    bag.write_to(&mut data, false)
        .map_err(|err| ApiError::contracts_create_send_grams_message_failed(err))?;

    Ok((data, id.into()))
}

pub(crate) fn encode_unsigned_message(context: &mut Context, params: ParamsOfEncodeUnsignedDeployMessage) -> ApiResult<ResultOfEncodeUnsignedDeployMessage> {
    let public = decode_public_key(&params.publicKeyHex)?;
    let image = create_image(&params.imageBase64, &public)?;
    let address_hex = image.account_id().to_hex_string();
    let encoded = ton_sdk::Contract::get_deploy_message_bytes_for_signing(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image
    ).map_err(|err| ApiError::contracts_create_deploy_message_failed(err))?;
    Ok(ResultOfEncodeUnsignedDeployMessage {
        encoded: EncodedUnsignedMessage {
            unsignedBytesBase64: base64::encode(&encoded.message),
            bytesToSignBase64: base64::encode(&encoded.data_to_sign),
        },
        addressHex: address_hex
    })
}

pub(crate) fn encode_send_grams_message(context: &mut Context, params: ParamsOfSendGrams) -> ApiResult<EncodedMessage> {
    let msg = create_external_transfer_funds_message(
        &account_decode(&params.fromAccount)?,
        &account_decode(&params.toAccount)?,
        params.amount);

    let (body, id) = serialize_message(msg)?;
    Ok(EncodedMessage {
        messageId: u256_encode(&id),
        messageIdBase64: base64::encode(id.as_slice()),
        messageBodyBase64: base64::encode(&body),
    })
}

// Internals

use rand::{thread_rng, RngCore};
use tvm::block::{
    Message,
    MessageId,
    MsgAddressExt,
    MsgAddressInt,
    InternalMessageHeader,
    Grams,
    ExternalInboundMessageHeader,
    CurrencyCollection,
    Serializable,
};
use tvm::types::AccountId;
use std::io::Cursor;
use ed25519_dalek::PublicKey;
use types::{ApiResult, ApiError};

use tvm::block::MessageProcessingStatus;
use ed25519_dalek::Keypair;
use client::Context;
use log::Level::Trace;

fn create_image(image_base64: &String, public_key: &PublicKey) -> ApiResult<ContractImage> {
    let bytes = base64::decode(image_base64)
        .map_err(|err| ApiError::contracts_deploy_invalid_image(err))?;
    let mut reader = Cursor::new(bytes);
    ContractImage::from_state_init_and_key(&mut reader, public_key)
        .map_err(|err| ApiError::contracts_deploy_image_creation_failed(err))
}

fn deploy_contract(params: &ParamsOfDeploy, image: ContractImage, keys: &Keypair) -> ApiResult<TransactionId> {
    let changes_stream = Contract::deploy_json(
        "constructor".to_owned(),
        params.constructorParams.to_string().to_owned(),
        params.abi.to_string().to_owned(),
        image,
        Some(keys))
        .expect("Error deploying contract");

    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(state) = state {
            debug!("deploy: {:?}", state.status);
            if state.status == TransactionProcessingStatus::Preliminary ||
                state.status == TransactionProcessingStatus::Proposed ||
                state.status == TransactionProcessingStatus::Finalized
            {
                tr_id = Some(state.id.clone());
                break;
            }
        }
    }
    tr_id.ok_or(ApiError::contracts_deploy_transaction_missing())
}

pub fn create_external_transfer_funds_message(src: &AccountId, dst: &AccountId, value: u128) -> Message {
    let mut rng = thread_rng();
    let mut random = [0u8;8];
    rng.fill_bytes(&mut random);
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(SliceData::from_raw(random.to_vec(), 64)).unwrap(),
            dst: MsgAddressInt::with_standart(None, 0, src.clone()).unwrap(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams(value.into());

    let int_msg_hdr = InternalMessageHeader::with_addresses(
        MsgAddressInt::with_standart(None, 0, src.clone()).unwrap(),
        MsgAddressInt::with_standart(None, 0, dst.clone()).unwrap(),
        balance);

    *msg.body_mut() = Some(int_msg_hdr.write_to_new_cell().unwrap().into());

    msg
}

pub fn send_message(msg: Message) -> ApiResult<TransactionId> {
    let changes_stream = Contract::send_message(msg)
        .map_err(|err| ApiError::contracts_send_message_failed(err))?;
    let mut tr_id = None;
    for state in changes_stream.wait() {
        match state {
            Ok(state) => {
                debug!("send message: {:?}", state.status);
                if state.status == TransactionProcessingStatus::Preliminary ||
                    state.status == TransactionProcessingStatus::Proposed ||
                    state.status == TransactionProcessingStatus::Finalized
                {
                    tr_id = Some(state.id.clone());
                }
            }
            Err(err) => return Err(ApiError::contracts_send_message_failed(err))
        }
    }

    let tr_id = tr_id.expect("Error: no transaction id");

    let transaction = Transaction::load(tr_id)
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction");

    if transaction.tr().is_aborted() {
        return Err(ApiError::contracts_send_message_failed("Transaction aborted"));
    }

    transaction.out_messages_id().iter().for_each(|msg_id| {
        wait_message_processed_by_id(msg_id.clone());
    });

    Err(ApiError::contracts_send_message_failed("Missing message"))
}

fn wait_message_processed_by_id(message_id: MessageId) -> TransactionId {
    let msg = ton_sdk::Message::load(message_id.clone())
        .expect("Error load message")
        .wait()
        .next();

    if msg.is_some() {
        let s = msg.expect("Error unwrap stream next while loading Message")
            .expect("Error unwrap result while loading Message")
            .expect("Error unwrap returned Message");
        println!("{} : {:?}", s.id().to_hex_string(), s.status());
        if s.status() == MessageProcessingStatus::Preliminary ||
            s.status() == MessageProcessingStatus::Proposed ||
            s.status() == MessageProcessingStatus::Finalized {
            return s.id().clone();
        }
    }

    let mut tr_id = None;
    for state in Contract::subscribe_updates(message_id.clone()).unwrap().wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("{} : {:?}", s.id.to_hex_string(), s.status);
            if s.status == TransactionProcessingStatus::Preliminary ||
                s.status == TransactionProcessingStatus::Proposed ||
                s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }

    tr_id.expect("No transaction ID")
}

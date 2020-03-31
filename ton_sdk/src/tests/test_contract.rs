/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ton_abi::json_abi::decode_function_response;
use super::*;
use crate::{ContractImage, init_json, MessageType};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use sha2::Sha512;
use ton_block::{AccountId, MsgAddressInt};
use ton_types::dictionary::HashmapE;
use crate::tests_common::{call_contract, deploy_contract_and_wait, get_config, get_grams_from_giver,
    init_node_connection, PROFESSOR_ABI, PROFESSOR_IMAGE, WALLET_ABI, WALLET_IMAGE,
    SUBSCRIBE_CONTRACT_IMAGE, SUBSCRIBE_CONTRACT_ABI};
use futures::StreamExt;

const FUNCTION_PARAMS: &str = r#"
{
	"value": "0000000000000000000000000000000000000000000000000000000000000001"
}"#;

async fn test_call_contract(client: &NodeClient, address: MsgAddressInt, key_pair: &Keypair) {

    let func = "createOperationLimit".to_string();
    let abi = WALLET_ABI.to_string();

    // call needed method
    let tr = Contract::call_json(
        client, address, func.clone(), None, FUNCTION_PARAMS.to_owned(), abi.clone(), Some(&key_pair))
            .await
            .expect("Error calling contract method");

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages(client)
        .expect("Error calling load out messages");
    
    futures::pin_mut!(out_msg);

    let out_msg = out_msg
        .filter(|msg| futures::future::ready(
            msg.as_ref()
                .expect("error unwrap out message 1")
                .msg_type() == MessageType::ExternalOutbound))
        .next()
        .await
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3");

    // take body from the message
    let response = out_msg.body().expect("erro unwrap out message body");


    // decode the body by ABI
    let result = decode_function_response(abi, func, response, false)
        .expect("Error decoding result");

    println!("result:/n{}", result);


    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body

}

#[tokio::main]
#[test]
pub async fn test_deploy_and_call_contract() {
   
    let client = init_node_connection();

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut WALLET_IMAGE.as_slice(), &keypair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.msg_address(0);

    // before deploying contract need to transfer some funds to its address
    get_grams_from_giver(&client, account_id.clone()).await;


    // call deploy method
    let func = "constructor".to_string();
    let abi = WALLET_ABI.to_string();

    let tr = Contract::deploy_json(&client, func, None, "{}".to_owned(), abi, contract_image, Some(&keypair), 0)
        .await
        .expect("Error deploying contract");

    if tr.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    test_call_contract(&client, account_id, &keypair).await;
}

#[test]
fn test_contract_image_from_file() {

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut SUBSCRIBE_CONTRACT_IMAGE.as_slice(), &keypair.public).expect("Unable to parse contract code file");

    println!("Account ID {:x}", contract_image.account_id());
}

#[tokio::main]
#[test]
async fn test_load_nonexistent_contract() {
    let client = init_node_connection();

    let acc_id = AccountId::from([67; 32]);
    let c = Contract::load(&client, &MsgAddressInt::with_standart(None, 0, acc_id).unwrap())
        .await
        .expect("Error unwrap result while loading Contract");

    assert!(c.is_none());
}

#[test]
#[ignore]
fn test_update_contract_data() {
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let mut contract_image = ContractImage::from_state_init_and_key(&mut SUBSCRIBE_CONTRACT_IMAGE.as_slice(), &keypair.public)
        .expect("Unable to parse contract code file");

    let new_data = r#"
        { "mywallet": "0:1111111111111111111111111111111111111111111111111111111111111111" }
    "#;

    contract_image.update_data(new_data, &SUBSCRIBE_CONTRACT_ABI).unwrap();
    let init = contract_image.state_init();
    let new_map = HashmapE::with_data(ton_abi::Contract::DATA_MAP_KEYLEN, init.data.unwrap().into());

    let key_slice = new_map.get(
        0u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(key_slice.get_bytestring(0), keypair.public.as_bytes().to_vec());
    let mywallet_slice = new_map.get(
        100u64.write_to_new_cell().unwrap().into(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        mywallet_slice,
        MsgAddressInt::with_standart(None, 0, vec![0x11; 32].into()).unwrap().write_to_new_cell().unwrap().into());
}

#[tokio::main]
#[test]
async fn test_expire() {
    let mut config = get_config();
    config["timeouts"]["message_retries_count"] = serde_json::Value::from(0);
    // connect to node
	let client = init_json(&config.to_string()).unwrap();

	// generate key pair
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let wallet_address = deploy_contract_and_wait(&client, &WALLET_IMAGE, &WALLET_ABI, "{}", &keypair, 0).await;

    let (msg, _) = Contract::construct_call_message_json(
        wallet_address.clone(),
        "setSubscriptionAccount".to_owned(),
        Some(json!({
            "expire": 123
        }).to_string()),
        json!({
            "addr": wallet_address.to_string()
        }).to_string(),
        WALLET_ABI.clone(),
        false,
        Some(&keypair)).unwrap();

    let result = Contract::send_message(&client, Contract::deserialize_message(&msg).unwrap(), Some(Contract::get_now().unwrap() + 1), 0).await;

    match result {
        Err(error) => match error.downcast_ref::<SdkErrorKind>().unwrap() {
            SdkErrorKind::MessageExpired => {},
            _ => panic!("Error `SdkErrorKind::MessageExpired` expected")
        },
        _ => panic!("Error expected")
    }
}

#[tokio::main]
#[test]
async fn test_retries() {
    let mut config = get_config();
    config["timeouts"]["message_expiration_timeout_grow_factor"] = serde_json::Value::from(1.1);
    config["timeouts"]["message_processing_timeout_grow_factor"] = serde_json::Value::from(0.9);
    // connect to node
	let client = init_json(&config.to_string()).unwrap();

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let wallet_address = deploy_contract_and_wait(&client, &WALLET_IMAGE, &WALLET_ABI, "{}", &keypair, 0).await;

    let mut futures = vec![];
    for i in 0..10 {
        let str_address = wallet_address.to_string();
        let str_address = str_address[..str_address.len() - 2].to_owned() + &format!("{:02x}", i);
        let fut = call_contract(
            &client,
            wallet_address.clone(),
            "setSubscriptionAccount",
            json!({
                "addr": str_address
            }).to_string(),
            &WALLET_ABI,
            Some(&keypair));

        futures.push(fut);
    }

    futures::future::join_all(futures).await;
}

#[test]
fn professor_test() {
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(
        &mut PROFESSOR_IMAGE.as_slice(),
        &keypair.public).expect("Unable to parse contract code file");

    let _message = Contract::construct_deploy_message_json(
            "constructor".to_owned(),
            None,
            json!({
                "parents": [1234, 1234],
                "timestamps": [1234, 1234],
                "amount": 1234,
                "details": [123, 123],
                "detailsDelimiter": [1]
            }).to_string(),
            PROFESSOR_ABI.to_owned(),
            contract_image,
            Some(&keypair),
            0).unwrap();
}

#[test]
fn test_contract_from_bytes() {
    let smc1 = Contract::from_json(r#"{
        "id": "-1:6666666666666666666666666666666666666666666666666666666666666666",
        "acc_type": 1,
        "balance": "0xe8d4a51000",
        "balance_other": null,
        "code": "te6ccgEBAQEAQAAAfP8AIN0gggFMl7qXMO1E0NcLH+Ck8mDTHwHtRNDTH9FmuvKh+AAB0wfU0YIYBKgXyABz+wIB+wCkyMsfye1U",
        "data": "te6ccgEBAQEABgAACAAAAAA=",
        "last_paid": 0
      }"#).unwrap();

    let smc2 = Contract::from_bytes(&base64::decode(
        "te6ccuECAwEAAIMAAHoA+gEGAnHP9mZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZiBoCgwAAAAAAAAAAAAAAAAXo1KUQAE0ABAgB8/wAg3SCCAUyXupcw7UTQ1wsf4KTyYNMfAe1E0NMf0Wa68qH4AAHTB9TRghgEqBfIAHP7AgH7AKTIyx/J7VQACAAAAAB1qkcn"
    ).unwrap()).unwrap();

    let smc3 = Contract::from_json(r#"{
        "id": "-1:6666666666666666666666666666666666666666666666666666666666666666",
        "acc_type": 1,
        "balance": "0xe8d4a510",
        "balance_other": null,
        "code": "te6ccgEBAQEAQAAAfP8AIN0gggFMl7qXMO1E0NcLH+Ck8mDTHwHtRNDTH9FmuvKh+AAB0wfU0YIYBKgXyABz+wIB+wCkyMsfye1U",
        "data": "te6ccgEBAQEABgAACAAAAAA=",
        "last_paid": 100
      }"#).unwrap();

    assert_eq!(format!("{:?}", smc1), format!("{:?}", smc2));
    assert_ne!(format!("{:?}", smc2), format!("{:?}", smc3));
}

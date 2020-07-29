/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use super::*;
use crate::{ContractImage, init_json};
use ed25519_dalek::Keypair;
use ton_block::{MsgAddressInt};
use ton_types::{AccountId, HashmapE};
use crate::tests_common::{call_contract_and_wait, deploy_contract_and_wait,
    get_config, init_node_connection, PROFESSOR_ABI, PROFESSOR_IMAGE, WALLET_ABI, WALLET_IMAGE,
    SUBSCRIBE_CONTRACT_IMAGE, SUBSCRIBE_CONTRACT_ABI};

const FUNCTION_PARAMS: &str = r#"
{
	"value": "0000000000000000000000000000000000000000000000000000000000000001"
}"#;

#[tokio::main]
#[test]
pub async fn test_deploy_and_call_contract() {
    let client = init_node_connection().await;

    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

    let address = deploy_contract_and_wait(&client, &WALLET_IMAGE, &WALLET_ABI, "{}", &keypair, 0).await;

    let result = call_contract_and_wait(
        &client, address, "createOperationLimit", FUNCTION_PARAMS.to_owned(), &WALLET_ABI, Some(&keypair),
    ).await;

    println!("Contract response {}", result);
}

#[test]
fn test_contract_image_from_file() {
    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut SUBSCRIBE_CONTRACT_IMAGE.as_slice(), &keypair.public).expect("Unable to parse contract code file");

    println!("Account ID {:x}", contract_image.account_id());
}

#[tokio::main]
#[test]
async fn test_load_nonexistent_contract() {
    let client = init_node_connection().await;

    let acc_id = AccountId::from([67; 32]);
    let c = Contract::load(&client, &MsgAddressInt::with_standart(None, 0, acc_id).unwrap())
        .await
        .expect("Error unwrap result while loading Contract");

    assert!(c.is_none());
}

#[test]
#[ignore]
fn test_update_contract_data() {
    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

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
    let client = init_json(&config.to_string()).await.unwrap();

    // generate key pair
    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

    let wallet_address = deploy_contract_and_wait(&client, &WALLET_IMAGE, &WALLET_ABI, "{}", &keypair, 0).await;

    let mut msg = Contract::construct_call_message_json(
        wallet_address.clone(),
        FunctionCallSet {
            func: "setSubscriptionAccount".to_owned(),
            header: Some(json!({
                "expire": 123
            }).to_string()),
            input: json!({
                "addr": wallet_address.to_string()
            }).to_string(),
            abi: WALLET_ABI.clone(),
        },
        false,
        Some(&keypair),
        None,
        None).unwrap();

    msg.expire = Some(Contract::now() + 1);

    let result = Contract::process_message(&client, &msg, false).await;

    match result {
        Err(error) => {
            println!("{}", error);
            match error.downcast_ref::<SdkError>().unwrap() {
                SdkError::MessageExpired { msg_id: _, expire: _, sending_time: _, block_time: _, block_id: _ } => {}
                _ => panic!("Error `SdkError::MessageExpired` expected")
            };
        }
        _ => panic!("Error expected")
    }
}

#[test]
fn professor_test() {
    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(
        &mut PROFESSOR_IMAGE.as_slice(),
        &keypair.public).expect("Unable to parse contract code file");

    let _message = Contract::construct_deploy_message_json(
        FunctionCallSet {
            func: "constructor".to_owned(),
            header: None,
            input: json!({
                "parents": [1234, 1234],
                "timestamps": [1234, 1234],
                "amount": 1234,
                "details": [123, 123],
                "detailsDelimiter": [1]
            }).to_string(),
            abi: PROFESSOR_ABI.to_owned(),
        },
        contract_image,
        Some(&keypair),
        0,
        None,
        None).unwrap();
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

#[test]
fn test_resolving_code_and_data_from_boc() {
    let contract = Contract::from_json(r#"{
        "id": "-1:6666666666666666666666666666666666666666666666666666666666666666",
        "acc_type": 1,
        "balance": "0xe8d4a51000",
        "balance_other": null,
        "boc": "te6ccuECAwEAAIMAAHoA+gEGAnHP9mZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZiBoCgwAAAAAAAAAAAAAAAAXo1KUQAE0ABAgB8/wAg3SCCAUyXupcw7UTQ1wsf4KTyYNMfAe1E0NMf0Wa68qH4AAHTB9TRghgEqBfIAHP7AgH7AKTIyx/J7VQACAAAAAB1qkcn",
        "last_paid": 0
      }"#).unwrap();
    assert_eq!(cell_to_base64(&contract.get_code().unwrap()).unwrap(), "te6ccgEBAQEAQAAAfP8AIN0gggFMl7qXMO1E0NcLH+Ck8mDTHwHtRNDTH9FmuvKh+AAB0wfU0YIYBKgXyABz+wIB+wCkyMsfye1U");
    assert_eq!(cell_to_base64(&contract.get_data().unwrap()).unwrap(), "te6ccgEBAQEABgAACAAAAAA=");
}

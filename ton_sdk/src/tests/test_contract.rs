/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
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
use contract::ContractImage;
use std::io::{Cursor};
use std::str::FromStr;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha512;
use ton_block::{AccountId, MsgAddressInt, TransactionProcessingStatus};
use ton_types::{BuilderData, IBitstring};
use ton_types::dictionary::HashmapE;
use tests_common::*;

/*
#[test]
#[ignore] // Rethink have to work on 127.0.0.1:32769. Run it and comment "ignore"
fn test_subscribe_message_updates() {

    // create database, table and record
    let r = Client::new();
    let mut conf = Config::default();
    conf.servers = vec!("127.0.0.1:32769".parse().unwrap());
    let conn = r.connect(conf).unwrap();

    let db_create_res = r.db_create(DB_NAME).run::<Value>(conn).unwrap().wait().next();
    println!("\n\n db_create \n {:#?}", db_create_res);

    let table_create_res = r.db(DB_NAME).table_create(MSG_TABLE_NAME).run::<Value>(conn).unwrap().wait().next();
    println!("\n\n table_create \n {:#?}", table_create_res);

    let msg_id = MessageId::default();

    let insert_doc = r.db(DB_NAME)
        .table(MSG_TABLE_NAME)
        .update( // TODO insert with "update" flag
            json!({
                "id": msg_id.to_hex_string(),
                MSG_STATE_FIELD_NAME: MessageProcessingStatus::Queued
                })
        )
        .run::<WriteStatus>(conn).unwrap().wait().next().unwrap();
    println!("\n\n insert \n {:#?}", insert_doc);

    // subscribe changes
    let changes_stream = Contract::subscribe_message_updates(msg_id.clone()).unwrap();

    // another thread - write changes into DB
    let msg_id_ = msg_id.clone();
    let another_thread = std::thread::spawn(move || {

        std::thread::sleep(std::time::Duration::from_secs(1));

        for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {

            let insert_doc = r.db(DB_NAME)
                .table(MSG_TABLE_NAME)
                .replace(json!({
                    "id": msg_id_.to_hex_string(),
                    MSG_STATE_FIELD_NAME: state
                 }))
                .run::<WriteStatus>(conn).unwrap().wait().next().unwrap();
            println!("\n\n insert \n {:#?}", insert_doc);
        }
    });

    // chech all changes were got    
    let mut changes_stream = changes_stream.wait();
    for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
        let ccs = ContractCallState {
            id: msg_id.clone(),
            status: state.clone(),
        };
        
        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}

#[test]
#[ignore] 
fn test_subscribe_message_updates_kafka_connector() {

    /* Connector config

connector.class=com.datamountaineer.streamreactor.connect.rethink.sink.ReThinkSinkConnector
tasks.max=1
topics=messages_statuses
connect.rethink.db=blockchain
connect.rethink.host=rethinkdb
connect.rethink.port=28015
key.converter.schemas.enable=false
name=rethink-sink
value.converter.schemas.enable=false
value.converter=org.apache.kafka.connect.json.JsonConverter
key.converter=org.apache.kafka.connect.json.JsonConverter
connect.rethink.kcql=UPSERT INTO messages_statuses SELECT * FROM messages_statuses AUTOCREATE PK id

    */


    // init SDK
    let config_json = CONFIG_JSON.clone();    
    init_json(config_json.into()).unwrap();


    let msg_id = MessageId::default();

    // subscribe changes
    let changes_stream = Contract::subscribe_message_updates(msg_id.clone()).unwrap();

    // another thread - write changes into DB though Kafka (emulate node activity)
    let msg_id_ = msg_id.clone();
    let another_thread = std::thread::spawn(move || {

        std::thread::sleep(std::time::Duration::from_secs(1));

        for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
            let key = format!("\"{}\"", msg_id_.to_hex_string());
            
            let doc = json!({
                "id": msg_id_.to_hex_string(),
                MSG_STATE_FIELD_NAME: state
            }).to_string();
            
            requests_helper::send_message_to_topic(
                    key.as_bytes(),
                    doc.as_bytes(),
                    "messages_statuses"
                )
                .unwrap();

            println!("NODE {}", doc);
        }
    });

    // chech all changes were got    
    let mut changes_stream = changes_stream.wait();
    for state in [MessageProcessingStatus::Processing, MessageProcessingStatus::Proposed, MessageProcessingStatus::Finalized].iter() {
        let ccs = ContractCallState {
            id: msg_id.clone(),
            status: state.clone(),
        };

        let json = serde_json::to_string(&ccs).unwrap();
        println!("CLIENT {}", json);

        assert_eq!(changes_stream.next().unwrap().unwrap(), ccs);
    }

    another_thread.join().unwrap();
}
*/

const FUNCTION_PARAMS: &str = r#"
{
	"value": "0000000000000000000000000000000000000000000000000000000000000001"
}"#;

fn test_call_contract(address: MsgAddressInt, key_pair: &Keypair) {

    let func = "createOperationLimit".to_string();
    let abi = test_piggy_bank::WALLET_ABI.to_string();

    // call needed method
    let changes_stream = Contract::call_json(
        address, func.clone(), FUNCTION_PARAMS.to_owned(), abi.clone(), Some(&key_pair))
            .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.status == TransactionProcessingStatus::Finalized {
                tr = Some(s);
                break;
            }
        }
    }
    let tr = tr.expect("Error: no transaction");

    // OR 
    // wait message will done and find transaction with the message

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                    .expect("error unwrap out message 2")
                    .msg_type() == MessageType::ExternalOutbound
        })
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3")
            .expect("erro unwrap out message 4");

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

#[test]
fn test_deploy_and_call_contract() {
   
    tests_common::init_node_connection();   
   
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/LimitWallet.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.msg_address(0);

    // before deploying contract need to transfer some funds to its address
    println!("Account ID to take some grams {}", account_id);
    
    tests_common::get_grams_from_giver(account_id.clone());


    // call deploy method
    let func = "constructor".to_string();
    let abi = test_piggy_bank::WALLET_ABI.to_string();

    let changes_stream = Contract::deploy_json(func, "{}".to_owned(), abi, contract_image, Some(&keypair), 0)
        .expect("Error deploying contract");

    // wait transaction id in message-status or 
    // wait message will done and find transaction with the message

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");

    test_call_contract(account_id, &keypair);
}

#[test]
fn test_contract_image_from_file() {
    let mut state_init = std::fs::File::open("src/tests/Subscription.tvc").expect("Unable to open contract code file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public).expect("Unable to parse contract code file");

    println!("Account ID {:x}", contract_image.account_id());
}

#[test]
#[ignore]
fn test_deploy_empty_contract() {
    init_node_connection();

    let mut csprng = OsRng::new().unwrap();

    let mut code_builder = BuilderData::new();
    code_builder.append_u32(csprng.next_u32()).expect("Unable to add u32");

    let mut data = Vec::new();
    BagOfCells::with_root(&code_builder.into()).write_to(&mut data, false).expect("Error serializing BOC");
                                        
    let mut data_cur = Cursor::new(data);
    
    let image = ContractImage::from_code_data_and_library(&mut data_cur, None, None).expect("Error creating ContractImage");
    let acc_id = image.msg_address(0);

    tests_common::get_grams_from_giver(acc_id.clone());

    println!("Account ID {}", acc_id);

    /*Contract::load(&acc_id)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");*/
        	// wait for grams recieving
	queries_helper::wait_for(
        "accounts",
        &json!({
			"id": { "eq": acc_id.to_string() },
			"balance": { "gt": "0" }
		}).to_string(),
		"id balance"
	).unwrap();
    println!("Contract got!!!");



    let changes_stream = Contract::deploy_no_constructor(image, 0)
        .expect("Error deploying contract");

        // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
            if s.status == TransactionProcessingStatus::Finalized {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");
    println!("Transaction got!!!");

}

#[test]
fn test_load_nonexistent_contract() {
    init_node_connection();

    let acc_id = AccountId::from([67; 32]);
    let c = Contract::load(&MsgAddressInt::with_standart(None, 0, acc_id).unwrap())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    assert!(c.is_none());
}

#[test]
#[ignore]
fn test_print_base64_address_from_hex() {
    let hex_address = "0:9f2bc8a81da52c6b8cb1878352120f21e254138fff0b897f44fb6ff2b8cae256";

    let address = MsgAddressInt::from_str(hex_address).unwrap();

    println!("{}", contract::encode_base64(&address, false, false, false).unwrap());
}

#[test]
fn test_update_contract_data() {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/Subscription.tvc")
        .expect("Unable to open Subscription contract file");

    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let mut contract_image = ContractImage::from_state_init_and_key(&mut state_init, &keypair.public)
        .expect("Unable to parse contract code file");

    let new_data = r#"
        { "mywallet": "0:1111111111111111111111111111111111111111111111111111111111111111" }
    "#;

    contract_image.update_data(new_data, test_piggy_bank::SUBSCRIBE_CONTRACT_ABI).unwrap();
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

#[test]
fn professor_test() {
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let contract_image = ContractImage::from_state_init_and_key(
        &mut base64::decode(PROFESSOR_IMAGE).unwrap().as_slice(),
        &keypair.public).expect("Unable to parse contract code file");

    let _message = Contract::construct_deploy_message_json(
            "constructor".to_owned(),
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

const PROFESSOR_ABI: &str = r#"{
	"ABI version": 1,
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"parents","type":"uint160[]"},
				{"name":"timestamps","type":"uint64[]"},
				{"name":"details","type":"uint8[]"},
				{"name":"detailsDelimiter","type":"uint8[]"},
				{"name":"amount","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "getData",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint160"},
				{"name":"value1","type":"uint64[]"},
				{"name":"value2","type":"uint32[]"},
				{"name":"value3","type":"uint8[]"}
			]
		}
	],
	"events": [
	],
	"data": [
	]
}"#;

const PROFESSOR_IMAGE: &str = "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";

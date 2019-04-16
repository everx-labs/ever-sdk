#[allow(unused_imports)]
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate hex;

extern crate ton_block;
extern crate tvm;
#[macro_use]
extern crate abi_lib;
extern crate ed25519_dalek;
extern crate rand;
extern crate sha2;
#[macro_use]
extern crate lazy_static;

use sha2::Sha512;
use ed25519_dalek::*;
use rand::rngs::OsRng;

use std::collections::hash_map::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::io::Cursor;
use std::env;

use abi_lib::abi_call::ABICall;
use abi_lib::abi_response::ABIResponse;

use abi_lib::types::Duint;

use ton_block::{
    Message,
    ExternalInboundMessageHeader,
    MsgAddressInt,
    Serializable,
    Deserializable};
use tvm::stack::{BuilderData, CellData, SliceData};
use tvm::cells_serialization::{BagOfCells, deserialize_cells_tree};
use tvm::bitstring::{Bit, Bitstring};
use tvm::types::{AccountId};

use jsonrpc_client_http::{HttpHandle, HttpTransport};

const SERVER: &str = "http://dev.walletapi.tonlabs.io:20080";
//const SERVER: &str = "http://127.0.0.1:3030";
lazy_static! {
    static ref ACCOUNT_ID: AccountId = AccountId::from_str("02c363b6e134600134a1f843138c7ef537795ec16d516e51353621897346e198").unwrap();
}
const SECRET_KEY: &str ="189fb5d30596e9e8c56214f05c64586e0d8b39e198aa6cb3077991779bf47cc0";
const PUBLIC_KEY: &str ="0e8bf0e2301b5ba65ed4be7f8ef41ffb50b10a18bf663dad9cf6eafd33a52fc3";

fn create_message(message_body: SliceData) -> String {
    let mut hdr = ExternalInboundMessageHeader::default();
    hdr.dst = MsgAddressInt::with_standart(None, -1, ACCOUNT_ID.to_owned()).unwrap();

    let mut msg = Message::with_ext_in_header(hdr);

    msg.body = Some(message_body.cell());

    let mut builder = BuilderData::new();
    msg.write_to(&mut builder).unwrap();

    // serialize tree into Vec<u8>
    let root_cell = Arc::<CellData>::from(&builder);
    let root = SliceData::from(root_cell);

    let mut message_data = Vec::new();
    BagOfCells::with_root(root)
        .write_to(&mut message_data, false)
        .unwrap();

    //println!("Message for contract \n{:X?}", message_data);

    base64::encode(&message_data)
}

fn call_contract(transport_handle: &mut  HttpHandle, abi_call: BuilderData) -> SliceData {
    let str_message = create_message(abi_call.into());

    let mut params = HashMap::<String, String>::new();
    params.insert("message".to_string(), str_message);

    let result: String = jsonrpc_client_core::call_method(transport_handle, String::from("call"), params).call().unwrap(); 

    let answer: Vec<u8> = base64::decode(&result).unwrap();

    //println!("Answer {:X?}", answer);

    let mut cursor = Cursor::new(answer);
    let roots = deserialize_cells_tree(&mut cursor).unwrap();

    assert_eq!(roots.len(), 1);

    let mut answer_message = Message::default();
    answer_message.read_from(&mut SliceData::from(roots[0].clone())).unwrap();

    //println!("Answer body {:X?}", SliceData::from(answer_message.clone().body.unwrap()));
    
    SliceData::from(answer_message.body.unwrap())
}

fn call_get_limit_by_id(transport_handle: &mut  HttpHandle, limit_id: u8) -> ((u64, u8, Bitstring), i8) {
    let message_body = ABICall::<(u8,), ((Duint, u8, Bitstring), i8)>::encode_function_call_into_slice(
        "getLimitById",
        (limit_id,)
    );

    let answer = call_contract(transport_handle, message_body);

    ABIResponse::<((u64, u8, Bitstring), i8)>::decode_response_from_slice(answer).unwrap()
}

fn call_get_limits(transport_handle: &mut  HttpHandle, pair: &Keypair) {
    let message_body = ABICall::<(), (Vec<u8>, i8)>::encode_signed_function_call_into_slice("getLimits", (), pair);
    
    let answer = call_contract(transport_handle, message_body);

    let (limits_id, _) = ABIResponse::<(Vec<u8>, i8)>::decode_response_from_slice(answer).unwrap();

    println!("Limits count {}", limits_id.len());

    for limit in limits_id {
        let ((value, limit_type, meta), _) = call_get_limit_by_id(transport_handle, limit);

        println!("\nLimit info:");
        println!("ID - {}", limit);
        println!("Value - {}", value);
        
        if limit_type == 0 {
             println!("Type - Single operation limit");
        } else {
            println!("Type - Arbitrary limit");
            let mut vec = Vec::new();
            meta.into_bitstring_with_completion_tag(&mut vec);
            let period = vec[0];
            println!("Period - {} days", period);
        }
    }
}

fn call_create_limit(transport_handle: &mut  HttpHandle, parameters: Vec<String>, pair: &Keypair) {
    if parameters.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let value = Duint::parse_bytes(parameters[0].as_bytes(), 10).unwrap();

    let limit_type = u8::from_str_radix(&parameters[1], 10).unwrap();

    let mut meta = Bitstring::new();

    if limit_type == 1 {
        if parameters.len() < 3
        {
            println!("Not enough parameters");
            return;
        }

        let period = u8::from_str_radix(&parameters[2], 10).unwrap();
        meta.append_u8(period);
    }

    let message_body = ABICall::<(u8, Duint, Bitstring), (u8, i8)>::encode_signed_function_call_into_slice("createLimit", (limit_type, value, meta), pair);

    let answer = call_contract(transport_handle, message_body);

    let (limit_id, _) = ABIResponse::<(u8, i8)>::decode_response_from_slice(answer).unwrap();

    println!("New limit ID {}", limit_id);
}

fn call_remove_limit(transport_handle: &mut  HttpHandle, parameters: Vec<String>, pair: &Keypair) {
    if parameters.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    let limit_id = u8::from_str_radix(&parameters[0], 10).unwrap();

    let message_body = ABICall::<(u8,), (i8,)>::encode_signed_function_call_into_slice("removeLimit", (limit_id,), pair);

    let _answer = call_contract(transport_handle, message_body);
}

fn call_get_version(transport_handle: &mut  HttpHandle) {

    let message_body = ABICall::<(), ((u16, u16), i8)>::encode_function_call_into_slice("getVersion", ());

    let answer = call_contract(transport_handle, message_body);

    let ((major, minor), _) = ABIResponse::<((u16, u16), i8)>::decode_response_from_slice(answer).unwrap();

    println!("Version {}.{}", major, minor);
}

fn call_change_limit(transport_handle: &mut  HttpHandle, parameters: Vec<String>, pair: &Keypair) {
    if parameters.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    
    let limit_id = u8::from_str_radix(&parameters[0], 10).unwrap();

    let value = Duint::parse_bytes(parameters[1].as_bytes(), 10).unwrap();

    let mut meta = Bitstring::new();

    if parameters.len() > 2 {
        let period = u8::from_str_radix(&parameters[2], 10).unwrap();
        meta.append_u8(period);
    }

    let message_body = ABICall::<(u8, Duint, Bitstring), (i8,)>::encode_signed_function_call_into_slice("changeLimitById", (limit_id, value, meta), pair);

    let _answer = call_contract(transport_handle, message_body);
}

fixed_abi_array!(Bit, 256, Bits256);

fn call_send_transaction(transport_handle: &mut  HttpHandle, parameters: Vec<String>, pair: &Keypair) {
    if parameters.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let mut recipient = hex::decode(parameters[0].clone()).unwrap();
    recipient.resize(32, 0);

    let mut bits_recipient = [Bit::Zero; 256];

    bits_recipient.copy_from_slice(&Bitstring::create(recipient, 256).bits(0..256).data);

    let value = Duint::parse_bytes(parameters[1].as_bytes(), 10).unwrap();

    let message_body = ABICall::<(Bits256, Duint), (i8,)>::encode_signed_function_call_into_slice("sendTransaction", (bits_recipient.into(), value), pair);

    let answer = call_contract(transport_handle, message_body);

    let (transaction_id, _) = ABIResponse::<(u64, i8)>::decode_response_from_slice(answer).unwrap();

    println!("Transaction ID {}", transaction_id);

}

fn call_get_balance(transport_handle: &mut  HttpHandle, pair: &Keypair) {

    let message_body = ABICall::<(), (u64,)>::encode_signed_function_call_into_slice("getBalance", (), pair);

    let answer = call_contract(transport_handle, message_body);

    let (balance,) = ABIResponse::<(u64,)>::decode_response_from_slice(answer).unwrap();

    println!("Current balance {} Gram", balance);
}

#[test]
fn test_function_with_right_key() {
    let right_pair = Keypair {
        secret: SecretKey::from_bytes(&hex::decode(SECRET_KEY).unwrap()[..]).unwrap(), 
        public: PublicKey::from_bytes(&hex::decode(PUBLIC_KEY).unwrap()[..]).unwrap(),
    };

    run_function("getVersion", vec![], &right_pair);
    run_function("getLimits",  vec![], &right_pair);
    run_function("getBalance", vec![], &right_pair);

    let wrong_pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());
    run_function("getVersion", vec![], &wrong_pair);
}

fn run_function(func_name: &str, parameters: Vec<String>, pair: &Keypair) {
    let transport = HttpTransport::new().standalone().unwrap();
    let mut transport_handle = transport.handle(SERVER).unwrap();

    match func_name {
        "getLimits" => call_get_limits(&mut transport_handle, pair),
        "createLimit" => call_create_limit(&mut transport_handle, parameters, pair),
        "removeLimit" => call_remove_limit(&mut transport_handle, parameters, pair),
        "changeLimitById" => call_change_limit(&mut transport_handle, parameters, pair),
        "getVersion" => call_get_version(&mut transport_handle),
        "sendTransaction" => call_send_transaction(&mut transport_handle, parameters, pair),
        "getBalance" => call_get_balance(&mut transport_handle, pair),
        &_ => println!("Function not supported"),
    }
}

fn main() {
    // println!("Server {}", server);

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let mut offset = 2;
    // println!("{} {}", args.len(), arg[2]);
    let pair = if args.len() > 2 && args[2] == "wrong" {
        offset += 1;
        Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap())
    } else {
        Keypair {
            secret: SecretKey::from_bytes(&hex::decode(SECRET_KEY).unwrap()[..]).unwrap(), 
            public: PublicKey::from_bytes(&hex::decode(PUBLIC_KEY).unwrap()[..]).unwrap(),
        }
    };

    run_function(args[1].clone().as_str(), (&args[offset..]).to_vec(), &pair)
}
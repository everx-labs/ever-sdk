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

use super::*;
use ed25519_dalek::Keypair;
use tests_common::*;

#[tokio::main]
#[test]
async fn full_test_piggy_bank() {

    // connect to node
	let client = init_node_connection();

	// generate key pair
    let mut csprng = rand::thread_rng();
    let keypair = Keypair::generate(&mut csprng);

    let now = std::time::Instant::now();

	// deploy wallet
    println!("Wallet contract deploying...\n");
    let wallet_address = deploy_contract_and_wait(&client, &WALLET_IMAGE, &WALLET_ABI, "{}", &keypair, 0).await;
	println!("Wallet contract deployed. Account address {}\n", wallet_address);

	// deploy piggy bank
    println!("Piggy bank contract deploying...\n");
	let piggy_bank_address = deploy_contract_and_wait(&client, &PIGGY_BANK_CONTRACT_IMAGE, &PIGGY_BANK_CONTRACT_ABI, PIGGY_BANK_CONSTRUCTOR_PARAMS, &keypair, 0).await;
	println!("Piggy bank contract deployed. Account address {}\n", piggy_bank_address);

    // get goal from piggy
    println!("Get goal from piggy...\n");
    //let (get_goal_answer, _) = call_contract_and_wait(piggy_bank_address.clone(), "getGoal", "{}".to_string(), PIGGY_BANK_CONTRACT_ABI, None);
    let get_goal_answer = contract_call_local(&client, piggy_bank_address.clone(), "getGoal", "{}", &PIGGY_BANK_CONTRACT_ABI, None).await;
    println!("piggy answer {}", get_goal_answer);

	// deploy subscription

    println!("Subscription contract deploying...\n");
	let subscription_constructor_params = format!("{{ \"wallet\" : \"{}\" }}", wallet_address);
	let subscripition_address = deploy_contract_and_wait(&client, &SUBSCRIBE_CONTRACT_IMAGE, &SUBSCRIBE_CONTRACT_ABI, &subscription_constructor_params, &keypair, 0).await;
	println!("Subscription contract deployed. Account address {}\n", subscripition_address);


    // call setSubscriptionAccount in wallet
    println!("Adding subscription address to the wallet...\n");
	let set_subscription_params = format!("{{ \"addr\" : \"{}\" }}", subscripition_address);

	let _set_subscription_answer = call_contract(&client, wallet_address, "setSubscriptionAccount", set_subscription_params, &WALLET_ABI, Some(&keypair)).await;

	println!("Subscription address added to the wallet.\n");

	// call subscribe in subscription
    println!("Adding subscription 1...\n");
    let subscr_id_str = hex::encode(&[0x11; 32]);
	let pubkey_str = hex::encode(keypair.public.as_bytes());
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"{}\", \"value\" : 123, \"period\" : 456 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address,
    );

	call_contract(&client, subscripition_address.clone(), "subscribe", subscribe_params, &SUBSCRIBE_CONTRACT_ABI, Some(&keypair)).await;
	println!("Subscription 1 added.\n");

    	// call subscribe in subscription
    println!("Adding subscription 2...\n");
    let subscr_id_str = hex::encode(&[0x22; 32]);
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"{}\", \"value\" : 5000000000, \"period\" : 86400 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address,
    );
	call_contract(&client, subscripition_address.clone(), "subscribe", subscribe_params, &SUBSCRIBE_CONTRACT_ABI, Some(&keypair)).await;
	println!("Subscription 2 added.\n");

    println!("Call getSubscription with id {}\n", &subscr_id_str);
    let get_params = format!("{{ \"subscriptionId\" : \"0x{}\" }}", &subscr_id_str);
    let answer = contract_call_local(&client, subscripition_address.clone(), "getSubscription", &get_params, &SUBSCRIBE_CONTRACT_ABI, Some(&keypair)).await;
    println!("getSubscription result:\n{}", answer);

    let t = now.elapsed();
	println!("Time: sec={}.{:06} ", t.as_secs(), t.subsec_micros());
}

// 123 Grams and "Some goal" as goal
const PIGGY_BANK_CONSTRUCTOR_PARAMS: &str = r#"
{
	"amount": 123,
	"goal": "536f6d6520676f616c"
}"#;

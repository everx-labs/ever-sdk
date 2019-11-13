use crate::test_piggy_bank::PIGGY_BANK_CONTRACT_ABI;

const CONTRACT: &str = r#"{"id": "0:e6392da8a96f648098f818501f0211f27c89675e5f196445d211947b48e7c85b", "code": "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","data": "te6ccgEBDAEAkgABIYAAALctU9fiAAAAAAANu6BgAQIDzmAIAgIDpMAEAwBBpQKvROEle3Z5y5bpAFe2QNwhd5qLpT2ASjizNtXw5hb4AgEgBgUAEQAAAAAAAAAe4AEBIAcAElNvbWUgZ29hbAIBYgsJAQHeCgAD0CAAQdqBV6Jwkr27POXLdIAr2yBuELvNRdKewCUcWZtq+HMLfA=="}"#;

#[test]
fn test_local_piggy_call() {
    let contract: crate::Contract = serde_json::from_str(CONTRACT).expect("Error parsing state init");
    let messages = contract.local_call_json(
        "getGoal".to_owned(),
        "{}".to_owned(),
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        None).expect("Error calling contract");
    println!("messages count {}", messages.len());
    assert!(messages.len() == 1);

    let answer = crate::Contract::decode_function_response_json(
        PIGGY_BANK_CONTRACT_ABI.to_owned(),
        "getGoal".to_owned(),
        messages[0].body().expect("Message has no body"),
        false)
            .expect("Error decoding result");

    println!("answer {}", answer);

    assert_eq!(answer, r#"{"value0":"536f6d6520676f616c"}"#);
}
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

/*use crate::*;

#[test]
fn test_init() {

    let res = requests_helper::send_message(&[0], &[0]);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkError::NotInitialized => (),
        other => panic!(format!("{:?}", other))
    };

    let res = init_json("{}".into());
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkError::InvalidArg(_) => (),
        other => panic!(format!("{:?}", other))
    };

    let config_json = r#"
        {
            "graphql_config": {
                "server": "services.tonlabs.io:4000/graphql"
            },
            "kafka_config": {
                "servers": ["services.tonlabs.io:8082"],
                "topic": "requests",
                "ack_timeout": 123
            }
        }"#;
    
    let res = init_json(config_json.into());
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkError::Kafka(_) => (),
        other => panic!(format!("{:?}", other))
    };

    let config : NodeClientConfig = serde_json::from_str(&config_json).unwrap();

    let res = requests_helper::init(config.kafka_config);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkError::Kafka(_) => (),
        other => panic!(format!("{:?}", other))
    };

    queries_helper::init(config.graphql_config)
}*/
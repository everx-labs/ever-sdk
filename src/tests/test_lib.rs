/*use crate::*;

const WORKCHAIN: i32 = 0;

#[test]
fn test_init() {

    let res = kafka_helper::send_message(&[0], &[0]);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::NotInitialized => (),
        other => panic!(format!("{:?}", other))
    };

    let res = init_json(Some(WORKCHAIN), "{}".into());
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::InvalidArg(_) => (),
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
    
    let res = init_json(Some(WORKCHAIN), config_json.into());
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::Kafka(_) => (),
        other => panic!(format!("{:?}", other))
    };

    let config : NodeClientConfig = serde_json::from_str(&config_json).unwrap();

    let res = kafka_helper::init(config.kafka_config);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::Kafka(_) => (),
        other => panic!(format!("{:?}", other))
    };

    db_helper::init(config.graphql_config)
}*/
use crate::*;

#[test]
fn test_init() {

    let res = kafka_helper::send_message(&[0], &[0]);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::NotInitialized => (),
        other => panic!(format!("{:?}", other))
    };

    let res = db_helper::load_record("table", "record_id");
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::NotInitialized => (),
        other => panic!(format!("{:?}", other))
    };

    let res = init_json("{}".into());
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::InvalidArg(_) => (),
        other => panic!(format!("{:?}", other))
    };

    let config_json = r#"
        {
            "db_config": {
                "servers": ["127.0.0.1:123"],
                "db_name": "some name"
            },
            "kafka_config": {
                "servers": ["127.0.0.1:123"],
                "topic": "topic",
                "ack_timeout": 123
            }
        }"#;
    
    let res = init_json(config_json.into());
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

    let res = db_helper::init(config.db_config);
    assert!(res.is_err());
    match res.err().unwrap().kind() {
        SdkErrorKind::DB(_) => (),
        other => panic!(format!("{:?}", other))
    };
}
use crate::*;
extern crate futures;
extern crate tokio;


static HOST: &str = "http://localhost:4000/graphql";
static WS_HOST: &str = "ws://localhost:4000/graphql";

static QUERY: &str = "{message}";

static MUTATION: &str = "";
static SUBSCRIPTION: &str = "";

/*
#[test]
fn test_connection() {
    
}

#[test]
fn test_connection_fail() {
    
}*/


//static QUERY_RESP: &str = "query { User(id: \\\"cjsccgkzh1d9l0119cu3n18yl\\\") { id name } }";

#[test]
fn test_query() {
    let stream = get_client().query(QUERY.to_string());
    stream.and_then();
    let res = tokio::run(stream);
    panic!("{:?}", res);
}

#[test]
fn test_query_fail() {
    
}

#[test]
fn test_mitation() {
    
}

#[test]
fn test_mitation_fail() {
    
}

#[test]
fn test_subscription_subscribe() {
    
}

#[test]
fn test_subscription_subscribe_fail() {
    
}

fn get_client() -> GqlClient {
    GqlClient::new(HOST.to_string(), WS_HOST.to_string())
}
use crate::client::api::ApiBuilder;

#[test]
fn api_doc() {
    let api = ApiBuilder::new().build();
    println!("{}", serde_json::to_string_pretty(&api).unwrap());
}

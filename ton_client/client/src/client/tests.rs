use crate::client::get_api;

#[test]
fn api_info() {
    let api = get_api();
    println!("{}", serde_json::to_string_pretty(&api).unwrap());
}

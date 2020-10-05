
#[test]
fn api_info() {
    let api = crate::client::get_api();
    println!("{}", serde_json::to_string_pretty(&api).unwrap());
}

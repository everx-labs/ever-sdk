#[test]
fn api_info() {
    let api = crate::client::get_api();
    if let Some(mut path) = dirs::home_dir() {
        path.push("api.json");
        let json = serde_json::to_string(&api).unwrap();
        std::fs::write(path, json).unwrap();
    }
    println!("{}", serde_json::to_string_pretty(&api).unwrap());
}

use crate::api::ClientModule;
use api_info::ApiModule;
use crate::tests::TestClient;
use crate::api::api_reference::ResultOfGetApiReference;

#[test]
fn api_reference() {
    let client = TestClient::new();
    let api: ResultOfGetApiReference = client.request_no_params(&format!(
        "{}.{}",
        ClientModule::api().name,
        crate::api::api_reference::get_api_reference_api().name
    ));
    if let Some(mut path) = dirs::home_dir() {
        path.push("api.json");
        let json = serde_json::to_string(&api.api).unwrap();
        std::fs::write(path, json).unwrap();
    }
    println!("{}", serde_json::to_string_pretty(&api).unwrap());
}

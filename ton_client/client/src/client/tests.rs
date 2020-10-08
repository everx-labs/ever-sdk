use api_info::ApiModule;
use crate::tests::TestClient;
use crate::client::ResultOfGetApiReference;
use crate::c_interface::modules::ClientModule;

#[test]
fn api_reference() {
    let client = TestClient::new();
    let api: ResultOfGetApiReference = client.request_no_params(&format!(
        "{}.{}",
        ClientModule::api().name,
        crate::client::get_api_reference_api().name
    ));
    assert_ne!(api.api.modules.len(), 0);
}

use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocSchema {

}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocInfo {
    title: String,
    description: Option<String>,
    terms_of_service: Option<String>,
    contact: Option<String>,
    license: Option<String>,
    version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocExternal {
    url: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocTag {
    name: String,
    summary: Option<String>,
    description: Option<String>,
    external_docs: Option<DocExternal>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocContentDescriptor {
    name: String,
    summary: Option<String>,
    description: Option<String>,
    required: Option<bool>,
    schema: DocSchema,
    deprecated: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocExample {
    name: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    value: Option<String>,
    external_value: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocExamplePairing {
    name: Option<String>,
    description: Option<String>,
    summary: Option<String>,
    params: Option<Vec<DocExample>>,
    result: Option<DocExample>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocLink {
    name: String,
    description: Option<String>,
    summary: Option<String>,
    method: Option<String>,
    params: Option<HashMap<String, String>>,
    server: Option<DocServer>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocError {
    code: String,
    message: String,
    data: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocServer {
    name: String,
    url: String,
    summary: Option<String>,
    description: Option<String>,
    variables: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocMethod {
    name: String,
    tags: Option<Vec<DocTag>>,
    summary: Option<String>,
    description: Option<String>,
    external_docs: Option<DocExternal>,
    params: Vec <DocContentDescriptor>,
    result: DocContentDescriptor,
    deprecated: Option<bool>,
    servers: Option<Vec<DocServer>>,
    errors: Option<Vec<DocError>>,
    links: Option<Vec<DocLink>>,
    param_structure: Option<String>,
    examples: Option<Vec<DocExamplePairing>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OpenDoc {
    openrpc: String,
    info: DocInfo,
    methods: Vec<DocMethod>,
}

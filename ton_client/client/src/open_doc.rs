use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocSchema {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocContact {
    name: Option<String>,
    url: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocLicense {
    name: String,
    url: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocInfo {
    title: String,
    description: Option<String>,
    terms_of_service: Option<String>,
    contact: Option<DocContact>,
    license: Option<DocLicense>,
    version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocExternalDocumentation {
    url: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocTag {
    name: String,
    summary: Option<String>,
    description: Option<String>,
    external_docs: Option<DocExternalDocumentation>,
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
    external_docs: Option<DocExternalDocumentation>,
    params: Vec<DocContentDescriptor>,
    result: DocContentDescriptor,
    deprecated: Option<bool>,
    servers: Option<Vec<DocServer>>,
    errors: Option<Vec<DocError>>,
    links: Option<Vec<DocLink>>,
    param_structure: Option<String>,
    examples: Option<Vec<DocExamplePairing>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocComponents {
    content_descriptors: Option<HashMap<String, DocContentDescriptor>>,
    schemas: Option<HashMap<String, DocSchema>>,
    examples: Option<HashMap<String, DocExample>>,
    links: Option<HashMap<String, DocLink>>,
    errors: Option<HashMap<String, DocError>>,
    example_pairing_objects: Option<HashMap<String, DocExamplePairing>>,
    tags: Option<HashMap<String, DocTag>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenDoc {
    openrpc: String,
    info: DocInfo,
    methods: Vec<DocMethod>,
    servers: Option<Vec<DocServer>>,
    external_docs: Option<DocExternalDocumentation>,
    components: Option<DocComponents>,
}

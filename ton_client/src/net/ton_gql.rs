/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use serde_json::Value;

use crate::error::ClientError;
use crate::net::gql::GraphQLMessageFromClient;
use crate::net::ParamsOfWaitForCollection;

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub enum SortDirection {
    ASC,
    DESC,
}

#[derive(Serialize, Deserialize, Clone, ApiType)]
pub struct OrderBy {
    pub path: String,
    pub direction: SortDirection,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub enum AggregationFn {
    /// Returns count of filtered record
    COUNT,
    /// Returns the minimal value for a field in filtered records
    MIN,
    /// Returns the maximal value for a field in filtered records
    MAX,
    /// Returns a sum of values for a field in filtered records
    SUM,
    /// Returns an average value for a field in filtered records
    AVERAGE,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct FieldAggregation {
    /// Dot separated path to the field
    pub field: String,
    /// Aggregation function that must be applied to field values
    #[serde(rename = "fn")]
    pub aggregation_fn: AggregationFn,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostRequest {
    pub id: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfAggregateCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter.
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub fields: Option<Vec<FieldAggregation>>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfQueryCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub result: String,
    /// Sorting order
    pub order: Option<Vec<OrderBy>>,
    /// Number of documents to return
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub enum ParamsOfQueryOperation {
    QueryCollection(ParamsOfQueryCollection),
    WaitForCollection(ParamsOfWaitForCollection),
    AggregateCollection(ParamsOfAggregateCollection),
}

impl ParamsOfQueryOperation {
    fn collection(&self) -> &str {
        match self {
            ParamsOfQueryOperation::AggregateCollection(p) => &p.collection,
            ParamsOfQueryOperation::QueryCollection(p) => &p.collection,
            ParamsOfQueryOperation::WaitForCollection(p) => &p.collection,
        }
    }

    fn doc_type(&self) -> String {
        let mut type_words: Vec<String> = self
            .collection()
            .split_terminator("_")
            .map(|word| {
                let mut word = word.to_owned();
                word[..1].make_ascii_uppercase();
                word
            })
            .collect();
        type_words[0] = type_words[0].trim_end_matches("s").to_owned();
        type_words.join("")
    }

    pub(crate) fn query_name(&self) -> String {
        match self {
            ParamsOfQueryOperation::AggregateCollection(_) => {
                let doc_type = self.doc_type();
                format!(
                    "aggregate{}{}",
                    doc_type,
                    if doc_type.ends_with("s") { "" } else { "s" }
                )
            }
            ParamsOfQueryOperation::QueryCollection(p) => p.collection.clone(),
            ParamsOfQueryOperation::WaitForCollection(p) => p.collection.clone(),
        }
    }

    fn query_result(&self) -> &str {
        match self {
            ParamsOfQueryOperation::AggregateCollection(_) => "",
            ParamsOfQueryOperation::QueryCollection(p) => &p.result,
            ParamsOfQueryOperation::WaitForCollection(p) => &p.result,
        }
    }
}

struct QueryOperationBuilder {
    is_batch: bool,
    default_wait_for_timeout: u32,
    param_count: u32,
    op_count: u32,
    op_param_count: u32,
    header: String,
    body: String,
    variables: Option<Value>,
}

impl QueryOperationBuilder {
    fn new(is_batch: bool, default_wait_for_timeout: u32) -> Self {
        Self {
            is_batch,
            default_wait_for_timeout,
            param_count: 0,
            op_count: 0,
            op_param_count: 0,
            header: String::new(),
            body: String::new(),
            variables: None,
        }
    }

    fn add_operations(&mut self, params: &[ParamsOfQueryOperation]) {
        for op in params {
            self.add_op(op);
        }
    }

    fn build(self) -> GraphQLOperation {
        GraphQLOperation {
            query: format!(
                "{}{} {{{}\n}}",
                self.header,
                if self.param_count > 0 { ")" } else { "" },
                self.body
            ),
            variables: self.variables,
        }
    }

    fn add_op(&mut self, op: &ParamsOfQueryOperation) {
        let query_name = op.query_name();
        if self.op_count == 0 {
            self.header = format!(
                "query {}",
                if self.is_batch {
                    "batch".to_string()
                } else {
                    query_name
                }
            );
        }
        self.add_body("\n    ");
        self.op_count += 1;
        self.op_param_count = 0;
        if self.is_batch {
            self.add_body(&format!("q{}: ", self.op_count));
        }
        self.add_body(op.query_name().as_str());
        let filter_type = op.doc_type() + "Filter";
        match op {
            ParamsOfQueryOperation::AggregateCollection(ref p) => {
                self.add_agg_op_params(&filter_type, &p.filter, &p.fields);
            }
            ParamsOfQueryOperation::QueryCollection(ref p) => {
                self.add_query_op_params(&filter_type, &p.filter, &p.order, p.limit, None);
            }
            ParamsOfQueryOperation::WaitForCollection(ref p) => {
                self.add_query_op_params(
                    &filter_type,
                    &p.filter,
                    &None,
                    Some(1),
                    Some(p.timeout.unwrap_or(self.default_wait_for_timeout)),
                );
            }
        }
        if self.op_param_count > 0 {
            self.add_body(")");
        }
        let result = op.query_result();
        if !result.is_empty() {
            self.add_body(" { ");
            self.add_body(result);
            self.add_body(" }");
        }
    }

    fn add_agg_op_params(
        &mut self,
        filter_type: &str,
        filter: &Option<Value>,
        fields: &Option<Vec<FieldAggregation>>,
    ) {
        if let Some(ref filter) = filter {
            self.add_op_param("filter", &filter_type, filter);
        }
        if let Some(ref fields) = fields {
            if !fields.is_empty() {
                self.add_op_param(
                    "fields",
                    "[FieldAggregation]",
                    &serde_json::to_value(fields).unwrap(),
                );
            }
        }
    }

    fn add_query_op_params(
        &mut self,
        filter_type: &str,
        filter: &Option<Value>,
        order_by: &Option<Vec<OrderBy>>,
        limit: Option<u32>,
        timeout: Option<u32>,
    ) {
        if let Some(ref filter) = filter {
            self.add_op_param("filter", filter_type, filter);
        }
        if let Some(order_by) = order_by {
            self.add_op_param(
                "orderBy",
                "[QueryOrderBy]",
                &serde_json::to_value(order_by).unwrap(),
            );
        }
        if let Some(limit) = limit {
            self.add_op_param("limit", "Int", &serde_json::to_value(limit).unwrap());
        }
        if let Some(timeout) = timeout {
            self.add_op_param("timeout", "Float", &Value::from(timeout));
        }
    }

    fn add_op_param(&mut self, name: &str, type_decl: &str, value: &Value) {
        self.add_header(if self.param_count == 0 { "(" } else { "," });
        self.param_count += 1;
        let param_name = format!("p{}", self.param_count);
        self.add_header(&format!("${}: {}", param_name, type_decl));
        self.add_body(if self.op_param_count == 0 { "(" } else { "," });
        self.op_param_count += 1;
        self.add_body(&format!("{}: ${}", name, param_name));
        if let Some(ref mut variables) = self.variables {
            variables[param_name] = value.clone();
        } else {
            self.variables = Some(json!({
                param_name: value.clone()
            }));
        }
    }

    fn add_body(&mut self, s: &str) {
        self.body.push_str(s);
    }

    fn add_header(&mut self, s: &str) {
        self.header.push_str(s);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GraphQLOperation {
    pub query: String,
    pub variables: Option<Value>,
}

impl GraphQLOperation {
    pub fn build(
        params: &[ParamsOfQueryOperation],
        default_wait_for_timeout: u32,
    ) -> GraphQLOperation {
        let mut builder = QueryOperationBuilder::new(params.len() > 1, default_wait_for_timeout);
        builder.add_operations(params);
        builder.build()
    }

    pub fn get_start_message(&self, id: String) -> GraphQLMessageFromClient {
        GraphQLMessageFromClient::Start {
            id,
            query: self.query.clone(),
            variables: self.variables.clone(),
            operation_name: None,
        }
    }

    pub fn with_subscription(table: &str, filter: &Value, fields: &str) -> Self {
        let mut scheme_type = (&table[0..table.len() - 1]).to_owned() + "Filter";
        scheme_type[..1].make_ascii_uppercase();

        let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
            type=scheme_type,
            table=table,
            fields=fields);
        let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");
        let variables = Some(json!({
            "filter" : filter,
        }));
        Self { query, variables }
    }

    pub fn with_post_requests(requests: &[PostRequest]) -> Self {
        let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}"
            .to_owned();
        let variables = Some(json!({ "requests": serde_json::json!(requests) }));
        Self { query, variables }
    }
}

#[derive(Debug)]
pub enum GraphQLOperationEvent {
    Id(u32),
    Data(Value),
    Error(ClientError),
    Complete,
}

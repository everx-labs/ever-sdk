/*
 * Copyright 2018-2021 TON DEV SOLUTIONS LTD.
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

use crate::error::{ClientError, ClientResult};
use crate::net::gql::GraphQLMessageFromClient;
use crate::net::ParamsOfWaitForCollection;
use serde::{de::Error, Deserialize, Deserializer};

const COUNTERPARTIES_COLLECTION: &str = "counterparties";
const FETCH_ADDITIONAL_TIMEOUT: u32 = 5000;

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

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfAggregateCollection {
    /// Collection name (accounts, blocks, transactions, messages, block_signatures)
    pub collection: String,
    /// Collection filter
    pub filter: Option<serde_json::Value>,
    /// Projection (result) string
    pub fields: Option<Vec<FieldAggregation>>,
}

#[derive(Serialize, ApiType, Default, Clone)]
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

#[derive(Deserialize)]
struct ParamsOfQueryCollectionFix {
    pub collection: String,
    pub filter: Option<serde_json::Value>,
    pub result: String,
    pub order: Option<Vec<OrderBy>>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<Vec<OrderBy>>,
    pub limit: Option<u32>,
}

impl<'de> Deserialize<'de> for ParamsOfQueryCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let verified = ParamsOfQueryCollectionFix::deserialize(deserializer);
        match verified {
            Ok(verified) => {
                if verified.order_by.is_none() {
                    Ok(Self {
                        collection: verified.collection,
                        filter: verified.filter,
                        result: verified.result,
                        order: verified.order,
                        limit: verified.limit,
                    })
                } else {
                    Err(D::Error::custom(
                        "Invalid parameter name \"orderBy\"`. Valid name is \"order\".",
                    ))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfQueryCounterparties {
    /// Account address
    pub account: String,
    /// Projection (result) string
    pub result: String,
    /// Number of counterparties to return
    pub first: Option<u32>,
    /// `cursor` field of the last received result
    pub after: Option<String>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
#[serde(tag = "type")]
pub enum ParamsOfQueryOperation {
    QueryCollection(ParamsOfQueryCollection),
    WaitForCollection(ParamsOfWaitForCollection),
    AggregateCollection(ParamsOfAggregateCollection),
    QueryCounterparties(ParamsOfQueryCounterparties),
}

impl ParamsOfQueryOperation {
    fn collection(&self) -> &str {
        match self {
            ParamsOfQueryOperation::AggregateCollection(p) => &p.collection,
            ParamsOfQueryOperation::QueryCollection(p) => &p.collection,
            ParamsOfQueryOperation::WaitForCollection(p) => &p.collection,
            ParamsOfQueryOperation::QueryCounterparties(_) => COUNTERPARTIES_COLLECTION,
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
            ParamsOfQueryOperation::QueryCounterparties(_) => COUNTERPARTIES_COLLECTION.to_owned(),
        }
    }

    fn query_result(&self) -> &str {
        match self {
            ParamsOfQueryOperation::AggregateCollection(_) => "",
            ParamsOfQueryOperation::QueryCollection(p) => &p.result,
            ParamsOfQueryOperation::WaitForCollection(p) => &p.result,
            ParamsOfQueryOperation::QueryCounterparties(p) => &p.result,
        }
    }
}

pub(crate) struct QueryOperationBuilder {
    is_batch: bool,
    default_wait_for_timeout: u32,
    param_count: u32,
    op_count: u32,
    op_param_count: u32,
    header: String,
    body: String,
    variables: Option<Value>,
    timeout: Option<u32>,
}

impl QueryOperationBuilder {
    pub fn new(is_batch: bool, default_wait_for_timeout: u32) -> Self {
        Self {
            is_batch,
            default_wait_for_timeout,
            param_count: 0,
            op_count: 0,
            op_param_count: 0,
            header: String::new(),
            body: String::new(),
            variables: None,
            timeout: None,
        }
    }

    pub fn add_operations(&mut self, params: &[ParamsOfQueryOperation]) {
        for op in params {
            self.add_op(op);
        }
    }

    pub fn build(self) -> GraphQLQuery {
        GraphQLQuery {
            query: format!(
                "{}{} {{{}\n}}",
                self.header,
                if self.param_count > 0 { ")" } else { "" },
                self.body
            ),
            variables: self.variables,
            timeout: self.timeout.map(|x| x + FETCH_ADDITIONAL_TIMEOUT),
            is_batch: self.is_batch,
        }
    }

    fn start_op(&mut self, name: &str) {
        if self.op_count == 0 {
            self.header = format!("query {}", if self.is_batch { "batch" } else { name });
        }
        self.add_body("\n    ");
        self.op_count += 1;
        self.op_param_count = 0;
        if self.is_batch {
            self.add_body(&format!("q{}: ", self.op_count));
        }
        self.add_body(name);
    }

    fn end_op(&mut self, result: &str) {
        if self.op_param_count > 0 {
            self.add_body(")");
        }
        if !result.is_empty() {
            self.add_body(" { ");
            self.add_body(result);
            self.add_body(" }");
        }
    }

    fn add_op(&mut self, op: &ParamsOfQueryOperation) {
        self.start_op(&op.query_name());
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
                self.timeout = match (self.timeout, p.timeout) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (None, Some(b)) => Some(b),
                    _ => self.timeout,
                };
            }
            ParamsOfQueryOperation::QueryCounterparties(ref p) => {
                self.add_query_counterparties_op_params(&p.account, &p.first, &p.after);
            }
        }
        self.end_op(&op.query_result());
    }

    fn add_info(&mut self) {
        self.start_op("info");
        self.end_op("version time");
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

    fn add_query_counterparties_op_params(
        &mut self,
        account: &str,
        first: &Option<u32>,
        after: &Option<String>,
    ) {
        self.add_op_param("account", "String!", &Value::from(account));
        if let Some(first) = first {
            self.add_op_param("first", "Int", &Value::from(*first));
        }
        if let Some(after) = after.as_ref() {
            self.add_op_param("after", "String", &Value::from(after.as_str()));
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
pub(crate) struct GraphQLQuery {
    pub query: String,
    pub variables: Option<Value>,
    pub timeout: Option<u32>,
    pub is_batch: bool,
}

impl GraphQLQuery {
    pub fn build(
        params: &[ParamsOfQueryOperation],
        include_info: bool,
        default_wait_for_timeout: u32,
    ) -> GraphQLQuery {
        let param_count = params.len() + if include_info { 1 } else { 0 };
        let mut builder = QueryOperationBuilder::new(param_count > 1, default_wait_for_timeout);
        builder.add_operations(params);
        if include_info {
            builder.add_info();
        }
        builder.build()
    }

    pub fn get_result(
        &self,
        params: &[ParamsOfQueryOperation],
        index: usize,
        result: &Value,
    ) -> ClientResult<Value> {
        let param = params.get(index);
        let result_name = if self.is_batch {
            format!("q{}", index + 1)
        } else if let Some(param) = param {
            param.query_name()
        } else {
            "info".to_string()
        };
        let mut result_data = &result["data"][result_name.as_str()];
        if result_data.is_null() {
            return Err(crate::net::Error::invalid_server_response(format!(
                "Missing data.{} in: {}",
                result_name, result
            )));
        }
        if let Some(ParamsOfQueryOperation::WaitForCollection(_)) = param {
            result_data = &result_data[0];
            if result_data.is_null() {
                return Err(crate::net::Error::wait_for_timeout());
            }
        }
        Ok(result_data.clone())
    }

    pub fn get_results(
        &self,
        params: &[ParamsOfQueryOperation],
        result: &Value,
    ) -> ClientResult<Vec<Value>> {
        let mut results = Vec::new();
        for i in 0..params.len() {
            results.push(self.get_result(params, i, result)?);
        }
        Ok(results)
    }

    pub fn get_server_info(
        &self,
        params: &[ParamsOfQueryOperation],
        result: &Value,
    ) -> ClientResult<Value> {
        self.get_result(params, params.len(), result)
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
        let filter_type = Self::filter_type_for_collection(&table);

        let query = format!("subscription {table}($filter: {type}) {{ {table}(filter: $filter) {{ {fields} }} }}",
            type=filter_type,
            table=table,
            fields=fields);
        let query = query.split_whitespace().collect::<Vec<&str>>().join(" ");
        let variables = Some(json!({
            "filter" : filter,
        }));
        Self {
            query,
            variables,
            timeout: None,
            is_batch: false,
        }
    }

    pub fn with_post_requests(requests: &[PostRequest]) -> Self {
        let query = "mutation postRequests($requests:[Request]){postRequests(requests:$requests)}"
            .to_owned();
        let variables = Some(json!({ "requests": serde_json::json!(requests) }));
        Self {
            query,
            variables,
            timeout: None,
            is_batch: false,
        }
    }

    pub fn filter_type_for_collection(collection: &str) -> String {
        let mut filter_type = if let Some(prefix) = collection.strip_suffix("ies") {
            format!("{}yFilter", prefix)
        } else {
            format!("{}Filter", collection[0..collection.len() - 1].to_string())
        };
        filter_type[..1].make_ascii_uppercase();
        filter_type
    }
}

#[derive(Debug)]
pub enum GraphQLQueryEvent {
    Id(u32),
    Data(Value),
    Error(ClientError),
    Complete,
}

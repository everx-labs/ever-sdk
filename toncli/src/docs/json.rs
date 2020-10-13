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

use crate::api_item::ApiItem;
use crate::errors::CliResult;
use crate::text_generator::Output;
use api_info::API;

pub fn doc_json(api: &API, item: ApiItem, output: &Output) -> CliResult<()> {
    let (file, json) = match item {
        ApiItem::Api => ("api".to_string(), serde_json::to_value(api)?),
        ApiItem::Module(m) => (format!("{}", m.name), serde_json::to_value(m)?),
        ApiItem::Function(m, f) => (format!("{}_{}", m.name, f.name), serde_json::to_value(f)?),
        ApiItem::Type(m, t) => (format!("{}_{}", m.name, t.name), serde_json::to_value(t)?),
    };
    let text = serde_json::to_string_pretty(&json).unwrap_or("".into());
    output.write(&format!("{}.json", file), &text)
}


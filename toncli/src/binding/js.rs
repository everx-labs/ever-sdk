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
use crate::errors::{CliError, CliResult};
use api_info::{Field, Function, Module, API};

fn not_implemented() -> CliResult<()> {
    Err(CliError::with_message("not implemented".into()))
}

fn js_api(_api: &API) -> CliResult<()> {
    not_implemented()
}

fn js_module(_api: &API, _module: &Module) -> CliResult<()> {
    not_implemented()
}

fn js_function(_api: &API, _module: &Module, _function: &Function) -> CliResult<()> {
    not_implemented()
}

fn js_type(_api: &API, _module: &Module, _ty: &Field) -> CliResult<()> {
    not_implemented()
}

pub fn binding_js(api: &API, item: ApiItem) -> CliResult<()> {
    match item {
        ApiItem::Api => js_api(api),
        ApiItem::Module(m) => js_module(api, m),
        ApiItem::Function(m, f) => js_function(api, m, f),
        ApiItem::Type(m, t) => js_type(api, m, t),
    }
}

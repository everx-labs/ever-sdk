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

use api_info::{API, Module, Function, Field};
use crate::errors::CliError;

pub enum ApiItem<'a> {
    Api,
    Module(&'a Module),
    Function(&'a Module, &'a Function),
    Type(&'a Module, &'a Field),
}

impl<'a> ApiItem<'a> {
    pub fn from_name(api: &'a API, name: &str) -> Result<ApiItem<'a>, CliError> {
        if name.is_empty() {
            return Ok(ApiItem::Api)
        }
        for module in &api.modules {
            if module.name == name {
                return Ok(ApiItem::Module(module));
            }
            for function in &module.functions {
                if function.name == name {
                    return Ok(ApiItem::Function(module, function));
                }
            }
            for ty in &module.types {
                if ty.name == name {
                    return Ok(ApiItem::Type(module, ty));
                }
            }
        }
        Err(CliError::with_message(format!("Api item not found: {}", name)))
    }
}


/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

use crate::command_line::CommandLine;
use crate::errors::{CliError, CliResult};
use api_info::{Field, Function, Module, Type, API};
use std::sync::Arc;
use ever_client::ClientContext;

fn find_type<'a>(
    name: &str,
    default_module: &Module,
    api: &'a API,
) -> Option<(&'a Module, &'a Field)> {
    let mut names = name.split('.');
    let (module_name, type_name) = if let (Some(m), Some(t)) = (names.next(), names.next()) {
        (m, t)
    } else {
        (default_module.name.as_str(), name)
    };
    let mut fallback = None;
    for module in &api.modules {
        for ty in &module.types {
            if ty.name == type_name {
                if module.name == module_name {
                    return Some((module, ty));
                }
                fallback = Some((module, ty));
            }
        }
    }
    fallback
}

fn reduce_type(ty: &Type, module: &Module, api: &API) -> Type {
    match ty {
        Type::Array { item } => Type::Array {
            item: Box::new(reduce_type(item, module, api)),
        },
        Type::Optional { inner } => Type::Optional {
            inner: Box::new(reduce_type(inner, module, api)),
        },
        Type::Generic { name, args } => Type::Generic {
            name: name.clone(),
            args: args.iter().map(|a| reduce_type(a, module, api)).collect(),
        },
        Type::Ref { name } => {
            if let Some((m, t)) = find_type(&name, module, api) {
                Type::Ref {
                    name: format!("{}.{}", m.name, t.name),
                }
            } else {
                ty.clone()
            }
        }
        Type::Struct { fields } => {
            if fields.len() == 1 && fields[0].name.is_empty() {
                reduce_type(&fields[0].value, module, api)
            } else {
                for f in fields {
                    if f.name.is_empty() {
                        panic!("API can't contains tuples")
                    }
                }
                Type::Struct {
                    fields: fields
                        .iter()
                        .map(|x| reduce_field(x, module, api))
                        .collect(),
                }
            }
        }
        Type::EnumOfTypes { types } => {
            let mut reduced_types = Vec::new();
            for variant in types {
                reduced_types.push(reduce_field(variant, module, api));
            }
            Type::EnumOfTypes {
                types: reduced_types,
            }
        }
        _ => ty.clone(),
    }
}

fn reduce_field(field: &Field, module: &Module, api: &API) -> Field {
    Field {
        name: field.name.clone(),
        summary: field.summary.clone(),
        description: field.description.clone(),
        value: reduce_type(&field.value, module, api),
    }
}

fn reduce_function(function: &Function, module: &Module, api: &API) -> Function {
    Function {
        name: function.name.clone(),
        summary: function.summary.clone(),
        description: function.description.clone(),
        params: function
            .params
            .iter()
            .map(|p| reduce_field(p, module, api))
            .collect(),
        result: reduce_type(&function.result, module, api),
        errors: function.errors.clone(),
    }
}

fn reduce_module(module: &Module, api: &API) -> Module {
    Module {
        name: module.name.clone(),
        types: module
            .types
            .iter()
            .map(|t| reduce_field(t, module, api))
            .collect(),
        functions: module
            .functions
            .iter()
            .map(|f| reduce_function(f, module, api))
            .collect(),
        description: module.description.clone(),
        summary: module.summary.clone(),
    }
}

fn reduce_api(api: &API) -> API {
    API {
        version: api.version.clone(),
        modules: api.modules.iter().map(|m| reduce_module(m, api)).collect(),
    }
}

fn get_api() -> CliResult<API> {
    let context = Arc::new(ClientContext::new(Default::default())?);
    let api = ever_client::client::get_api_reference(context)?.api;
    Ok(reduce_api(&api))
}

fn write_text_to_out_dir(text: String, out_dir: String) -> CliResult<()> {
    let out_dir = if out_dir.starts_with("~/") {
        dirs::home_dir()
            .ok_or(CliError::with_message("Home dir not found".into()))?
            .join(&out_dir[2..])
    } else {
        out_dir.into()
    };
    let file_path = out_dir.join("api.json");
    if let Some(parent_dir) = file_path.parent() {
        std::fs::create_dir_all(parent_dir)?
    }
    std::fs::write(file_path, text)?;
    Ok(())
}

pub fn command(args: &[String]) -> Result<(), CliError> {
    let command_line = CommandLine::parse(args)?;
    let json = serde_json::to_value(get_api()?)?;
    let mut text = serde_json::to_string_pretty(&json).unwrap_or("".into());
    text += "\n";
    let out_dir = command_line.get_opt("o|out-dir").map(|x| x.to_string());
    if let Some(out_dir) = out_dir {
        write_text_to_out_dir(text, out_dir)
    } else {
        println!("{}", text);
        Ok(())
    }
}

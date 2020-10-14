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

use crate::errors::CliResult;
use crate::text_generator::Output;
use api_info::{ConstValue, Field, Function, Module, Type, API};

const MODULES_HEADER: &str = r#"
import {ResponseHandler} from "./bin";

interface IClient {
    request(
        functionName: string,
        functionParams: any,
        responseHandler?: ResponseHandler
    ): Promise<any>;
}

"#;

fn upper_first(ident: &str) -> String {
    if ident.is_empty() {
        String::new()
    } else {
        format!("{}{}", ident[0..1].to_uppercase(), &ident[1..])
    }
}

fn lower_first(ident: &str) -> String {
    if ident.is_empty() {
        String::new()
    } else {
        format!("{}{}", ident[0..1].to_lowercase(), &ident[1..])
    }
}

fn parse_snake(ident: &str) -> Vec<String> {
    ident.split('_').map(|x| x.to_string()).collect()
}

fn pascal(words: Vec<String>) -> String {
    let mut pascal = String::new();
    for word in words {
        pascal.push_str(&upper_first(&word));
    }
    pascal
}

fn camel(words: Vec<String>) -> String {
    lower_first(&pascal(words))
}

fn generate_module(api: &API, module: &Module, ts: &mut String) -> CliResult<()> {
    ts.push_str(&format!("// {} module\n\n", module.name));

    for ty in &module.types {
        generate_type(api, module, ty, ts)?;
    }

    ts.push_str(&format!(
        r#"
export class {}Module {{
    client: IClient;

    constructor(client: IClient) {{
        this.client = client;
    }}
"#,
        upper_first(&module.name)
    ));

    for function in &module.functions {
        generate_function(api, module, function, ts)?;
    }

    ts.push_str("}\n\n");
    Ok(())
}

fn generate_type_decl(ty: &Type, ident: &str, ts: &mut String) -> CliResult<()> {
    match ty {
        Type::None {} => {
            ts.push_str("void");
        }
        Type::Ref(ref_type_name) => {
            if ref_type_name == "Value" || ref_type_name == "API" {
                ts.push_str("any");
            } else {
                let mut parts = ref_type_name.split('.');
                let ref_name = match (parts.next(), parts.next()) {
                    (Some(_), Some(func)) => func.to_string(),
                    (Some(func), None) => func.to_string(),
                    _ => "".into(),
                };
                ts.push_str(&format!("{}", ref_name));
            }
        }
        Type::Optional(inner) => {
            generate_type_decl(&*inner, ident, ts)?;
            ts.push_str(" | null");
        }
        Type::Struct(fields) => {
            if fields.len() == 1 && fields[0].name.is_empty() {
                generate_type_decl(&fields[0].value, ident, ts)?;
            } else {
                ts.push_str("{\n");
                let fields_ident = format!("{}    ", ident);
                for field in fields {
                    ts.push_str(&fields_ident);
                    ts.push_str(&format!("{}: ", field.name));
                    generate_type_decl(&field.value, &fields_ident, ts)?;
                    ts.push_str(",\n");
                }
                ts.push_str(ident);
                ts.push_str("}");
            }
        }
        Type::EnumOfTypes(variants) => {
            let mut is_first = true;
            for variant in variants {
                if !is_first {
                    ts.push_str(" | ");
                }
                ts.push_str(&format!("{{ {}: ", variant.name));
                match &variant.value {
                    Type::Struct(fields) if fields.len() == 1 && fields[0].name.is_empty() => {
                        generate_type_decl(&fields[0].value, ident, ts)?;
                    }
                    _ => {
                        generate_type_decl(&variant.value, ident, ts)?;
                    }
                }
                ts.push_str(" }");
                is_first = false;
            }
        }
        Type::Array(item) => {
            generate_type_decl(&item, ident, ts)?;
            ts.push_str("[]");
        }
        Type::String { .. } => {
            ts.push_str("string");
        }
        Type::Any { .. } => {
            ts.push_str("any");
        }
        Type::BigInt { .. } => {
            ts.push_str("bigint");
        }
        Type::Boolean { .. } => {
            ts.push_str("boolean");
        }
        Type::EnumOfConsts(variants) => {
            let mut is_first = true;
            for variant in variants {
                if !is_first {
                    ts.push_str(" | ");
                }
                match &variant.value {
                    ConstValue::Number(n) => ts.push_str(n.as_str()),
                    ConstValue::String(s) => ts.push_str(&format!("\"{}\"", s)),
                    ConstValue::Bool(b) => ts.push_str(b.as_str()),
                    ConstValue::None {} => ts.push_str(&format!("\"{}\"", variant.name)),
                }
                is_first = false;
            }
        }
        Type::Number {} => ts.push_str("number"),
        Type::Generic { name, .. } => ts.push_str(&format!("{}<>", name)),
    }
    Ok(())
}

fn generate_type(_api: &API, _module: &Module, ty: &Field, ts: &mut String) -> CliResult<()> {
    ts.push_str(&format!("export type {} = ", ty.name));
    generate_type_decl(&ty.value, "", ts)?;
    ts.push_str(";\n\n");
    Ok(())
}

fn generate_function(
    _api: &API,
    module: &Module,
    function: &Function,
    ts: &mut String,
) -> CliResult<()> {
    ts.push_str(&format!("    {}(", camel(parse_snake(&function.name))));
    let mut is_first = true;
    for param in &function.params {
        if !is_first {
            ts.push_str(", ");
        }
        ts.push_str(&format!("\n        {}: ", param.name));
        generate_type_decl(&param.value, "", ts)?;
        is_first = false;
    }
    if !is_first {
        ts.push_str(", ");
    }
    ts.push_str("\n        responseHandler?: ResponseHandler\n    ): Promise<");
    generate_type_decl(&function.result, "", ts)?;
    ts.push_str(&format!(
        "> {{\n        return this.client.request(\n            '{}.{}',",
        module.name, function.name
    ));
    ts.push_str(&format!(
        "\n            {},",
        if !function.params.is_empty() {
            &function.params[0].name
        } else {
            "undefined"
        }
    ));
    ts.push_str("\n            responseHandler,\n        );\n    }\n");
    Ok(())
}

pub fn binding_ts(api: &API, output: Output) -> CliResult<()> {
    let mut ts = String::new();
    ts.push_str(MODULES_HEADER);
    for module in &api.modules {
        generate_module(api, module, &mut ts)?;
    }
    output.write("modules.ts", &ts)?;
    Ok(())
}

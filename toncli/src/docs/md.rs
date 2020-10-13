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
use crate::text_generator::{Output, StringWriter};
use api_info::{Field, Function, Module, Type, API};
use std::sync::Arc;

fn summary(summary: &Option<String>) -> String {
    if let Some(summary) = summary {
        let summary = summary.trim();
        format!(
            " – {}{}",
            summary[0..1].to_string().to_lowercase(),
            summary[1..].to_string()
        )
    } else {
        "".into()
    }
}

fn description(description: &Option<String>) -> String {
    if let Some(description) = description {
        format!("{}\n", description)
    } else {
        "".into()
    }
}

fn module_file(module: &Module) -> String {
    format!("{}.md", module.name)
}

fn module_member_file(module: &Module, member: &str) -> String {
    format!("{}_{}.md", module.name, member)
}

fn doc_children(
    embed: bool,
    parent_md: &mut String,
    parent_output: &Output,
    children: impl Fn(&Output) -> CliResult<()>,
) -> CliResult<()> {
    if embed {
        let writer = Arc::new(StringWriter::new());
        children(&parent_output.clone_with_writer(writer.clone()))?;

        let text = writer.text();
        if !text.is_empty() {
            parent_md.push_str(&text);
        }
    } else {
        children(parent_output)?;
    }
    Ok(())
}

fn doc_api(api: &API, output: &Output) -> CliResult<()> {
    let mut md = String::new();
    md.push_str("# Modules\n");
    for module in &api.modules {
        md.push_str(&format!(
            "## [{}]({}){}\n\n",
            module.name,
            module_file(module),
            summary(&module.summary)
        ));
        for function in &module.functions {
            md.push_str(&format!(
                "[{}]({}){}\n\n",
                function.name,
                module_member_file(module, &function.name),
                summary(&function.summary)
            ));
        }
    }
    doc_children(output.embed_modules, &mut md, output, |output| {
        for module in &api.modules {
            doc_module(api, module, output)?
        }
        Ok(())
    })?;
    output.write("modules.md", &md)?;
    Ok(())
}

fn doc_module(api: &API, module: &Module, output: &Output) -> CliResult<()> {
    let mut md = String::new();
    md.push_str(&format!("# {}\n", module.name));
    md.push_str("# Functions\n");
    for function in &module.functions {
        md.push_str(&format!(
            "[{}](#{}){}\n\n",
            function.name,
            function.name,
            summary(&function.summary)
        ));
    }
    md.push_str("# Types\n");
    for ty in &module.types {
        md.push_str(&format!(
            "[{}](#{}){}\n\n",
            ty.name,
            ty.name,
            summary(&ty.summary)
        ));
    }

    doc_children(output.embed_functions, &mut md, output, |output| {
        for function in &module.functions {
            doc_function(api, module, function, output)?;
        }
        Ok(())
    })?;

    doc_children(output.embed_types, &mut md, output, |output| {
        for ty in &module.types {
            doc_type(api, module, ty, output)?;
        }
        Ok(())
    })?;

    output.write(&format!("{}.md", module.name), &md)
}

fn find_type<'a>(api: &'a API, name: &str) -> Option<&'a Field> {
    for module in &api.modules {
        for ty in &module.types {
            if format!("{}.{}", module.name, ty.name) == name {
                return Some(ty);
            }
        }
    }
    None
}

fn doc_type_field(api: &API, field: &Field, md: &mut String) -> CliResult<()> {
    md.push_str(&format!(
        "- `{}`: _{}_{}\n",
        field.name,
        doc_type_name(&field.value),
        summary(&field.summary)
    ));
    Ok(())
}

fn doc_type_fields(api: &API, module: &Module, ty: &Type, md: &mut String) -> CliResult<()> {
    match ty {
        Type::Ref(ref_type_name) => {
            if let Some(ty) = find_type(api, ref_type_name.as_str()) {
                doc_type_fields(api, module, &ty.value, md)?;
            }
        }
        Type::Optional(ty) => {
            doc_type_fields(api, module, &*ty, md)?;
        }
        Type::Struct(ty) => {
            for field in ty {
                doc_type_field(api, field, md)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn doc_type_name(ty: &Type) -> String {
    match ty {
        Type::Ref(ref_type_name) => {
            if ref_type_name == "Value" {
                "any".into()
            } else {
                let mut parts = ref_type_name.split('.');
                let ref_name = match (parts.next(), parts.next()) {
                    (Some(_), Some(func)) => func.to_string(),
                    (Some(func), None) => func.to_string(),
                    _ => "".into(),
                };
                format!("[{}]({})", ref_name, ref_name)
            }
        }
        Type::Optional(ty) => format!("{}?", doc_type_name(&*ty)),
        Type::Struct(_) => "{}".into(),
        Type::EnumOfTypes(_) => "(A|B)".into(),
        Type::Array(item) => format!("{}[]", doc_type_name(&*item)),
        Type::String { .. } => "string".into(),
        Type::Any { .. } => "any".into(),
        Type::BigInt { .. } => "BigInt".into(),
        Type::Boolean { .. } => "boolean".into(),
        Type::EnumOfConsts(_) => "(0|1)".into(),
        Type::Number {} => "number".into(),
        Type::Generic { name, .. } => format!("{}<>", name),
        _ => "".into(),
    }
}

fn doc_function(api: &API, module: &Module, function: &Function, output: &Output) -> CliResult<()> {
    let mut md = String::new();
    md.push_str(&format!("# {}\n", function.name));
    md.push_str(&description(&function.description));
    if function.params.len() > 0 {
        md.push_str("## Parameters\n");
        doc_type_fields(api, module, &function.params[0].value, &mut md);
    }
    md.push_str("## Result\n");
    doc_type_fields(api, module, &function.result, &mut md);
    output.write(&format!("{}_{}.md", module.name, function.name), &md)
}

fn doc_type(_api: &API, module: &Module, ty: &Field, output: &Output) -> CliResult<()> {
    let mut md = String::new();
    md.push_str(&format!("# {}\n", ty.name));
    md.push_str(&description(&ty.description));
    output.write(&format!("{}_{}.md", module.name, ty.name), &md)
}

pub fn doc_md(api: &API, item: ApiItem, output: &Output) -> CliResult<()> {
    match item {
        ApiItem::Api => doc_api(api, output),
        ApiItem::Module(m) => doc_module(api, m, output),
        ApiItem::Function(m, f) => doc_function(api, m, f, output),
        ApiItem::Type(m, t) => doc_type(api, m, t, output),
    }
}

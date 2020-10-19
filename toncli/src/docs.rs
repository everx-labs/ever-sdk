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
use crate::command_line::CommandLine;
use crate::errors::{CliError, CliResult};
use api_info::API;
use std::sync::Arc;
use ton_client::ClientContext;
use std::path::PathBuf;

pub trait OutputWriter {
    fn write(&self, file: &str, text: &str) -> CliResult<()>;
}

#[derive(Clone)]
pub struct Output {
    pub embed_modules: bool,
    pub embed_functions: bool,
    pub embed_types: bool,
    writer: Arc<dyn OutputWriter>,
}

impl Output {
    pub fn parse(command_line: &CommandLine) -> CliResult<Self> {
        let writer: Arc<dyn OutputWriter> =
            if let Some(dir) = command_line.get_opt("o|out-dir").map(|x| x.to_string()) {
                Arc::new(DirWriter::new(dir)?)
            } else {
                Arc::new(ConsoleWriter {})
            };
        let embed = command_line.get_opt_set("e|embed");
        let embed_all = embed.contains("all");
        Ok(Self {
            writer,
            embed_modules: embed_all || embed.contains("modules"),
            embed_functions: embed_all || embed.contains("functions"),
            embed_types: embed_all || embed.contains("types"),
        })
    }
    pub fn write(&self, file: &str, text: &str) -> CliResult<()> {
        self.writer.write(file, text)
    }
}

pub struct ConsoleWriter;

impl OutputWriter for ConsoleWriter {
    fn write(&self, _file: &str, text: &str) -> CliResult<()> {
        println!("{}", text);
        Ok(())
    }
}

pub struct DirWriter {
    dir: PathBuf,
}

impl DirWriter {
    fn new(dir: String) -> CliResult<Self> {
        let dir = if dir.starts_with("~/") {
            dirs::home_dir()
                .ok_or(CliError::with_message("Home dir not found".into()))?
                .join(&dir[2..])
        } else {
            dir.into()
        };
        Ok(Self { dir })
    }
}

impl OutputWriter for DirWriter {
    fn write(&self, file: &str, text: &str) -> CliResult<()> {
        let file_path = self.dir.join(file);
        if let Some(parent_dir) = file_path.parent() {
            std::fs::create_dir_all(parent_dir)?
        }
        std::fs::write(file_path, text)?;
        Ok(())
    }
}

pub fn doc_json(api: &API, item: ApiItem, output: Output) -> CliResult<()> {
    let (file, json) = match item {
        ApiItem::Api => ("api".to_string(), serde_json::to_value(api)?),
        ApiItem::Module(m) => (format!("{}", m.name), serde_json::to_value(m)?),
        ApiItem::Function(m, f) => (format!("{}_{}", m.name, f.name), serde_json::to_value(f)?),
        ApiItem::Type(m, t) => (format!("{}_{}", m.name, t.name), serde_json::to_value(t)?),
    };
    let text = serde_json::to_string_pretty(&json).unwrap_or("".into());
    output.write(&format!("{}.json", file), &text)
}

pub fn command(args: &[String]) -> Result<(), CliError> {
    let command_line = CommandLine::parse(args)?;
    let mut args = command_line.args.iter();
    let api = ton_client::client::get_api_reference(Arc::new(ClientContext::new(None)?))?.api;
    let name = args.next().map(|x| x.as_str()).unwrap_or("");
    doc_json(
        &api,
        ApiItem::from_name(&api, name)?,
        Output::parse(&command_line)?,
    )
}

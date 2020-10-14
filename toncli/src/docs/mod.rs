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

mod json;
mod md;

use crate::api_item::ApiItem;
use crate::command_line::CommandLine;
use crate::docs::md::doc_md;
use crate::errors::{CliError, CliResult};
use crate::text_generator::Output;
use std::sync::Arc;
use ton_client::ClientContext;

#[derive(Clone)]
pub enum Format {
    MD,
    JSON,
}

#[derive(Clone)]
pub struct Options {
    format: Format,
    output: Output,
}

impl Options {
    pub fn parse(command_line: &CommandLine) -> CliResult<Self> {
        let format = match command_line.get_opt("f|format") {
            Some("md") => Format::MD,
            Some("json") => Format::JSON,
            Some(unknown) => {
                return Err(CliError::with_message(format!(
                    "Invalid doc format: {}",
                    unknown
                )))
            }
            None => Format::MD,
        };
        Ok(Self {
            format,
            output: Output::parse(command_line)?,
        })
    }
}

pub fn command(args: &[String]) -> Result<(), CliError> {
    let command_line = CommandLine::parse(args)?;
    let mut args = command_line.args.iter();
    let api = ton_client::client::get_api_reference(Arc::new(ClientContext::new(None)?))?.api;
    let name = args.next().map(|x| x.as_str()).unwrap_or("");
    let item = ApiItem::from_name(&api, name)?;
    let options = Options::parse(&command_line)?;
    match options.format {
        Format::MD => doc_md(&api, item, &options.output)?,
        Format::JSON => json::doc_json(&api, item, &options.output)?,
    }
    Ok(())
}

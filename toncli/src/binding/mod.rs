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

mod ts;

use crate::binding::ts::binding_ts;
use crate::command_line::CommandLine;
use crate::errors::{CliError, CliResult};
use crate::text_generator::Output;
use std::sync::Arc;
use ton_client::ClientContext;

enum Lang {
    TS,
}

impl Lang {
    fn parse(command_line: &CommandLine) -> CliResult<Self> {
        match command_line.get_opt("format") {
            Some("ts") => Ok(Lang::TS),
            Some(unk) => Err(CliError::with_message(format!(
                "Invalid binding language: {}",
                unk
            ))),
            _ => Ok(Lang::TS),
        }
    }
}

pub fn command(args: &[String]) -> Result<(), CliError> {
    let command_line = CommandLine::parse(args)?;
    let api = ton_client::client::get_api_reference(Arc::new(ClientContext::new(None)?))?.api;
    match Lang::parse(&command_line)? {
        Lang::TS => binding_ts(&api, Output::parse(&command_line)?)?,
    }
    Ok(())
}

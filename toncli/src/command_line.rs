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

use crate::errors::CliError;
use std::collections::{HashMap};

enum ParseState {
    OptionOrArg,
    OptionValue,
    Arg,
}

pub struct CommandLine {
    pub args: Vec<String>,
    pub options: HashMap<String, String>,
}

impl CommandLine {
    pub fn get_opt(&self, names: &str) -> Option<&str> {
        names
            .split("|")
            .find_map(|x| self.options.get(x.trim()).map(|x| x.as_str()))
    }

    pub fn parse(args: &[String]) -> Result<Self, CliError> {
        let mut command_line = Self {
            args: Vec::new(),
            options: HashMap::new(),
        };
        let mut state = ParseState::OptionOrArg;
        let mut option = String::new();

        for arg in args {
            match state {
                ParseState::OptionOrArg if arg.starts_with("-") => {
                    option = arg[1..].to_string();
                    state = ParseState::OptionValue
                }
                ParseState::OptionOrArg if arg.starts_with("--") => {
                    option = arg[2..].to_string();
                    state = ParseState::OptionValue
                }
                ParseState::OptionOrArg => {
                    command_line.args.push(arg.clone());
                    state = ParseState::Arg
                }
                ParseState::OptionValue => {
                    command_line.options.insert(option.clone(), arg.clone());
                    state = ParseState::OptionOrArg
                }
                ParseState::Arg => command_line.args.push(arg.clone()),
            }
        }
        Ok(command_line)
    }
}

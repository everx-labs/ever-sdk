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
 */
use std::env;
use ton_client::{create_context, json_sync_request, destroy_context, client::ResultOfCreateContext};
use serde_json::Value;
use regex::{Regex, Captures};

enum ParseState {
    OptionOrMethodName,
    OptionValue,
    Parameters,
}

struct CliError {
    message: String
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError { message: e.to_string() }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError { message: e.to_string() }
    }
}


impl From<regex::Error> for CliError {
    fn from(e: regex::Error) -> Self {
        CliError { message: e.to_string() }
    }
}


impl From<json5::Error> for CliError {
    fn from(e: json5::Error) -> Self {
        CliError { message: e.to_string() }
    }
}


fn main() {
    if let Err(err) = main_internal() {
        eprintln!("{}", err.message);
        std::process::exit(1)
    }
}

fn reformat_json(json: &String) -> Result<String, CliError> {
    if json.trim().is_empty() {
        Ok(String::new())
    } else {
        let value: Value = serde_json::from_str(json)?;
        serde_json::to_string_pretty(&value).map_err(|e| CliError { message: e.to_string() })
    }
}

fn resolve_json_path<'a>(value: &'a Value, _path: &str) -> &'a Value {
    value
}

fn include_json(json_ref: &str) -> Result<String, CliError> {
    let ref_parts: Vec<&str> = json_ref.split("+").collect();
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_string();
    let ref_file = ref_parts[0].replace("~", home.as_str());
    let ref_path = if ref_parts.len() > 1 { ref_parts[1] } else { "" };
    let ref_string = std::fs::read_to_string(&ref_file)
        .map_err(|e| CliError { message: format!("Include [{}] failed: {}", ref_file, e) })?;
    let value: Value = serde_json::from_str(&ref_string)
        .map_err(|e| CliError { message: format!("Include [{}] failed: {}", ref_file, e) })?;
    let value = resolve_json_path(&value, ref_path);
    Ok(value.to_string())
}

fn main_internal() -> Result<(), CliError> {
    let mut network = "net.ton.dev".to_string();
    let mut state = ParseState::OptionOrMethodName;
    let mut option = String::new();
    let mut method = String::new();
    let mut parameters = String::new();
    let api = ton_client::get_api();
    for arg in env::args().skip(1) {
        match state {
            ParseState::OptionOrMethodName if arg.starts_with("-") => {
                option = arg[1..].to_string();
                state = ParseState::OptionValue
            }
            ParseState::OptionOrMethodName if arg.starts_with("--") => {
                option = arg[2..].to_string();
                state = ParseState::OptionValue
            }
            ParseState::OptionOrMethodName => {
                method = arg;
                state = ParseState::Parameters
            }
            ParseState::OptionValue => {
                match option.as_str() {
                    "n" | "network" => {
                        network = arg
                    }
                    _ => {}
                }
                state = ParseState::OptionOrMethodName
            }
            ParseState::Parameters => {
                if !parameters.is_empty() {
                    parameters.push(' ');
                }
                parameters.push_str(arg.as_str())
            }
        }
    }
    if !parameters.trim().is_empty() {
        let file_refs = Regex::new(r"@(\S*)")?;
        parameters = file_refs.replace(&parameters, |caps: &Captures| {
            match include_json(&caps[1]) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("{}", e.message);
                    std::process::exit(1)
                }
            }
        }).to_string();
        let json: Value = json5::from_str(parameters.as_str())?;
        parameters = json.to_string();
    }

    let method_names: Vec<String> = api.methods.iter().map(|x|x.name.clone()).collect();
    if method.is_empty() {
        return Err(CliError {
            message: format!(
                "Method doesn't specified. Available methods are:\n{}",
                method_names.join("\n")
            )
        });
    }

    let mut names = Vec::<String>::new();
    for name in method_names {
        if name == method {
            names.clear();
            names.push(name);
            break;
        }
        if name.contains(method.as_str()) {
            names.push(name);
        }
    }
    if names.is_empty() {
        return Err(CliError {
            message: format!(
                "Unknown method [{}]. Available methods are:\n{}",
                method,
                names.join("\n")
            )
        });
    }
    if names.len() > 1 {
        return Err(CliError {
            message: format!(
                "Unknown method [{}]. May be you mean one of following:\n{}",
                method,
                names.join("\n")
            )
        });
    }
    if names[0] != method {
        eprintln!("Unknown method [{}]. [{}] used instead.", method, names[0]);
        method = names[0].clone();
    }

    let config = serde_json::json!({
        "network": {
            "server_address": network
        }
    });
    let response = create_context(config.to_string());

    if !response.error_json.trim().is_empty() {
        return Err(CliError { message: reformat_json(&response.error_json)? });
    };

    let context: ResultOfCreateContext = serde_json::from_str(&response.result_json)?;

    let response = json_sync_request(context.handle, method, parameters);
    let result = if response.error_json.trim().is_empty() {
        println!("{}", reformat_json(&response.result_json)?);
        Ok(())
    } else {
        Err(CliError { message: reformat_json(&response.error_json)? })
    };
    destroy_context(context.handle);
    result
}


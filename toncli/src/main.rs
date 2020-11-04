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
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod api;
mod errors;
mod command_line;
mod request;

const USAGE: &str = r#"
toncli <command> args...
where command is:

api – export ton client api JSON

request <function> <params> – executes ton client api function
    <function> – any possible api function name in form of module.function
    <params> – all args next to <function> collected as a JSON5 function parameters
"#;

fn print_usage_and_exit() {
    println!("{}", USAGE);
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.iter().skip(1).next().map(|x| x.as_str());
    let result = match cmd.unwrap_or("") {
        "api" => api::command(&args[2..]),
        "request" => request::command(&args[2..]),
        _ => {
            print_usage_and_exit();
            Ok(())
        }
    };
    if let Err(err) = result {
        eprintln!("{}", err.message);
        std::process::exit(1);
    }
}

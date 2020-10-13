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

mod docs;
mod errors;
mod command_line;
mod execute;
mod api_item;
mod binding;
mod text_generator;

const USAGE: &str = r#"
toncli <command> args...
where command is:

api <function> args...
    executes ton client api function

docs
    generates ton client api documentation

binding
    generates ton client binding code
"#;

fn print_usage_and_exit() {
    println!("{}", USAGE);
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.iter().skip(1).next().map(|x| x.as_str());
    let result = match cmd.unwrap_or("") {
        "api" => execute::command(&args[2..]),
        "docs" => docs::command(&args[2..]),
        "binding" => binding::command(&args[2..]),
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

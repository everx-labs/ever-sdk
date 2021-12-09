/*
 * Copyright 2018-2021 TON Labs LTD.
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
mod command_line;
mod errors;
mod request;

const USAGE: &str = r#"
Usage:  toncli [OPTIONS] <command> [args...]

Commands:
    api      Exports ton client api JSON
    request  Executes ton client api function

api [OPTIONS]
Options:
    -o, --out-dir string  Path to folder where the `api.json` will be stored.
                          If omitted, then api json will be printed to console.
Example:
    toncli api -o ~/ton

request <function> [params...]
    function  Any possible api function name in form of `module.function`.
    params    All params collected as a JSON5 function parameters.
              You can use file includes in params. File include must be specified in
              form of `@filename`. If file has an extension `.json` then content of json
              will be inserted as is. Elsewhere the content of specified file
              will be encoded with base64 and enclosed in double quotes.

Example:
    toncli request client.version
    toncli request crypto.generate_random_bytes { length: 32 }
    toncli request abi.attach_signature_to_message_body { \
        abi: { type: \"Contract\", value: @hello.abi.json }, \
        public_key: \"...\", \
        message: @message.boc, \
        signature: \"...\" }
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

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

use std::io::{self};
use std::sync::Arc;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use ton_types::{Cell, SliceData};
use ton_abi::{TokenValue, ParamType};
use crate::ClientContext;
use crate::debot::cpprun::get_acc_balance;
use crate::debot::cpp_browser::CppBrowserCallbacks;

fn get_message_text(msg_json: &Value) -> String {
    let msg_text_code = msg_json["message"].as_str().unwrap();
    let message_text = String::from_utf8(
        hex::decode(msg_text_code).unwrap()
    ).unwrap();
    return message_text;
}

fn get_params_cell(msg_json: &Value) -> Cell {
    let params_text_code = msg_json["params"].as_str().unwrap();
    let bytes = base64::decode(&params_text_code).unwrap();
    let cl:Cell = ton_types::cells_serialization::deserialize_tree_of_cells(&mut bytes.as_slice())
        .unwrap();
    return cl;
}

fn process_printf_param(context: Arc<ClientContext>, sl: &mut SliceData, param_type_name: &String) -> String {
    if param_type_name == "datetime" {
        let time_u32 = sl.get_next_u32().unwrap();
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time_u32.into(), 0), Utc);
        return format!("{}", time);
    }
    if param_type_name == "value" {
        // TODO: support big ints
        sl.get_next_bits(256 - 64).unwrap();
        let val = sl.get_next_int(64).unwrap();
        return format!("{:.2}T", (val as f64) / 1000000000.0f64);
    }
    // TODO: prepare full list of params and use decode_params() to correctly process last fields
    let param_type = ton_abi::param_type::read_type(&param_type_name).unwrap();
    let (token, next_sl) = TokenValue::read_from(&param_type, sl.clone(), false, 2).unwrap();
    *sl = next_sl;
    if param_type == ParamType::Address {
        let addr_str = token.to_string();
        let balance = get_acc_balance(context, &addr_str);
        return format!("{} ({:.2}T)", addr_str, (balance as f64) / 1000000000.0f64);
    }
    return token.to_string();
}

pub struct CppConsole {
}
impl CppConsole {
    pub async fn print(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        browser.log(get_message_text(&msg_json)).await;
        return Some(r#"{ "value0": "true" }"#.to_string());
    }

    pub async fn printf(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        let params_cell = get_params_cell(&msg_json);
        let mut sl = SliceData::from(params_cell);
        let mut cur_param = String::new();
        let mut cur_str = String::new();
        let mut param_opened = false;
        let mut escaped_next = false;
        for ch in msg_text.chars() {
            if param_opened {
                if ch == '}' {
                    cur_str.push_str(&process_printf_param(context.clone(), &mut sl, &cur_param));
                    param_opened = false;
                    cur_param.clear();
                } else {
                    cur_param.push(ch);
                }
            } else {
                if ch == '{' && !escaped_next {
                    param_opened = true;
                } else if ch == '\\' && !escaped_next {
                    escaped_next = true;
                } else {
                    if escaped_next && ch != '{' && ch != '}' && ch != '\\' {
                        cur_str.push('\\');
                    }
                    escaped_next = false;
                    cur_str.push(ch);
                }
            }
        }
        browser.log(cur_str).await;
        return Some(r#"{ "value0": "true" }"#.to_string());
    }

    pub async fn input(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, hex::encode(v)))
    }

    pub async fn input_address(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_address(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, v))
    }

    pub async fn input_uint256(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_uint256(msg_text).await
            .map(|v|format!(r#"{{ "value0": "0x{}" }}"#, v.to_hex_string()))
    }

    pub async fn input_pubkey(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_pubkey(msg_text).await
            .map(|v|format!(r#"{{ "value0": "0x{}" }}"#, hex::encode(&v.to_bytes())))
    }

    pub async fn input_tons(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_tons(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, v))
    }

    pub async fn input_yes_or_no(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_yes_or_no(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, v))
    }

    pub async fn input_datetime(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_datetime(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, v.timestamp()))
    }

    pub async fn input_deploy_message(
        browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        browser.input_deploy_message(msg_text).await
            .map(|v|format!(r#"{{ "value0": "{}" }}"#, v))
    }

    pub async fn input_cell(
        _browser: Arc<dyn CppBrowserCallbacks + Send + Sync>,
        _context: Arc<ClientContext>,
        msg_json: &Value
    ) -> Option<String> {
        let msg_text = get_message_text(&msg_json);
        println!("{}", msg_text);

        let val = (||
        loop {
            let mut input_text = String::new();
            io::stdin()
                .read_line(&mut input_text)
                .expect("failed to read from stdin");
            return input_text;
        })();
        return Some(format!(r#"{{ "value0": "{}" }}"#, val));
    }

    pub async fn i_am_home(
        cur_account: &String,
        last_home: &mut String
    ) -> Option<String> {
        *last_home = cur_account.clone();
        return Some(r#"{ "value0": "true" }"#.to_string());
    }
}

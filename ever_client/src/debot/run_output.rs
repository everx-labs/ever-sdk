use super::action::DAction;
use super::calltype::DebotCallType;
use super::{JsonValue, DEBOT_WC};
use crate::boc::internal::{deserialize_object_from_base64, serialize_object_to_base64};
use crate::encoding::account_decode;
use crate::error::ClientError;
use std::collections::VecDeque;
use ever_block::{Message, MsgAddressInt};

#[derive(Default)]
pub(super) struct RunOutput {
    pub account: String,
    pub return_value: Option<JsonValue>,
    pub calls: VecDeque<DebotCallType>,
    pub actions: Vec<DAction>,
    pub std_addr: Option<MsgAddressInt>,
}

impl RunOutput {
    pub fn new(
        account: String,
        debot_addr: String,
        return_value: Option<JsonValue>,
        mut msgs: Vec<String>,
    ) -> Result<Self, ClientError> {
        let mut output = RunOutput::default();
        output.account = account;
        output.return_value = return_value;
        output.std_addr = Some(account_decode(&debot_addr)?);
        while let Some(msg_base64) = msgs.pop() {
            let msg: Message = deserialize_object_from_base64(&msg_base64, "message")?.object;
            output.filter_msg(msg, msg_base64);
        }

        Ok(output)
    }
}

impl RunOutput {
    pub fn decode_actions(&self) -> Result<Option<Vec<DAction>>, String> {
        match self.return_value.as_ref() {
            Some(val) => serde_json::from_value(val["actions"].clone())
                .map_err(|_| format!("internal error: failed to parse actions"))
                .map(|v| Some(v)),
            None => Ok(None),
        }
    }

    pub fn append(&mut self, mut output: RunOutput) {
        self.calls.append(&mut output.calls);
        self.actions.append(&mut output.actions);
        self.return_value = output.return_value;
    }

    pub fn pop(&mut self) -> Option<DebotCallType> {
        self.calls.pop_front()
    }

    fn filter_msg(&mut self, msg: Message, msg_base64: String) {
        let msg = (&msg, msg_base64);
        self.filter_interface_call(msg)
            .and_then(|msg| self.filter_invoke_call(msg))
            .and_then(|msg| self.filter_external_call(msg))
            .and_then(|msg| self.filter_getmethod_call(msg));
    }

    fn filter_interface_call<'a>(
        &mut self,
        msg: (&'a Message, String),
    ) -> Option<(&'a Message, String)> {
        if msg.0.is_internal() {
            let wc_id = msg.0.workchain_id().unwrap_or(0);
            if DEBOT_WC as i32 == wc_id {
                let std_addr = msg.0.dst_ref().cloned().unwrap_or_default();
                let addr = std_addr.to_string();
                let wc_and_addr: Vec<&str> = addr.split(':').collect();

                let mut msg = msg.0.clone();
                if let Some(std_addr) = &self.std_addr {
                    msg.set_src_address(std_addr.clone());
                }
                if let Ok(msg_base64) = serialize_object_to_base64(&msg, "message") {
                    self.calls.push_back(DebotCallType::Interface {
                        msg: msg_base64,
                        id: wc_and_addr
                            .get(1)
                            .map(|x| x.to_owned())
                            .unwrap_or("0")
                            .to_string(),
                    });
                }
                return None;
            }
        }
        Some(msg)
    }

    fn filter_invoke_call<'a>(
        &mut self,
        msg: (&'a Message, String),
    ) -> Option<(&'a Message, String)> {
        if msg.0.is_internal() {
            let wc_id = msg.0.workchain_id().unwrap_or(0);
            if wc_id != DEBOT_WC as i32 {
                let mut msg = msg.0.clone();
                if let Some(std_addr) = &self.std_addr {
                    msg.set_src_address(std_addr.clone());
                }
                if let Ok(msg_base64) = serialize_object_to_base64(&msg, "message") {
                    self.calls
                        .push_back(DebotCallType::Invoke { msg: msg_base64 });
                }
                return None;
            }
        }
        Some(msg)
    }

    fn filter_external_call<'a>(
        &mut self,
        msg: (&'a Message, String),
    ) -> Option<(&'a Message, String)> {
        self.filter_external_inbound_msg(msg, true)
    }

    fn filter_getmethod_call<'a>(
        &mut self,
        msg: (&'a Message, String),
    ) -> Option<(&'a Message, String)> {
        self.filter_external_inbound_msg(msg, false)
    }

    fn filter_external_inbound_msg<'a>(
        &mut self,
        msg: (&'a Message, String),
        call_or_get: bool,
    ) -> Option<(&'a Message, String)> {
        if msg.0.is_inbound_external() {
            if let Some(body_slice) = msg.0.body() {
                // TODO: currently using an unreliable way to
                // distinguish a get-method call from external call.
                // Most accurate method - to check flags in src address.
                let mut body_slice = body_slice.clone();
                let dest = msg
                    .0
                    .header()
                    .get_dst_address()
                    .map(|x| x.to_string())
                    .unwrap_or_default();
                if let Ok(bit) = body_slice.get_next_bit() {
                    if call_or_get && bit {
                        self.calls
                            .push_back(DebotCallType::External { msg: msg.1, dest });
                        return None;
                    } else if !call_or_get && !bit {
                        self.calls
                            .push_back(DebotCallType::GetMethod { msg: msg.1, dest });
                        return None;
                    }
                }
            }
        }
        Some(msg)
    }
}

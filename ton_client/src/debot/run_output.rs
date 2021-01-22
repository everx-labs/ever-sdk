use super::action::DAction;
use super::{JsonValue, DEBOT_WC};
use crate::boc::internal::deserialize_object_from_base64;
use crate::error::ClientError;
use ton_block::Message;

#[derive(Default)]
pub(super) struct RunOutput {
    pub account: String,
    pub return_value: Option<JsonValue>,
    pub interface_calls: Vec<(String, String)>,
    pub external_calls: Vec<String>,
    pub get_method_calls: Vec<String>,
    pub debot_invokes: Vec<String>,
    pub actions: Vec<DAction>,
}

impl RunOutput {
    pub fn new(
        account: String,
        return_value: Option<JsonValue>,
        mut msgs: Vec<String>,
    ) -> Result<Self, ClientError> {
        let mut output = RunOutput::default();
        output.account = account;
        output.return_value = return_value;

        while let Some(msg_base64) = msgs.pop() {
            let msg: Message = deserialize_object_from_base64(&msg_base64, "message")?.object;
            output.filter_msg(msg, msg_base64);
        }

        Ok(output)
    }
}

impl RunOutput {
    fn filter_msg(&mut self, msg: Message, msg_base64: String) {
        let msg = (&msg, msg_base64);
        self.filter_interface_call(msg)
            .and_then(|msg| self.filter_external_call(msg))
            .and_then(|msg| self.filter_getmethod_call(msg));
    }

    pub fn decode_actions(&self) -> Result<Option<Vec<DAction>>, String> {
        match self.return_value.as_ref() {
            Some(val) => serde_json::from_value(val["actions"].clone())
                .map_err(|_| format!("internal error: failed to parse actions"))
                .map(|v| Some(v)),
            None => Ok(None),
        }
    }

    fn filter_interface_call<'a>(
        &mut self,
        msg: (&'a Message, String),
    ) -> Option<(&'a Message, String)> {
        if msg.0.is_internal() {
            let wc_id = msg.0.workchain_id().unwrap();
            if DEBOT_WC as i32 == wc_id {
                let account_id = msg.0.int_dst_account_id().unwrap();
                self.interface_calls
                    .push((msg.1.clone(), account_id.to_string()));
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
                // distinguish a get method call from external call.
                // Most accurate method - to check flags in src address.
                let mut body_slice = body_slice.clone();
                if let Ok(bit) = body_slice.get_next_bit() {
                    if call_or_get && bit {
                        self.external_calls.push(msg.1);
                        return None;
                    } else if !call_or_get && !bit {
                        self.get_method_calls.push(msg.1);
                        return None;
                    }
                }
            }
        }
        Some(msg)
    }
}

use super::errors::Error;
use super::helpers::build_internal_message;
use super::TonClient;
use crate::boc::internal::{deserialize_object_from_base64, serialize_object_to_base64};
use crate::error::{ClientResult};
use crate::tvm::{run_tvm, ParamsOfRunTvm};
use ton_abi::Contract;
use ton_block::Message;
use ton_types::{BuilderData, IBitstring};

pub(super) struct GetMethod {}

impl GetMethod {
    pub async fn run(
        ton: TonClient,
        msg: String,
        target_state: String,
        debot_abi: &String,
        debot_addr: &String,
    ) -> ClientResult<String> {
        let debot_abi = Contract::load(debot_abi.as_bytes())
            .map_err(|e| Error::invalid_debot_abi(e.to_string()))?;
        let mut message: Message = deserialize_object_from_base64(&msg, "message")?.object;
        let dest = message
            .header()
            .get_dst_address()
            .map(|x| x.to_string())
            .unwrap_or_default();
        let mut in_body_slice = message.body().ok_or(Error::invalid_msg("empty body"))?;
        let mut pubkey_bit_present = false;
        // skip signature bit (must be 0)
        in_body_slice.get_next_bit().unwrap();
        let slice_clone = in_body_slice.clone();
        // skip timestamp in miliseconds
        let mut timestamp = in_body_slice.get_next_u64().unwrap();
        // `expire` is a function id
        let mut answer_id = in_body_slice.get_next_u32().unwrap();
        // remember function id
        let mut func_id = in_body_slice.get_next_u32().unwrap();

        let result = debot_abi
            .function_by_id(func_id, true)
            .map_err(|e| Error::invalid_function_id(e));
        if result.is_err() {
            println!("WARNING function with answer id not found. 2nd try.");
            in_body_slice = slice_clone;
            // skip pubkey bit (must be 0)
            in_body_slice.get_next_bit().unwrap();
            pubkey_bit_present = true;
            timestamp = in_body_slice.get_next_u64().unwrap();
            answer_id = in_body_slice.get_next_u32().unwrap();
            func_id = in_body_slice.get_next_u32().unwrap();

            debot_abi.function_by_id(func_id, true).map_err(|e| {
                println!("FAIL func_id not found");
                Error::invalid_function_id(e)
            })?;
        }

        // rebuild msg body - insert correct `expire` header instead of answerId
        let mut new_body = BuilderData::new();
        // signature bit = 0
        new_body.append_bit_zero().unwrap();
        if pubkey_bit_present {
            // pubkey bit = 0
            new_body.append_bit_zero().unwrap();
        }
        new_body
            .append_u64(timestamp).unwrap()
            .append_u32(((timestamp >> 32) as u32) + 100).unwrap()
            .append_u32(func_id).unwrap()
            .append_builder(&BuilderData::from_slice(&in_body_slice))
            .unwrap();

        message.set_body(new_body.into());

        let mut result = run_tvm(
            ton.clone(),
            ParamsOfRunTvm {
                account: target_state,
                message: serialize_object_to_base64(&message, "message")?,
                abi: None,
                execution_options: None,
            },
        )
        .await?;

        if result.out_messages.len() != 1 {
            return Err(Error::execute_failed(
                "get-metod returns more than 1 message",
            ));
        }
        let out_msg = result.out_messages.pop().unwrap();
        let out_message: Message = deserialize_object_from_base64(&out_msg, "message")?.object;
        let mut out_body = out_message.body();
        let mut new_body = BuilderData::new();
        new_body.append_u32(answer_id).unwrap();

        if let Some(body_slice) = out_body.as_mut() {
            let response_id = body_slice.get_next_u32().unwrap();
            let request_id = response_id & !(1u32 << 31);
            if func_id != request_id {
                return Err(Error::invalid_msg(
                    "get-method returns msg with incorrect response id",
                ));
            }
            new_body
                .append_builder(&BuilderData::from_slice(&body_slice))
                .unwrap();
        }

        build_internal_message(&dest, &debot_addr, new_body.into())
    }
}

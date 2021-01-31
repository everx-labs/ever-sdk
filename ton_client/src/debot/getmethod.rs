use super::TonClient;
use crate::boc::internal::{deserialize_cell_from_base64};
use ton_types::{UInt256, BuilderData, SliceData, IBitstring, Cell};
use crate::error::{ClientError, ClientResult};
use crate::boc::{parse_message, ParamsOfParse};
use crate::tvm::{run_tvm, ParamsOfRunTvm};
use super::errors::Error;

pub(super) struct GetMethod {
    client: TonClient,
    dest: String,
    in_body: String,
    msg: String,
}

impl GetMethod {
    pub fn new(client: TonClient, msg: String) -> ClientResult<Self> {
        let (dest, in_body) = Self::parse_dest_and_body(client.clone(), msg)?;
        Ok(Self {client, dest, in_body, msg})
    }

    pub fn dest(&self) -> &String {
        &self.dest
    }

    pub async fn run(&self, smc: String) -> ClientResult<SliceData> {
        let (_, in_body_cell) = deserialize_cell_from_base64(&self.in_body, "message body")?;
        let mut in_body_slice: SliceData = in_body_cell.into();
        
        let mut result = run_tvm(
            self.client.clone(),
            ParamsOfRunTvm {
                account: smc,
                message: self.msg.clone(),
                abi: None,
                execution_options: None,
            },
        ).await?;

        if result.out_messages.len() != 1 {
            return Err(Error::execute_failed("target contract returns more than 1 message"));
        }
        let out_msg = result.out_messages.pop().unwrap();
        
        let (_, out_body) = Self::parse_dest_and_body(self.client.clone(), out_msg)?;

        let (_, out_body_cell) = deserialize_cell_from_base64(&out_body, "message body")?;
        let mut out_body_slice: SliceData = out_body_cell.into();
        let request_id = out_body_slice.get_next_u32().unwrap() & !(1u32 << 31);
        
        // skip signature bit (must be 0)
        in_body_slice.get_next_bit().unwrap();
        let slice_clone = in_body_slice.clone();
        // skip timestamp in miliseconds
        in_body_slice.get_next_u64().unwrap();
        // `expire` is a function id
        let mut answer_id = in_body_slice.get_next_u32().unwrap();
        let mut func_id = in_body_slice.get_next_u32().unwrap();

        if func_id != request_id {
            println!("WARNING func_id != request_id");
            in_body_slice = slice_clone;
            // skip pubkey bit (must be 0)
            in_body_slice.get_next_bit().unwrap();
            in_body_slice.get_next_u64().unwrap();
            answer_id = in_body_slice.get_next_u32().unwrap();
            func_id = in_body_slice.get_next_u32().unwrap();
            println!("FAIL func_id != request_id");
            assert_eq!(func_id, request_id);
        }

        let mut new_body = BuilderData::new();
        new_body.append_u32(answer_id).unwrap();
        new_body.append_builder(&BuilderData::from_slice(&out_body_slice)).unwrap();

        Ok(new_body.into())
    }


    fn parse_dest_and_body(client: TonClient, boc: String) ->ClientResult<(String, String)> {
        let parsed_msg = parse_message(
            client.clone(),
            ParamsOfParse { boc },
        )?.parsed;
        let dest_addr = parsed_msg["dst"].as_str().unwrap().to_string();
        let body = parsed_msg["body"].as_str().unwrap().to_string();
        Ok((dest_addr, body))
    }
}



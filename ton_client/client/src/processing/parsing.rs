use crate::abi::{decode_message, Abi, DecodedMessageType, ParamsOfDecodeMessage};
use crate::boc::{parse_message, parse_transaction, ParamsOfParse};
use crate::client::ClientContext;
use crate::error::ApiResult;
use crate::processing::fetching::TransactionBoc;
use crate::processing::types::DecodedOutput;
use serde_json::Value;
use std::sync::Arc;

pub(crate) fn parse_transaction_boc(
    context: Arc<ClientContext>,
    transaction: &TransactionBoc,
) -> ApiResult<(Value, Vec<Value>)> {
    let mut parsed_messages = Vec::<Value>::new();
    for message in &transaction.out_messages {
        parsed_messages.push(
            parse_message(
                context.clone(),
                ParamsOfParse {
                    boc: message.boc.clone(),
                },
            )?
            .parsed,
        );
    }
    Ok((
        parse_transaction(
            context,
            ParamsOfParse {
                boc: transaction.boc.clone(),
            },
        )?
        .parsed,
        parsed_messages,
    ))
}

pub(crate) fn decode_output(
    context: &Arc<ClientContext>,
    abi: &Abi,
    parsed_messages: &Vec<Value>,
) -> ApiResult<DecodedOutput> {
    let mut out_messages = Vec::new();
    let mut output = None;
    for parsed_message in parsed_messages {
        let decoded = match &parsed_message["boc"] {
            Value::String(boc) => match decode_message(
                context.clone(),
                ParamsOfDecodeMessage {
                    message: boc.clone(),
                    abi: abi.clone(),
                },
            ) {
                Ok(decoded) => {
                    if decoded.message_type == DecodedMessageType::FunctionOutput {
                        output = Some(decoded.value.clone());
                    }
                    Some(decoded)
                }
                _ => None,
            },
            _ => None,
        };
        out_messages.push(decoded);
    }
    Ok(DecodedOutput {
        out_messages,
        output,
    })
}

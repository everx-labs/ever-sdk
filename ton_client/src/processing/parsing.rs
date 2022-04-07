use crate::abi::{Abi, ParamsOfDecodeMessage, DecodedMessageBody};
use crate::boc::{parse_transaction, ParamsOfParse};
use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::processing::fetching::TransactionBoc;
use crate::processing::types::DecodedOutput;
use serde_json::Value;
use std::sync::Arc;
use crate::abi::decode_message::ResponsibleCall;

pub(crate) async fn parse_transaction_boc(
    context: Arc<ClientContext>,
    transaction: TransactionBoc,
) -> ClientResult<(Value, Vec<String>)> {
    let mut messages = Vec::new();
    for message in transaction.out_messages {
        messages.push(message.boc);
    }
    Ok((
        parse_transaction(
            context,
            ParamsOfParse {
                boc: transaction.boc,
            },
        )
        .await?
        .parsed,
        messages,
    ))
}

pub(crate) async fn decode_output<'a>(
    context: &Arc<ClientContext>,
    abi: &Abi,
    messages: Vec<String>,
    responsible: Option<&ResponsibleCall<'a>>,
) -> ClientResult<DecodedOutput> {
    let mut out_messages = Vec::new();
    let mut output = None;
    for message in messages {
        let decode_result = DecodedMessageBody::decode_message(
            context.clone(),
            ParamsOfDecodeMessage {
                message,
                abi: abi.clone(),
                allow_partial: false,
            },
            responsible
        ).await;
        let decoded = match decode_result {
            Ok(decoded) => {
                if decoded.body_type.is_output() {
                    output = decoded.value.clone();
                }
                Some(decoded)
            }
            Err(_) => None,
        };
        out_messages.push(decoded);
    }
    Ok(DecodedOutput {
        out_messages,
        output,
    })
}

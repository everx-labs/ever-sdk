use crate::abi::ParamsOfEncodeMessage;
use crate::processing::types::{CallbackParams,  ProcessingOptions, TransactionResult};

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub enum MessageSource {
    Message(String),
    AbiEncoding(ParamsOfEncodeMessage),
}

#[derive(Serialize, Deserialize, TypeInfo, Debug)]
pub struct ParamsOfProcessMessage {
    /// Message source.
    pub message: MessageSource,
    /// Processing options.
    pub processing_options: Option<ProcessingOptions>,
    /// Processing callback.
    pub callback: Option<CallbackParams>,
}

#[derive(Serialize, Deserialize, TypeInfo, PartialEq, Debug)]
pub struct ResultOfProcessMessage {
    pub transaction: Option<TransactionResult>,
}

// fn ensure_message(
//     context: &Arc<ClientContext>,
//     source: &MessageSource,
//     retry_count: u32,
//     callback: &Option<CallbackParams>,
// ) -> ApiResult<(String, Option<u32>)> {
//     Ok(match source {
//         MessageSource::Message(boc) => (boc.clone(), None),
//         MessageSource::AbiEncoding(encode_params) => {
//             emit_event(context, callback, || MessageProcessingEvent::EncodeMessage);
//             let encoded = crate::abi::encode_message(context.clone(), encode_params.clone())?;
//             (encoded.message, None)
//         }
//     })
// }
//
// pub async fn process_message(
//     context: Arc<ClientContext>,
//     params: ParamsOfProcessMessage,
// ) -> ApiResult<ResultOfProcessMessage> {
//     let mut retry_count = 0;
//     loop {
//         let (message, expiration_time) =
//             ensure_message(&context, &params.message, retry_count, &params.callback)?;
//         let transaction_waiting_state = send_message(
//             context.clone(),
//             ParamsOfSendMessage {
//                 message: message.clone(),
//                 message_expiration_time: None,
//                 callback: params.callback.clone(),
//             },
//         )
//         .await?
//         .transaction_waiting_state;
//
//         let result = wait_for_transaction(
//             context.clone(),
//             ParamsOfWaitForTransaction {
//                 message: message.clone(),
//                 message_expiration_time: expiration_time,
//                 callback: params.callback.clone(),
//                 waiting_state: transaction_waiting_state,
//             },
//         )
//         .await?;
//         match result {
//             ResultOfWaitForTransaction::Complete(transaction) => {
//                 emit_event(&context, &params.callback, || {
//                     MessageProcessingEvent::TransactionReceived {
//                         transaction: transaction.clone(),
//                     }
//                 });
//                 return Ok(ResultOfProcessMessage { transaction });
//             }
//
//             ResultOfWaitForTransaction::Incomplete(waiting_state) => {}
//         }
//         retry_count += 1;
//     }
// }

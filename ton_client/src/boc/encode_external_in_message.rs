use crate::boc::internal::{
    deserialize_cell_from_boc, deserialize_object_from_boc, serialize_object_to_boc,
};
use crate::boc::BocCacheType;
use crate::client::ClientContext;
use crate::encoding::{account_decode, slice_from_cell};
use crate::error::ClientResult;
use std::str::FromStr;
use ton_block::{ExternalInboundMessageHeader, GetRepresentationHash, MsgAddressExt, StateInit};

#[derive(Serialize, Deserialize, Clone, Debug, ApiType, Default)]
pub struct ParamsOfEncodeExternalInMessage {
    /// Source address.
    pub src: Option<String>,

    /// Destination address.
    pub dst: String,

    /// Bag of cells with state init (used in deploy messages).
    pub init: Option<String>,

    /// Bag of cells with the message body encoded as base64.
    pub body: Option<String>,

    /// Cache type to put the result. The BOC itself returned if no cache type provided
    pub boc_cache: Option<BocCacheType>,
}

#[derive(Serialize, Deserialize, ApiType, Default)]
pub struct ResultOfEncodeExternalInMessage {
    /// Message BOC encoded with `base64`.
    pub message: String,

    /// Message id.
    pub message_id: String,
}

/// Encodes a message
///
/// Allows to encode any external inbound message.
///
#[api_function]
pub fn encode_external_in_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfEncodeExternalInMessage,
) -> ClientResult<ResultOfEncodeExternalInMessage> {
    let src = params.src.clone();
    let header = ExternalInboundMessageHeader {
        dst: account_decode(&params.dst)?,
        src: src
            .as_ref()
            .map(|x| MsgAddressExt::from_str(x.as_str()))
            .unwrap_or_else(|| Ok(MsgAddressExt::AddrNone))
            .map_err(|err| {
                crate::client::errors::Error::invalid_address(
                    &err.to_string(),
                    &src.unwrap_or_default(),
                )
            })?,
        ..Default::default()
    };

    let mut msg = ton_block::Message::with_ext_in_header(header);
    if let Some(init) = params.init {
        msg.set_state_init(
            deserialize_object_from_boc::<StateInit>(&context, &init, "state init")?.object,
        );
    }
    if let Some(body) = params.body {
        let (_, cell) = deserialize_cell_from_boc(&context, &body, "message body")?;
        msg.set_body(slice_from_cell(cell)?);
    }

    let hash = msg
        .hash()
        .map_err(|err| crate::client::errors::Error::internal_error(err))?;
    let boc = serialize_object_to_boc(&context, &msg, "message", params.boc_cache)?;
    Ok(ResultOfEncodeExternalInMessage {
        message: boc,
        message_id: hex::encode(hash),
    })
}

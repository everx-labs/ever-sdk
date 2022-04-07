use crate::abi::DecodedMessageBody;
use crate::error::ClientError;
use serde_json::Value;
use ton_sdk::TransactionFees;

#[derive(Serialize, Deserialize, ApiType, Default, Debug, PartialEq, Clone)]
pub struct DecodedOutput {
    /// Decoded bodies of the out messages.
    ///
    /// If the message can't be decoded, then `None` will be stored in
    /// the appropriate position.
    pub out_messages: Vec<Option<DecodedMessageBody>>,

    /// Decoded body of the function output message.
    pub output: Option<Value>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Debug, PartialEq, Clone)]
pub struct ResultOfProcessMessage {
    /// Parsed transaction.
    ///
    /// In addition to the regular transaction fields there is a
    /// `boc` field encoded with `base64` which contains source
    /// transaction BOC.
    pub transaction: Value,

    /// List of output messages' BOCs. Encoded as `base64`
    pub out_messages: Vec<String>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// Transaction fees
    pub fees: TransactionFees,
}

#[derive(Clone, num_derive::FromPrimitive, PartialEq, Debug)]
pub enum ProcessingResponseType {
    ProcessingEvent = 100,
}

#[derive(Serialize, Deserialize, ApiType, Debug, Clone)]
#[serde(tag = "type")]
pub enum ProcessingEvent {
    /// Notifies the application that the account's current shard block will be fetched
    /// from the network.
    /// This step is performed before the message sending so that sdk knows starting
    /// from which block it will search for the transaction.
    ///
    /// Fetched block will be used later in waiting phase.
    WillFetchFirstBlock {},

    /// Notifies the app that the client has failed to fetch the account's current
    /// shard block. This may happen due to the network issues.
    /// Receiving this event means that message processing will not proceed -
    /// message was not sent, and Developer can try to run `process_message` again,
    /// in the hope that the connection is restored.
    FetchFirstBlockFailed { error: ClientError },

    /// Notifies the app that the message will be sent to the network.
    /// This event means that the account's current shard block was successfully fetched
    /// and the message was successfully created (`abi.encode_message` function was executed successfully).
    WillSend {
        shard_block_id: String,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the message was sent to the network, i.e `processing.send_message` was successfuly executed.
    /// Now, the message is in the blockchain.
    /// If Application exits at this phase, Developer needs to proceed with processing
    /// after the application is restored with `wait_for_transaction` function, passing
    /// shard_block_id and message from this event. Do not forget to specify abi of your contract
    /// as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.
    DidSend {
        shard_block_id: String,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the sending operation was failed with
    /// network error.
    ///
    /// Nevertheless the processing will be continued at the waiting
    /// phase because the message possibly has been delivered to the
    /// node.
    /// If Application exits at this phase, Developer needs to proceed with processing
    /// after the application is restored with `wait_for_transaction` function, passing
    /// shard_block_id and message from this event. Do not forget to specify abi of your contract
    /// as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.
    SendFailed {
        shard_block_id: String,
        message_id: String,
        message: String,
        error: ClientError,
    },

    /// Notifies the app that the next shard block will be fetched
    /// from the network.
    ///
    /// Event can occurs more than one time due to block walking
    /// procedure.
    /// If Application exits at this phase, Developer needs to proceed with processing
    /// after the application is restored with `wait_for_transaction` function, passing
    /// shard_block_id and message from this event. Do not forget to specify abi of your contract
    /// as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.
    WillFetchNextBlock {
        shard_block_id: String,
        message_id: String,
        message: String,
    },

    /// Notifies the app that the next block can't be fetched.
    ///
    /// If no block was fetched within `NetworkConfig.wait_for_timeout` then processing stops.
    /// This may happen when the shard stops, or there are other network issues.
    /// In this case Developer should resume message processing with `wait_for_transaction`, passing shard_block_id,
    /// message and contract abi to it. Note that passing ABI is crucial, because it will influence the processing strategy.
    ///
    /// Another way to tune this is to specify long timeout in `NetworkConfig.wait_for_timeout`
    FetchNextBlockFailed {
        shard_block_id: String,
        message_id: String,
        message: String,
        error: ClientError,
    },

    /// Notifies the app that the message was not executed within expire timeout on-chain and will
    /// never be because it is already expired.
    /// The expiration timeout can be configured with `AbiConfig` parameters.
    ///
    /// This event occurs only for the contracts which ABI includes "expire" header.
    ///
    /// If Application specifies `NetworkConfig.message_retries_count` > 0, then `process_message`
    /// will perform retries: will create a new message and send it again and repeat it untill it reaches
    /// the maximum retries count or receives a successful result.  All the processing
    /// events will be repeated.
    MessageExpired {
        message_id: String,
        message: String,
        error: ClientError,
    },
}

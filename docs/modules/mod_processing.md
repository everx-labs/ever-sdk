# Module processing

## Module processing

Message processing module.

This module incorporates functions related to complex message processing scenarios.

### Functions

[send_message](mod_processing.md#send_message) – Sends message to the network

[wait_for_transaction](mod_processing.md#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](mod_processing.md#process_message) – Creates message, sends it to the network and monitors its processing.

### Types

[ProcessingErrorCode](mod_processing.md#ProcessingErrorCode)

[ProcessingEvent](mod_processing.md#ProcessingEvent)

[ResultOfProcessMessage](mod_processing.md#ResultOfProcessMessage)

[DecodedOutput](mod_processing.md#DecodedOutput)

[ParamsOfSendMessage](mod_processing.md#ParamsOfSendMessage)

[ResultOfSendMessage](mod_processing.md#ResultOfSendMessage)

[ParamsOfWaitForTransaction](mod_processing.md#ParamsOfWaitForTransaction)

[ParamsOfProcessMessage](mod_processing.md#ParamsOfProcessMessage)

## Functions

### send_message

Sends message to the network

Sends message to the network and returns the last generated shard block of the destination account before the message was sent. It will be required later for message processing.

```typescript
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
}

type ResultOfSendMessage = {
    shard_block_id: string,
    sending_endpoints: string[]
}

function send_message(
    params: ParamsOfSendMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfSendMessage>;
```

#### Parameters

* `message`: _string_ – Message BOC.
*   `abi`?: [_Abi_](mod_abi.md#Abi) – Optional message ABI.

    \
    If this parameter is specified and the message has the\
    `expire` header then expiration time will be checked against\
    the current time to prevent unnecessary sending of already expired message.\
    \
    The `message already expired` error will be returned in this\
    case.\
    \
    Note, that specifying `abi` for ABI compliant contracts is\
    strongly recommended, so that proper processing strategy can be\
    chosen.
* `send_events`: _boolean_ – Flag for requesting events sending
* `responseHandler`?: [_ResponseHandler_](./#ResponseHandler) – additional responses handler.

#### Result

*   `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.

    \
    This block id must be used as a parameter of the\
    `wait_for_transaction`.
*   `sending_endpoints`: _string\[]_ – The list of endpoints to which the message was sent.

    \
    This list id must be used as a parameter of the\
    `wait_for_transaction`.

### wait_for_transaction

Performs monitoring of the network for the result transaction of the external inbound message processing.

`send_events` enables intermediate events, such as `WillFetchNextBlock`, `FetchNextBlockFailed` that may be useful for logging of new shard blocks creation during message processing.

Note, that presence of the `abi` parameter is critical for ABI compliant contracts. Message processing uses drastically different strategy for processing message for contracts which ABI includes "expire" header.

When the ABI header `expire` is present, the processing uses `message expiration` strategy:

*   The maximum block gen time is set to

    `message_expiration_timeout + transaction_wait_timeout`.
*   When maximum block gen time is reached, the processing will

    be finished with `MessageExpired` error.

When the ABI header `expire` isn't present or `abi` parameter isn't specified, the processing uses `transaction waiting` strategy:

* The maximum block gen time is set to `now() + transaction_wait_timeout`.
* If maximum block gen time is reached and no result transaction is found, the processing will exit with an error.

```typescript
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean,
    sending_endpoints?: string[]
}

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}

function wait_for_transaction(
    params: ParamsOfWaitForTransaction,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```

#### Parameters

*   `abi`?: [_Abi_](mod_abi.md#Abi) – Optional ABI for decoding the transaction result.

    \
    If it is specified, then the output messages' bodies will be\
    decoded according to this ABI.\
    \
    The `abi_decoded` result field will be filled out.
*   `message`: _string_ – Message BOC.

    \
    Encoded with `base64`.
*   `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.

    \
    You must provide the same value as the `send_message` has returned.
* `send_events`: _boolean_ – Flag that enables/disables intermediate events
*   `sending_endpoints`?: _string\[]_ – The list of endpoints to which the message was sent.

    \
    Use this field to get more informative errors.\
    Provide the same value as the `send_message` has returned.\
    If the message was not delivered (expired), SDK will log the endpoint URLs, used for its sending.
* `responseHandler`?: [_ResponseHandler_](./#ResponseHandler) – additional responses handler.

#### Result

*   `transaction`: _any_ – Parsed transaction.

    \
    In addition to the regular transaction fields there is a\
    `boc` field encoded with `base64` which contains source\
    transaction BOC.
*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
* `fees`: [_TransactionFees_](mod_tvm.md#TransactionFees) – Transaction fees

### process_message

Creates message, sends it to the network and monitors its processing.

Creates ABI-compatible message, sends it to the network and monitors for the result transaction. Decodes the output messages' bodies.

If contract's ABI includes "expire" header, then SDK implements retries in case of unsuccessful message delivery within the expiration timeout: SDK recreates the message, sends it and processes it again.

The intermediate events, such as `WillFetchFirstBlock`, `WillSend`, `DidSend`, `WillFetchNextBlock`, etc - are switched on/off by `send_events` flag and logged into the supplied callback function.

The retry configuration parameters are defined in the client's `NetworkConfig` and `AbiConfig`.

If contract's ABI does not include "expire" header then, if no transaction is found within the network timeout (see config parameter ), exits with error.

```typescript
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
}

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}

function process_message(
    params: ParamsOfProcessMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```

#### Parameters

* `message_encode_params`: [_ParamsOfEncodeMessage_](mod_abi.md#ParamsOfEncodeMessage) – Message encode parameters.
* `send_events`: _boolean_ – Flag for requesting events sending
* `responseHandler`?: [_ResponseHandler_](./#ResponseHandler) – additional responses handler.

#### Result

*   `transaction`: _any_ – Parsed transaction.

    \
    In addition to the regular transaction fields there is a\
    `boc` field encoded with `base64` which contains source\
    transaction BOC.
*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
* `fees`: [_TransactionFees_](mod_tvm.md#TransactionFees) – Transaction fees

## Types

### ProcessingErrorCode

```typescript
enum ProcessingErrorCode {
    MessageAlreadyExpired = 501,
    MessageHasNotDestinationAddress = 502,
    CanNotBuildMessageCell = 503,
    FetchBlockFailed = 504,
    SendMessageFailed = 505,
    InvalidMessageBoc = 506,
    MessageExpired = 507,
    TransactionWaitTimeout = 508,
    InvalidBlockReceived = 509,
    CanNotCheckBlockShard = 510,
    BlockNotFound = 511,
    InvalidData = 512,
    ExternalSignerMustNotBeUsed = 513
}
```

One of the following value:

* `MessageAlreadyExpired = 501`
* `MessageHasNotDestinationAddress = 502`
* `CanNotBuildMessageCell = 503`
* `FetchBlockFailed = 504`
* `SendMessageFailed = 505`
* `InvalidMessageBoc = 506`
* `MessageExpired = 507`
* `TransactionWaitTimeout = 508`
* `InvalidBlockReceived = 509`
* `CanNotCheckBlockShard = 510`
* `BlockNotFound = 511`
* `InvalidData = 512`
* `ExternalSignerMustNotBeUsed = 513`

### ProcessingEvent

```typescript
type ProcessingEvent = {
    type: 'WillFetchFirstBlock'
} | {
    type: 'FetchFirstBlockFailed'
    error: ClientError
} | {
    type: 'WillSend'
    shard_block_id: string,
    message_id: string,
    message: string
} | {
    type: 'DidSend'
    shard_block_id: string,
    message_id: string,
    message: string
} | {
    type: 'SendFailed'
    shard_block_id: string,
    message_id: string,
    message: string,
    error: ClientError
} | {
    type: 'WillFetchNextBlock'
    shard_block_id: string,
    message_id: string,
    message: string
} | {
    type: 'FetchNextBlockFailed'
    shard_block_id: string,
    message_id: string,
    message: string,
    error: ClientError
} | {
    type: 'MessageExpired'
    message_id: string,
    message: string,
    error: ClientError
}
```

Depends on value of the `type` field.

When _type_ is _'WillFetchFirstBlock'_

Notifies the application that the account's current shard block will be fetched from the network. This step is performed before the message sending so that sdk knows starting from which block it will search for the transaction.

Fetched block will be used later in waiting phase.

When _type_ is _'FetchFirstBlockFailed'_

Notifies the app that the client has failed to fetch the account's current shard block.

This may happen due to the network issues. Receiving this event means that message processing will not proceed - message was not sent, and Developer can try to run `process_message` again, in the hope that the connection is restored.

* `error`: [_ClientError_](mod_client.md#ClientError)

When _type_ is _'WillSend'_

Notifies the app that the message will be sent to the network. This event means that the account's current shard block was successfully fetched and the message was successfully created (`abi.encode_message` function was executed successfully).

* `shard_block_id`: _string_
* `message_id`: _string_
* `message`: _string_

When _type_ is _'DidSend'_

Notifies the app that the message was sent to the network, i.e `processing.send_message` was successfuly executed. Now, the message is in the blockchain. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event.

Do not forget to specify abi of your contract as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.

* `shard_block_id`: _string_
* `message_id`: _string_
* `message`: _string_

When _type_ is _'SendFailed'_

Notifies the app that the sending operation was failed with network error.

Nevertheless the processing will be continued at the waiting phase because the message possibly has been delivered to the node. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event. Do not forget to specify abi of your contract as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.

* `shard_block_id`: _string_
* `message_id`: _string_
* `message`: _string_
* `error`: [_ClientError_](mod_client.md#ClientError)

When _type_ is _'WillFetchNextBlock'_

Notifies the app that the next shard block will be fetched from the network.

Event can occurs more than one time due to block walking procedure. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event. Do not forget to specify abi of your contract as well, it is crucial for proccessing. See `processing.wait_for_transaction` documentation.

* `shard_block_id`: _string_
* `message_id`: _string_
* `message`: _string_

When _type_ is _'FetchNextBlockFailed'_

Notifies the app that the next block can't be fetched.

If no block was fetched within `NetworkConfig.wait_for_timeout` then processing stops. This may happen when the shard stops, or there are other network issues. In this case Developer should resume message processing with `wait_for_transaction`, passing shard_block_id, message and contract abi to it. Note that passing ABI is crucial, because it will influence the processing strategy.

Another way to tune this is to specify long timeout in `NetworkConfig.wait_for_timeout`

* `shard_block_id`: _string_
* `message_id`: _string_
* `message`: _string_
* `error`: [_ClientError_](mod_client.md#ClientError)

When _type_ is _'MessageExpired'_

Notifies the app that the message was not executed within expire timeout on-chain and will never be because it is already expired. The expiration timeout can be configured with `AbiConfig` parameters.

This event occurs only for the contracts which ABI includes "expire" header.

If Application specifies `NetworkConfig.message_retries_count` > 0, then `process_message` will perform retries: will create a new message and send it again and repeat it untill it reaches the maximum retries count or receives a successful result. All the processing events will be repeated.

* `message_id`: _string_
* `message`: _string_
* `error`: [_ClientError_](mod_client.md#ClientError)

Variant constructors:

```typescript
function processingEventWillFetchFirstBlock(): ProcessingEvent;
function processingEventFetchFirstBlockFailed(error: ClientError): ProcessingEvent;
function processingEventWillSend(shard_block_id: string, message_id: string, message: string): ProcessingEvent;
function processingEventDidSend(shard_block_id: string, message_id: string, message: string): ProcessingEvent;
function processingEventSendFailed(shard_block_id: string, message_id: string, message: string, error: ClientError): ProcessingEvent;
function processingEventWillFetchNextBlock(shard_block_id: string, message_id: string, message: string): ProcessingEvent;
function processingEventFetchNextBlockFailed(shard_block_id: string, message_id: string, message: string, error: ClientError): ProcessingEvent;
function processingEventMessageExpired(message_id: string, message: string, error: ClientError): ProcessingEvent;
```

### ResultOfProcessMessage

```typescript
type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}
```

*   `transaction`: _any_ – Parsed transaction.

    \
    In addition to the regular transaction fields there is a\
    `boc` field encoded with `base64` which contains source\
    transaction BOC.
*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
* `fees`: [_TransactionFees_](mod_tvm.md#TransactionFees) – Transaction fees

### DecodedOutput

```typescript
type DecodedOutput = {
    out_messages: DecodedMessageBody | null[],
    output?: any
}
```

*   `out_messages`: [_DecodedMessageBody_](mod_abi.md#DecodedMessageBody)_?\[]_ – Decoded bodies of the out messages.

    \
    If the message can't be decoded, then `None` will be stored in\
    the appropriate position.
* `output`?: _any_ – Decoded body of the function output message.

### ParamsOfSendMessage

```typescript
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
}
```

* `message`: _string_ – Message BOC.
*   `abi`?: [_Abi_](mod_abi.md#Abi) – Optional message ABI.

    \
    If this parameter is specified and the message has the\
    `expire` header then expiration time will be checked against\
    the current time to prevent unnecessary sending of already expired message.\
    \
    The `message already expired` error will be returned in this\
    case.\
    \
    Note, that specifying `abi` for ABI compliant contracts is\
    strongly recommended, so that proper processing strategy can be\
    chosen.
* `send_events`: _boolean_ – Flag for requesting events sending

### ResultOfSendMessage

```typescript
type ResultOfSendMessage = {
    shard_block_id: string,
    sending_endpoints: string[]
}
```

*   `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.

    \
    This block id must be used as a parameter of the\
    `wait_for_transaction`.
*   `sending_endpoints`: _string\[]_ – The list of endpoints to which the message was sent.

    \
    This list id must be used as a parameter of the\
    `wait_for_transaction`.

### ParamsOfWaitForTransaction

```typescript
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean,
    sending_endpoints?: string[]
}
```

*   `abi`?: [_Abi_](mod_abi.md#Abi) – Optional ABI for decoding the transaction result.

    \
    If it is specified, then the output messages' bodies will be\
    decoded according to this ABI.\
    \
    The `abi_decoded` result field will be filled out.
*   `message`: _string_ – Message BOC.

    \
    Encoded with `base64`.
*   `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.

    \
    You must provide the same value as the `send_message` has returned.
* `send_events`: _boolean_ – Flag that enables/disables intermediate events
*   `sending_endpoints`?: _string\[]_ – The list of endpoints to which the message was sent.

    \
    Use this field to get more informative errors.\
    Provide the same value as the `send_message` has returned.\
    If the message was not delivered (expired), SDK will log the endpoint URLs, used for its sending.

### ParamsOfProcessMessage

```typescript
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
}
```

* `message_encode_params`: [_ParamsOfEncodeMessage_](mod_abi.md#ParamsOfEncodeMessage) – Message encode parameters.
* `send_events`: _boolean_ – Flag for requesting events sending
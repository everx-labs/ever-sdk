# Module processing

 Message processing module.

 This module incorporates functions related to complex message
 processing scenarios.
## Functions
[send_message](#send_message) – Sends message to the network

[wait_for_transaction](#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](#process_message) – Creates message, sends it to the network and monitors its processing.

## Types
[ProcessingEvent](#ProcessingEvent)

[ResultOfProcessMessage](#ResultOfProcessMessage)

[DecodedOutput](#DecodedOutput)

[ParamsOfSendMessage](#ParamsOfSendMessage)

[ResultOfSendMessage](#ResultOfSendMessage)

[ParamsOfWaitForTransaction](#ParamsOfWaitForTransaction)

[ParamsOfProcessMessage](#ParamsOfProcessMessage)


# Functions
## send_message

Sends message to the network

Sends message to the network and returns the last generated shard block of the destination account
before the message was sent. It will be required later for message processing.

```ts
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
};

type ResultOfSendMessage = {
    shard_block_id: string
};

function send_message(
    params: ParamsOfSendMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfSendMessage>;
```
### Parameters
- `message`: _string_ – Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Optional message ABI.
<br>If this parameter is specified and the message has the<br>`expire` header then expiration time will be checked against<br>the current time to prevent an unnecessary sending of already expired message.<br><br>The `message already expired` error will be returned in this<br>case.<br><br>Note that specifying `abi` for ABI compliant contracts is<br>strongly recommended due to choosing proper processing<br>strategy.
- `send_events`: _boolean_ – Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.
<br>This block id must be used as a parameter of the<br>`wait_for_transaction`.


## wait_for_transaction

Performs monitoring of the network for the result transaction of the external inbound message processing.

`send_events` enables intermediate events, such as `WillFetchNextBlock`,
`FetchNextBlockFailed` that may be useful for logging of new shard blocks creation
during message processing.

Note that presence of the `abi` parameter is critical for ABI
compliant contracts. Message processing uses drastically
different strategy for processing message for contracts which
ABI includes "expire" header.

When the ABI header `expire` is present, the processing uses
`message expiration` strategy:
- The maximum block gen time is set to
  `message_expiration_time + transaction_wait_timeout`.
- When maximum block gen time is reached the processing will
  be finished with `MessageExpired` error.

When the ABI header `expire` isn't present or `abi` parameter
isn't specified, the processing uses `transaction waiting`
strategy:
- The maximum block gen time is set to
  `now() + transaction_wait_timeout`.

- If maximum block gen time is reached and no result transaction is found
the processing will exit with an error.

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean
};

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
};

function wait_for_transaction(
    params: ParamsOfWaitForTransaction,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```
### Parameters
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Optional ABI for decoding the transaction result.
<br>If it is specified then the output messages' bodies will be<br>decoded according to this ABI.<br><br>The `abi_decoded` result field will be filled out.
- `message`: _string_ – Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.
<br>You must provide the same value as the `send_message` has returned.
- `send_events`: _boolean_ – Flag that enables/disables intermediate events
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod_tvm.md#TransactionFees)_ – Transaction fees


## process_message

Creates message, sends it to the network and monitors its processing.

Creates ABI-compatible message,
sends it to the network and monitors for the result transaction.
Decodes the output messages's bodies.

If contract's ABI includes "expire" header then
SDK implements retries in case of unsuccessful message delivery within the expiration
timeout: SDK recreates the message, sends it and processes it again.

The intermediate events, such as `WillFetchFirstBlock`, `WillSend`, `DidSend`,
`WillFetchNextBlock`, etc - are switched on/off by `send_events` flag
and logged into the supplied callback function.
The retry configuration parameters are defined in config:
<add correct config params here>
pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 3; - max number of retries
pub const DEFAULT_EXPIRATION_TIMEOUT: u32 = 40000;  - message expiration timeout in ms.
pub const DEFAULT_....expiration_timeout_grow_factor... = 1.5 - factor that increases the expiration timeout for each retry

If contract's ABI does not include "expire" header
then if no transaction is found within the network timeout (see config parameter ), exits with error.

```ts
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
};

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
};

function process_message(
    params: ParamsOfProcessMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```
### Parameters
- `message_encode_params`: _[ParamsOfEncodeMessage](mod_abi.md#ParamsOfEncodeMessage)_ – Message encode parameters.
- `send_events`: _boolean_ – Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod_tvm.md#TransactionFees)_ – Transaction fees


# Types
## ProcessingEvent
```ts
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
};
```
Depends on value of the  `type` field.

When _type_ is _'WillFetchFirstBlock'_

Notifies the app that the current shard block will be fetched from the network.

Fetched block will be used later in waiting phase.


When _type_ is _'FetchFirstBlockFailed'_

Notifies the app that the client has failed to fetch current shard block.

Message processing has finished.


- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'WillSend'_

Notifies the app that the message will be sent to the network.


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'DidSend'_

Notifies the app that the message was sent to the network.


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'SendFailed'_

Notifies the app that the sending operation was failed with network error.

Nevertheless the processing will be continued at the waiting
phase because the message possibly has been delivered to the
node.


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_
- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'WillFetchNextBlock'_

Notifies the app that the next shard block will be fetched from the network.

Event can occurs more than one time due to block walking
procedure.


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'FetchNextBlockFailed'_

Notifies the app that the next block can't be fetched due to error.

Processing will be continued after `network_resume_timeout`.


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_
- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'MessageExpired'_

Notifies the app that the message was expired.

Event occurs for contracts which ABI includes header "expire"

Processing will be continued from encoding phase after
`expiration_retries_timeout`.


- `message_id`: _string_
- `message`: _string_
- `error`: _[ClientError](mod_client.md#ClientError)_


## ResultOfProcessMessage
```ts
type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
};
```
- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod_tvm.md#TransactionFees)_ – Transaction fees


## DecodedOutput
```ts
type DecodedOutput = {
    out_messages: DecodedMessageBody | null[],
    output?: any
};
```
- `out_messages`: _[DecodedMessageBody](mod_abi.md#DecodedMessageBody)?[]_ – Decoded bodies of the out messages.
<br>If the message can't be decoded then `None` will be stored in<br>the appropriate position.
- `output`?: _any_ – Decoded body of the function output message.


## ParamsOfSendMessage
```ts
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
};
```
- `message`: _string_ – Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Optional message ABI.
<br>If this parameter is specified and the message has the<br>`expire` header then expiration time will be checked against<br>the current time to prevent an unnecessary sending of already expired message.<br><br>The `message already expired` error will be returned in this<br>case.<br><br>Note that specifying `abi` for ABI compliant contracts is<br>strongly recommended due to choosing proper processing<br>strategy.
- `send_events`: _boolean_ – Flag for requesting events sending


## ResultOfSendMessage
```ts
type ResultOfSendMessage = {
    shard_block_id: string
};
```
- `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.
<br>This block id must be used as a parameter of the<br>`wait_for_transaction`.


## ParamsOfWaitForTransaction
```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean
};
```
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Optional ABI for decoding the transaction result.
<br>If it is specified then the output messages' bodies will be<br>decoded according to this ABI.<br><br>The `abi_decoded` result field will be filled out.
- `message`: _string_ – Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.
<br>You must provide the same value as the `send_message` has returned.
- `send_events`: _boolean_ – Flag that enables/disables intermediate events


## ParamsOfProcessMessage
```ts
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
};
```
- `message_encode_params`: _[ParamsOfEncodeMessage](mod_abi.md#ParamsOfEncodeMessage)_ – Message encode parameters.
- `send_events`: _boolean_ – Flag for requesting events sending



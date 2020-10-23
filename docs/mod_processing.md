# Module processing

 Message processing module.

 This module incorporates functions related to complex message
 processing scenarios.
## Functions
[send_message](#send_message) –  Sends message to the network

[wait_for_transaction](#wait_for_transaction) –  Performs monitoring of the network for the result transaction

[process_message](#process_message) –  Creates message, sends it to the network and monitors its processing.

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
- `message`: _string_ –  Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional message ABI.
- `send_events`: _boolean_ –  Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `shard_block_id`: _string_ –  The last generated shard block of the message destination account before the


## wait_for_transaction

 Performs monitoring of the network for the result transaction
 of the external inbound message processing.
 
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
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding the transaction result.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  The last generated block id of the destination account shard before the message was sent.
- `send_events`: _boolean_ –  Flag that enables/disables intermediate events
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _string[]_ –  List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional
- `fees`: _TransactionFees_ –  Transaction fees


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
- `message_encode_params`: _[ParamsOfEncodeMessage](mod_abi.md#ParamsOfEncodeMessage)_ –  Message encode parameters.
- `send_events`: _boolean_ –  Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _string[]_ –  List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional
- `fees`: _TransactionFees_ –  Transaction fees


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


When _type_ is _'FetchFirstBlockFailed'_


- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'WillSend'_


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'DidSend'_


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'SendFailed'_


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_
- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'WillFetchNextBlock'_


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_

When _type_ is _'FetchNextBlockFailed'_


- `shard_block_id`: _string_
- `message_id`: _string_
- `message`: _string_
- `error`: _[ClientError](mod_client.md#ClientError)_

When _type_ is _'MessageExpired'_


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
- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _string[]_ –  List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional
- `fees`: _TransactionFees_ –  Transaction fees


## DecodedOutput

```ts
type DecodedOutput = {
    out_messages: DecodedMessageBody | null[],
    output?: any
};
```
- `out_messages`: _[DecodedMessageBody](mod_abi.md#DecodedMessageBody)?[]_ –  Decoded bodies of the out messages.
- `output`?: _any_ –  Decoded body of the function output message.


## ParamsOfSendMessage

```ts
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
};
```
- `message`: _string_ –  Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional message ABI.
- `send_events`: _boolean_ –  Flag for requesting events sending


## ResultOfSendMessage

```ts
type ResultOfSendMessage = {
    shard_block_id: string
};
```
- `shard_block_id`: _string_ –  The last generated shard block of the message destination account before the


## ParamsOfWaitForTransaction

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean
};
```
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding the transaction result.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  The last generated block id of the destination account shard before the message was sent.
- `send_events`: _boolean_ –  Flag that enables/disables intermediate events


## ParamsOfProcessMessage

```ts
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
};
```
- `message_encode_params`: _[ParamsOfEncodeMessage](mod_abi.md#ParamsOfEncodeMessage)_ –  Message encode parameters.
- `send_events`: _boolean_ –  Flag for requesting events sending



# Module processing

 Message processing module.

 This module incorporates functions related to complex message
 processing scenarios.
## Functions
[send_message](#send_message)

[wait_for_transaction](#wait_for_transaction) –  Performs monitoring of the network for a results of the external

[process_message](#process_message) –  Sends message to the network and monitors network for a result of

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

- `shard_block_id`: _string_ –  Shard block related to the message dst account before the


## wait_for_transaction

 Performs monitoring of the network for a results of the external
 inbound message processing.

 Note that presence of the `abi` parameter is critical for ABI
 compliant contracts. Message processing uses drastically
 different strategy for processing message with an ABI expiration
 replay protection.

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
 - When maximum block gen time is reached the processing will
   be finished with `Incomplete` result.

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean
};

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: any[],
    decoded?: DecodedOutput
};

function wait_for_transaction(
    params: ParamsOfWaitForTransaction,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```
### Parameters
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding transaction results.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  Dst account shard block id before the message had been sent.
- `send_events`: _boolean_ –  Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


## process_message

 Sends message to the network and monitors network for a result of
 message processing.

```ts
type ParamsOfProcessMessage = {
    message: MessageSource,
    send_events: boolean
};

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: any[],
    decoded?: DecodedOutput
};

function process_message(
    params: ParamsOfProcessMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;
```
### Parameters
- `message`: _[MessageSource](mod_abi.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


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
} | {
    type: 'TransactionReceived'
    message_id: string,
    message: string,
    result: ResultOfProcessMessage
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

When _type_ is _'TransactionReceived'_


- `message_id`: _string_ –  Input message id. Encoded with `hex`.
- `message`: _string_ –  Input message. BOC encoded with `base64`.
- `result`: _[ResultOfProcessMessage](mod_processing.md#ResultOfProcessMessage)_ –  Results of transaction.


## ResultOfProcessMessage

```ts
type ResultOfProcessMessage = {
    transaction: any,
    out_messages: any[],
    decoded?: DecodedOutput
};
```
- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


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
- `shard_block_id`: _string_ –  Shard block related to the message dst account before the


## ParamsOfWaitForTransaction

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean
};
```
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding transaction results.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  Dst account shard block id before the message had been sent.
- `send_events`: _boolean_ –  Flag for requesting events sending


## ParamsOfProcessMessage

```ts
type ParamsOfProcessMessage = {
    message: MessageSource,
    send_events: boolean
};
```
- `message`: _[MessageSource](mod_abi.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending



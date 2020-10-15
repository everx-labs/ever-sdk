# Module processing

 Message processing module.

 This module incorporates functions related to complex message
 processing scenarios.
## Functions
[send_message](#send_message)

[wait_for_transaction](#wait_for_transaction) –  Performs monitoring of the network for a results of the external

[process_message](#process_message) –  Sends message to the network and monitors network for a result of

## Types
[MessageSource](#MessageSource)

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
    message: String,
    abi?: Abi,
    send_events: Boolean
};

type ResultOfSendMessage = {
    shard_block_id: String
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
    message: String,
    shard_block_id: String,
    send_events: Boolean
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
    send_events: Boolean
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
- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


# Types
## MessageSource

```ts
type MessageSource = {
    type: 'Encoded'
    message: String,
    abi?: Abi
} | {
    type: 'EncodingParams'
    abi: Abi,
    address?: String,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: Number
};
```
Depends on value of the  `type` field.

When _type_ is _'Encoded'_


- `message`: _string_
- `abi`?: _[Abi](mod_abi.md#Abi)_

When _type_ is _'EncodingParams'_


- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `address`?: _string_ –  Contract address.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ –  Deploy parameters.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.


## ProcessingEvent

```ts
type ProcessingEvent = {
    type: 'WillFetchFirstBlock'
} | {
    type: 'FetchFirstBlockFailed'
    error: ClientError
} | {
    type: 'WillSend'
    shard_block_id: String,
    message_id: String,
    message: String
} | {
    type: 'DidSend'
    shard_block_id: String,
    message_id: String,
    message: String
} | {
    type: 'SendFailed'
    shard_block_id: String,
    message_id: String,
    message: String,
    error: ClientError
} | {
    type: 'WillFetchNextBlock'
    shard_block_id: String,
    message_id: String,
    message: String
} | {
    type: 'FetchNextBlockFailed'
    shard_block_id: String,
    message_id: String,
    message: String,
    error: ClientError
} | {
    type: 'MessageExpired'
    message_id: String,
    message: String,
    error: ClientError
} | {
    type: 'TransactionReceived'
    message_id: String,
    message: String,
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
    message: String,
    abi?: Abi,
    send_events: Boolean
};
```
- `message`: _string_ –  Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional message ABI.
- `send_events`: _boolean_ –  Flag for requesting events sending


## ResultOfSendMessage

```ts
type ResultOfSendMessage = {
    shard_block_id: String
};
```
- `shard_block_id`: _string_ –  Shard block related to the message dst account before the


## ParamsOfWaitForTransaction

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: String,
    shard_block_id: String,
    send_events: Boolean
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
    send_events: Boolean
};
```
- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending



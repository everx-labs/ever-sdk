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

function sendMessage(
    params: ParamsOfSendMessage,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfSendMessage>;

```
### Parameters
- `message`: _string_ –  Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional message ABI.
- `send_events`: _boolean_ –  Flag for requesting events sending
### Result

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

function waitForTransaction(
    params: ParamsOfWaitForTransaction,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfProcessMessage>;

```
### Parameters
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding transaction results.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  Dst account shard block id before the message had been sent.
- `send_events`: _boolean_ –  Flag for requesting events sending
### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


## process_message

 Sends message to the network and monitors network for a result of
 message processing.

```ts

function processMessage(
    params: ParamsOfProcessMessage,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfProcessMessage>;

```
### Parameters
- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending
### Result

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


# Types
## MessageSource



## ProcessingEvent



## ResultOfProcessMessage

- `transaction`: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional


## DecodedOutput

- `out_messages`: _[DecodedMessageBody](mod_abi.md#DecodedMessageBody)?[]_ –  Decoded bodies of the out messages.
- `output`?: _any_ –  Decoded body of the function output message.


## ParamsOfSendMessage

- `message`: _string_ –  Message BOC.
- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional message ABI.
- `send_events`: _boolean_ –  Flag for requesting events sending


## ResultOfSendMessage

- `shard_block_id`: _string_ –  Shard block related to the message dst account before the


## ParamsOfWaitForTransaction

- `abi`?: _[Abi](mod_abi.md#Abi)_ –  Optional ABI for decoding transaction results.
- `message`: _string_ –  Message BOC. Encoded with `base64`.
- `shard_block_id`: _string_ –  Dst account shard block id before the message had been sent.
- `send_events`: _boolean_ –  Flag for requesting events sending


## ParamsOfProcessMessage

- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Message source.
- `send_events`: _boolean_ –  Flag for requesting events sending



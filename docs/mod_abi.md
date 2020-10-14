# Module abi

 Functions for encoding and decoding messages due to ABI
 specification.
## Functions
[encode_message](#encode_message)

[attach_signature](#attach_signature)

[decode_message](#decode_message)

[encode_account](#encode_account) –  Encodes account state as it will be

## Types
[Abi](#Abi)

[AbiHandle](#AbiHandle)

[FunctionHeader](#FunctionHeader) –  The ABI function header.

[CallSet](#CallSet)

[DeploySet](#DeploySet)

[Signer](#Signer)

[DecodedMessageType](#DecodedMessageType)

[StateInitSource](#StateInitSource)

[StateInitParams](#StateInitParams)

[ParamsOfEncodeMessage](#ParamsOfEncodeMessage)

[ResultOfEncodeMessage](#ResultOfEncodeMessage)

[ParamsOfAttachSignature](#ParamsOfAttachSignature)

[ResultOfAttachSignature](#ResultOfAttachSignature)

[ParamsOfDecodeMessage](#ParamsOfDecodeMessage)

[DecodedMessageBody](#DecodedMessageBody)

[ParamsOfEncodeAccount](#ParamsOfEncodeAccount)

[ResultOfEncodeAccount](#ResultOfEncodeAccount)


# Functions
## encode_message

```ts

function encodeMessage(
    params: ParamsOfEncodeMessage,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfEncodeMessage>;

```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `address`?: _string_ –  Contract address.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ –  Deploy parameters.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.
### Result

- `message`: _string_ –  Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ –  Optional data to sign. Encoded with `base64`.
- `address`: _string_ –  Destination address.
- `message_id`: _string_ –  Message id.


## attach_signature

```ts

function attachSignature(
    params: ParamsOfAttachSignature,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfAttachSignature>;

```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.
### Result

- `message`: _string_
- `message_id`: _string_


## decode_message

```ts

function decodeMessage(
    params: ParamsOfDecodeMessage,
    responseHandler: ResponseHandler | null,
): Promise<DecodedMessageBody>;

```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  contract ABI
- `message`: _string_ –  Message BOC
### Result

- `message_type`: _[DecodedMessageType](mod_abi.md#DecodedMessageType)_ –  Type of the message body content.
- `name`: _string_ –  Function or event name.
- `value`: _any_ –  Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.


## encode_account

 Encodes account state as it will be

```ts

function encodeAccount(
    params: ParamsOfEncodeAccount,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfEncodeAccount>;

```
### Parameters
- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ –  Source of the account state init.
- `balance`?: _bigint_ –  Initial balance.
- `last_trans_lt`?: _bigint_ –  Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ –  Initial value for the `last_paid`.
### Result

- `account`: _string_ –  Account BOC. Encoded with `base64`.
- `id`: _string_ –  Account id. Encoded with `hex`.


# Types
## Abi



## AbiHandle

- ``: _number_


## FunctionHeader

 The ABI function header.

 Includes several hidden function parameters that contract
 uses for security and replay protection reasons.

 The actual set of header fields depends on the contract's ABI.

- `expire`?: _number_ –  Message expiration time in seconds.
- `time`?: _bigint_ –  Message creation time in milliseconds.
- `pubkey`?: _string_ –  Public key used to sign message. Encoded with `hex`.


## CallSet

- `function_name`: _string_ –  Function name.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.
- `input`?: _any_ –  Function input according to ABI.


## DeploySet

- `tvc`: _string_ –  Content of TVC file. Must be encoded with `base64`.
- `workchain_id`?: _number_ –  Target workchain for destination address. Default is `0`.
- `initial_data`?: _any_ –  List of initial values for contract's public variables.


## Signer



## DecodedMessageType



## StateInitSource



## StateInitParams

- `abi`: _[Abi](mod_abi.md#Abi)_
- `value`: _any_


## ParamsOfEncodeMessage

- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `address`?: _string_ –  Contract address.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ –  Deploy parameters.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.


## ResultOfEncodeMessage

- `message`: _string_ –  Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ –  Optional data to sign. Encoded with `base64`.
- `address`: _string_ –  Destination address.
- `message_id`: _string_ –  Message id.


## ParamsOfAttachSignature

- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.


## ResultOfAttachSignature

- `message`: _string_
- `message_id`: _string_


## ParamsOfDecodeMessage

- `abi`: _[Abi](mod_abi.md#Abi)_ –  contract ABI
- `message`: _string_ –  Message BOC


## DecodedMessageBody

- `message_type`: _[DecodedMessageType](mod_abi.md#DecodedMessageType)_ –  Type of the message body content.
- `name`: _string_ –  Function or event name.
- `value`: _any_ –  Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.


## ParamsOfEncodeAccount

- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ –  Source of the account state init.
- `balance`?: _bigint_ –  Initial balance.
- `last_trans_lt`?: _bigint_ –  Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ –  Initial value for the `last_paid`.


## ResultOfEncodeAccount

- `account`: _string_ –  Account BOC. Encoded with `base64`.
- `id`: _string_ –  Account id. Encoded with `hex`.



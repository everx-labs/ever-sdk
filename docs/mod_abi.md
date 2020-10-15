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
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: String,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: Number
};

type ResultOfEncodeMessage = {
    message: String,
    data_to_sign?: String,
    address: String,
    message_id: String
};

function encode_message(
    params: ParamsOfEncodeMessage,
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
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: String,
    message: String,
    signature: String
};

type ResultOfAttachSignature = {
    message: String,
    message_id: String
};

function attach_signature(
    params: ParamsOfAttachSignature,
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
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: String
};

type DecodedMessageBody = {
    message_type: DecodedMessageType,
    name: String,
    value: any,
    header?: FunctionHeader
};

function decode_message(
    params: ParamsOfDecodeMessage,
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
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: Number
};

type ResultOfEncodeAccount = {
    account: String,
    id: String
};

function encode_account(
    params: ParamsOfEncodeAccount,
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

```ts
type Abi = {
    type: 'Serialized'
    value: any
} | {
    type: 'Handle'
    value: Number
};
```
Depends on value of the  `type` field.

When _type_ is _'Serialized'_


- `value`: _any_

When _type_ is _'Handle'_


- `value`: _number_


## AbiHandle

```ts
type AbiHandle = Number;
```
- ``: _number_


## FunctionHeader
 The ABI function header.

 Includes several hidden function parameters that contract
 uses for security and replay protection reasons.

 The actual set of header fields depends on the contract's ABI.


```ts
type FunctionHeader = {
    expire?: Number,
    time?: bigint,
    pubkey?: String
};
```
- `expire`?: _number_ –  Message expiration time in seconds.
- `time`?: _bigint_ –  Message creation time in milliseconds.
- `pubkey`?: _string_ –  Public key used to sign message. Encoded with `hex`.


## CallSet

```ts
type CallSet = {
    function_name: String,
    header?: FunctionHeader,
    input?: any
};
```
- `function_name`: _string_ –  Function name.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.
- `input`?: _any_ –  Function input according to ABI.


## DeploySet

```ts
type DeploySet = {
    tvc: String,
    workchain_id?: Number,
    initial_data?: any
};
```
- `tvc`: _string_ –  Content of TVC file. Must be encoded with `base64`.
- `workchain_id`?: _number_ –  Target workchain for destination address. Default is `0`.
- `initial_data`?: _any_ –  List of initial values for contract's public variables.


## Signer

```ts
type Signer = {
    type: 'None'
} | {
    type: 'External'
    public_key: String
} | {
    type: 'Keys'
    keys: KeyPair
} | {
    type: 'SigningBox'
    handle: SigningBoxHandle
};
```
Depends on value of the  `type` field.

When _type_ is _'None'_


When _type_ is _'External'_


- `public_key`: _string_

When _type_ is _'Keys'_


- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_

When _type_ is _'SigningBox'_


- `handle`: _[SigningBoxHandle](mod_crypto.md#SigningBoxHandle)_


## DecodedMessageType

```ts
type DecodedMessageType = 'FunctionInput' | 'FunctionOutput' | 'ForeignFunctionInput' | 'Event';
```
One of the following value:

- `FunctionInput` –  Message contains the input of the ABI function.
- `FunctionOutput` –  Message contains the output of the ABI function.
- `ForeignFunctionInput` –  Message contains the input of the foreign ABI function.
- `Event` –  Message contains the input of the ABI event.


## StateInitSource

```ts
type StateInitSource = {
    type: 'Message'
    source: MessageSource
} | {
    type: 'StateInit'
    code: String,
    data: String,
    library?: String
} | {
    type: 'Tvc'
    tvc: String,
    public_key?: String,
    init_params?: StateInitParams
};
```
Depends on value of the  `type` field.

When _type_ is _'Message'_


- `source`: _[MessageSource](mod_processing.md#MessageSource)_

When _type_ is _'StateInit'_


- `code`: _string_ –  Code BOC. Encoded with `base64`.
- `data`: _string_ –  Data BOC. Encoded with `base64`.
- `library`?: _string_ –  Library BOC. Encoded with `base64`.

When _type_ is _'Tvc'_


- `tvc`: _string_
- `public_key`?: _string_
- `init_params`?: _[StateInitParams](mod_abi.md#StateInitParams)_


## StateInitParams

```ts
type StateInitParams = {
    abi: Abi,
    value: any
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_
- `value`: _any_


## ParamsOfEncodeMessage

```ts
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: String,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: Number
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `address`?: _string_ –  Contract address.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ –  Deploy parameters.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.


## ResultOfEncodeMessage

```ts
type ResultOfEncodeMessage = {
    message: String,
    data_to_sign?: String,
    address: String,
    message_id: String
};
```
- `message`: _string_ –  Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ –  Optional data to sign. Encoded with `base64`.
- `address`: _string_ –  Destination address.
- `message_id`: _string_ –  Message id.


## ParamsOfAttachSignature

```ts
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: String,
    message: String,
    signature: String
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.


## ResultOfAttachSignature

```ts
type ResultOfAttachSignature = {
    message: String,
    message_id: String
};
```
- `message`: _string_
- `message_id`: _string_


## ParamsOfDecodeMessage

```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: String
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  contract ABI
- `message`: _string_ –  Message BOC


## DecodedMessageBody

```ts
type DecodedMessageBody = {
    message_type: DecodedMessageType,
    name: String,
    value: any,
    header?: FunctionHeader
};
```
- `message_type`: _[DecodedMessageType](mod_abi.md#DecodedMessageType)_ –  Type of the message body content.
- `name`: _string_ –  Function or event name.
- `value`: _any_ –  Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.


## ParamsOfEncodeAccount

```ts
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: Number
};
```
- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ –  Source of the account state init.
- `balance`?: _bigint_ –  Initial balance.
- `last_trans_lt`?: _bigint_ –  Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ –  Initial value for the `last_paid`.


## ResultOfEncodeAccount

```ts
type ResultOfEncodeAccount = {
    account: String,
    id: String
};
```
- `account`: _string_ –  Account BOC. Encoded with `base64`.
- `id`: _string_ –  Account id. Encoded with `hex`.



# Module abi

 Provides message encoding and decoding according to the ABI
 specification.
## Functions
[encode_message_body](#encode_message_body) –  Encodes message body according to ABI function call.

[attach_signature_to_message_body](#attach_signature_to_message_body)

[encode_message](#encode_message)

[attach_signature](#attach_signature)

[decode_message](#decode_message) –  Decodes message body using provided message BOC and ABI.

[decode_message_body](#decode_message_body) –  Decodes message body using provided body BOC and ABI.

[encode_account](#encode_account) –  Encodes account state as it will be

## Types
[Abi](#Abi)

[AbiHandle](#AbiHandle)

[FunctionHeader](#FunctionHeader) –  The ABI function header.

[CallSet](#CallSet)

[DeploySet](#DeploySet)

[Signer](#Signer)

[BodyType](#BodyType)

[StateInitSource](#StateInitSource)

[StateInitParams](#StateInitParams)

[ParamsOfEncodeMessageBody](#ParamsOfEncodeMessageBody)

[ResultOfEncodeMessageBody](#ResultOfEncodeMessageBody)

[ParamsOfAttachSignatureToMessageBody](#ParamsOfAttachSignatureToMessageBody)

[ResultOfAttachSignatureToMessageBody](#ResultOfAttachSignatureToMessageBody)

[ParamsOfEncodeMessage](#ParamsOfEncodeMessage)

[ResultOfEncodeMessage](#ResultOfEncodeMessage)

[ParamsOfAttachSignature](#ParamsOfAttachSignature)

[ResultOfAttachSignature](#ResultOfAttachSignature)

[ParamsOfDecodeMessage](#ParamsOfDecodeMessage)

[DecodedBody](#DecodedBody)

[ParamsOfDecodeMessageBody](#ParamsOfDecodeMessageBody)

[ParamsOfEncodeAccount](#ParamsOfEncodeAccount)

[ResultOfEncodeAccount](#ResultOfEncodeAccount)


# Functions
## encode_message_body

 Encodes message body according to ABI function call.

```ts
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number
};

type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
};

function encode_message_body(
    params: ParamsOfEncodeMessageBody,
): Promise<ResultOfEncodeMessageBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `call_set`: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `is_internal`: _boolean_ –  True if internal message body must be encoded.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.
### Result

- `body`: _string_ –  Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ –  Optional data to sign. Encoded with `base64`.


## attach_signature_to_message_body

```ts
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
};

type ResultOfAttachSignatureToMessageBody = {
    body: string
};

function attach_signature_to_message_body(
    params: ParamsOfAttachSignatureToMessageBody,
): Promise<ResultOfAttachSignatureToMessageBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.
### Result

- `body`: _string_


## encode_message

```ts
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number
};

type ResultOfEncodeMessage = {
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
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
    public_key: string,
    message: string,
    signature: string
};

type ResultOfAttachSignature = {
    message: string,
    message_id: string
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

 Decodes message body using provided message BOC and ABI.

```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
};

type DecodedBody = {
    body_type: BodyType,
    name: string,
    value: any,
    header?: FunctionHeader
};

function decode_message(
    params: ParamsOfDecodeMessage,
): Promise<DecodedBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  contract ABI
- `message`: _string_ –  Message BOC
### Result

- `body_type`: _[BodyType](mod_abi.md#BodyType)_ –  Type of the message body content.
- `name`: _string_ –  Function or event name.
- `value`: _any_ –  Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.


## decode_message_body

 Decodes message body using provided body BOC and ABI.

```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
};

type DecodedBody = {
    body_type: BodyType,
    name: string,
    value: any,
    header?: FunctionHeader
};

function decode_message_body(
    params: ParamsOfDecodeMessageBody,
): Promise<DecodedBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI used to decode.
- `body`: _string_ –  Message body BOC. Must be encoded with `base64`.
- `is_internal`: _boolean_ –  True if the body belongs to the internal message.
### Result

- `body_type`: _[BodyType](mod_abi.md#BodyType)_ –  Type of the message body content.
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
    last_paid?: number
};

type ResultOfEncodeAccount = {
    account: string,
    id: string
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
    value: number
};
```
Depends on value of the  `type` field.

When _type_ is _'Serialized'_


- `value`: _any_

When _type_ is _'Handle'_


- `value`: _number_


## AbiHandle

```ts
type AbiHandle = number;
```
- _number_


## FunctionHeader
 The ABI function header.

 Includes several hidden function parameters that contract
 uses for security and replay protection reasons.

 The actual set of header fields depends on the contract's ABI.


```ts
type FunctionHeader = {
    expire?: number,
    time?: bigint,
    pubkey?: string
};
```
- `expire`?: _number_ –  Message expiration time in seconds.
- `time`?: _bigint_ –  Message creation time in milliseconds.
- `pubkey`?: _string_ –  Public key used to sign message. Encoded with `hex`.


## CallSet

```ts
type CallSet = {
    function_name: string,
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
    tvc: string,
    workchain_id?: number,
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
    public_key: string
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


## BodyType

```ts
type BodyType = 'Input' | 'Output' | 'InternalOutput' | 'Event';
```
One of the following value:

- `Input` –  Message contains the input of the ABI function.
- `Output` –  Message contains the output of the ABI function.
- `InternalOutput` –  Message contains the input of the imported ABI function.
- `Event` –  Message contains the input of the ABI event.


## StateInitSource

```ts
type StateInitSource = {
    type: 'Message'
    source: MessageSource
} | {
    type: 'StateInit'
    code: string,
    data: string,
    library?: string
} | {
    type: 'Tvc'
    tvc: string,
    public_key?: string,
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


## ParamsOfEncodeMessageBody

```ts
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI.
- `call_set`: _[CallSet](mod_abi.md#CallSet)_ –  Function call parameters.
- `is_internal`: _boolean_ –  True if internal message body must be encoded.
- `signer`: _[Signer](mod_abi.md#Signer)_ –  Signing parameters.
- `processing_try_index`?: _number_ –  Processing try index.


## ResultOfEncodeMessageBody

```ts
type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
};
```
- `body`: _string_ –  Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ –  Optional data to sign. Encoded with `base64`.


## ParamsOfAttachSignatureToMessageBody

```ts
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.


## ResultOfAttachSignatureToMessageBody

```ts
type ResultOfAttachSignatureToMessageBody = {
    body: string
};
```
- `body`: _string_


## ParamsOfEncodeMessage

```ts
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number
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
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
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
    public_key: string,
    message: string,
    signature: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI
- `public_key`: _string_ –  Public key. Must be encoded with `hex`.
- `message`: _string_ –  Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ –  Signature. Must be encoded with `hex`.


## ResultOfAttachSignature

```ts
type ResultOfAttachSignature = {
    message: string,
    message_id: string
};
```
- `message`: _string_
- `message_id`: _string_


## ParamsOfDecodeMessage

```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  contract ABI
- `message`: _string_ –  Message BOC


## DecodedBody

```ts
type DecodedBody = {
    body_type: BodyType,
    name: string,
    value: any,
    header?: FunctionHeader
};
```
- `body_type`: _[BodyType](mod_abi.md#BodyType)_ –  Type of the message body content.
- `name`: _string_ –  Function or event name.
- `value`: _any_ –  Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ –  Function header.


## ParamsOfDecodeMessageBody

```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ –  Contract ABI used to decode.
- `body`: _string_ –  Message body BOC. Must be encoded with `base64`.
- `is_internal`: _boolean_ –  True if the body belongs to the internal message.


## ParamsOfEncodeAccount

```ts
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: number
};
```
- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ –  Source of the account state init.
- `balance`?: _bigint_ –  Initial balance.
- `last_trans_lt`?: _bigint_ –  Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ –  Initial value for the `last_paid`.


## ResultOfEncodeAccount

```ts
type ResultOfEncodeAccount = {
    account: string,
    id: string
};
```
- `account`: _string_ –  Account BOC. Encoded with `base64`.
- `id`: _string_ –  Account id. Encoded with `hex`.



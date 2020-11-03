# Module abi

 Provides message encoding and decoding according to the ABI
 specification.
## Functions
[encode_message_body](#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](#attach_signature_to_message_body)

[encode_message](#encode_message) – Encodes an ABI-compatible message

[attach_signature](#attach_signature) – Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

[decode_message](#decode_message) – Decodes message body using provided message BOC and ABI.

[decode_message_body](#decode_message_body) – Decodes message body using provided body BOC and ABI.

[encode_account](#encode_account) – Creates account state BOC

## Types
[Abi](#Abi)

[AbiHandle](#AbiHandle)

[FunctionHeader](#FunctionHeader) – The ABI function header.

[CallSet](#CallSet)

[DeploySet](#DeploySet)

[Signer](#Signer)

[MessageBodyType](#MessageBodyType)

[StateInitSource](#StateInitSource)

[StateInitParams](#StateInitParams)

[MessageSource](#MessageSource)

[AbiParam](#AbiParam)

[AbiEvent](#AbiEvent)

[AbiData](#AbiData)

[AbiFunction](#AbiFunction)

[AbiContract](#AbiContract)

[ParamsOfEncodeMessageBody](#ParamsOfEncodeMessageBody)

[ResultOfEncodeMessageBody](#ResultOfEncodeMessageBody)

[ParamsOfAttachSignatureToMessageBody](#ParamsOfAttachSignatureToMessageBody)

[ResultOfAttachSignatureToMessageBody](#ResultOfAttachSignatureToMessageBody)

[ParamsOfEncodeMessage](#ParamsOfEncodeMessage)

[ResultOfEncodeMessage](#ResultOfEncodeMessage)

[ParamsOfAttachSignature](#ParamsOfAttachSignature)

[ResultOfAttachSignature](#ResultOfAttachSignature)

[ParamsOfDecodeMessage](#ParamsOfDecodeMessage)

[DecodedMessageBody](#DecodedMessageBody)

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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI.
- `call_set`: _[CallSet](mod_abi.md#CallSet)_ – Function call parameters.
<br>Must be specified in non deploy message.<br><br>In case of deploy message contains parameters of constructor.
- `is_internal`: _boolean_ – True if internal message body must be encoded.
- `signer`: _[Signer](mod_abi.md#Signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries.<br><br>Encoder uses the provided try index to calculate message<br>expiration time.<br><br>Expiration timeouts will grow with every retry.<br><br>Default value is 0.
### Result

- `body`: _string_ – Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to sign. Encoded with `base64`.
<br>Presents when `message` is unsigned. Can be used for external<br>message signing. Is this case you need to sing this data and<br>produce signed message using `abi.attach_signature`.


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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI
- `public_key`: _string_ – Public key. Must be encoded with `hex`.
- `message`: _string_ – Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ – Signature. Must be encoded with `hex`.
### Result

- `body`: _string_


## encode_message

Encodes an ABI-compatible message

Allows to encode deploy and function call messages,
both signed and unsigned.

Use cases include messages of any possible type:
- deploy with initial function call (i.e. `constructor` or any other function that is used for some kind
of initialization);
- deploy without initial function call;
- signed/unsigned + data for signing.

`Signer` defines how the message should or shouldn't be signed:

`Signer::None` creates an unsigned message. This may be needed in case of some public methods,
that do not require authorization by pubkey.

`Signer::External` takes public key and returns `data_to_sign` for later signing.
Use `attach_signature` method with the result signature to get the signed message.

`Signer::Keys` creates a signed message with provided key pair.

[SOON] `Signer::SigningBox` Allows using a special interface to imlepement signing
without private key disclosure to SDK. For instance, in case of using a cold wallet or HSM,
when application calls some API to sign data.

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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `signer`: _[Signer](mod_abi.md#Signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries (if contract's ABI includes "expire" header).<br><br>Encoder uses the provided try index to calculate message<br>expiration time. The 1st message expiration time is specified in<br>Client config.<br><br>Expiration timeouts will grow with every retry.<br>Retry grow factor is set in Client config:<br><.....add config parameter with default value here><br><br>Default value is 0.
### Result

- `message`: _string_ – Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.
<br>Returned in case of `Signer::External`. Can be used for external<br>message signing. Is this case you need to use this data to create signature and<br>then produce signed message using `abi.attach_signature`.
- `address`: _string_ – Destination address.
- `message_id`: _string_ – Message id.


## attach_signature

Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI
- `public_key`: _string_ – Public key encoded in `hex`.
- `message`: _string_ – Unsigned message BOC encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.
### Result

- `message`: _string_ – Signed message BOC
- `message_id`: _string_ – Message ID


## decode_message

Decodes message body using provided message BOC and ABI.

```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
};

type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
};

function decode_message(
    params: ParamsOfDecodeMessage,
): Promise<DecodedMessageBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ – contract ABI
- `message`: _string_ – Message BOC
### Result

- `body_type`: _[MessageBodyType](mod_abi.md#MessageBodyType)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ – Function header.


## decode_message_body

Decodes message body using provided body BOC and ABI.

```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
};

type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
};

function decode_message_body(
    params: ParamsOfDecodeMessageBody,
): Promise<DecodedMessageBody>;
```
### Parameters
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI used to decode.
- `body`: _string_ – Message body BOC encoded in `base64`.
- `is_internal`: _boolean_ – True if the body belongs to the internal message.
### Result

- `body_type`: _[MessageBodyType](mod_abi.md#MessageBodyType)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ – Function header.


## encode_account

Creates account state BOC

Creates account state provided with one of these sets of data :
1. BOC of code, BOC of data, BOC of library
2. TVC (string in `base64`), keys, init params

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
- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ – Source of the account state init.
- `balance`?: _bigint_ – Initial balance.
- `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ – Initial value for the `last_paid`.
### Result

- `account`: _string_ – Account BOC encoded in `base64`.
- `id`: _string_ – Account ID  encoded in `hex`.


# Types
## Abi
```ts
type Abi = {
    type: 'Serialized'
    'ABI version': number,
    header?: string[],
    functions?: AbiFunction[],
    events?: AbiEvent[],
    data?: AbiData[]
} | {
    type: 'Handle'
    value: number
};
```
Depends on value of the  `type` field.

When _type_ is _'Serialized'_


- `ABI version`: _number_
- `header`?: _string[]_
- `functions`?: _[AbiFunction](mod_abi.md#AbiFunction)[]_
- `events`?: _[AbiEvent](mod_abi.md#AbiEvent)[]_
- `data`?: _[AbiData](mod_abi.md#AbiData)[]_

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
- `expire`?: _number_ – Message expiration time in seconds.
- `time`?: _bigint_ – Message creation time in milliseconds.
- `pubkey`?: _string_ – Public key used to sign message. Encoded with `hex`.


## CallSet
```ts
type CallSet = {
    function_name: string,
    header?: FunctionHeader,
    input?: any
};
```
- `function_name`: _string_ – Function name that is being called.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ – Function header.
<br>If an application omits some header parameters required by the<br>contract's ABI, the library will set the default values for<br>them.
- `input`?: _any_ – Function input parameters according to ABI.


## DeploySet
```ts
type DeploySet = {
    tvc: string,
    workchain_id?: number,
    initial_data?: any
};
```
- `tvc`: _string_ – Content of TVC file encoded in `base64`.
- `workchain_id`?: _number_ – Target workchain for destination address. Default is `0`.
- `initial_data`?: _any_ – List of initial values for contract's public variables.


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

No keys are provided. Creates an unsigned message.


When _type_ is _'External'_

Only public key is provided to generate unsigned message and `data_to_sign` which can be signed later.


- `public_key`: _string_

When _type_ is _'Keys'_

Key pair is provided for signing


- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_

When _type_ is _'SigningBox'_

Signing Box interface is provided for signing, allows Dapps to sign messages using external APIs, such as HSM, cold wallet, etc.


- `handle`: _[SigningBoxHandle](mod_crypto.md#SigningBoxHandle)_


## MessageBodyType
```ts
type MessageBodyType = 'Input' | 'Output' | 'InternalOutput' | 'Event';
```
One of the following value:

- `Input` – Message contains the input of the ABI function.
- `Output` – Message contains the output of the ABI function.
- `InternalOutput` – Message contains the input of the imported ABI function.
<br>Occurs when contract sends an internal message to other<br>contract.
- `Event` – Message contains the input of the ABI event.


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

Deploy message.


- `source`: _[MessageSource](mod_abi.md#MessageSource)_

When _type_ is _'StateInit'_

State init data.


- `code`: _string_ – Code BOC. Encoded in `base64`.
- `data`: _string_ – Data BOC. Encoded in `base64`.
- `library`?: _string_ – Library BOC. Encoded in `base64`.

When _type_ is _'Tvc'_

Content of the TVC file. Encoded in `base64`.


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


## MessageSource
```ts
type MessageSource = {
    type: 'Encoded'
    message: string,
    abi?: Abi
} | {
    type: 'EncodingParams'
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number
};
```
Depends on value of the  `type` field.

When _type_ is _'Encoded'_


- `message`: _string_
- `abi`?: _[Abi](mod_abi.md#Abi)_

When _type_ is _'EncodingParams'_


- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `signer`: _[Signer](mod_abi.md#Signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries (if contract's ABI includes "expire" header).<br><br>Encoder uses the provided try index to calculate message<br>expiration time. The 1st message expiration time is specified in<br>Client config.<br><br>Expiration timeouts will grow with every retry.<br>Retry grow factor is set in Client config:<br><.....add config parameter with default value here><br><br>Default value is 0.


## AbiParam
```ts
type AbiParam = {
    name: string,
    type: string,
    components?: AbiParam[]
};
```
- `name`: _string_
- `type`: _string_
- `components`?: _[AbiParam](mod_abi.md#AbiParam)[]_


## AbiEvent
```ts
type AbiEvent = {
    name: string,
    inputs: AbiParam[],
    id?: number | null
};
```
- `name`: _string_
- `inputs`: _[AbiParam](mod_abi.md#AbiParam)[]_
- `id`?: _number?_


## AbiData
```ts
type AbiData = {
    key: bigint,
    name: string,
    type: string,
    components?: AbiParam[]
};
```
- `key`: _bigint_
- `name`: _string_
- `type`: _string_
- `components`?: _[AbiParam](mod_abi.md#AbiParam)[]_


## AbiFunction
```ts
type AbiFunction = {
    name: string,
    inputs: AbiParam[],
    outputs: AbiParam[],
    id?: number | null
};
```
- `name`: _string_
- `inputs`: _[AbiParam](mod_abi.md#AbiParam)[]_
- `outputs`: _[AbiParam](mod_abi.md#AbiParam)[]_
- `id`?: _number?_


## AbiContract
```ts
type AbiContract = {
    'ABI version': number,
    header?: string[],
    functions?: AbiFunction[],
    events?: AbiEvent[],
    data?: AbiData[]
};
```
- `ABI version`: _number_
- `header`?: _string[]_
- `functions`?: _[AbiFunction](mod_abi.md#AbiFunction)[]_
- `events`?: _[AbiEvent](mod_abi.md#AbiEvent)[]_
- `data`?: _[AbiData](mod_abi.md#AbiData)[]_


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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI.
- `call_set`: _[CallSet](mod_abi.md#CallSet)_ – Function call parameters.
<br>Must be specified in non deploy message.<br><br>In case of deploy message contains parameters of constructor.
- `is_internal`: _boolean_ – True if internal message body must be encoded.
- `signer`: _[Signer](mod_abi.md#Signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries.<br><br>Encoder uses the provided try index to calculate message<br>expiration time.<br><br>Expiration timeouts will grow with every retry.<br><br>Default value is 0.


## ResultOfEncodeMessageBody
```ts
type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
};
```
- `body`: _string_ – Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to sign. Encoded with `base64`.
<br>Presents when `message` is unsigned. Can be used for external<br>message signing. Is this case you need to sing this data and<br>produce signed message using `abi.attach_signature`.


## ParamsOfAttachSignatureToMessageBody
```ts
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI
- `public_key`: _string_ – Public key. Must be encoded with `hex`.
- `message`: _string_ – Unsigned message BOC. Must be encoded with `base64`.
- `signature`: _string_ – Signature. Must be encoded with `hex`.


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
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `deploy_set`?: _[DeploySet](mod_abi.md#DeploySet)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod_abi.md#CallSet)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `signer`: _[Signer](mod_abi.md#Signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries (if contract's ABI includes "expire" header).<br><br>Encoder uses the provided try index to calculate message<br>expiration time. The 1st message expiration time is specified in<br>Client config.<br><br>Expiration timeouts will grow with every retry.<br>Retry grow factor is set in Client config:<br><.....add config parameter with default value here><br><br>Default value is 0.


## ResultOfEncodeMessage
```ts
type ResultOfEncodeMessage = {
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
};
```
- `message`: _string_ – Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.
<br>Returned in case of `Signer::External`. Can be used for external<br>message signing. Is this case you need to use this data to create signature and<br>then produce signed message using `abi.attach_signature`.
- `address`: _string_ – Destination address.
- `message_id`: _string_ – Message id.


## ParamsOfAttachSignature
```ts
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI
- `public_key`: _string_ – Public key encoded in `hex`.
- `message`: _string_ – Unsigned message BOC encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## ResultOfAttachSignature
```ts
type ResultOfAttachSignature = {
    message: string,
    message_id: string
};
```
- `message`: _string_ – Signed message BOC
- `message_id`: _string_ – Message ID


## ParamsOfDecodeMessage
```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ – contract ABI
- `message`: _string_ – Message BOC


## DecodedMessageBody
```ts
type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
};
```
- `body_type`: _[MessageBodyType](mod_abi.md#MessageBodyType)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod_abi.md#FunctionHeader)_ – Function header.


## ParamsOfDecodeMessageBody
```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
};
```
- `abi`: _[Abi](mod_abi.md#Abi)_ – Contract ABI used to decode.
- `body`: _string_ – Message body BOC encoded in `base64`.
- `is_internal`: _boolean_ – True if the body belongs to the internal message.


## ParamsOfEncodeAccount
```ts
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: number
};
```
- `state_init`: _[StateInitSource](mod_abi.md#StateInitSource)_ – Source of the account state init.
- `balance`?: _bigint_ – Initial balance.
- `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ – Initial value for the `last_paid`.


## ResultOfEncodeAccount
```ts
type ResultOfEncodeAccount = {
    account: string,
    id: string
};
```
- `account`: _string_ – Account BOC encoded in `base64`.
- `id`: _string_ – Account ID  encoded in `hex`.



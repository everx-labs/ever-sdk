# Module abi

## Module abi

Provides message encoding and decoding according to the ABI specification.

### Functions

[encode_message_body](mod_abi.md#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](mod_abi.md#attach_signature_to_message_body)

[encode_message](mod_abi.md#encode_message) – Encodes an ABI-compatible message

[encode_internal_message](mod_abi.md#encode_internal_message) – Encodes an internal ABI-compatible message

[attach_signature](mod_abi.md#attach_signature) – Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

[decode_message](mod_abi.md#decode_message) – Decodes message body using provided message BOC and ABI.

[decode_message_body](mod_abi.md#decode_message_body) – Decodes message body using provided body BOC and ABI.

[encode_account](mod_abi.md#encode_account) – Creates account state BOC

[decode_account_data](mod_abi.md#decode_account_data) – Decodes account data using provided data BOC and ABI.

### Types

[AbiErrorCode](mod_abi.md#AbiErrorCode)

[Abi](mod_abi.md#Abi)

[AbiHandle](mod_abi.md#AbiHandle)

[FunctionHeader](mod_abi.md#FunctionHeader) – The ABI function header.

[CallSet](mod_abi.md#CallSet)

[DeploySet](mod_abi.md#DeploySet)

[Signer](mod_abi.md#Signer)

[MessageBodyType](mod_abi.md#MessageBodyType)

[StateInitSource](mod_abi.md#StateInitSource)

[StateInitParams](mod_abi.md#StateInitParams)

[MessageSource](mod_abi.md#MessageSource)

[AbiParam](mod_abi.md#AbiParam)

[AbiEvent](mod_abi.md#AbiEvent)

[AbiData](mod_abi.md#AbiData)

[AbiFunction](mod_abi.md#AbiFunction)

[AbiContract](mod_abi.md#AbiContract)

[ParamsOfEncodeMessageBody](mod_abi.md#ParamsOfEncodeMessageBody)

[ResultOfEncodeMessageBody](mod_abi.md#ResultOfEncodeMessageBody)

[ParamsOfAttachSignatureToMessageBody](mod_abi.md#ParamsOfAttachSignatureToMessageBody)

[ResultOfAttachSignatureToMessageBody](mod_abi.md#ResultOfAttachSignatureToMessageBody)

[ParamsOfEncodeMessage](mod_abi.md#ParamsOfEncodeMessage)

[ResultOfEncodeMessage](mod_abi.md#ResultOfEncodeMessage)

[ParamsOfEncodeInternalMessage](mod_abi.md#ParamsOfEncodeInternalMessage)

[ResultOfEncodeInternalMessage](mod_abi.md#ResultOfEncodeInternalMessage)

[ParamsOfAttachSignature](mod_abi.md#ParamsOfAttachSignature)

[ResultOfAttachSignature](mod_abi.md#ResultOfAttachSignature)

[ParamsOfDecodeMessage](mod_abi.md#ParamsOfDecodeMessage)

[DecodedMessageBody](mod_abi.md#DecodedMessageBody)

[ParamsOfDecodeMessageBody](mod_abi.md#ParamsOfDecodeMessageBody)

[ParamsOfEncodeAccount](mod_abi.md#ParamsOfEncodeAccount)

[ResultOfEncodeAccount](mod_abi.md#ResultOfEncodeAccount)

[ParamsOfDecodeAccountData](mod_abi.md#ParamsOfDecodeAccountData)

[ResultOfDecodeData](mod_abi.md#ResultOfDecodeData)

## Functions

### encode_message_body

Encodes message body according to ABI function call.

```typescript
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number
}

type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
}

function encode_message_body(
    params: ParamsOfEncodeMessageBody,
): Promise<ResultOfEncodeMessageBody>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI.
*   `call_set`: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in non deploy message.\
    \
    In case of deploy message contains parameters of constructor.
* `is_internal`: _boolean_ – True if internal message body must be encoded.
* `signer`: [_Signer_](mod_abi.md#Signer) – Signing parameters.
*   `processing_try_index`?: _number_ – Processing try index.

    \
    Used in message processing with retries.\
    \
    Encoder uses the provided try index to calculate message\
    expiration time.\
    \
    Expiration timeouts will grow with every retry.\
    \
    Default value is 0.

#### Result

* `body`: _string_ – Message body BOC encoded with `base64`.
*   `data_to_sign`?: _string_ – Optional data to sign.

    \
    Encoded with `base64`. \
    Presents when `message` is unsigned. Can be used for external\
    message signing. Is this case you need to sing this data and\
    produce signed message using `abi.attach_signature`.

### attach_signature_to_message_body

```typescript
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}

type ResultOfAttachSignatureToMessageBody = {
    body: string
}

function attach_signature_to_message_body(
    params: ParamsOfAttachSignatureToMessageBody,
): Promise<ResultOfAttachSignatureToMessageBody>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
*   `public_key`: _string_ – Public key.

    \
    Must be encoded with `hex`.
*   `message`: _string_ – Unsigned message body BOC.

    \
    Must be encoded with `base64`.
*   `signature`: _string_ – Signature.

    \
    Must be encoded with `hex`.

#### Result

* `body`: _string_

### encode_message

Encodes an ABI-compatible message

Allows to encode deploy and function call messages, both signed and unsigned.

Use cases include messages of any possible type:

*   deploy with initial function call (i.e. `constructor` or any other function that is used for some kind

    of initialization);
* deploy without initial function call;
* signed/unsigned + data for signing.

`Signer` defines how the message should or shouldn't be signed:

`Signer::None` creates an unsigned message. This may be needed in case of some public methods, that do not require authorization by pubkey.

`Signer::External` takes public key and returns `data_to_sign` for later signing. Use `attach_signature` method with the result signature to get the signed message.

`Signer::Keys` creates a signed message with provided key pair.

\[SOON] `Signer::SigningBox` Allows using a special interface to implement signing without private key disclosure to SDK. For instance, in case of using a cold wallet or HSM, when application calls some API to sign data.

There is an optional public key can be provided in deploy set in order to substitute one in TVM file.

Public key resolving priority: 1. Public key from deploy set. 2. Public key, specified in TVM file. 3. Public key, provided by signer.

```typescript
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number
}

type ResultOfEncodeMessage = {
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
}

function encode_message(
    params: ParamsOfEncodeMessage,
): Promise<ResultOfEncodeMessage>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI.
*   `address`?: _string_ – Target address the message will be sent to.

    \
    Must be specified in case of non-deploy message.
*   `deploy_set`?: [_DeploySet_](mod_abi.md#DeploySet) – Deploy parameters.

    \
    Must be specified in case of deploy message.
*   `call_set`?: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in case of non-deploy message.\
    \
    In case of deploy message it is optional and contains parameters\
    of the functions that will to be called upon deploy transaction.
* `signer`: [_Signer_](mod_abi.md#Signer) – Signing parameters.
*   `processing_try_index`?: _number_ – Processing try index.

    \
    Used in message processing with retries (if contract's ABI includes "expire" header).\
    \
    Encoder uses the provided try index to calculate message\
    expiration time. The 1st message expiration time is specified in\
    Client config.\
    \
    Expiration timeouts will grow with every retry.\
    Retry grow factor is set in Client config:\
    <.....add config parameter with default value here>\
    \
    Default value is 0.

#### Result

* `message`: _string_ – Message BOC encoded with `base64`.
*   `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.

    \
    Returned in case of `Signer::External`. Can be used for external\
    message signing. Is this case you need to use this data to create signature and\
    then produce signed message using `abi.attach_signature`.
* `address`: _string_ – Destination address.
* `message_id`: _string_ – Message id.

### encode_internal_message

Encodes an internal ABI-compatible message

Allows to encode deploy and function call messages.

Use cases include messages of any possible type:

*   deploy with initial function call (i.e. `constructor` or any other function that is used for some kind

    of initialization);
* deploy without initial function call;
* simple function call

There is an optional public key can be provided in deploy set in order to substitute one in TVM file.

Public key resolving priority: 1. Public key from deploy set. 2. Public key, specified in TVM file.

```typescript
type ParamsOfEncodeInternalMessage = {
    abi?: Abi,
    address?: string,
    src_address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    value: string,
    bounce?: boolean,
    enable_ihr?: boolean
}

type ResultOfEncodeInternalMessage = {
    message: string,
    address: string,
    message_id: string
}

function encode_internal_message(
    params: ParamsOfEncodeInternalMessage,
): Promise<ResultOfEncodeInternalMessage>;
```

#### Parameters

*   `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI.

    \
    Can be None if both deploy_set and call_set are None.
*   `address`?: _string_ – Target address the message will be sent to.

    \
    Must be specified in case of non-deploy message.
* `src_address`?: _string_ – Source address of the message.
*   `deploy_set`?: [_DeploySet_](mod_abi.md#DeploySet) – Deploy parameters.

    \
    Must be specified in case of deploy message.
*   `call_set`?: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in case of non-deploy message.\
    \
    In case of deploy message it is optional and contains parameters\
    of the functions that will to be called upon deploy transaction.
* `value`: _string_ – Value in nanotokens to be sent with message.
*   `bounce`?: _boolean_ – Flag of bounceable message.

    \
    Default is true.
*   `enable_ihr`?: _boolean_ – Enable Instant Hypercube Routing for the message.

    \
    Default is false.

#### Result

* `message`: _string_ – Message BOC encoded with `base64`.
* `address`: _string_ – Destination address.
* `message_id`: _string_ – Message id.

### attach_signature

Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

```typescript
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}

type ResultOfAttachSignature = {
    message: string,
    message_id: string
}

function attach_signature(
    params: ParamsOfAttachSignature,
): Promise<ResultOfAttachSignature>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
* `public_key`: _string_ – Public key encoded in `hex`.
* `message`: _string_ – Unsigned message BOC encoded in `base64`.
* `signature`: _string_ – Signature encoded in `hex`.

#### Result

* `message`: _string_ – Signed message BOC
* `message_id`: _string_ – Message ID

### decode_message

Decodes message body using provided message BOC and ABI.

```typescript
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
}

type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
}

function decode_message(
    params: ParamsOfDecodeMessage,
): Promise<DecodedMessageBody>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – contract ABI
* `message`: _string_ – Message BOC

#### Result

* `body_type`: [_MessageBodyType_](mod_abi.md#MessageBodyType) – Type of the message body content.
* `name`: _string_ – Function or event name.
* `value`?: _any_ – Parameters or result value.
* `header`?: [_FunctionHeader_](mod_abi.md#FunctionHeader) – Function header.

### decode_message_body

Decodes message body using provided body BOC and ABI.

```typescript
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
}

type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
}

function decode_message_body(
    params: ParamsOfDecodeMessageBody,
): Promise<DecodedMessageBody>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI used to decode.
* `body`: _string_ – Message body BOC encoded in `base64`.
* `is_internal`: _boolean_ – True if the body belongs to the internal message.

#### Result

* `body_type`: [_MessageBodyType_](mod_abi.md#MessageBodyType) – Type of the message body content.
* `name`: _string_ – Function or event name.
* `value`?: _any_ – Parameters or result value.
* `header`?: [_FunctionHeader_](mod_abi.md#FunctionHeader) – Function header.

### encode_account

Creates account state BOC

Creates account state provided with one of these sets of data : 1. BOC of code, BOC of data, BOC of library 2. TVC (string in `base64`), keys, init params

```typescript
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: number,
    boc_cache?: BocCacheType
}

type ResultOfEncodeAccount = {
    account: string,
    id: string
}

function encode_account(
    params: ParamsOfEncodeAccount,
): Promise<ResultOfEncodeAccount>;
```

#### Parameters

* `state_init`: [_StateInitSource_](mod_abi.md#StateInitSource) – Source of the account state init.
* `balance`?: _bigint_ – Initial balance.
* `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
* `last_paid`?: _number_ – Initial value for the `last_paid`.
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided

#### Result

* `account`: _string_ – Account BOC encoded in `base64`.
* `id`: _string_ – Account ID  encoded in `hex`.

### decode_account_data

Decodes account data using provided data BOC and ABI.

Note: this feature requires ABI 2.1 or higher.

```typescript
type ParamsOfDecodeAccountData = {
    abi: Abi,
    data: string
}

type ResultOfDecodeData = {
    data: any
}

function decode_account_data(
    params: ParamsOfDecodeAccountData,
): Promise<ResultOfDecodeData>;
```

#### Parameters

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
* `data`: _string_ – Data BOC or BOC handle

#### Result

* `data`: _any_ – Decoded data as a JSON structure.

## Types

### AbiErrorCode

```typescript
enum AbiErrorCode {
    RequiredAddressMissingForEncodeMessage = 301,
    RequiredCallSetMissingForEncodeMessage = 302,
    InvalidJson = 303,
    InvalidMessage = 304,
    EncodeDeployMessageFailed = 305,
    EncodeRunMessageFailed = 306,
    AttachSignatureFailed = 307,
    InvalidTvcImage = 308,
    RequiredPublicKeyMissingForFunctionHeader = 309,
    InvalidSigner = 310,
    InvalidAbi = 311,
    InvalidFunctionId = 312,
    InvalidData = 313
}
```

One of the following value:

* `RequiredAddressMissingForEncodeMessage = 301`
* `RequiredCallSetMissingForEncodeMessage = 302`
* `InvalidJson = 303`
* `InvalidMessage = 304`
* `EncodeDeployMessageFailed = 305`
* `EncodeRunMessageFailed = 306`
* `AttachSignatureFailed = 307`
* `InvalidTvcImage = 308`
* `RequiredPublicKeyMissingForFunctionHeader = 309`
* `InvalidSigner = 310`
* `InvalidAbi = 311`
* `InvalidFunctionId = 312`
* `InvalidData = 313`

### Abi

```typescript
type Abi = {
    type: 'Contract'
    value: AbiContract
} | {
    type: 'Json'
    value: string
} | {
    type: 'Handle'
    value: AbiHandle
} | {
    type: 'Serialized'
    value: AbiContract
}
```

Depends on value of the `type` field.

When _type_ is _'Contract'_

* `value`: [_AbiContract_](mod_abi.md#AbiContract)

When _type_ is _'Json'_

* `value`: _string_

When _type_ is _'Handle'_

* `value`: [_AbiHandle_](mod_abi.md#AbiHandle)

When _type_ is _'Serialized'_

* `value`: [_AbiContract_](mod_abi.md#AbiContract)

Variant constructors:

```typescript
function abiContract(value: AbiContract): Abi;
function abiJson(value: string): Abi;
function abiHandle(value: AbiHandle): Abi;
function abiSerialized(value: AbiContract): Abi;
```

### AbiHandle

```typescript
type AbiHandle = number
```

### FunctionHeader

The ABI function header.

Includes several hidden function parameters that contract uses for security, message delivery monitoring and replay protection reasons.

The actual set of header fields depends on the contract's ABI. If a contract's ABI does not include some headers, then they are not filled.

```typescript
type FunctionHeader = {
    expire?: number,
    time?: bigint,
    pubkey?: string
}
```

* `expire`?: _number_ – Message expiration time in seconds. If not specified - calculated automatically from message_expiration_timeout(), try_index and message_expiration_timeout_grow_factor() (if ABI includes `expire` header).
*   `time`?: _bigint_ – Message creation time in milliseconds.

    \
    If not specified, `now` is used (if ABI includes `time` header).
*   `pubkey`?: _string_ – Public key is used by the contract to check the signature.

    \
    Encoded in `hex`. If not specified, method fails with exception (if ABI includes `pubkey` header)..

### CallSet

```typescript
type CallSet = {
    function_name: string,
    header?: FunctionHeader,
    input?: any
}
```

* `function_name`: _string_ – Function name that is being called. Or function id encoded as string in hex (starting with 0x).
*   `header`?: [_FunctionHeader_](mod_abi.md#FunctionHeader) – Function header.

    \
    If an application omits some header parameters required by the\
    contract's ABI, the library will set the default values for\
    them.
* `input`?: _any_ – Function input parameters according to ABI.

### DeploySet

```typescript
type DeploySet = {
    tvc: string,
    workchain_id?: number,
    initial_data?: any,
    initial_pubkey?: string
}
```

* `tvc`: _string_ – Content of TVC file encoded in `base64`.
*   `workchain_id`?: _number_ – Target workchain for destination address.

    \
    Default is `0`.
* `initial_data`?: _any_ – List of initial values for contract's public variables.
*   `initial_pubkey`?: _string_ – Optional public key that can be provided in deploy set in order to substitute one in TVM file or provided by Signer.

    \
    Public key resolving priority:\
    1\. Public key from deploy set.\
    2\. Public key, specified in TVM file.\
    3\. Public key, provided by Signer.

### Signer

```typescript
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
}
```

Depends on value of the `type` field.

When _type_ is _'None'_

No keys are provided.

Creates an unsigned message.

When _type_ is _'External'_

Only public key is provided in unprefixed hex string format to generate unsigned message and `data_to_sign` which can be signed later.

* `public_key`: _string_

When _type_ is _'Keys'_

Key pair is provided for signing

* `keys`: [_KeyPair_](mod_crypto.md#KeyPair)

When _type_ is _'SigningBox'_

Signing Box interface is provided for signing, allows Dapps to sign messages using external APIs, such as HSM, cold wallet, etc.

* `handle`: [_SigningBoxHandle_](mod_crypto.md#SigningBoxHandle)

Variant constructors:

```typescript
function signerNone(): Signer;
function signerExternal(public_key: string): Signer;
function signerKeys(keys: KeyPair): Signer;
function signerSigningBox(handle: SigningBoxHandle): Signer;
```

### MessageBodyType

```typescript
enum MessageBodyType {
    Input = "Input",
    Output = "Output",
    InternalOutput = "InternalOutput",
    Event = "Event"
}
```

One of the following value:

* `Input = "Input"` – Message contains the input of the ABI function.
* `Output = "Output"` – Message contains the output of the ABI function.
*   `InternalOutput = "InternalOutput"` – Message contains the input of the imported ABI function.

    \
    Occurs when contract sends an internal message to other\
    contract.
* `Event = "Event"` – Message contains the input of the ABI event.

### StateInitSource

```typescript
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
}
```

Depends on value of the `type` field.

When _type_ is _'Message'_

Deploy message.

* `source`: [_MessageSource_](mod_abi.md#MessageSource)

When _type_ is _'StateInit'_

State init data.

*   `code`: _string_ – Code BOC.

    \
    Encoded in `base64`.
*   `data`: _string_ – Data BOC.

    \
    Encoded in `base64`.
*   `library`?: _string_ – Library BOC.

    \
    Encoded in `base64`.

When _type_ is _'Tvc'_

Content of the TVC file.

Encoded in `base64`.

* `tvc`: _string_
* `public_key`?: _string_
* `init_params`?: [_StateInitParams_](mod_abi.md#StateInitParams)

Variant constructors:

```typescript
function stateInitSourceMessage(source: MessageSource): StateInitSource;
function stateInitSourceStateInit(code: string, data: string, library?: string): StateInitSource;
function stateInitSourceTvc(tvc: string, public_key?: string, init_params?: StateInitParams): StateInitSource;
```

### StateInitParams

```typescript
type StateInitParams = {
    abi: Abi,
    value: any
}
```

* `abi`: [_Abi_](mod_abi.md#Abi)
* `value`: _any_

### MessageSource

```typescript
type MessageSource = {
    type: 'Encoded'
    message: string,
    abi?: Abi
} | ({
    type: 'EncodingParams'
} & ParamsOfEncodeMessage)
```

Depends on value of the `type` field.

When _type_ is _'Encoded'_

* `message`: _string_
* `abi`?: [_Abi_](mod_abi.md#Abi)

When _type_ is _'EncodingParams'_

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI.
*   `address`?: _string_ – Target address the message will be sent to.

    \
    Must be specified in case of non-deploy message.
*   `deploy_set`?: [_DeploySet_](mod_abi.md#DeploySet) – Deploy parameters.

    \
    Must be specified in case of deploy message.
*   `call_set`?: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in case of non-deploy message.\
    \
    In case of deploy message it is optional and contains parameters\
    of the functions that will to be called upon deploy transaction.
* `signer`: [_Signer_](mod_abi.md#Signer) – Signing parameters.
*   `processing_try_index`?: _number_ – Processing try index.

    \
    Used in message processing with retries (if contract's ABI includes "expire" header).\
    \
    Encoder uses the provided try index to calculate message\
    expiration time. The 1st message expiration time is specified in\
    Client config.\
    \
    Expiration timeouts will grow with every retry.\
    Retry grow factor is set in Client config:\
    <.....add config parameter with default value here>\
    \
    Default value is 0.

Variant constructors:

```typescript
function messageSourceEncoded(message: string, abi?: Abi): MessageSource;
function messageSourceEncodingParams(params: ParamsOfEncodeMessage): MessageSource;
```

### AbiParam

```typescript
type AbiParam = {
    name: string,
    type: string,
    components?: AbiParam[]
}
```

* `name`: _string_
* `type`: _string_
* `components`?: [_AbiParam_](mod_abi.md#AbiParam)_\[]_

### AbiEvent

```typescript
type AbiEvent = {
    name: string,
    inputs: AbiParam[],
    id?: string | null
}
```

* `name`: _string_
* `inputs`: [_AbiParam_](mod_abi.md#AbiParam)_\[]_
* `id`?: _string?_

### AbiData

```typescript
type AbiData = {
    key: number,
    name: string,
    type: string,
    components?: AbiParam[]
}
```

* `key`: _number_
* `name`: _string_
* `type`: _string_
* `components`?: [_AbiParam_](mod_abi.md#AbiParam)_\[]_

### AbiFunction

```typescript
type AbiFunction = {
    name: string,
    inputs: AbiParam[],
    outputs: AbiParam[],
    id?: string | null
}
```

* `name`: _string_
* `inputs`: [_AbiParam_](mod_abi.md#AbiParam)_\[]_
* `outputs`: [_AbiParam_](mod_abi.md#AbiParam)_\[]_
* `id`?: _string?_

### AbiContract

```typescript
type AbiContract = {
    'ABI version'?: number,
    abi_version?: number,
    version?: string | null,
    header?: string[],
    functions?: AbiFunction[],
    events?: AbiEvent[],
    data?: AbiData[],
    fields?: AbiParam[]
}
```

* `ABI version`?: _number_
* `abi_version`?: _number_
* `version`?: _string?_
* `header`?: _string\[]_
* `functions`?: [_AbiFunction_](mod_abi.md#AbiFunction)_\[]_
* `events`?: [_AbiEvent_](mod_abi.md#AbiEvent)_\[]_
* `data`?: [_AbiData_](mod_abi.md#AbiData)_\[]_
* `fields`?: [_AbiParam_](mod_abi.md#AbiParam)_\[]_

### ParamsOfEncodeMessageBody

```typescript
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI.
*   `call_set`: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in non deploy message.\
    \
    In case of deploy message contains parameters of constructor.
* `is_internal`: _boolean_ – True if internal message body must be encoded.
* `signer`: [_Signer_](mod_abi.md#Signer) – Signing parameters.
*   `processing_try_index`?: _number_ – Processing try index.

    \
    Used in message processing with retries.\
    \
    Encoder uses the provided try index to calculate message\
    expiration time.\
    \
    Expiration timeouts will grow with every retry.\
    \
    Default value is 0.

### ResultOfEncodeMessageBody

```typescript
type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
}
```

* `body`: _string_ – Message body BOC encoded with `base64`.
*   `data_to_sign`?: _string_ – Optional data to sign.

    \
    Encoded with `base64`. \
    Presents when `message` is unsigned. Can be used for external\
    message signing. Is this case you need to sing this data and\
    produce signed message using `abi.attach_signature`.

### ParamsOfAttachSignatureToMessageBody

```typescript
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
*   `public_key`: _string_ – Public key.

    \
    Must be encoded with `hex`.
*   `message`: _string_ – Unsigned message body BOC.

    \
    Must be encoded with `base64`.
*   `signature`: _string_ – Signature.

    \
    Must be encoded with `hex`.

### ResultOfAttachSignatureToMessageBody

```typescript
type ResultOfAttachSignatureToMessageBody = {
    body: string
}
```

* `body`: _string_

### ParamsOfEncodeMessage

```typescript
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI.
*   `address`?: _string_ – Target address the message will be sent to.

    \
    Must be specified in case of non-deploy message.
*   `deploy_set`?: [_DeploySet_](mod_abi.md#DeploySet) – Deploy parameters.

    \
    Must be specified in case of deploy message.
*   `call_set`?: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in case of non-deploy message.\
    \
    In case of deploy message it is optional and contains parameters\
    of the functions that will to be called upon deploy transaction.
* `signer`: [_Signer_](mod_abi.md#Signer) – Signing parameters.
*   `processing_try_index`?: _number_ – Processing try index.

    \
    Used in message processing with retries (if contract's ABI includes "expire" header).\
    \
    Encoder uses the provided try index to calculate message\
    expiration time. The 1st message expiration time is specified in\
    Client config.\
    \
    Expiration timeouts will grow with every retry.\
    Retry grow factor is set in Client config:\
    <.....add config parameter with default value here>\
    \
    Default value is 0.

### ResultOfEncodeMessage

```typescript
type ResultOfEncodeMessage = {
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
}
```

* `message`: _string_ – Message BOC encoded with `base64`.
*   `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.

    \
    Returned in case of `Signer::External`. Can be used for external\
    message signing. Is this case you need to use this data to create signature and\
    then produce signed message using `abi.attach_signature`.
* `address`: _string_ – Destination address.
* `message_id`: _string_ – Message id.

### ParamsOfEncodeInternalMessage

```typescript
type ParamsOfEncodeInternalMessage = {
    abi?: Abi,
    address?: string,
    src_address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    value: string,
    bounce?: boolean,
    enable_ihr?: boolean
}
```

*   `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI.

    \
    Can be None if both deploy_set and call_set are None.
*   `address`?: _string_ – Target address the message will be sent to.

    \
    Must be specified in case of non-deploy message.
* `src_address`?: _string_ – Source address of the message.
*   `deploy_set`?: [_DeploySet_](mod_abi.md#DeploySet) – Deploy parameters.

    \
    Must be specified in case of deploy message.
*   `call_set`?: [_CallSet_](mod_abi.md#CallSet) – Function call parameters.

    \
    Must be specified in case of non-deploy message.\
    \
    In case of deploy message it is optional and contains parameters\
    of the functions that will to be called upon deploy transaction.
* `value`: _string_ – Value in nanotokens to be sent with message.
*   `bounce`?: _boolean_ – Flag of bounceable message.

    \
    Default is true.
*   `enable_ihr`?: _boolean_ – Enable Instant Hypercube Routing for the message.

    \
    Default is false.

### ResultOfEncodeInternalMessage

```typescript
type ResultOfEncodeInternalMessage = {
    message: string,
    address: string,
    message_id: string
}
```

* `message`: _string_ – Message BOC encoded with `base64`.
* `address`: _string_ – Destination address.
* `message_id`: _string_ – Message id.

### ParamsOfAttachSignature

```typescript
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
* `public_key`: _string_ – Public key encoded in `hex`.
* `message`: _string_ – Unsigned message BOC encoded in `base64`.
* `signature`: _string_ – Signature encoded in `hex`.

### ResultOfAttachSignature

```typescript
type ResultOfAttachSignature = {
    message: string,
    message_id: string
}
```

* `message`: _string_ – Signed message BOC
* `message_id`: _string_ – Message ID

### ParamsOfDecodeMessage

```typescript
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – contract ABI
* `message`: _string_ – Message BOC

### DecodedMessageBody

```typescript
type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
}
```

* `body_type`: [_MessageBodyType_](mod_abi.md#MessageBodyType) – Type of the message body content.
* `name`: _string_ – Function or event name.
* `value`?: _any_ – Parameters or result value.
* `header`?: [_FunctionHeader_](mod_abi.md#FunctionHeader) – Function header.

### ParamsOfDecodeMessageBody

```typescript
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI used to decode.
* `body`: _string_ – Message body BOC encoded in `base64`.
* `is_internal`: _boolean_ – True if the body belongs to the internal message.

### ParamsOfEncodeAccount

```typescript
type ParamsOfEncodeAccount = {
    state_init: StateInitSource,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: number,
    boc_cache?: BocCacheType
}
```

* `state_init`: [_StateInitSource_](mod_abi.md#StateInitSource) – Source of the account state init.
* `balance`?: _bigint_ – Initial balance.
* `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
* `last_paid`?: _number_ – Initial value for the `last_paid`.
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided

### ResultOfEncodeAccount

```typescript
type ResultOfEncodeAccount = {
    account: string,
    id: string
}
```

* `account`: _string_ – Account BOC encoded in `base64`.
* `id`: _string_ – Account ID  encoded in `hex`.

### ParamsOfDecodeAccountData

```typescript
type ParamsOfDecodeAccountData = {
    abi: Abi,
    data: string
}
```

* `abi`: [_Abi_](mod_abi.md#Abi) – Contract ABI
* `data`: _string_ – Data BOC or BOC handle

### ResultOfDecodeData

```typescript
type ResultOfDecodeData = {
    data: any
}
```

* `data`: _any_ – Decoded data as a JSON structure.

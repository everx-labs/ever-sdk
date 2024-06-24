# Module abi

Provides message encoding and decoding according to the ABI specification.


## Functions
[encode_message_body](mod\_abi.md#encode_message_body) – Encodes message body according to ABI function call.

[attach_signature_to_message_body](mod\_abi.md#attach_signature_to_message_body)

[encode_message](mod\_abi.md#encode_message) – Encodes an ABI-compatible message

[encode_internal_message](mod\_abi.md#encode_internal_message) – Encodes an internal ABI-compatible message

[attach_signature](mod\_abi.md#attach_signature) – Combines `hex`-encoded `signature` with `base64`-encoded `unsigned_message`. Returns signed message encoded in `base64`.

[decode_message](mod\_abi.md#decode_message) – Decodes message body using provided message BOC and ABI.

[decode_message_body](mod\_abi.md#decode_message_body) – Decodes message body using provided body BOC and ABI.

[encode_account](mod\_abi.md#encode_account) – Creates account state BOC

[decode_account_data](mod\_abi.md#decode_account_data) – Decodes account data using provided data BOC and ABI.

[update_initial_data](mod\_abi.md#update_initial_data) – Updates initial account data with initial values for the contract's static variables and owner's public key. This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

[encode_initial_data](mod\_abi.md#encode_initial_data) – Encodes initial account data with initial values for the contract's static variables and owner's public key into a data BOC that can be passed to `boc.encode_external_in_message` function afterwards.

[decode_initial_data](mod\_abi.md#decode_initial_data) – Decodes initial values of a contract's static variables and owner's public key from account initial data This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

[decode_boc](mod\_abi.md#decode_boc) – Decodes BOC into JSON as a set of provided parameters.

[encode_boc](mod\_abi.md#encode_boc) – Encodes given parameters in JSON into a BOC using param types from ABI.

[calc_function_id](mod\_abi.md#calc_function_id) – Calculates contract function ID by contract ABI

[get_signature_data](mod\_abi.md#get_signature_data) – Extracts signature from message body and calculates hash to verify the signature

## Types
[AbiErrorCode](mod\_abi.md#abierrorcode)

[AbiContractVariant](mod\_abi.md#abicontractvariant)

[AbiJsonVariant](mod\_abi.md#abijsonvariant)

[AbiHandleVariant](mod\_abi.md#abihandlevariant)

[AbiSerializedVariant](mod\_abi.md#abiserializedvariant)

[Abi](mod\_abi.md#abi)

[AbiHandle](mod\_abi.md#abihandle)

[FunctionHeader](mod\_abi.md#functionheader) – The ABI function header.

[CallSet](mod\_abi.md#callset)

[DeploySet](mod\_abi.md#deployset)

[SignerNoneVariant](mod\_abi.md#signernonevariant) – No keys are provided.

[SignerExternalVariant](mod\_abi.md#signerexternalvariant) – Only public key is provided in unprefixed hex string format to generate unsigned message and `data_to_sign` which can be signed later.

[SignerKeysVariant](mod\_abi.md#signerkeysvariant) – Key pair is provided for signing

[SignerSigningBoxVariant](mod\_abi.md#signersigningboxvariant) – Signing Box interface is provided for signing, allows Dapps to sign messages using external APIs, such as HSM, cold wallet, etc.

[Signer](mod\_abi.md#signer)

[MessageBodyType](mod\_abi.md#messagebodytype)

[AbiParam](mod\_abi.md#abiparam)

[AbiEvent](mod\_abi.md#abievent)

[AbiData](mod\_abi.md#abidata)

[AbiFunction](mod\_abi.md#abifunction)

[AbiContract](mod\_abi.md#abicontract)

[DataLayout](mod\_abi.md#datalayout)

[ParamsOfEncodeMessageBody](mod\_abi.md#paramsofencodemessagebody)

[ResultOfEncodeMessageBody](mod\_abi.md#resultofencodemessagebody)

[ParamsOfAttachSignatureToMessageBody](mod\_abi.md#paramsofattachsignaturetomessagebody)

[ResultOfAttachSignatureToMessageBody](mod\_abi.md#resultofattachsignaturetomessagebody)

[ParamsOfEncodeMessage](mod\_abi.md#paramsofencodemessage)

[ResultOfEncodeMessage](mod\_abi.md#resultofencodemessage)

[ParamsOfEncodeInternalMessage](mod\_abi.md#paramsofencodeinternalmessage)

[ResultOfEncodeInternalMessage](mod\_abi.md#resultofencodeinternalmessage)

[ParamsOfAttachSignature](mod\_abi.md#paramsofattachsignature)

[ResultOfAttachSignature](mod\_abi.md#resultofattachsignature)

[ParamsOfDecodeMessage](mod\_abi.md#paramsofdecodemessage)

[DecodedMessageBody](mod\_abi.md#decodedmessagebody)

[ParamsOfDecodeMessageBody](mod\_abi.md#paramsofdecodemessagebody)

[ParamsOfEncodeAccount](mod\_abi.md#paramsofencodeaccount)

[ResultOfEncodeAccount](mod\_abi.md#resultofencodeaccount)

[ParamsOfDecodeAccountData](mod\_abi.md#paramsofdecodeaccountdata)

[ResultOfDecodeAccountData](mod\_abi.md#resultofdecodeaccountdata)

[ParamsOfUpdateInitialData](mod\_abi.md#paramsofupdateinitialdata)

[ResultOfUpdateInitialData](mod\_abi.md#resultofupdateinitialdata)

[ParamsOfEncodeInitialData](mod\_abi.md#paramsofencodeinitialdata)

[ResultOfEncodeInitialData](mod\_abi.md#resultofencodeinitialdata)

[ParamsOfDecodeInitialData](mod\_abi.md#paramsofdecodeinitialdata)

[ResultOfDecodeInitialData](mod\_abi.md#resultofdecodeinitialdata)

[ParamsOfDecodeBoc](mod\_abi.md#paramsofdecodeboc)

[ResultOfDecodeBoc](mod\_abi.md#resultofdecodeboc)

[ParamsOfAbiEncodeBoc](mod\_abi.md#paramsofabiencodeboc)

[ResultOfAbiEncodeBoc](mod\_abi.md#resultofabiencodeboc)

[ParamsOfCalcFunctionId](mod\_abi.md#paramsofcalcfunctionid)

[ResultOfCalcFunctionId](mod\_abi.md#resultofcalcfunctionid)

[ParamsOfGetSignatureData](mod\_abi.md#paramsofgetsignaturedata)

[ResultOfGetSignatureData](mod\_abi.md#resultofgetsignaturedata)


# Functions
## encode_message_body

Encodes message body according to ABI function call.

```ts
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number,
    address?: string,
    signature_id?: number
}

type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
}

function encode_message_body(
    params: ParamsOfEncodeMessageBody,
): Promise<ResultOfEncodeMessageBody>;

function encode_message_body_sync(
    params: ParamsOfEncodeMessageBody,
): ResultOfEncodeMessageBody;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `call_set`: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in non deploy message.<br><br>In case of deploy message contains parameters of constructor.
- `is_internal`: _boolean_ – True if internal message body must be encoded.
- `signer`: _[Signer](mod\_abi.md#signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries.<br><br>Encoder uses the provided try index to calculate message<br>expiration time.<br><br>Expiration timeouts will grow with every retry.<br><br>Default value is 0.
- `address`?: _string_ – Destination address of the message
<br>Since ABI version 2.3 destination address of external inbound message is used in message<br>body signature calculation. Should be provided when signed external inbound message body is<br>created. Otherwise can be omitted.
- `signature_id`?: _number_ – Signature ID to be used in data to sign preparing when CapSignatureWithId capability is enabled


### Result

- `body`: _string_ – Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to sign.
<br>Encoded with `base64`. <br>Presents when `message` is unsigned. Can be used for external<br>message signing. Is this case you need to sing this data and<br>produce signed message using `abi.attach_signature`.


## attach_signature_to_message_body

```ts
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

function attach_signature_to_message_body_sync(
    params: ParamsOfAttachSignatureToMessageBody,
): ResultOfAttachSignatureToMessageBody;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `public_key`: _string_ – Public key.
<br>Must be encoded with `hex`.
- `message`: _string_ – Unsigned message body BOC.
<br>Must be encoded with `base64`.
- `signature`: _string_ – Signature.
<br>Must be encoded with `hex`.


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

[SOON] `Signer::SigningBox` Allows using a special interface to implement signing
without private key disclosure to SDK. For instance, in case of using a cold wallet or HSM,
when application calls some API to sign data.

There is an optional public key can be provided in deploy set in order to substitute one
in TVM file.

Public key resolving priority:
1. Public key from deploy set.
2. Public key, specified in TVM file.
3. Public key, provided by signer.

```ts
type ParamsOfEncodeMessage = {
    abi: Abi,
    address?: string,
    deploy_set?: DeploySet,
    call_set?: CallSet,
    signer: Signer,
    processing_try_index?: number,
    signature_id?: number
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

function encode_message_sync(
    params: ParamsOfEncodeMessage,
): ResultOfEncodeMessage;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `deploy_set`?: _[DeploySet](mod\_abi.md#deployset)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `signer`: _[Signer](mod\_abi.md#signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries (if contract's ABI includes "expire" header).<br><br>Encoder uses the provided try index to calculate message<br>expiration time. The 1st message expiration time is specified in<br>Client config.<br><br>Expiration timeouts will grow with every retry.<br>Retry grow factor is set in Client config:<br><.....add config parameter with default value here><br><br>Default value is 0.
- `signature_id`?: _number_ – Signature ID to be used in data to sign preparing when CapSignatureWithId capability is enabled


### Result

- `message`: _string_ – Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.
<br>Returned in case of `Signer::External`. Can be used for external<br>message signing. Is this case you need to use this data to create signature and<br>then produce signed message using `abi.attach_signature`.
- `address`: _string_ – Destination address.
- `message_id`: _string_ – Message id.


## encode_internal_message

Encodes an internal ABI-compatible message

Allows to encode deploy and function call messages.

Use cases include messages of any possible type:
- deploy with initial function call (i.e. `constructor` or any other function that is used for some kind
of initialization);
- deploy without initial function call;
- simple function call

There is an optional public key can be provided in deploy set in order to substitute one
in TVM file.

Public key resolving priority:
1. Public key from deploy set.
2. Public key, specified in TVM file.

```ts
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

function encode_internal_message_sync(
    params: ParamsOfEncodeInternalMessage,
): ResultOfEncodeInternalMessage;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
<br>Can be None if both deploy_set and call_set are None.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `src_address`?: _string_ – Source address of the message.
- `deploy_set`?: _[DeploySet](mod\_abi.md#deployset)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `value`: _string_ – Value in nanotokens to be sent with message.
- `bounce`?: _boolean_ – Flag of bounceable message.
<br>Default is true.
- `enable_ihr`?: _boolean_ – Enable Instant Hypercube Routing for the message.
<br>Default is false.


### Result

- `message`: _string_ – Message BOC encoded with `base64`.
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
}

type ResultOfAttachSignature = {
    message: string,
    message_id: string
}

function attach_signature(
    params: ParamsOfAttachSignature,
): Promise<ResultOfAttachSignature>;

function attach_signature_sync(
    params: ParamsOfAttachSignature,
): ResultOfAttachSignature;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
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
    message: string,
    allow_partial?: boolean,
    function_name?: string,
    data_layout?: DataLayout
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

function decode_message_sync(
    params: ParamsOfDecodeMessage,
): DecodedMessageBody;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – contract ABI
- `message`: _string_ – Message BOC
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)
- `function_name`?: _string_ – Function name or function id if is known in advance
- `data_layout`?: _[DataLayout](mod\_abi.md#datalayout)_


### Result

- `body_type`: _[MessageBodyType](mod\_abi.md#messagebodytype)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod\_abi.md#functionheader)_ – Function header.


## decode_message_body

Decodes message body using provided body BOC and ABI.

```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean,
    allow_partial?: boolean,
    function_name?: string,
    data_layout?: DataLayout
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

function decode_message_body_sync(
    params: ParamsOfDecodeMessageBody,
): DecodedMessageBody;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI used to decode.
- `body`: _string_ – Message body BOC encoded in `base64`.
- `is_internal`: _boolean_ – True if the body belongs to the internal message.
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)
- `function_name`?: _string_ – Function name or function id if is known in advance
- `data_layout`?: _[DataLayout](mod\_abi.md#datalayout)_


### Result

- `body_type`: _[MessageBodyType](mod\_abi.md#messagebodytype)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod\_abi.md#functionheader)_ – Function header.


## encode_account

Creates account state BOC

```ts
type ParamsOfEncodeAccount = {
    state_init: string,
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

function encode_account_sync(
    params: ParamsOfEncodeAccount,
): ResultOfEncodeAccount;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `state_init`: _string_ – Account state init.
- `balance`?: _bigint_ – Initial balance.
- `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ – Initial value for the `last_paid`.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


### Result

- `account`: _string_ – Account BOC encoded in `base64`.
- `id`: _string_ – Account ID  encoded in `hex`.


## decode_account_data

Decodes account data using provided data BOC and ABI.

Note: this feature requires ABI 2.1 or higher.

```ts
type ParamsOfDecodeAccountData = {
    abi: Abi,
    data: string,
    allow_partial?: boolean
}

type ResultOfDecodeAccountData = {
    data: any
}

function decode_account_data(
    params: ParamsOfDecodeAccountData,
): Promise<ResultOfDecodeAccountData>;

function decode_account_data_sync(
    params: ParamsOfDecodeAccountData,
): ResultOfDecodeAccountData;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `data`: _string_ – Data BOC or BOC handle
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)


### Result

- `data`: _any_ – Decoded data as a JSON structure.


## update_initial_data

Updates initial account data with initial values for the contract's static variables and owner's public key. This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

Doesn't support ABI version >= 2.4. Use `encode_initial_data` instead

```ts
type ParamsOfUpdateInitialData = {
    abi: Abi,
    data: string,
    initial_data?: any,
    initial_pubkey?: string,
    boc_cache?: BocCacheType
}

type ResultOfUpdateInitialData = {
    data: string
}

function update_initial_data(
    params: ParamsOfUpdateInitialData,
): Promise<ResultOfUpdateInitialData>;

function update_initial_data_sync(
    params: ParamsOfUpdateInitialData,
): ResultOfUpdateInitialData;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `data`: _string_ – Data BOC or BOC handle
- `initial_data`?: _any_ – List of initial values for contract's static variables.
<br>`abi` parameter should be provided to set initial data
- `initial_pubkey`?: _string_ – Initial account owner's public key to set into account data
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `data`: _string_ – Updated data BOC or BOC handle


## encode_initial_data

Encodes initial account data with initial values for the contract's static variables and owner's public key into a data BOC that can be passed to `boc.encode_external_in_message` function afterwards.

This function is analogue of `tvm.buildDataInit` function in Solidity.

```ts
type ParamsOfEncodeInitialData = {
    abi: Abi,
    initial_data?: any,
    initial_pubkey?: string,
    boc_cache?: BocCacheType
}

type ResultOfEncodeInitialData = {
    data: string
}

function encode_initial_data(
    params: ParamsOfEncodeInitialData,
): Promise<ResultOfEncodeInitialData>;

function encode_initial_data_sync(
    params: ParamsOfEncodeInitialData,
): ResultOfEncodeInitialData;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `initial_data`?: _any_ – List of initial values for contract's static variables.
<br>`abi` parameter should be provided to set initial data
- `initial_pubkey`?: _string_ – Initial account owner's public key to set into account data
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `data`: _string_ – Updated data BOC or BOC handle


## decode_initial_data

Decodes initial values of a contract's static variables and owner's public key from account initial data This operation is applicable only for initial account data (before deploy). If the contract is already deployed, its data doesn't contain this data section any more.

Doesn't support ABI version >= 2.4. Use `decode_account_data` instead

```ts
type ParamsOfDecodeInitialData = {
    abi: Abi,
    data: string,
    allow_partial?: boolean
}

type ResultOfDecodeInitialData = {
    initial_data: any,
    initial_pubkey: string
}

function decode_initial_data(
    params: ParamsOfDecodeInitialData,
): Promise<ResultOfDecodeInitialData>;

function decode_initial_data_sync(
    params: ParamsOfDecodeInitialData,
): ResultOfDecodeInitialData;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
<br>Initial data is decoded if this parameter is provided
- `data`: _string_ – Data BOC or BOC handle
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)


### Result

- `initial_data`: _any_ – List of initial values of contract's public variables.
<br>Initial data is decoded if `abi` input parameter is provided
- `initial_pubkey`: _string_ – Initial account owner's public key


## decode_boc

Decodes BOC into JSON as a set of provided parameters.

Solidity functions use ABI types for [builder encoding](https://github.com/everx-labs/TON-Solidity-Compiler/blob/master/API.md#tvmbuilderstore).
The simplest way to decode such a BOC is to use ABI decoding.
ABI has it own rules for fields layout in cells so manually encoded
BOC can not be described in terms of ABI rules.

To solve this problem we introduce a new ABI type `Ref(<ParamType>)`
which allows to store `ParamType` ABI parameter in cell reference and, thus,
decode manually encoded BOCs. This type is available only in `decode_boc` function
and will not be available in ABI messages encoding until it is included into some ABI revision.

Such BOC descriptions covers most users needs. If someone wants to decode some BOC which
can not be described by these rules (i.e. BOC with TLB containing constructors of flags
defining some parsing conditions) then they can decode the fields up to fork condition,
check the parsed data manually, expand the parsing schema and then decode the whole BOC
with the full schema.

```ts
type ParamsOfDecodeBoc = {
    params: AbiParam[],
    boc: string,
    allow_partial: boolean
}

type ResultOfDecodeBoc = {
    data: any
}

function decode_boc(
    params: ParamsOfDecodeBoc,
): Promise<ResultOfDecodeBoc>;

function decode_boc_sync(
    params: ParamsOfDecodeBoc,
): ResultOfDecodeBoc;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `params`: _[AbiParam](mod\_abi.md#abiparam)[]_ – Parameters to decode from BOC
- `boc`: _string_ – Data BOC or BOC handle
- `allow_partial`: _boolean_


### Result

- `data`: _any_ – Decoded data as a JSON structure.


## encode_boc

Encodes given parameters in JSON into a BOC using param types from ABI.

```ts
type ParamsOfAbiEncodeBoc = {
    params: AbiParam[],
    data: any,
    boc_cache?: BocCacheType
}

type ResultOfAbiEncodeBoc = {
    boc: string
}

function encode_boc(
    params: ParamsOfAbiEncodeBoc,
): Promise<ResultOfAbiEncodeBoc>;

function encode_boc_sync(
    params: ParamsOfAbiEncodeBoc,
): ResultOfAbiEncodeBoc;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `params`: _[AbiParam](mod\_abi.md#abiparam)[]_ – Parameters to encode into BOC
- `data`: _any_ – Parameters and values as a JSON structure
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


### Result

- `boc`: _string_ – BOC encoded as base64


## calc_function_id

Calculates contract function ID by contract ABI

```ts
type ParamsOfCalcFunctionId = {
    abi: Abi,
    function_name: string,
    output?: boolean
}

type ResultOfCalcFunctionId = {
    function_id: number
}

function calc_function_id(
    params: ParamsOfCalcFunctionId,
): Promise<ResultOfCalcFunctionId>;

function calc_function_id_sync(
    params: ParamsOfCalcFunctionId,
): ResultOfCalcFunctionId;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `function_name`: _string_ – Contract function name
- `output`?: _boolean_ – If set to `true` output function ID will be returned which is used in contract response. Default is `false`


### Result

- `function_id`: _number_ – Contract function ID


## get_signature_data

Extracts signature from message body and calculates hash to verify the signature

```ts
type ParamsOfGetSignatureData = {
    abi: Abi,
    message: string,
    signature_id?: number
}

type ResultOfGetSignatureData = {
    signature: string,
    unsigned: string
}

function get_signature_data(
    params: ParamsOfGetSignatureData,
): Promise<ResultOfGetSignatureData>;

function get_signature_data_sync(
    params: ParamsOfGetSignatureData,
): ResultOfGetSignatureData;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI used to decode.
- `message`: _string_ – Message BOC encoded in `base64`.
- `signature_id`?: _number_ – Signature ID to be used in unsigned data preparing when CapSignatureWithId capability is enabled


### Result

- `signature`: _string_ – Signature from the message in `hex`.
- `unsigned`: _string_ – Data to verify the signature in `base64`.


# Types
## AbiErrorCode
```ts
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
    InvalidData = 313,
    EncodeInitialDataFailed = 314,
    InvalidFunctionName = 315,
    PubKeyNotSupported = 316
}
```
One of the following value:

- `RequiredAddressMissingForEncodeMessage = 301`
- `RequiredCallSetMissingForEncodeMessage = 302`
- `InvalidJson = 303`
- `InvalidMessage = 304`
- `EncodeDeployMessageFailed = 305`
- `EncodeRunMessageFailed = 306`
- `AttachSignatureFailed = 307`
- `InvalidTvcImage = 308`
- `RequiredPublicKeyMissingForFunctionHeader = 309`
- `InvalidSigner = 310`
- `InvalidAbi = 311`
- `InvalidFunctionId = 312`
- `InvalidData = 313`
- `EncodeInitialDataFailed = 314`
- `InvalidFunctionName = 315`
- `PubKeyNotSupported = 316`


## AbiContractVariant
```ts
type AbiContractVariant = {
    value: AbiContract
}
```
- `value`: _[AbiContract](mod\_abi.md#abicontract)_


## AbiJsonVariant
```ts
type AbiJsonVariant = {
    value: string
}
```
- `value`: _string_


## AbiHandleVariant
```ts
type AbiHandleVariant = {
    value: AbiHandle
}
```
- `value`: _[AbiHandle](mod\_abi.md#abihandle)_


## AbiSerializedVariant
```ts
type AbiSerializedVariant = {
    value: AbiContract
}
```
- `value`: _[AbiContract](mod\_abi.md#abicontract)_


## Abi
```ts
type Abi = ({
    type: 'Contract'
} & AbiContractVariant) | ({
    type: 'Json'
} & AbiJsonVariant) | ({
    type: 'Handle'
} & AbiHandleVariant) | ({
    type: 'Serialized'
} & AbiSerializedVariant)
```
Depends on value of the  `type` field.

When _type_ is _'Contract'_

- `value`: _[AbiContract](mod\_abi.md#abicontract)_

When _type_ is _'Json'_

- `value`: _string_

When _type_ is _'Handle'_

- `value`: _[AbiHandle](mod\_abi.md#abihandle)_

When _type_ is _'Serialized'_

- `value`: _[AbiContract](mod\_abi.md#abicontract)_


Variant constructors:

```ts
function abiContract(value: AbiContract): Abi;
function abiJson(value: string): Abi;
function abiHandle(value: AbiHandle): Abi;
function abiSerialized(value: AbiContract): Abi;
```

## AbiHandle
```ts
type AbiHandle = number
```


## FunctionHeader
The ABI function header.

Includes several hidden function parameters that contract
uses for security, message delivery monitoring and replay protection reasons.

The actual set of header fields depends on the contract's ABI.
If a contract's ABI does not include some headers, then they are not filled.

```ts
type FunctionHeader = {
    expire?: number,
    time?: bigint,
    pubkey?: string
}
```
- `expire`?: _number_ – Message expiration timestamp (UNIX time) in seconds.
<br>If not specified - calculated automatically from message_expiration_timeout(),<br>try_index and message_expiration_timeout_grow_factor() (if ABI includes `expire` header).
- `time`?: _bigint_ – Message creation time in milliseconds.
<br>If not specified, `now` is used (if ABI includes `time` header).
- `pubkey`?: _string_ – Public key is used by the contract to check the signature.
<br>Encoded in `hex`. If not specified, method fails with exception (if ABI includes `pubkey` header)..


## CallSet
```ts
type CallSet = {
    function_name: string,
    header?: FunctionHeader,
    input?: any
}
```
- `function_name`: _string_ – Function name that is being called. Or function id encoded as string in hex (starting with 0x).
- `header`?: _[FunctionHeader](mod\_abi.md#functionheader)_ – Function header.
<br>If an application omits some header parameters required by the<br>contract's ABI, the library will set the default values for<br>them.
- `input`?: _any_ – Function input parameters according to ABI.


## DeploySet
```ts
type DeploySet = {
    tvc?: string,
    code?: string,
    state_init?: string,
    workchain_id?: number,
    initial_data?: any,
    initial_pubkey?: string
}
```
- `tvc`?: _string_ – Content of TVC file encoded in `base64`. For compatibility reason this field can contain an encoded  `StateInit`.
- `code`?: _string_ – Contract code BOC encoded with base64.
- `state_init`?: _string_ – State init BOC encoded with base64.
- `workchain_id`?: _number_ – Target workchain for destination address.
<br>Default is `0`.
- `initial_data`?: _any_ – List of initial values for contract's public variables.
- `initial_pubkey`?: _string_ – Optional public key that can be provided in deploy set in order to substitute one in TVM file or provided by Signer.
<br>Public key resolving priority:<br>1. Public key from deploy set.<br>2. Public key, specified in TVM file.<br>3. Public key, provided by Signer.<br><br>Applicable only for contracts with ABI version < 2.4. Contract initial public key should be<br>explicitly provided inside `initial_data` since ABI 2.4


## SignerNoneVariant
No keys are provided.

Creates an unsigned message.

```ts
type SignerNoneVariant = {

}
```


## SignerExternalVariant
Only public key is provided in unprefixed hex string format to generate unsigned message and `data_to_sign` which can be signed later.

```ts
type SignerExternalVariant = {
    public_key: string
}
```
- `public_key`: _string_


## SignerKeysVariant
Key pair is provided for signing

```ts
type SignerKeysVariant = {
    keys: KeyPair
}
```
- `keys`: _[KeyPair](mod\_crypto.md#keypair)_


## SignerSigningBoxVariant
Signing Box interface is provided for signing, allows Dapps to sign messages using external APIs, such as HSM, cold wallet, etc.

```ts
type SignerSigningBoxVariant = {
    handle: SigningBoxHandle
}
```
- `handle`: _[SigningBoxHandle](mod\_crypto.md#signingboxhandle)_


## Signer
```ts
type Signer = ({
    type: 'None'
} & SignerNoneVariant) | ({
    type: 'External'
} & SignerExternalVariant) | ({
    type: 'Keys'
} & SignerKeysVariant) | ({
    type: 'SigningBox'
} & SignerSigningBoxVariant)
```
Depends on value of the  `type` field.

When _type_ is _'None'_

No keys are provided.

Creates an unsigned message.


When _type_ is _'External'_

Only public key is provided in unprefixed hex string format to generate unsigned message and `data_to_sign` which can be signed later.

- `public_key`: _string_

When _type_ is _'Keys'_

Key pair is provided for signing

- `keys`: _[KeyPair](mod\_crypto.md#keypair)_

When _type_ is _'SigningBox'_

Signing Box interface is provided for signing, allows Dapps to sign messages using external APIs, such as HSM, cold wallet, etc.

- `handle`: _[SigningBoxHandle](mod\_crypto.md#signingboxhandle)_


Variant constructors:

```ts
function signerNone(): Signer;
function signerExternal(public_key: string): Signer;
function signerKeys(keys: KeyPair): Signer;
function signerSigningBox(handle: SigningBoxHandle): Signer;
```

## MessageBodyType
```ts
enum MessageBodyType {
    Input = "Input",
    Output = "Output",
    InternalOutput = "InternalOutput",
    Event = "Event"
}
```
One of the following value:

- `Input = "Input"` – Message contains the input of the ABI function.
- `Output = "Output"` – Message contains the output of the ABI function.
- `InternalOutput = "InternalOutput"` – Message contains the input of the imported ABI function.
<br>Occurs when contract sends an internal message to other<br>contract.
- `Event = "Event"` – Message contains the input of the ABI event.


## AbiParam
```ts
type AbiParam = {
    name: string,
    type: string,
    components?: AbiParam[],
    init?: boolean
}
```
- `name`: _string_
- `type`: _string_
- `components`?: _[AbiParam](mod\_abi.md#abiparam)[]_
- `init`?: _boolean_


## AbiEvent
```ts
type AbiEvent = {
    name: string,
    inputs: AbiParam[],
    id?: string | null
}
```
- `name`: _string_
- `inputs`: _[AbiParam](mod\_abi.md#abiparam)[]_
- `id`?: _string?_


## AbiData
```ts
type AbiData = {
    key: number,
    name: string,
    type: string,
    components?: AbiParam[]
}
```
- `key`: _number_
- `name`: _string_
- `type`: _string_
- `components`?: _[AbiParam](mod\_abi.md#abiparam)[]_


## AbiFunction
```ts
type AbiFunction = {
    name: string,
    inputs: AbiParam[],
    outputs: AbiParam[],
    id?: string | null
}
```
- `name`: _string_
- `inputs`: _[AbiParam](mod\_abi.md#abiparam)[]_
- `outputs`: _[AbiParam](mod\_abi.md#abiparam)[]_
- `id`?: _string?_


## AbiContract
```ts
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
- `ABI version`?: _number_
- `abi_version`?: _number_
- `version`?: _string?_
- `header`?: _string[]_
- `functions`?: _[AbiFunction](mod\_abi.md#abifunction)[]_
- `events`?: _[AbiEvent](mod\_abi.md#abievent)[]_
- `data`?: _[AbiData](mod\_abi.md#abidata)[]_
- `fields`?: _[AbiParam](mod\_abi.md#abiparam)[]_


## DataLayout
```ts
enum DataLayout {
    Input = "Input",
    Output = "Output"
}
```
One of the following value:

- `Input = "Input"` – Decode message body as function input parameters.
- `Output = "Output"` – Decode message body as function output.


## ParamsOfEncodeMessageBody
```ts
type ParamsOfEncodeMessageBody = {
    abi: Abi,
    call_set: CallSet,
    is_internal: boolean,
    signer: Signer,
    processing_try_index?: number,
    address?: string,
    signature_id?: number
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `call_set`: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in non deploy message.<br><br>In case of deploy message contains parameters of constructor.
- `is_internal`: _boolean_ – True if internal message body must be encoded.
- `signer`: _[Signer](mod\_abi.md#signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries.<br><br>Encoder uses the provided try index to calculate message<br>expiration time.<br><br>Expiration timeouts will grow with every retry.<br><br>Default value is 0.
- `address`?: _string_ – Destination address of the message
<br>Since ABI version 2.3 destination address of external inbound message is used in message<br>body signature calculation. Should be provided when signed external inbound message body is<br>created. Otherwise can be omitted.
- `signature_id`?: _number_ – Signature ID to be used in data to sign preparing when CapSignatureWithId capability is enabled


## ResultOfEncodeMessageBody
```ts
type ResultOfEncodeMessageBody = {
    body: string,
    data_to_sign?: string
}
```
- `body`: _string_ – Message body BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to sign.
<br>Encoded with `base64`. <br>Presents when `message` is unsigned. Can be used for external<br>message signing. Is this case you need to sing this data and<br>produce signed message using `abi.attach_signature`.


## ParamsOfAttachSignatureToMessageBody
```ts
type ParamsOfAttachSignatureToMessageBody = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `public_key`: _string_ – Public key.
<br>Must be encoded with `hex`.
- `message`: _string_ – Unsigned message body BOC.
<br>Must be encoded with `base64`.
- `signature`: _string_ – Signature.
<br>Must be encoded with `hex`.


## ResultOfAttachSignatureToMessageBody
```ts
type ResultOfAttachSignatureToMessageBody = {
    body: string
}
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
    processing_try_index?: number,
    signature_id?: number
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `deploy_set`?: _[DeploySet](mod\_abi.md#deployset)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `signer`: _[Signer](mod\_abi.md#signer)_ – Signing parameters.
- `processing_try_index`?: _number_ – Processing try index.
<br>Used in message processing with retries (if contract's ABI includes "expire" header).<br><br>Encoder uses the provided try index to calculate message<br>expiration time. The 1st message expiration time is specified in<br>Client config.<br><br>Expiration timeouts will grow with every retry.<br>Retry grow factor is set in Client config:<br><.....add config parameter with default value here><br><br>Default value is 0.
- `signature_id`?: _number_ – Signature ID to be used in data to sign preparing when CapSignatureWithId capability is enabled


## ResultOfEncodeMessage
```ts
type ResultOfEncodeMessage = {
    message: string,
    data_to_sign?: string,
    address: string,
    message_id: string
}
```
- `message`: _string_ – Message BOC encoded with `base64`.
- `data_to_sign`?: _string_ – Optional data to be signed encoded in `base64`.
<br>Returned in case of `Signer::External`. Can be used for external<br>message signing. Is this case you need to use this data to create signature and<br>then produce signed message using `abi.attach_signature`.
- `address`: _string_ – Destination address.
- `message_id`: _string_ – Message id.


## ParamsOfEncodeInternalMessage
```ts
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
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
<br>Can be None if both deploy_set and call_set are None.
- `address`?: _string_ – Target address the message will be sent to.
<br>Must be specified in case of non-deploy message.
- `src_address`?: _string_ – Source address of the message.
- `deploy_set`?: _[DeploySet](mod\_abi.md#deployset)_ – Deploy parameters.
<br>Must be specified in case of deploy message.
- `call_set`?: _[CallSet](mod\_abi.md#callset)_ – Function call parameters.
<br>Must be specified in case of non-deploy message.<br><br>In case of deploy message it is optional and contains parameters<br>of the functions that will to be called upon deploy transaction.
- `value`: _string_ – Value in nanotokens to be sent with message.
- `bounce`?: _boolean_ – Flag of bounceable message.
<br>Default is true.
- `enable_ihr`?: _boolean_ – Enable Instant Hypercube Routing for the message.
<br>Default is false.


## ResultOfEncodeInternalMessage
```ts
type ResultOfEncodeInternalMessage = {
    message: string,
    address: string,
    message_id: string
}
```
- `message`: _string_ – Message BOC encoded with `base64`.
- `address`: _string_ – Destination address.
- `message_id`: _string_ – Message id.


## ParamsOfAttachSignature
```ts
type ParamsOfAttachSignature = {
    abi: Abi,
    public_key: string,
    message: string,
    signature: string
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `public_key`: _string_ – Public key encoded in `hex`.
- `message`: _string_ – Unsigned message BOC encoded in `base64`.
- `signature`: _string_ – Signature encoded in `hex`.


## ResultOfAttachSignature
```ts
type ResultOfAttachSignature = {
    message: string,
    message_id: string
}
```
- `message`: _string_ – Signed message BOC
- `message_id`: _string_ – Message ID


## ParamsOfDecodeMessage
```ts
type ParamsOfDecodeMessage = {
    abi: Abi,
    message: string,
    allow_partial?: boolean,
    function_name?: string,
    data_layout?: DataLayout
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – contract ABI
- `message`: _string_ – Message BOC
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)
- `function_name`?: _string_ – Function name or function id if is known in advance
- `data_layout`?: _[DataLayout](mod\_abi.md#datalayout)_


## DecodedMessageBody
```ts
type DecodedMessageBody = {
    body_type: MessageBodyType,
    name: string,
    value?: any,
    header?: FunctionHeader
}
```
- `body_type`: _[MessageBodyType](mod\_abi.md#messagebodytype)_ – Type of the message body content.
- `name`: _string_ – Function or event name.
- `value`?: _any_ – Parameters or result value.
- `header`?: _[FunctionHeader](mod\_abi.md#functionheader)_ – Function header.


## ParamsOfDecodeMessageBody
```ts
type ParamsOfDecodeMessageBody = {
    abi: Abi,
    body: string,
    is_internal: boolean,
    allow_partial?: boolean,
    function_name?: string,
    data_layout?: DataLayout
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI used to decode.
- `body`: _string_ – Message body BOC encoded in `base64`.
- `is_internal`: _boolean_ – True if the body belongs to the internal message.
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)
- `function_name`?: _string_ – Function name or function id if is known in advance
- `data_layout`?: _[DataLayout](mod\_abi.md#datalayout)_


## ParamsOfEncodeAccount
```ts
type ParamsOfEncodeAccount = {
    state_init: string,
    balance?: bigint,
    last_trans_lt?: bigint,
    last_paid?: number,
    boc_cache?: BocCacheType
}
```
- `state_init`: _string_ – Account state init.
- `balance`?: _bigint_ – Initial balance.
- `last_trans_lt`?: _bigint_ – Initial value for the `last_trans_lt`.
- `last_paid`?: _number_ – Initial value for the `last_paid`.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


## ResultOfEncodeAccount
```ts
type ResultOfEncodeAccount = {
    account: string,
    id: string
}
```
- `account`: _string_ – Account BOC encoded in `base64`.
- `id`: _string_ – Account ID  encoded in `hex`.


## ParamsOfDecodeAccountData
```ts
type ParamsOfDecodeAccountData = {
    abi: Abi,
    data: string,
    allow_partial?: boolean
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `data`: _string_ – Data BOC or BOC handle
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)


## ResultOfDecodeAccountData
```ts
type ResultOfDecodeAccountData = {
    data: any
}
```
- `data`: _any_ – Decoded data as a JSON structure.


## ParamsOfUpdateInitialData
```ts
type ParamsOfUpdateInitialData = {
    abi: Abi,
    data: string,
    initial_data?: any,
    initial_pubkey?: string,
    boc_cache?: BocCacheType
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `data`: _string_ – Data BOC or BOC handle
- `initial_data`?: _any_ – List of initial values for contract's static variables.
<br>`abi` parameter should be provided to set initial data
- `initial_pubkey`?: _string_ – Initial account owner's public key to set into account data
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfUpdateInitialData
```ts
type ResultOfUpdateInitialData = {
    data: string
}
```
- `data`: _string_ – Updated data BOC or BOC handle


## ParamsOfEncodeInitialData
```ts
type ParamsOfEncodeInitialData = {
    abi: Abi,
    initial_data?: any,
    initial_pubkey?: string,
    boc_cache?: BocCacheType
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI
- `initial_data`?: _any_ – List of initial values for contract's static variables.
<br>`abi` parameter should be provided to set initial data
- `initial_pubkey`?: _string_ – Initial account owner's public key to set into account data
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfEncodeInitialData
```ts
type ResultOfEncodeInitialData = {
    data: string
}
```
- `data`: _string_ – Updated data BOC or BOC handle


## ParamsOfDecodeInitialData
```ts
type ParamsOfDecodeInitialData = {
    abi: Abi,
    data: string,
    allow_partial?: boolean
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
<br>Initial data is decoded if this parameter is provided
- `data`: _string_ – Data BOC or BOC handle
- `allow_partial`?: _boolean_ – Flag allowing partial BOC decoding when ABI doesn't describe the full body BOC. Controls decoder behaviour when after decoding all described in ABI params there are some data left in BOC: `true` - return decoded values `false` - return error of incomplete BOC deserialization (default)


## ResultOfDecodeInitialData
```ts
type ResultOfDecodeInitialData = {
    initial_data: any,
    initial_pubkey: string
}
```
- `initial_data`: _any_ – List of initial values of contract's public variables.
<br>Initial data is decoded if `abi` input parameter is provided
- `initial_pubkey`: _string_ – Initial account owner's public key


## ParamsOfDecodeBoc
```ts
type ParamsOfDecodeBoc = {
    params: AbiParam[],
    boc: string,
    allow_partial: boolean
}
```
- `params`: _[AbiParam](mod\_abi.md#abiparam)[]_ – Parameters to decode from BOC
- `boc`: _string_ – Data BOC or BOC handle
- `allow_partial`: _boolean_


## ResultOfDecodeBoc
```ts
type ResultOfDecodeBoc = {
    data: any
}
```
- `data`: _any_ – Decoded data as a JSON structure.


## ParamsOfAbiEncodeBoc
```ts
type ParamsOfAbiEncodeBoc = {
    params: AbiParam[],
    data: any,
    boc_cache?: BocCacheType
}
```
- `params`: _[AbiParam](mod\_abi.md#abiparam)[]_ – Parameters to encode into BOC
- `data`: _any_ – Parameters and values as a JSON structure
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


## ResultOfAbiEncodeBoc
```ts
type ResultOfAbiEncodeBoc = {
    boc: string
}
```
- `boc`: _string_ – BOC encoded as base64


## ParamsOfCalcFunctionId
```ts
type ParamsOfCalcFunctionId = {
    abi: Abi,
    function_name: string,
    output?: boolean
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI.
- `function_name`: _string_ – Contract function name
- `output`?: _boolean_ – If set to `true` output function ID will be returned which is used in contract response. Default is `false`


## ResultOfCalcFunctionId
```ts
type ResultOfCalcFunctionId = {
    function_id: number
}
```
- `function_id`: _number_ – Contract function ID


## ParamsOfGetSignatureData
```ts
type ParamsOfGetSignatureData = {
    abi: Abi,
    message: string,
    signature_id?: number
}
```
- `abi`: _[Abi](mod\_abi.md#abi)_ – Contract ABI used to decode.
- `message`: _string_ – Message BOC encoded in `base64`.
- `signature_id`?: _number_ – Signature ID to be used in unsigned data preparing when CapSignatureWithId capability is enabled


## ResultOfGetSignatureData
```ts
type ResultOfGetSignatureData = {
    signature: string,
    unsigned: string
}
```
- `signature`: _string_ – Signature from the message in `hex`.
- `unsigned`: _string_ – Data to verify the signature in `base64`.



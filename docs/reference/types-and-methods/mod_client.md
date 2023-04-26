# Module client

Provides information about library.


## Functions
[get_api_reference](mod\_client.md#get_api_reference) – Returns Core Library API reference

[version](mod\_client.md#version) – Returns Core Library version

[config](mod\_client.md#config) – Returns Core Library API reference

[build_info](mod\_client.md#build_info) – Returns detailed information about this build.

[resolve_app_request](mod\_client.md#resolve_app_request) – Resolves application request processing result

## Types
[ClientErrorCode](mod\_client.md#clienterrorcode)

[ClientError](mod\_client.md#clienterror)

[ClientConfig](mod\_client.md#clientconfig)

[NetworkConfig](mod\_client.md#networkconfig)

[BindingConfig](mod\_client.md#bindingconfig)

[NetworkQueriesProtocol](mod\_client.md#networkqueriesprotocol) – Network protocol used to perform GraphQL queries.

[CryptoConfig](mod\_client.md#cryptoconfig) – Crypto config.

[AbiConfig](mod\_client.md#abiconfig)

[BocConfig](mod\_client.md#bocconfig)

[ProofsConfig](mod\_client.md#proofsconfig)

[BuildInfoDependency](mod\_client.md#buildinfodependency)

[ParamsOfAppRequest](mod\_client.md#paramsofapprequest)

[AppRequestResultErrorVariant](mod\_client.md#apprequestresulterrorvariant) – Error occurred during request processing

[AppRequestResultOkVariant](mod\_client.md#apprequestresultokvariant) – Request processed successfully

[AppRequestResult](mod\_client.md#apprequestresult)

[ResultOfGetApiReference](mod\_client.md#resultofgetapireference)

[ResultOfVersion](mod\_client.md#resultofversion)

[ResultOfBuildInfo](mod\_client.md#resultofbuildinfo)

[ParamsOfResolveAppRequest](mod\_client.md#paramsofresolveapprequest)


# Functions
## get_api_reference

Returns Core Library API reference

```ts
type ResultOfGetApiReference = {
    api: any
}

function get_api_reference(): Promise<ResultOfGetApiReference>;
```


### Result

- `api`: _API_


## version

Returns Core Library version

```ts
type ResultOfVersion = {
    version: string
}

function version(): Promise<ResultOfVersion>;
```


### Result

- `version`: _string_ – Core Library version


## config

Returns Core Library API reference

```ts
type ClientConfig = {
    binding?: BindingConfig,
    network?: NetworkConfig,
    crypto?: CryptoConfig,
    abi?: AbiConfig,
    boc?: BocConfig,
    proofs?: ProofsConfig,
    local_storage_path?: string
}

function config(): Promise<ClientConfig>;
```


### Result

- `binding`?: _[BindingConfig](mod\_client.md#bindingconfig)_
- `network`?: _[NetworkConfig](mod\_client.md#networkconfig)_
- `crypto`?: _[CryptoConfig](mod\_client.md#cryptoconfig)_
- `abi`?: _[AbiConfig](mod\_client.md#abiconfig)_
- `boc`?: _[BocConfig](mod\_client.md#bocconfig)_
- `proofs`?: _[ProofsConfig](mod\_client.md#proofsconfig)_
- `local_storage_path`?: _string_ – For file based storage is a folder name where SDK will store its data. For browser based is a browser async storage key prefix. Default (recommended) value is "~/.tonclient" for native environments and ".tonclient" for web-browser.


## build_info

Returns detailed information about this build.

```ts
type ResultOfBuildInfo = {
    build_number: number,
    dependencies: BuildInfoDependency[]
}

function build_info(): Promise<ResultOfBuildInfo>;
```


### Result

- `build_number`: _number_ – Build number assigned to this build by the CI.
- `dependencies`: _[BuildInfoDependency](mod\_client.md#buildinfodependency)[]_ – Fingerprint of the most important dependencies.


## resolve_app_request

Resolves application request processing result

```ts
type ParamsOfResolveAppRequest = {
    app_request_id: number,
    result: AppRequestResult
}

function resolve_app_request(
    params: ParamsOfResolveAppRequest,
): Promise<void>;
```
### Parameters
- `app_request_id`: _number_ – Request ID received from SDK
- `result`: _[AppRequestResult](mod\_client.md#apprequestresult)_ – Result of request processing


# Types
## ClientErrorCode
```ts
enum ClientErrorCode {
    NotImplemented = 1,
    InvalidHex = 2,
    InvalidBase64 = 3,
    InvalidAddress = 4,
    CallbackParamsCantBeConvertedToJson = 5,
    WebsocketConnectError = 6,
    WebsocketReceiveError = 7,
    WebsocketSendError = 8,
    HttpClientCreateError = 9,
    HttpRequestCreateError = 10,
    HttpRequestSendError = 11,
    HttpRequestParseError = 12,
    CallbackNotRegistered = 13,
    NetModuleNotInit = 14,
    InvalidConfig = 15,
    CannotCreateRuntime = 16,
    InvalidContextHandle = 17,
    CannotSerializeResult = 18,
    CannotSerializeError = 19,
    CannotConvertJsValueToJson = 20,
    CannotReceiveSpawnedResult = 21,
    SetTimerError = 22,
    InvalidParams = 23,
    ContractsAddressConversionFailed = 24,
    UnknownFunction = 25,
    AppRequestError = 26,
    NoSuchRequest = 27,
    CanNotSendRequestResult = 28,
    CanNotReceiveRequestResult = 29,
    CanNotParseRequestResult = 30,
    UnexpectedCallbackResponse = 31,
    CanNotParseNumber = 32,
    InternalError = 33,
    InvalidHandle = 34,
    LocalStorageError = 35,
    InvalidData = 36
}
```
One of the following value:

- `NotImplemented = 1`
- `InvalidHex = 2`
- `InvalidBase64 = 3`
- `InvalidAddress = 4`
- `CallbackParamsCantBeConvertedToJson = 5`
- `WebsocketConnectError = 6`
- `WebsocketReceiveError = 7`
- `WebsocketSendError = 8`
- `HttpClientCreateError = 9`
- `HttpRequestCreateError = 10`
- `HttpRequestSendError = 11`
- `HttpRequestParseError = 12`
- `CallbackNotRegistered = 13`
- `NetModuleNotInit = 14`
- `InvalidConfig = 15`
- `CannotCreateRuntime = 16`
- `InvalidContextHandle = 17`
- `CannotSerializeResult = 18`
- `CannotSerializeError = 19`
- `CannotConvertJsValueToJson = 20`
- `CannotReceiveSpawnedResult = 21`
- `SetTimerError = 22`
- `InvalidParams = 23`
- `ContractsAddressConversionFailed = 24`
- `UnknownFunction = 25`
- `AppRequestError = 26`
- `NoSuchRequest = 27`
- `CanNotSendRequestResult = 28`
- `CanNotReceiveRequestResult = 29`
- `CanNotParseRequestResult = 30`
- `UnexpectedCallbackResponse = 31`
- `CanNotParseNumber = 32`
- `InternalError = 33`
- `InvalidHandle = 34`
- `LocalStorageError = 35`
- `InvalidData = 36`


## ClientError
```ts
type ClientError = {
    code: number,
    message: string,
    data: any
}
```
- `code`: _number_
- `message`: _string_
- `data`: _any_


## ClientConfig
```ts
type ClientConfig = {
    binding?: BindingConfig,
    network?: NetworkConfig,
    crypto?: CryptoConfig,
    abi?: AbiConfig,
    boc?: BocConfig,
    proofs?: ProofsConfig,
    local_storage_path?: string
}
```
- `binding`?: _[BindingConfig](mod\_client.md#bindingconfig)_
- `network`?: _[NetworkConfig](mod\_client.md#networkconfig)_
- `crypto`?: _[CryptoConfig](mod\_client.md#cryptoconfig)_
- `abi`?: _[AbiConfig](mod\_client.md#abiconfig)_
- `boc`?: _[BocConfig](mod\_client.md#bocconfig)_
- `proofs`?: _[ProofsConfig](mod\_client.md#proofsconfig)_
- `local_storage_path`?: _string_ – For file based storage is a folder name where SDK will store its data. For browser based is a browser async storage key prefix. Default (recommended) value is "~/.tonclient" for native environments and ".tonclient" for web-browser.


## NetworkConfig
```ts
type NetworkConfig = {
    server_address?: string,
    endpoints?: string[],
    network_retries_count?: number,
    max_reconnect_timeout?: number,
    reconnect_timeout?: number,
    message_retries_count?: number,
    message_processing_timeout?: number,
    wait_for_timeout?: number,
    out_of_sync_threshold?: number,
    sending_endpoint_count?: number,
    latency_detection_interval?: number,
    max_latency?: number,
    query_timeout?: number,
    queries_protocol?: NetworkQueriesProtocol,
    first_remp_status_timeout?: number,
    next_remp_status_timeout?: number,
    signature_id?: number,
    access_key?: string
}
```
- `server_address`?: _string_ – **This field is deprecated, but left for backward-compatibility.** Evernode endpoint.
- `endpoints`?: _string[]_ – List of Evernode endpoints.
<br>Any correct URL format can be specified, including IP addresses. This parameter is prevailing over `server_address`.<br>Check the full list of [supported network endpoints](https://docs.evercloud.dev/products/evercloud/networks-endpoints).
- `network_retries_count`?: _number_ – Deprecated.
<br>You must use `network.max_reconnect_timeout` that allows to specify maximum network resolving timeout.
- `max_reconnect_timeout`?: _number_ – Maximum time for sequential reconnections.
<br>Must be specified in milliseconds. Default is 120000 (2 min).
- `reconnect_timeout`?: _number_ – Deprecated
- `message_retries_count`?: _number_ – The number of automatic message processing retries that SDK performs in case of `Message Expired (507)` error - but only for those messages which local emulation was successful or failed with replay protection error.
<br>Default is 5.
- `message_processing_timeout`?: _number_ – Timeout that is used to process message delivery for the contracts which ABI does not include "expire" header. If the message is not delivered within the specified timeout the appropriate error occurs.
<br>Must be specified in milliseconds. Default is 40000 (40 sec).
- `wait_for_timeout`?: _number_ – Maximum timeout that is used for query response.
<br>Must be specified in milliseconds. Default is 40000 (40 sec).
- `out_of_sync_threshold`?: _number_ – **DEPRECATED**: This parameter was deprecated.
- `sending_endpoint_count`?: _number_ – Maximum number of randomly chosen endpoints the library uses to broadcast a message.
<br>Default is 1.
- `latency_detection_interval`?: _number_ – Frequency of sync latency detection.
<br>Library periodically checks the current endpoint for blockchain data synchronization latency.<br>If the latency (time-lag) is less then `NetworkConfig.max_latency`<br>then library selects another endpoint.<br><br>Must be specified in milliseconds. Default is 60000 (1 min).
- `max_latency`?: _number_ – Maximum value for the endpoint's blockchain data synchronization latency (time-lag). Library periodically checks the current endpoint for blockchain data synchronization latency. If the latency (time-lag) is less then `NetworkConfig.max_latency` then library selects another endpoint.
<br>Must be specified in milliseconds. Default is 60000 (1 min).
- `query_timeout`?: _number_ – Default timeout for http requests.
<br>Is is used when no timeout specified for the request to limit the answer waiting time. If no answer received during the timeout requests ends with<br>error.<br><br>Must be specified in milliseconds. Default is 60000 (1 min).
- `queries_protocol`?: _[NetworkQueriesProtocol](mod\_client.md#networkqueriesprotocol)_ – Queries protocol.
<br>`HTTP` or `WS`. <br>Default is `HTTP`.
- `first_remp_status_timeout`?: _number_ – UNSTABLE.
<br>First REMP status awaiting timeout. If no status received during the timeout than fallback transaction scenario is activated.<br><br>Must be specified in milliseconds. Default is 1 (1 ms) in order to start fallback scenario<br>together with REMP statuses processing while REMP is not properly tuned yet.
- `next_remp_status_timeout`?: _number_ – UNSTABLE.
<br>Subsequent REMP status awaiting timeout. If no status received during the timeout than fallback transaction scenario is activated.<br><br>Must be specified in milliseconds. Default is 5000 (5 sec).
- `signature_id`?: _number_ – Network signature ID which is used by VM in signature verifying instructions if capability `CapSignatureWithId` is enabled in blockchain configuration parameters.
<br>This parameter should be set to `global_id` field from any blockchain block if network can<br>not be reachable at the moment of message encoding and the message is aimed to be sent into<br>network with `CapSignatureWithId` enabled. Otherwise signature ID is detected automatically<br>inside message encoding functions
- `access_key`?: _string_ – Access key to GraphQL API (Project secret)


## BindingConfig
```ts
type BindingConfig = {
    library?: string,
    version?: string
}
```
- `library`?: _string_
- `version`?: _string_


## NetworkQueriesProtocol
Network protocol used to perform GraphQL queries.

```ts
enum NetworkQueriesProtocol {
    HTTP = "HTTP",
    WS = "WS"
}
```
One of the following value:

- `HTTP = "HTTP"` – Each GraphQL query uses separate HTTP request.
- `WS = "WS"` – All GraphQL queries will be served using single web socket connection. SDK is tested to reliably handle 5000 parallel network requests (sending and processing messages, quering and awaiting blockchain data)


## CryptoConfig
Crypto config.

```ts
type CryptoConfig = {
    mnemonic_dictionary?: MnemonicDictionary,
    mnemonic_word_count?: number,
    hdkey_derivation_path?: string
}
```
- `mnemonic_dictionary`?: _[MnemonicDictionary](mod\_crypto.md#mnemonicdictionary)_ – Mnemonic dictionary that will be used by default in crypto functions. If not specified, `English` dictionary will be used.
- `mnemonic_word_count`?: _number_ – Mnemonic word count that will be used by default in crypto functions. If not specified the default value will be 12.
- `hdkey_derivation_path`?: _string_ – Derivation path that will be used by default in crypto functions. If not specified `m/44'/396'/0'/0/0` will be used.


## AbiConfig
```ts
type AbiConfig = {
    workchain?: number,
    message_expiration_timeout?: number,
    message_expiration_timeout_grow_factor?: number
}
```
- `workchain`?: _number_ – Workchain id that is used by default in DeploySet
- `message_expiration_timeout`?: _number_ – Message lifetime for contracts which ABI includes "expire" header. The default value is 40 sec.
- `message_expiration_timeout_grow_factor`?: _number_ – Factor that increases the expiration timeout for each retry The default value is 1.5


## BocConfig
```ts
type BocConfig = {
    cache_max_size?: number
}
```
- `cache_max_size`?: _number_ – Maximum BOC cache size in kilobytes.
<br>Default is 10 MB


## ProofsConfig
```ts
type ProofsConfig = {
    cache_in_local_storage?: boolean
}
```
- `cache_in_local_storage`?: _boolean_ – Cache proofs in the local storage.
<br>Default is `true`. If this value is set to `true`, downloaded proofs and master-chain BOCs are saved into the<br>persistent local storage (e.g. file system for native environments or browser's IndexedDB<br>for the web); otherwise all the data is cached only in memory in current client's context<br>and will be lost after destruction of the client.


## BuildInfoDependency
```ts
type BuildInfoDependency = {
    name: string,
    git_commit: string
}
```
- `name`: _string_ – Dependency name.
<br>Usually it is a crate name.
- `git_commit`: _string_ – Git commit hash of the related repository.


## ParamsOfAppRequest
```ts
type ParamsOfAppRequest = {
    app_request_id: number,
    request_data: any
}
```
- `app_request_id`: _number_ – Request ID.
<br>Should be used in `resolve_app_request` call
- `request_data`: _any_ – Request describing data


## AppRequestResultErrorVariant
Error occurred during request processing

```ts
type AppRequestResultErrorVariant = {
    text: string
}
```
- `text`: _string_ – Error description


## AppRequestResultOkVariant
Request processed successfully

```ts
type AppRequestResultOkVariant = {
    result: any
}
```
- `result`: _any_ – Request processing result


## AppRequestResult
```ts
type AppRequestResult = ({
    type: 'Error'
} & AppRequestResultErrorVariant) | ({
    type: 'Ok'
} & AppRequestResultOkVariant)
```
Depends on value of the  `type` field.

When _type_ is _'Error'_

Error occurred during request processing

- `text`: _string_ – Error description

When _type_ is _'Ok'_

Request processed successfully

- `result`: _any_ – Request processing result


Variant constructors:

```ts
function appRequestResultError(text: string): AppRequestResult;
function appRequestResultOk(result: any): AppRequestResult;
```

## ResultOfGetApiReference
```ts
type ResultOfGetApiReference = {
    api: any
}
```
- `api`: _API_


## ResultOfVersion
```ts
type ResultOfVersion = {
    version: string
}
```
- `version`: _string_ – Core Library version


## ResultOfBuildInfo
```ts
type ResultOfBuildInfo = {
    build_number: number,
    dependencies: BuildInfoDependency[]
}
```
- `build_number`: _number_ – Build number assigned to this build by the CI.
- `dependencies`: _[BuildInfoDependency](mod\_client.md#buildinfodependency)[]_ – Fingerprint of the most important dependencies.


## ParamsOfResolveAppRequest
```ts
type ParamsOfResolveAppRequest = {
    app_request_id: number,
    result: AppRequestResult
}
```
- `app_request_id`: _number_ – Request ID received from SDK
- `result`: _[AppRequestResult](mod\_client.md#apprequestresult)_ – Result of request processing



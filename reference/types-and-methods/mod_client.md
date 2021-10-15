# Module client

## Module client

Provides information about library.

### Functions

[get_api_reference](mod_client.md#get_api_reference) – Returns Core Library API reference

[version](mod_client.md#version) – Returns Core Library version

[build_info](mod_client.md#build_info) – Returns detailed information about this build.

[resolve_app_request](mod_client.md#resolve_app_request) – Resolves application request processing result

### Types

[ClientErrorCode](mod_client.md#ClientErrorCode)

[ClientError](mod_client.md#ClientError)

[ClientConfig](mod_client.md#ClientConfig)

[NetworkConfig](mod_client.md#NetworkConfig)

[CryptoConfig](mod_client.md#CryptoConfig) – Crypto config.

[AbiConfig](mod_client.md#AbiConfig)

[BocConfig](mod_client.md#BocConfig)

[BuildInfoDependency](mod_client.md#BuildInfoDependency)

[ParamsOfAppRequest](mod_client.md#ParamsOfAppRequest)

[AppRequestResult](mod_client.md#AppRequestResult)

[ResultOfGetApiReference](mod_client.md#ResultOfGetApiReference)

[ResultOfVersion](mod_client.md#ResultOfVersion)

[ResultOfBuildInfo](mod_client.md#ResultOfBuildInfo)

[ParamsOfResolveAppRequest](mod_client.md#ParamsOfResolveAppRequest)

## Functions

### get_api_reference

Returns Core Library API reference

```typescript
type ResultOfGetApiReference = {
    api: any
}

function get_api_reference(): Promise<ResultOfGetApiReference>;
```

#### Result

* `api`: _API_

### version

Returns Core Library version

```typescript
type ResultOfVersion = {
    version: string
}

function version(): Promise<ResultOfVersion>;
```

#### Result

* `version`: _string_ – Core Library version

### build_info

Returns detailed information about this build.

```typescript
type ResultOfBuildInfo = {
    build_number: number,
    dependencies: BuildInfoDependency[]
}

function build_info(): Promise<ResultOfBuildInfo>;
```

#### Result

* `build_number`: _number_ – Build number assigned to this build by the CI.
* `dependencies`: [_BuildInfoDependency_](mod_client.md#BuildInfoDependency)_\[]_ – Fingerprint of the most important dependencies.

### resolve_app_request

Resolves application request processing result

```typescript
type ParamsOfResolveAppRequest = {
    app_request_id: number,
    result: AppRequestResult
}

function resolve_app_request(
    params: ParamsOfResolveAppRequest,
): Promise<void>;
```

#### Parameters

* `app_request_id`: _number_ – Request ID received from SDK
* `result`: [_AppRequestResult_](mod_client.md#AppRequestResult) – Result of request processing

## Types

### ClientErrorCode

```typescript
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
    InvalidHandle = 34
}
```

One of the following value:

* `NotImplemented = 1`
* `InvalidHex = 2`
* `InvalidBase64 = 3`
* `InvalidAddress = 4`
* `CallbackParamsCantBeConvertedToJson = 5`
* `WebsocketConnectError = 6`
* `WebsocketReceiveError = 7`
* `WebsocketSendError = 8`
* `HttpClientCreateError = 9`
* `HttpRequestCreateError = 10`
* `HttpRequestSendError = 11`
* `HttpRequestParseError = 12`
* `CallbackNotRegistered = 13`
* `NetModuleNotInit = 14`
* `InvalidConfig = 15`
* `CannotCreateRuntime = 16`
* `InvalidContextHandle = 17`
* `CannotSerializeResult = 18`
* `CannotSerializeError = 19`
* `CannotConvertJsValueToJson = 20`
* `CannotReceiveSpawnedResult = 21`
* `SetTimerError = 22`
* `InvalidParams = 23`
* `ContractsAddressConversionFailed = 24`
* `UnknownFunction = 25`
* `AppRequestError = 26`
* `NoSuchRequest = 27`
* `CanNotSendRequestResult = 28`
* `CanNotReceiveRequestResult = 29`
* `CanNotParseRequestResult = 30`
* `UnexpectedCallbackResponse = 31`
* `CanNotParseNumber = 32`
* `InternalError = 33`
* `InvalidHandle = 34`

### ClientError

```typescript
type ClientError = {
    code: number,
    message: string,
    data: any
}
```

* `code`: _number_
* `message`: _string_
* `data`: _any_

### ClientConfig

```typescript
type ClientConfig = {
    network?: NetworkConfig,
    crypto?: CryptoConfig,
    abi?: AbiConfig,
    boc?: BocConfig
}
```

* `network`?: [_NetworkConfig_](mod_client.md#NetworkConfig)
* `crypto`?: [_CryptoConfig_](mod_client.md#CryptoConfig)
* `abi`?: [_AbiConfig_](mod_client.md#AbiConfig)
* `boc`?: [_BocConfig_](mod_client.md#BocConfig)

### NetworkConfig

```typescript
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
    access_key?: string
}
```

* `server_address`?: _string_ – DApp Server public address. For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be net.ton.dev
*   `endpoints`?: _string\[]_ – List of DApp Server addresses.

    \
    Any correct URL format can be specified, including IP addresses This parameter is prevailing over `server_address`.
*   `network_retries_count`?: _number_ – Deprecated.

    \
    You must use `network.max_reconnect_timeout` that allows to specify maximum network resolving timeout.
*   `max_reconnect_timeout`?: _number_ – Maximum time for sequential reconnections.

    \
    Must be specified in milliseconds. Default is 120000 (2 min).
* `reconnect_timeout`?: _number_ – Deprecated
*   `message_retries_count`?: _number_ – The number of automatic message processing retries that SDK performs in case of `Message Expired (507)` error - but only for those messages which local emulation was successful or failed with replay protection error.

    \
    Default is 5.
*   `message_processing_timeout`?: _number_ – Timeout that is used to process message delivery for the contracts which ABI does not include "expire" header. If the message is not delivered within the specified timeout the appropriate error occurs.

    \
    Must be specified in milliseconds. Default is 40000 (40 sec).
*   `wait_for_timeout`?: _number_ – Maximum timeout that is used for query response.

    \
    Must be specified in milliseconds. Default is 40000 (40 sec).
*   `out_of_sync_threshold`?: _number_ – Maximum time difference between server and client.

    \
    If client's device time is out of sync and difference is more than the threshold then error will occur. Also an error will occur if the specified threshold is more than\
    `message_processing_timeout/2`.\
    \
    Must be specified in milliseconds. Default is 15000 (15 sec).
*   `sending_endpoint_count`?: _number_ – Maximum number of randomly chosen endpoints the library uses to broadcast a message.

    \
    Default is 2.
*   `latency_detection_interval`?: _number_ – Frequency of sync latency detection.

    \
    Library periodically checks the current endpoint for blockchain data syncronization latency.\
    If the latency (time-lag) is less then `NetworkConfig.max_latency`\
    then library selects another endpoint.\
    \
    Must be specified in milliseconds. Default is 60000 (1 min).
*   `max_latency`?: _number_ – Maximum value for the endpoint's blockchain data syncronization latency (time-lag). Library periodically checks the current endpoint for blockchain data syncronization latency. If the latency (time-lag) is less then `NetworkConfig.max_latency` then library selects another endpoint.

    \
    Must be specified in milliseconds. Default is 60000 (1 min).
*   `query_timeout`?: _number_ – Default timeout for http requests.

    \
    Is is used when no timeout specified for the request to limit the answer waiting time. If no answer received during the timeout requests ends with\
    error.\
    \
    Must be specified in milliseconds. Default is 60000 (1 min).
*   `access_key`?: _string_ – Access key to GraphQL API.

    \
    At the moment is not used in production.

### CryptoConfig

Crypto config.

```typescript
type CryptoConfig = {
    mnemonic_dictionary?: number,
    mnemonic_word_count?: number,
    hdkey_derivation_path?: string
}
```

* `mnemonic_dictionary`?: _number_ – Mnemonic dictionary that will be used by default in crypto functions. If not specified, 1 dictionary will be used.
* `mnemonic_word_count`?: _number_ – Mnemonic word count that will be used by default in crypto functions. If not specified the default value will be 12.
* `hdkey_derivation_path`?: _string_ – Derivation path that will be used by default in crypto functions. If not specified `m/44'/396'/0'/0/0` will be used.

### AbiConfig

```typescript
type AbiConfig = {
    workchain?: number,
    message_expiration_timeout?: number,
    message_expiration_timeout_grow_factor?: number
}
```

* `workchain`?: _number_ – Workchain id that is used by default in DeploySet
* `message_expiration_timeout`?: _number_ – Message lifetime for contracts which ABI includes "expire" header. The default value is 40 sec.
* `message_expiration_timeout_grow_factor`?: _number_ – Factor that increases the expiration timeout for each retry The default value is 1.5

### BocConfig

```typescript
type BocConfig = {
    cache_max_size?: number
}
```

*   `cache_max_size`?: _number_ – Maximum BOC cache size in kilobytes.

    \
    Default is 10 MB

### BuildInfoDependency

```typescript
type BuildInfoDependency = {
    name: string,
    git_commit: string
}
```

*   `name`: _string_ – Dependency name.

    \
    Usually it is a crate name.
* `git_commit`: _string_ – Git commit hash of the related repository.

### ParamsOfAppRequest

```typescript
type ParamsOfAppRequest = {
    app_request_id: number,
    request_data: any
}
```

*   `app_request_id`: _number_ – Request ID.

    \
    Should be used in `resolve_app_request` call
* `request_data`: _any_ – Request describing data

### AppRequestResult

```typescript
type AppRequestResult = {
    type: 'Error'
    text: string
} | {
    type: 'Ok'
    result: any
}
```

Depends on value of the `type` field.

When _type_ is _'Error'_

Error occurred during request processing

* `text`: _string_ – Error description

When _type_ is _'Ok'_

Request processed successfully

* `result`: _any_ – Request processing result

Variant constructors:

```typescript
function appRequestResultError(text: string): AppRequestResult;
function appRequestResultOk(result: any): AppRequestResult;
```

### ResultOfGetApiReference

```typescript
type ResultOfGetApiReference = {
    api: any
}
```

* `api`: _API_

### ResultOfVersion

```typescript
type ResultOfVersion = {
    version: string
}
```

* `version`: _string_ – Core Library version

### ResultOfBuildInfo

```typescript
type ResultOfBuildInfo = {
    build_number: number,
    dependencies: BuildInfoDependency[]
}
```

* `build_number`: _number_ – Build number assigned to this build by the CI.
* `dependencies`: [_BuildInfoDependency_](mod_client.md#BuildInfoDependency)_\[]_ – Fingerprint of the most important dependencies.

### ParamsOfResolveAppRequest

```typescript
type ParamsOfResolveAppRequest = {
    app_request_id: number,
    result: AppRequestResult
}
```

* `app_request_id`: _number_ – Request ID received from SDK
* `result`: [_AppRequestResult_](mod_client.md#AppRequestResult) – Result of request processing

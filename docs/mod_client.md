# Module client

null
## Functions
[get_api_reference](#get_api_reference) – Returns Core Library API reference

[version](#version) – Returns Core Library version

[build_info](#build_info) – Returns detailed information about this build.

[resolve_app_request](#resolve_app_request) – Resolves application request processing result

## Types
[ClientErrorCode](#ClientErrorCode)

[ClientError](#ClientError)

[ClientConfig](#ClientConfig)

[NetworkConfig](#NetworkConfig)

[CryptoConfig](#CryptoConfig) – Crypto config.

[AbiConfig](#AbiConfig)

[BuildInfoDependency](#BuildInfoDependency)

[ParamsOfAppRequest](#ParamsOfAppRequest)

[AppRequestResult](#AppRequestResult)

[ResultOfGetApiReference](#ResultOfGetApiReference)

[ResultOfVersion](#ResultOfVersion)

[ResultOfBuildInfo](#ResultOfBuildInfo)

[ParamsOfResolveAppRequest](#ParamsOfResolveAppRequest)


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
- `dependencies`: _[BuildInfoDependency](mod_client.md#BuildInfoDependency)[]_ – Fingerprint of the most important dependencies.


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
- `result`: _[AppRequestResult](mod_client.md#AppRequestResult)_ – Result of request processing
### Result



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
    InternalError = 33
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
    network?: NetworkConfig,
    crypto?: CryptoConfig,
    abi?: AbiConfig
}
```
- `network`?: _[NetworkConfig](mod_client.md#NetworkConfig)_
- `crypto`?: _[CryptoConfig](mod_client.md#CryptoConfig)_
- `abi`?: _[AbiConfig](mod_client.md#AbiConfig)_


## NetworkConfig
```ts
type NetworkConfig = {
    server_address?: string,
    endpoints?: string[],
    network_retries_count?: number,
    message_retries_count?: number,
    message_processing_timeout?: number,
    wait_for_timeout?: number,
    out_of_sync_threshold?: number,
    reconnect_timeout?: number,
    access_key?: string
}
```
- `server_address`?: _string_ – DApp Server public address. For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be net.ton.dev
- `endpoints`?: _string[]_ – List of DApp Server addresses.
<br>Any correct URL format can be specified, including IP addresses
- `network_retries_count`?: _number_ – The number of automatic network retries that SDK performs in case of connection problems The default value is 5.
- `message_retries_count`?: _number_ – The number of automatic message processing retries that SDK performs in case of `Message Expired (507)` error - but only for those messages which local emulation was successfull or failed with replay protection error. The default value is 5.
- `message_processing_timeout`?: _number_ – Timeout that is used to process message delivery for the contracts which ABI does not include "expire" header. If the message is not delivered within the speficied timeout the appropriate error occurs.
- `wait_for_timeout`?: _number_ – Maximum timeout that is used for query response. The default value is 40 sec.
- `out_of_sync_threshold`?: _number_ – Maximum time difference between server and client.
<br>If client's device time is out of sink and difference is more thanthe threshhold then error will occur. Also the error will occur if the specified threshhold is more than<br>`message_processing_timeout/2`.<br>The default value is 15 sec.
- `reconnect_timeout`?: _number_ – Timeout between reconnect attempts
- `access_key`?: _string_ – Access key to GraphQL API.
<br>At the moment is not used in production


## CryptoConfig
Crypto config.

```ts
type CryptoConfig = {
    mnemonic_dictionary?: number,
    mnemonic_word_count?: number,
    hdkey_derivation_path?: string
}
```
- `mnemonic_dictionary`?: _number_ – Mnemonic dictionary that will be used by default in crypto funcions. If not specified, 1 dictionary will be used.
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


## AppRequestResult
```ts
type AppRequestResult = {
    type: 'Error'
    text: string
} | {
    type: 'Ok'
    result: any
}
```
Depends on value of the  `type` field.

When _type_ is _'Error'_

Error occured during request processing


- `text`: _string_ – Error description

When _type_ is _'Ok'_

Request processed successfully


- `result`: _any_ – Request processing result


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
- `dependencies`: _[BuildInfoDependency](mod_client.md#BuildInfoDependency)[]_ – Fingerprint of the most important dependencies.


## ParamsOfResolveAppRequest
```ts
type ParamsOfResolveAppRequest = {
    app_request_id: number,
    result: AppRequestResult
}
```
- `app_request_id`: _number_ – Request ID received from SDK
- `result`: _[AppRequestResult](mod_client.md#AppRequestResult)_ – Result of request processing



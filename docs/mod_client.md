# Module client

 BOC manipulation module.
## Functions
[get_api_reference](#get_api_reference)

[version](#version)

## Types
[ClientError](#ClientError)

[ClientConfig](#ClientConfig)

[NetworkConfig](#NetworkConfig)

[CryptoConfig](#CryptoConfig)

[AbiConfig](#AbiConfig)

[ResultOfGetApiReference](#ResultOfGetApiReference)

[ResultOfVersion](#ResultOfVersion)


# Functions
## get_api_reference

```ts
type ResultOfGetApiReference = {
    api: any
};

function get_api_reference(): Promise<ResultOfGetApiReference>;
```
### Result

- `api`: _API_


## version

```ts
type ResultOfVersion = {
    version: String
};

function version(): Promise<ResultOfVersion>;
```
### Result

- `version`: _string_ –  core version


# Types
## ClientError

```ts
type ClientError = {
    code: Number,
    message: String,
    data: any
};
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
};
```
- `network`?: _[NetworkConfig](mod_client.md#NetworkConfig)_
- `crypto`?: _[CryptoConfig](mod_client.md#CryptoConfig)_
- `abi`?: _[AbiConfig](mod_client.md#AbiConfig)_


## NetworkConfig

```ts
type NetworkConfig = {
    server_address: String,
    message_retries_count?: Number,
    message_processing_timeout?: Number,
    wait_for_timeout?: Number,
    out_of_sync_threshold?: bigint,
    access_key?: String
};
```
- `server_address`: _string_
- `message_retries_count`?: _number_
- `message_processing_timeout`?: _number_
- `wait_for_timeout`?: _number_
- `out_of_sync_threshold`?: _bigint_
- `access_key`?: _string_


## CryptoConfig

```ts
type CryptoConfig = {
    fish_param?: String
};
```
- `fish_param`?: _string_


## AbiConfig

```ts
type AbiConfig = {
    message_expiration_timeout?: Number,
    message_expiration_timeout_grow_factor?: Number
};
```
- `message_expiration_timeout`?: _number_
- `message_expiration_timeout_grow_factor`?: _number_


## ResultOfGetApiReference

```ts
type ResultOfGetApiReference = {
    api: any
};
```
- `api`: _API_


## ResultOfVersion

```ts
type ResultOfVersion = {
    version: String
};
```
- `version`: _string_ –  core version



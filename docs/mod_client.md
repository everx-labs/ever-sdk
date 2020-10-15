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
    version: string
};

function version(): Promise<ResultOfVersion>;
```
### Result

- `version`: _string_ –  core version


# Types
## ClientError

```ts
type ClientError = {
    code: number,
    message: string,
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
    server_address: string,
    message_retries_count?: number,
    message_processing_timeout?: number,
    wait_for_timeout?: number,
    out_of_sync_threshold?: bigint,
    access_key?: string
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
    fish_param?: string
};
```
- `fish_param`?: _string_


## AbiConfig

```ts
type AbiConfig = {
    message_expiration_timeout?: number,
    message_expiration_timeout_grow_factor?: number
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
    version: string
};
```
- `version`: _string_ –  core version



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

function getApiReference(
    responseHandler: ResponseHandler | null,
): Promise<ResultOfGetApiReference>;

```
### Result

- `api`: _API_


## version

```ts

function version(
    responseHandler: ResponseHandler | null,
): Promise<ResultOfVersion>;

```
### Result

- `version`: _string_ –  core version


# Types
## ClientError

- `code`: _number_
- `message`: _string_
- `data`: _any_


## ClientConfig

- `network`?: _[NetworkConfig](mod_client.md#NetworkConfig)_
- `crypto`?: _[CryptoConfig](mod_client.md#CryptoConfig)_
- `abi`?: _[AbiConfig](mod_client.md#AbiConfig)_


## NetworkConfig

- `server_address`: _string_
- `message_retries_count`?: _number_
- `message_processing_timeout`?: _number_
- `wait_for_timeout`?: _number_
- `out_of_sync_threshold`?: _bigint_
- `access_key`?: _string_


## CryptoConfig

- `fish_param`?: _string_


## AbiConfig

- `message_expiration_timeout`?: _number_
- `message_expiration_timeout_grow_factor`?: _number_


## ResultOfGetApiReference

- `api`: _API_


## ResultOfVersion

- `version`: _string_ –  core version



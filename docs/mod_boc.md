# Module boc

BOC manipulation module.


## Functions
[parse_message](#parse_message) – Parses message boc into a JSON

[parse_transaction](#parse_transaction) – Parses transaction boc into a JSON

[parse_account](#parse_account) – Parses account boc into a JSON

[parse_block](#parse_block) – Parses block boc into a JSON

[parse_shardstate](#parse_shardstate) – Parses shardstate boc into a JSON

[get_blockchain_config](#get_blockchain_config)

[get_boc_hash](#get_boc_hash) – Calculates BOC root hash

[get_code_from_tvc](#get_code_from_tvc) – Extracts code from TVC contract image

[cache_get](#cache_get) – Get BOC from cache

[cache_set](#cache_set) – Save BOC into cache

[cache_unpin](#cache_unpin) – Unpin BOCs with specified pin.

## Types
[BocCacheType](#BocCacheType)

[BocErrorCode](#BocErrorCode)

[ParamsOfParse](#ParamsOfParse)

[ResultOfParse](#ResultOfParse)

[ParamsOfParseShardstate](#ParamsOfParseShardstate)

[ParamsOfGetBlockchainConfig](#ParamsOfGetBlockchainConfig)

[ResultOfGetBlockchainConfig](#ResultOfGetBlockchainConfig)

[ParamsOfGetBocHash](#ParamsOfGetBocHash)

[ResultOfGetBocHash](#ResultOfGetBocHash)

[ParamsOfGetCodeFromTvc](#ParamsOfGetCodeFromTvc)

[ResultOfGetCodeFromTvc](#ResultOfGetCodeFromTvc)

[ParamsOfBocCacheGet](#ParamsOfBocCacheGet)

[ResultOfBocCacheGet](#ResultOfBocCacheGet)

[ParamsOfBocCacheSet](#ParamsOfBocCacheSet)

[ResultOfBocCacheSet](#ResultOfBocCacheSet)

[ParamsOfBocCacheUnpin](#ParamsOfBocCacheUnpin)


# Functions
## parse_message

Parses message boc into a JSON

JSON structure is compatible with GraphQL API message object

```ts
type ParamsOfParse = {
    boc: string
}

type ResultOfParse = {
    parsed: any
}

function parse_message(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## parse_transaction

Parses transaction boc into a JSON

JSON structure is compatible with GraphQL API transaction object

```ts
type ParamsOfParse = {
    boc: string
}

type ResultOfParse = {
    parsed: any
}

function parse_transaction(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## parse_account

Parses account boc into a JSON

JSON structure is compatible with GraphQL API account object

```ts
type ParamsOfParse = {
    boc: string
}

type ResultOfParse = {
    parsed: any
}

function parse_account(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## parse_block

Parses block boc into a JSON

JSON structure is compatible with GraphQL API block object

```ts
type ParamsOfParse = {
    boc: string
}

type ResultOfParse = {
    parsed: any
}

function parse_block(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## parse_shardstate

Parses shardstate boc into a JSON

JSON structure is compatible with GraphQL API shardstate object

```ts
type ParamsOfParseShardstate = {
    boc: string,
    id: string,
    workchain_id: number
}

type ResultOfParse = {
    parsed: any
}

function parse_shardstate(
    params: ParamsOfParseShardstate,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64
- `id`: _string_ – Shardstate identificator
- `workchain_id`: _number_ – Workchain shardstate belongs to


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## get_blockchain_config

```ts
type ParamsOfGetBlockchainConfig = {
    block_boc: string
}

type ResultOfGetBlockchainConfig = {
    config_boc: string
}

function get_blockchain_config(
    params: ParamsOfGetBlockchainConfig,
): Promise<ResultOfGetBlockchainConfig>;
```
### Parameters
- `block_boc`: _string_ – Key block BOC encoded as base64


### Result

- `config_boc`: _string_ – Blockchain config BOC encoded as base64


## get_boc_hash

Calculates BOC root hash

```ts
type ParamsOfGetBocHash = {
    boc: string
}

type ResultOfGetBocHash = {
    hash: string
}

function get_boc_hash(
    params: ParamsOfGetBocHash,
): Promise<ResultOfGetBocHash>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64


### Result

- `hash`: _string_ – BOC root hash encoded with hex


## get_code_from_tvc

Extracts code from TVC contract image

```ts
type ParamsOfGetCodeFromTvc = {
    tvc: string
}

type ResultOfGetCodeFromTvc = {
    code: string
}

function get_code_from_tvc(
    params: ParamsOfGetCodeFromTvc,
): Promise<ResultOfGetCodeFromTvc>;
```
### Parameters
- `tvc`: _string_ – Contract TVC image encoded as base64


### Result

- `code`: _string_ – Contract code encoded as base64


## cache_get

Get BOC from cache

```ts
type ParamsOfBocCacheGet = {
    boc_ref: string
}

type ResultOfBocCacheGet = {
    boc?: string
}

function cache_get(
    params: ParamsOfBocCacheGet,
): Promise<ResultOfBocCacheGet>;
```
### Parameters
- `boc_ref`: _string_ – Reference to the cached BOC


### Result

- `boc`?: _string_ – BOC encoded as base64.


## cache_set

Save BOC into cache

```ts
type ParamsOfBocCacheSet = {
    boc: string,
    cache_type: BocCacheType
}

type ResultOfBocCacheSet = {
    boc_ref: string
}

function cache_set(
    params: ParamsOfBocCacheSet,
): Promise<ResultOfBocCacheSet>;
```
### Parameters
- `boc`: _string_ – BOC encoded as base64 or BOC reference
- `cache_type`: _[BocCacheType](mod_boc.md#BocCacheType)_ – Cache type


### Result

- `boc_ref`: _string_ – Reference to the cached BOC


## cache_unpin

Unpin BOCs with specified pin.

BOCs which don't have another pins will be removed from cache

```ts
type ParamsOfBocCacheUnpin = {
    pin: string,
    boc_ref?: string
}

function cache_unpin(
    params: ParamsOfBocCacheUnpin,
): Promise<void>;
```
### Parameters
- `pin`: _string_ – Pinned name
- `boc_ref`?: _string_ – Reference to the cached BOC.
<br>If it is provided then only referenced BOC is unpinned


# Types
## BocCacheType
```ts
type BocCacheType = {
    type: 'Pinned'
    pin: string
} | {
    type: 'Unpinned'
}
```
Depends on value of the  `type` field.

When _type_ is _'Pinned'_

Pin the BOC with `pin` name.

Such BOC will not be removed from cache until it is unpinned


- `pin`: _string_

When _type_ is _'Unpinned'_

 



Variant constructors:

```ts
function bocCacheTypePinned(pin: string): BocCacheType;
function bocCacheTypeUnpinned(): BocCacheType;
```

## BocErrorCode
```ts
enum BocErrorCode {
    InvalidBoc = 201,
    SerializationError = 202,
    InappropriateBlock = 203,
    MissingSourceBoc = 204,
    InsufficientCacheSize = 205,
    BocRefNotFound = 206,
    InvalidBocRef = 207
}
```
One of the following value:

- `InvalidBoc = 201`
- `SerializationError = 202`
- `InappropriateBlock = 203`
- `MissingSourceBoc = 204`
- `InsufficientCacheSize = 205`
- `BocRefNotFound = 206`
- `InvalidBocRef = 207`


## ParamsOfParse
```ts
type ParamsOfParse = {
    boc: string
}
```
- `boc`: _string_ – BOC encoded as base64


## ResultOfParse
```ts
type ResultOfParse = {
    parsed: any
}
```
- `parsed`: _any_ – JSON containing parsed BOC


## ParamsOfParseShardstate
```ts
type ParamsOfParseShardstate = {
    boc: string,
    id: string,
    workchain_id: number
}
```
- `boc`: _string_ – BOC encoded as base64
- `id`: _string_ – Shardstate identificator
- `workchain_id`: _number_ – Workchain shardstate belongs to


## ParamsOfGetBlockchainConfig
```ts
type ParamsOfGetBlockchainConfig = {
    block_boc: string
}
```
- `block_boc`: _string_ – Key block BOC encoded as base64


## ResultOfGetBlockchainConfig
```ts
type ResultOfGetBlockchainConfig = {
    config_boc: string
}
```
- `config_boc`: _string_ – Blockchain config BOC encoded as base64


## ParamsOfGetBocHash
```ts
type ParamsOfGetBocHash = {
    boc: string
}
```
- `boc`: _string_ – BOC encoded as base64


## ResultOfGetBocHash
```ts
type ResultOfGetBocHash = {
    hash: string
}
```
- `hash`: _string_ – BOC root hash encoded with hex


## ParamsOfGetCodeFromTvc
```ts
type ParamsOfGetCodeFromTvc = {
    tvc: string
}
```
- `tvc`: _string_ – Contract TVC image encoded as base64


## ResultOfGetCodeFromTvc
```ts
type ResultOfGetCodeFromTvc = {
    code: string
}
```
- `code`: _string_ – Contract code encoded as base64


## ParamsOfBocCacheGet
```ts
type ParamsOfBocCacheGet = {
    boc_ref: string
}
```
- `boc_ref`: _string_ – Reference to the cached BOC


## ResultOfBocCacheGet
```ts
type ResultOfBocCacheGet = {
    boc?: string
}
```
- `boc`?: _string_ – BOC encoded as base64.


## ParamsOfBocCacheSet
```ts
type ParamsOfBocCacheSet = {
    boc: string,
    cache_type: BocCacheType
}
```
- `boc`: _string_ – BOC encoded as base64 or BOC reference
- `cache_type`: _[BocCacheType](mod_boc.md#BocCacheType)_ – Cache type


## ResultOfBocCacheSet
```ts
type ResultOfBocCacheSet = {
    boc_ref: string
}
```
- `boc_ref`: _string_ – Reference to the cached BOC


## ParamsOfBocCacheUnpin
```ts
type ParamsOfBocCacheUnpin = {
    pin: string,
    boc_ref?: string
}
```
- `pin`: _string_ – Pinned name
- `boc_ref`?: _string_ – Reference to the cached BOC.
<br>If it is provided then only referenced BOC is unpinned



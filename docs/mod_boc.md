# Module boc

null
## Functions
[parse_message](#parse_message) – Parses message boc into a JSON

[parse_transaction](#parse_transaction) – Parses transaction boc into a JSON

[parse_account](#parse_account) – Parses account boc into a JSON

[parse_block](#parse_block) – Parses block boc into a JSON

[parse_shardstate](#parse_shardstate) – Parses shardstate boc into a JSON

[get_blockchain_config](#get_blockchain_config)

[get_boc_hash](#get_boc_hash) – Calculates BOC root hash

[get_code_from_tvc](#get_code_from_tvc) – Extracts code from TVC contract image

## Types
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


# Types
## BocErrorCode
```ts
enum BocErrorCode {
    InvalidBoc = 201,
    SerializationError = 202,
    InappropriateBlock = 203,
    MissingSourceBoc = 204
}
```
One of the following value:

- `InvalidBoc = 201`
- `SerializationError = 202`
- `InappropriateBlock = 203`
- `MissingSourceBoc = 204`


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



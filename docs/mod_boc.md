# Module boc

 BOC manipulation module.
## Functions
[parse_message](#parse_message)

[parse_transaction](#parse_transaction)

[parse_account](#parse_account)

[parse_block](#parse_block)

[get_blockchain_config](#get_blockchain_config)

## Types
[ParamsOfParse](#ParamsOfParse)

[ResultOfParse](#ResultOfParse)

[ParamsOfGetBlockchainConfig](#ParamsOfGetBlockchainConfig)

[ResultOfGetBlockchainConfig](#ResultOfGetBlockchainConfig)


# Functions
## parse_message

```ts
type ParamsOfParse = {
    boc: string
};

type ResultOfParse = {
    parsed: any
};

function parse_message(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_transaction

```ts
type ParamsOfParse = {
    boc: string
};

type ResultOfParse = {
    parsed: any
};

function parse_transaction(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_account

```ts
type ParamsOfParse = {
    boc: string
};

type ResultOfParse = {
    parsed: any
};

function parse_account(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_block

```ts
type ParamsOfParse = {
    boc: string
};

type ResultOfParse = {
    parsed: any
};

function parse_block(
    params: ParamsOfParse,
): Promise<ResultOfParse>;
```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## get_blockchain_config

```ts
type ParamsOfGetBlockchainConfig = {
    block_boc: string
};

type ResultOfGetBlockchainConfig = {
    config_boc: string
};

function get_blockchain_config(
    params: ParamsOfGetBlockchainConfig,
): Promise<ResultOfGetBlockchainConfig>;
```
### Parameters
- `block_boc`: _string_ –  Key block BOC encoded as base64
### Result

- `config_boc`: _string_ –  Blockchain config BOC encoded as base64


# Types
## ParamsOfParse

```ts
type ParamsOfParse = {
    boc: string
};
```
- `boc`: _string_ –  BOC encoded as base64


## ResultOfParse

```ts
type ResultOfParse = {
    parsed: any
};
```
- `parsed`: _any_ –  JSON containing parsed BOC


## ParamsOfGetBlockchainConfig

```ts
type ParamsOfGetBlockchainConfig = {
    block_boc: string
};
```
- `block_boc`: _string_ –  Key block BOC encoded as base64


## ResultOfGetBlockchainConfig

```ts
type ResultOfGetBlockchainConfig = {
    config_boc: string
};
```
- `config_boc`: _string_ –  Blockchain config BOC encoded as base64



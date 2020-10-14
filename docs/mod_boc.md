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

function parseMessage(
    params: ParamsOfParse,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfParse>;

```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_transaction

```ts

function parseTransaction(
    params: ParamsOfParse,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfParse>;

```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_account

```ts

function parseAccount(
    params: ParamsOfParse,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfParse>;

```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## parse_block

```ts

function parseBlock(
    params: ParamsOfParse,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfParse>;

```
### Parameters
- `boc`: _string_ –  BOC encoded as base64
### Result

- `parsed`: _any_ –  JSON containing parsed BOC


## get_blockchain_config

```ts

function getBlockchainConfig(
    params: ParamsOfGetBlockchainConfig,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfGetBlockchainConfig>;

```
### Parameters
- `block_boc`: _string_ –  Key block BOC encoded as base64
### Result

- `config_boc`: _string_ –  Blockchain config BOC encoded as base64


# Types
## ParamsOfParse

- `boc`: _string_ –  BOC encoded as base64


## ResultOfParse

- `parsed`: _any_ –  JSON containing parsed BOC


## ParamsOfGetBlockchainConfig

- `block_boc`: _string_ –  Key block BOC encoded as base64


## ResultOfGetBlockchainConfig

- `config_boc`: _string_ –  Blockchain config BOC encoded as base64



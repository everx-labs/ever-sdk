# Module boc

BOC manipulation module.


## Functions
[decode_tvc](mod\_boc.md#decode_tvc) – Decodes tvc according to the tvc spec. Read more about tvc structure here https://github.com/everx-labs/ever-struct/blob/main/src/scheme/mod.rs#L30

[parse_message](mod\_boc.md#parse_message) – Parses message boc into a JSON

[parse_transaction](mod\_boc.md#parse_transaction) – Parses transaction boc into a JSON

[parse_account](mod\_boc.md#parse_account) – Parses account boc into a JSON

[parse_block](mod\_boc.md#parse_block) – Parses block boc into a JSON

[parse_shardstate](mod\_boc.md#parse_shardstate) – Parses shardstate boc into a JSON

[get_blockchain_config](mod\_boc.md#get_blockchain_config) – Extract blockchain configuration from key block and also from zerostate.

[get_boc_hash](mod\_boc.md#get_boc_hash) – Calculates BOC root hash

[get_boc_depth](mod\_boc.md#get_boc_depth) – Calculates BOC depth

[get_code_from_tvc](mod\_boc.md#get_code_from_tvc) – Extracts code from TVC contract image

[cache_get](mod\_boc.md#cache_get) – Get BOC from cache

[cache_set](mod\_boc.md#cache_set) – Save BOC into cache or increase pin counter for existing pinned BOC

[cache_unpin](mod\_boc.md#cache_unpin) – Unpin BOCs with specified pin defined in the `cache_set`. Decrease pin reference counter for BOCs with specified pin defined in the `cache_set`. BOCs which have only 1 pin and its reference counter become 0 will be removed from cache

[encode_boc](mod\_boc.md#encode_boc) – Encodes bag of cells (BOC) with builder operations. This method provides the same functionality as Solidity TvmBuilder. Resulting BOC of this method can be passed into Solidity and C++ contracts as TvmCell type.

[get_code_salt](mod\_boc.md#get_code_salt) – Returns the contract code's salt if it is present.

[set_code_salt](mod\_boc.md#set_code_salt) – Sets new salt to contract code.

[decode_state_init](mod\_boc.md#decode_state_init) – Decodes contract's initial state into code, data, libraries and special options.

[encode_state_init](mod\_boc.md#encode_state_init) – Encodes initial contract state from code, data, libraries ans special options (see input params)

[encode_external_in_message](mod\_boc.md#encode_external_in_message) – Encodes a message

[get_compiler_version](mod\_boc.md#get_compiler_version) – Returns the compiler version used to compile the code.

## Types
[BocCacheTypePinnedVariant](mod\_boc.md#boccachetypepinnedvariant) – Pin the BOC with `pin` name.

[BocCacheTypeUnpinnedVariant](mod\_boc.md#boccachetypeunpinnedvariant) – BOC is placed into a common BOC pool with limited size regulated by LRU (least recently used) cache lifecycle.

[BocCacheType](mod\_boc.md#boccachetype)

[BuilderOpIntegerVariant](mod\_boc.md#builderopintegervariant) – Append integer to cell data.

[BuilderOpBitStringVariant](mod\_boc.md#builderopbitstringvariant) – Append bit string to cell data.

[BuilderOpCellVariant](mod\_boc.md#builderopcellvariant) – Append ref to nested cells.

[BuilderOpCellBocVariant](mod\_boc.md#builderopcellbocvariant) – Append ref to nested cell.

[BuilderOpAddressVariant](mod\_boc.md#builderopaddressvariant) – Address.

[BuilderOp](mod\_boc.md#builderop) – Cell builder operation.

[TvcV1Variant](mod\_boc.md#tvcv1variant)

[Tvc](mod\_boc.md#tvc)

[TvcV1](mod\_boc.md#tvcv1)

[BocErrorCode](mod\_boc.md#bocerrorcode)

[ParamsOfDecodeTvc](mod\_boc.md#paramsofdecodetvc)

[ResultOfDecodeTvc](mod\_boc.md#resultofdecodetvc)

[ParamsOfParse](mod\_boc.md#paramsofparse)

[ResultOfParse](mod\_boc.md#resultofparse)

[ParamsOfParseShardstate](mod\_boc.md#paramsofparseshardstate)

[ParamsOfGetBlockchainConfig](mod\_boc.md#paramsofgetblockchainconfig)

[ResultOfGetBlockchainConfig](mod\_boc.md#resultofgetblockchainconfig)

[ParamsOfGetBocHash](mod\_boc.md#paramsofgetbochash)

[ResultOfGetBocHash](mod\_boc.md#resultofgetbochash)

[ParamsOfGetBocDepth](mod\_boc.md#paramsofgetbocdepth)

[ResultOfGetBocDepth](mod\_boc.md#resultofgetbocdepth)

[ParamsOfGetCodeFromTvc](mod\_boc.md#paramsofgetcodefromtvc)

[ResultOfGetCodeFromTvc](mod\_boc.md#resultofgetcodefromtvc)

[ParamsOfBocCacheGet](mod\_boc.md#paramsofboccacheget)

[ResultOfBocCacheGet](mod\_boc.md#resultofboccacheget)

[ParamsOfBocCacheSet](mod\_boc.md#paramsofboccacheset)

[ResultOfBocCacheSet](mod\_boc.md#resultofboccacheset)

[ParamsOfBocCacheUnpin](mod\_boc.md#paramsofboccacheunpin)

[ParamsOfEncodeBoc](mod\_boc.md#paramsofencodeboc)

[ResultOfEncodeBoc](mod\_boc.md#resultofencodeboc)

[ParamsOfGetCodeSalt](mod\_boc.md#paramsofgetcodesalt)

[ResultOfGetCodeSalt](mod\_boc.md#resultofgetcodesalt)

[ParamsOfSetCodeSalt](mod\_boc.md#paramsofsetcodesalt)

[ResultOfSetCodeSalt](mod\_boc.md#resultofsetcodesalt)

[ParamsOfDecodeStateInit](mod\_boc.md#paramsofdecodestateinit)

[ResultOfDecodeStateInit](mod\_boc.md#resultofdecodestateinit)

[ParamsOfEncodeStateInit](mod\_boc.md#paramsofencodestateinit)

[ResultOfEncodeStateInit](mod\_boc.md#resultofencodestateinit)

[ParamsOfEncodeExternalInMessage](mod\_boc.md#paramsofencodeexternalinmessage)

[ResultOfEncodeExternalInMessage](mod\_boc.md#resultofencodeexternalinmessage)

[ParamsOfGetCompilerVersion](mod\_boc.md#paramsofgetcompilerversion)

[ResultOfGetCompilerVersion](mod\_boc.md#resultofgetcompilerversion)


# Functions
## decode_tvc

Decodes tvc according to the tvc spec. Read more about tvc structure here https://github.com/everx-labs/ever-struct/blob/main/src/scheme/mod.rs#L30

```ts
type ParamsOfDecodeTvc = {
    tvc: string
}

type ResultOfDecodeTvc = {
    tvc: Tvc
}

function decode_tvc(
    params: ParamsOfDecodeTvc,
): Promise<ResultOfDecodeTvc>;

function decode_tvc_sync(
    params: ParamsOfDecodeTvc,
): ResultOfDecodeTvc;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `tvc`: _string_ – Contract TVC BOC encoded as base64 or BOC handle


### Result

- `tvc`: _[Tvc](mod\_boc.md#tvc)_ – Decoded TVC


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

function parse_message_sync(
    params: ParamsOfParse,
): ResultOfParse;
```
NOTE: Sync version is available only for `lib-node` binding.
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

function parse_transaction_sync(
    params: ParamsOfParse,
): ResultOfParse;
```
NOTE: Sync version is available only for `lib-node` binding.
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

function parse_account_sync(
    params: ParamsOfParse,
): ResultOfParse;
```
NOTE: Sync version is available only for `lib-node` binding.
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

function parse_block_sync(
    params: ParamsOfParse,
): ResultOfParse;
```
NOTE: Sync version is available only for `lib-node` binding.
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

function parse_shardstate_sync(
    params: ParamsOfParseShardstate,
): ResultOfParse;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `boc`: _string_ – BOC encoded as base64
- `id`: _string_ – Shardstate identifier
- `workchain_id`: _number_ – Workchain shardstate belongs to


### Result

- `parsed`: _any_ – JSON containing parsed BOC


## get_blockchain_config

Extract blockchain configuration from key block and also from zerostate.

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

function get_blockchain_config_sync(
    params: ParamsOfGetBlockchainConfig,
): ResultOfGetBlockchainConfig;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `block_boc`: _string_ – Key block BOC or zerostate BOC encoded as base64


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

function get_boc_hash_sync(
    params: ParamsOfGetBocHash,
): ResultOfGetBocHash;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `boc`: _string_ – BOC encoded as base64 or BOC handle


### Result

- `hash`: _string_ – BOC root hash encoded with hex


## get_boc_depth

Calculates BOC depth

```ts
type ParamsOfGetBocDepth = {
    boc: string
}

type ResultOfGetBocDepth = {
    depth: number
}

function get_boc_depth(
    params: ParamsOfGetBocDepth,
): Promise<ResultOfGetBocDepth>;

function get_boc_depth_sync(
    params: ParamsOfGetBocDepth,
): ResultOfGetBocDepth;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `boc`: _string_ – BOC encoded as base64 or BOC handle


### Result

- `depth`: _number_ – BOC root cell depth


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

function get_code_from_tvc_sync(
    params: ParamsOfGetCodeFromTvc,
): ResultOfGetCodeFromTvc;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `tvc`: _string_ – Contract TVC image or image BOC handle


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

function cache_get_sync(
    params: ParamsOfBocCacheGet,
): ResultOfBocCacheGet;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `boc_ref`: _string_ – Reference to the cached BOC


### Result

- `boc`?: _string_ – BOC encoded as base64.


## cache_set

Save BOC into cache or increase pin counter for existing pinned BOC

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

function cache_set_sync(
    params: ParamsOfBocCacheSet,
): ResultOfBocCacheSet;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `boc`: _string_ – BOC encoded as base64 or BOC reference
- `cache_type`: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type


### Result

- `boc_ref`: _string_ – Reference to the cached BOC


## cache_unpin

Unpin BOCs with specified pin defined in the `cache_set`. Decrease pin reference counter for BOCs with specified pin defined in the `cache_set`. BOCs which have only 1 pin and its reference counter become 0 will be removed from cache

```ts
type ParamsOfBocCacheUnpin = {
    pin: string,
    boc_ref?: string
}

function cache_unpin(
    params: ParamsOfBocCacheUnpin,
): Promise<void>;

function cache_unpin_sync(
    params: ParamsOfBocCacheUnpin,
): void;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `pin`: _string_ – Pinned name
- `boc_ref`?: _string_ – Reference to the cached BOC.
<br>If it is provided then only referenced BOC is unpinned


## encode_boc

Encodes bag of cells (BOC) with builder operations. This method provides the same functionality as Solidity TvmBuilder. Resulting BOC of this method can be passed into Solidity and C++ contracts as TvmCell type.

```ts
type ParamsOfEncodeBoc = {
    builder: BuilderOp[],
    boc_cache?: BocCacheType
}

type ResultOfEncodeBoc = {
    boc: string
}

function encode_boc(
    params: ParamsOfEncodeBoc,
): Promise<ResultOfEncodeBoc>;

function encode_boc_sync(
    params: ParamsOfEncodeBoc,
): ResultOfEncodeBoc;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `builder`: _[BuilderOp](mod\_boc.md#builderop)[]_ – Cell builder operations.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `boc`: _string_ – Encoded cell BOC or BOC cache key.


## get_code_salt

Returns the contract code's salt if it is present.

```ts
type ParamsOfGetCodeSalt = {
    code: string,
    boc_cache?: BocCacheType
}

type ResultOfGetCodeSalt = {
    salt?: string
}

function get_code_salt(
    params: ParamsOfGetCodeSalt,
): Promise<ResultOfGetCodeSalt>;

function get_code_salt_sync(
    params: ParamsOfGetCodeSalt,
): ResultOfGetCodeSalt;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `salt`?: _string_ – Contract code salt if present.
<br>BOC encoded as base64 or BOC handle


## set_code_salt

Sets new salt to contract code.

Returns the new contract code with salt.

```ts
type ParamsOfSetCodeSalt = {
    code: string,
    salt: string,
    boc_cache?: BocCacheType
}

type ResultOfSetCodeSalt = {
    code: string
}

function set_code_salt(
    params: ParamsOfSetCodeSalt,
): Promise<ResultOfSetCodeSalt>;

function set_code_salt_sync(
    params: ParamsOfSetCodeSalt,
): ResultOfSetCodeSalt;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle
- `salt`: _string_ – Code salt to set.
<br>BOC encoded as base64 or BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `code`: _string_ – Contract code with salt set.
<br>BOC encoded as base64 or BOC handle


## decode_state_init

Decodes contract's initial state into code, data, libraries and special options.

```ts
type ParamsOfDecodeStateInit = {
    state_init: string,
    boc_cache?: BocCacheType
}

type ResultOfDecodeStateInit = {
    code?: string,
    code_hash?: string,
    code_depth?: number,
    data?: string,
    data_hash?: string,
    data_depth?: number,
    library?: string,
    tick?: boolean,
    tock?: boolean,
    split_depth?: number,
    compiler_version?: string
}

function decode_state_init(
    params: ParamsOfDecodeStateInit,
): Promise<ResultOfDecodeStateInit>;

function decode_state_init_sync(
    params: ParamsOfDecodeStateInit,
): ResultOfDecodeStateInit;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `state_init`: _string_ – Contract StateInit image BOC encoded as base64 or BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `code`?: _string_ – Contract code BOC encoded as base64 or BOC handle
- `code_hash`?: _string_ – Contract code hash
- `code_depth`?: _number_ – Contract code depth
- `data`?: _string_ – Contract data BOC encoded as base64 or BOC handle
- `data_hash`?: _string_ – Contract data hash
- `data_depth`?: _number_ – Contract data depth
- `library`?: _string_ – Contract library BOC encoded as base64 or BOC handle
- `tick`?: _boolean_ – `special.tick` field.
<br>Specifies the contract ability to handle tick transactions
- `tock`?: _boolean_ – `special.tock` field.
<br>Specifies the contract ability to handle tock transactions
- `split_depth`?: _number_ – Is present and non-zero only in instances of large smart contracts
- `compiler_version`?: _string_ – Compiler version, for example 'sol 0.49.0'


## encode_state_init

Encodes initial contract state from code, data, libraries ans special options (see input params)

```ts
type ParamsOfEncodeStateInit = {
    code?: string,
    data?: string,
    library?: string,
    tick?: boolean,
    tock?: boolean,
    split_depth?: number,
    boc_cache?: BocCacheType
}

type ResultOfEncodeStateInit = {
    state_init: string
}

function encode_state_init(
    params: ParamsOfEncodeStateInit,
): Promise<ResultOfEncodeStateInit>;

function encode_state_init_sync(
    params: ParamsOfEncodeStateInit,
): ResultOfEncodeStateInit;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `code`?: _string_ – Contract code BOC encoded as base64 or BOC handle
- `data`?: _string_ – Contract data BOC encoded as base64 or BOC handle
- `library`?: _string_ – Contract library BOC encoded as base64 or BOC handle
- `tick`?: _boolean_ – `special.tick` field.
<br>Specifies the contract ability to handle tick transactions
- `tock`?: _boolean_ – `special.tock` field.
<br>Specifies the contract ability to handle tock transactions
- `split_depth`?: _number_ – Is present and non-zero only in instances of large smart contracts
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


### Result

- `state_init`: _string_ – Contract StateInit image BOC encoded as base64 or BOC handle of boc_cache parameter was specified


## encode_external_in_message

Encodes a message

Allows to encode any external inbound message.

```ts
type ParamsOfEncodeExternalInMessage = {
    src?: string,
    dst: string,
    init?: string,
    body?: string,
    boc_cache?: BocCacheType
}

type ResultOfEncodeExternalInMessage = {
    message: string,
    message_id: string
}

function encode_external_in_message(
    params: ParamsOfEncodeExternalInMessage,
): Promise<ResultOfEncodeExternalInMessage>;

function encode_external_in_message_sync(
    params: ParamsOfEncodeExternalInMessage,
): ResultOfEncodeExternalInMessage;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `src`?: _string_ – Source address.
- `dst`: _string_ – Destination address.
- `init`?: _string_ – Bag of cells with state init (used in deploy messages).
- `body`?: _string_ – Bag of cells with the message body encoded as base64.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


### Result

- `message`: _string_ – Message BOC encoded with `base64`.
- `message_id`: _string_ – Message id.


## get_compiler_version

Returns the compiler version used to compile the code.

```ts
type ParamsOfGetCompilerVersion = {
    code: string
}

type ResultOfGetCompilerVersion = {
    version?: string
}

function get_compiler_version(
    params: ParamsOfGetCompilerVersion,
): Promise<ResultOfGetCompilerVersion>;

function get_compiler_version_sync(
    params: ParamsOfGetCompilerVersion,
): ResultOfGetCompilerVersion;
```
NOTE: Sync version is available only for `lib-node` binding.
### Parameters
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle


### Result

- `version`?: _string_ – Compiler version, for example 'sol 0.49.0'


# Types
## BocCacheTypePinnedVariant
Pin the BOC with `pin` name.

Such BOC will not be removed from cache until it is unpinned BOCs can have several pins and each of the pins has reference counter indicating how many
times the BOC was pinned with the pin. BOC is removed from cache after all references for all
pins are unpinned with `cache_unpin` function calls.

```ts
type BocCacheTypePinnedVariant = {
    pin: string
}
```
- `pin`: _string_


## BocCacheTypeUnpinnedVariant
BOC is placed into a common BOC pool with limited size regulated by LRU (least recently used) cache lifecycle.

BOC resides there until it is replaced with other BOCs if it is not used

```ts
type BocCacheTypeUnpinnedVariant = {

}
```


## BocCacheType
```ts
type BocCacheType = ({
    type: 'Pinned'
} & BocCacheTypePinnedVariant) | ({
    type: 'Unpinned'
} & BocCacheTypeUnpinnedVariant)
```
Depends on value of the  `type` field.

When _type_ is _'Pinned'_

Pin the BOC with `pin` name.

Such BOC will not be removed from cache until it is unpinned BOCs can have several pins and each of the pins has reference counter indicating how many
times the BOC was pinned with the pin. BOC is removed from cache after all references for all
pins are unpinned with `cache_unpin` function calls.

- `pin`: _string_

When _type_ is _'Unpinned'_

BOC is placed into a common BOC pool with limited size regulated by LRU (least recently used) cache lifecycle.

BOC resides there until it is replaced with other BOCs if it is not used



Variant constructors:

```ts
function bocCacheTypePinned(pin: string): BocCacheType;
function bocCacheTypeUnpinned(): BocCacheType;
```

## BuilderOpIntegerVariant
Append integer to cell data.

```ts
type BuilderOpIntegerVariant = {
    size: number,
    value: any
}
```
- `size`: _number_ – Bit size of the value.
- `value`: _any_ – Value: - `Number` containing integer number.
<br>e.g. `123`, `-123`. - Decimal string. e.g. `"123"`, `"-123"`.<br>- `0x` prefixed hexadecimal string.<br>  e.g `0x123`, `0X123`, `-0x123`.


## BuilderOpBitStringVariant
Append bit string to cell data.

```ts
type BuilderOpBitStringVariant = {
    value: string
}
```
- `value`: _string_ – Bit string content using bitstring notation. See `TON VM specification` 1.0.
<br>Contains hexadecimal string representation:<br>- Can end with `_` tag.<br>- Can be prefixed with `x` or `X`.<br>- Can be prefixed with `x{` or `X{` and ended with `}`.<br><br>Contains binary string represented as a sequence<br>of `0` and `1` prefixed with `n` or `N`.<br><br>Examples:<br>`1AB`, `x1ab`, `X1AB`, `x{1abc}`, `X{1ABC}`<br>`2D9_`, `x2D9_`, `X2D9_`, `x{2D9_}`, `X{2D9_}`<br>`n00101101100`, `N00101101100`


## BuilderOpCellVariant
Append ref to nested cells.

```ts
type BuilderOpCellVariant = {
    builder: BuilderOp[]
}
```
- `builder`: _[BuilderOp](mod\_boc.md#builderop)[]_ – Nested cell builder.


## BuilderOpCellBocVariant
Append ref to nested cell.

```ts
type BuilderOpCellBocVariant = {
    boc: string
}
```
- `boc`: _string_ – Nested cell BOC encoded with `base64` or BOC cache key.


## BuilderOpAddressVariant
Address.

```ts
type BuilderOpAddressVariant = {
    address: string
}
```
- `address`: _string_ – Address in a common `workchain:account` or base64 format.


## BuilderOp
Cell builder operation.

```ts
type BuilderOp = ({
    type: 'Integer'
} & BuilderOpIntegerVariant) | ({
    type: 'BitString'
} & BuilderOpBitStringVariant) | ({
    type: 'Cell'
} & BuilderOpCellVariant) | ({
    type: 'CellBoc'
} & BuilderOpCellBocVariant) | ({
    type: 'Address'
} & BuilderOpAddressVariant)
```
Depends on value of the  `type` field.

When _type_ is _'Integer'_

Append integer to cell data.

- `size`: _number_ – Bit size of the value.
- `value`: _any_ – Value: - `Number` containing integer number.
<br>e.g. `123`, `-123`. - Decimal string. e.g. `"123"`, `"-123"`.<br>- `0x` prefixed hexadecimal string.<br>  e.g `0x123`, `0X123`, `-0x123`.

When _type_ is _'BitString'_

Append bit string to cell data.

- `value`: _string_ – Bit string content using bitstring notation. See `TON VM specification` 1.0.
<br>Contains hexadecimal string representation:<br>- Can end with `_` tag.<br>- Can be prefixed with `x` or `X`.<br>- Can be prefixed with `x{` or `X{` and ended with `}`.<br><br>Contains binary string represented as a sequence<br>of `0` and `1` prefixed with `n` or `N`.<br><br>Examples:<br>`1AB`, `x1ab`, `X1AB`, `x{1abc}`, `X{1ABC}`<br>`2D9_`, `x2D9_`, `X2D9_`, `x{2D9_}`, `X{2D9_}`<br>`n00101101100`, `N00101101100`

When _type_ is _'Cell'_

Append ref to nested cells.

- `builder`: _[BuilderOp](mod\_boc.md#builderop)[]_ – Nested cell builder.

When _type_ is _'CellBoc'_

Append ref to nested cell.

- `boc`: _string_ – Nested cell BOC encoded with `base64` or BOC cache key.

When _type_ is _'Address'_

Address.

- `address`: _string_ – Address in a common `workchain:account` or base64 format.


Variant constructors:

```ts
function builderOpInteger(size: number, value: any): BuilderOp;
function builderOpBitString(value: string): BuilderOp;
function builderOpCell(builder: BuilderOp[]): BuilderOp;
function builderOpCellBoc(boc: string): BuilderOp;
function builderOpAddress(address: string): BuilderOp;
```

## TvcV1Variant
```ts
type TvcV1Variant = {
    value: TvcV1
}
```
- `value`: _[TvcV1](mod\_boc.md#tvcv1)_


## Tvc
```ts
type Tvc = ({
    type: 'V1'
} & TvcV1Variant)
```
Depends on value of the  `type` field.

When _type_ is _'V1'_

- `value`: _[TvcV1](mod\_boc.md#tvcv1)_


Variant constructors:

```ts
function tvcV1(value: TvcV1): Tvc;
```

## TvcV1
```ts
type TvcV1 = {
    code?: string,
    description?: string
}
```
- `code`?: _string_
- `description`?: _string_


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


## ParamsOfDecodeTvc
```ts
type ParamsOfDecodeTvc = {
    tvc: string
}
```
- `tvc`: _string_ – Contract TVC BOC encoded as base64 or BOC handle


## ResultOfDecodeTvc
```ts
type ResultOfDecodeTvc = {
    tvc: Tvc
}
```
- `tvc`: _[Tvc](mod\_boc.md#tvc)_ – Decoded TVC


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
- `id`: _string_ – Shardstate identifier
- `workchain_id`: _number_ – Workchain shardstate belongs to


## ParamsOfGetBlockchainConfig
```ts
type ParamsOfGetBlockchainConfig = {
    block_boc: string
}
```
- `block_boc`: _string_ – Key block BOC or zerostate BOC encoded as base64


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
- `boc`: _string_ – BOC encoded as base64 or BOC handle


## ResultOfGetBocHash
```ts
type ResultOfGetBocHash = {
    hash: string
}
```
- `hash`: _string_ – BOC root hash encoded with hex


## ParamsOfGetBocDepth
```ts
type ParamsOfGetBocDepth = {
    boc: string
}
```
- `boc`: _string_ – BOC encoded as base64 or BOC handle


## ResultOfGetBocDepth
```ts
type ResultOfGetBocDepth = {
    depth: number
}
```
- `depth`: _number_ – BOC root cell depth


## ParamsOfGetCodeFromTvc
```ts
type ParamsOfGetCodeFromTvc = {
    tvc: string
}
```
- `tvc`: _string_ – Contract TVC image or image BOC handle


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
- `cache_type`: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type


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


## ParamsOfEncodeBoc
```ts
type ParamsOfEncodeBoc = {
    builder: BuilderOp[],
    boc_cache?: BocCacheType
}
```
- `builder`: _[BuilderOp](mod\_boc.md#builderop)[]_ – Cell builder operations.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfEncodeBoc
```ts
type ResultOfEncodeBoc = {
    boc: string
}
```
- `boc`: _string_ – Encoded cell BOC or BOC cache key.


## ParamsOfGetCodeSalt
```ts
type ParamsOfGetCodeSalt = {
    code: string,
    boc_cache?: BocCacheType
}
```
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfGetCodeSalt
```ts
type ResultOfGetCodeSalt = {
    salt?: string
}
```
- `salt`?: _string_ – Contract code salt if present.
<br>BOC encoded as base64 or BOC handle


## ParamsOfSetCodeSalt
```ts
type ParamsOfSetCodeSalt = {
    code: string,
    salt: string,
    boc_cache?: BocCacheType
}
```
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle
- `salt`: _string_ – Code salt to set.
<br>BOC encoded as base64 or BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfSetCodeSalt
```ts
type ResultOfSetCodeSalt = {
    code: string
}
```
- `code`: _string_ – Contract code with salt set.
<br>BOC encoded as base64 or BOC handle


## ParamsOfDecodeStateInit
```ts
type ParamsOfDecodeStateInit = {
    state_init: string,
    boc_cache?: BocCacheType
}
```
- `state_init`: _string_ – Contract StateInit image BOC encoded as base64 or BOC handle
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfDecodeStateInit
```ts
type ResultOfDecodeStateInit = {
    code?: string,
    code_hash?: string,
    code_depth?: number,
    data?: string,
    data_hash?: string,
    data_depth?: number,
    library?: string,
    tick?: boolean,
    tock?: boolean,
    split_depth?: number,
    compiler_version?: string
}
```
- `code`?: _string_ – Contract code BOC encoded as base64 or BOC handle
- `code_hash`?: _string_ – Contract code hash
- `code_depth`?: _number_ – Contract code depth
- `data`?: _string_ – Contract data BOC encoded as base64 or BOC handle
- `data_hash`?: _string_ – Contract data hash
- `data_depth`?: _number_ – Contract data depth
- `library`?: _string_ – Contract library BOC encoded as base64 or BOC handle
- `tick`?: _boolean_ – `special.tick` field.
<br>Specifies the contract ability to handle tick transactions
- `tock`?: _boolean_ – `special.tock` field.
<br>Specifies the contract ability to handle tock transactions
- `split_depth`?: _number_ – Is present and non-zero only in instances of large smart contracts
- `compiler_version`?: _string_ – Compiler version, for example 'sol 0.49.0'


## ParamsOfEncodeStateInit
```ts
type ParamsOfEncodeStateInit = {
    code?: string,
    data?: string,
    library?: string,
    tick?: boolean,
    tock?: boolean,
    split_depth?: number,
    boc_cache?: BocCacheType
}
```
- `code`?: _string_ – Contract code BOC encoded as base64 or BOC handle
- `data`?: _string_ – Contract data BOC encoded as base64 or BOC handle
- `library`?: _string_ – Contract library BOC encoded as base64 or BOC handle
- `tick`?: _boolean_ – `special.tick` field.
<br>Specifies the contract ability to handle tick transactions
- `tock`?: _boolean_ – `special.tock` field.
<br>Specifies the contract ability to handle tock transactions
- `split_depth`?: _number_ – Is present and non-zero only in instances of large smart contracts
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result. The BOC itself returned if no cache type provided.


## ResultOfEncodeStateInit
```ts
type ResultOfEncodeStateInit = {
    state_init: string
}
```
- `state_init`: _string_ – Contract StateInit image BOC encoded as base64 or BOC handle of boc_cache parameter was specified


## ParamsOfEncodeExternalInMessage
```ts
type ParamsOfEncodeExternalInMessage = {
    src?: string,
    dst: string,
    init?: string,
    body?: string,
    boc_cache?: BocCacheType
}
```
- `src`?: _string_ – Source address.
- `dst`: _string_ – Destination address.
- `init`?: _string_ – Bag of cells with state init (used in deploy messages).
- `body`?: _string_ – Bag of cells with the message body encoded as base64.
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided


## ResultOfEncodeExternalInMessage
```ts
type ResultOfEncodeExternalInMessage = {
    message: string,
    message_id: string
}
```
- `message`: _string_ – Message BOC encoded with `base64`.
- `message_id`: _string_ – Message id.


## ParamsOfGetCompilerVersion
```ts
type ParamsOfGetCompilerVersion = {
    code: string
}
```
- `code`: _string_ – Contract code BOC encoded as base64 or code BOC handle


## ResultOfGetCompilerVersion
```ts
type ResultOfGetCompilerVersion = {
    version?: string
}
```
- `version`?: _string_ – Compiler version, for example 'sol 0.49.0'



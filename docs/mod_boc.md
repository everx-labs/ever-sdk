# Module boc

## Module boc

BOC manipulation module.

### Functions

[parse\_message](mod_boc.md#parse_message) – Parses message boc into a JSON

[parse\_transaction](mod_boc.md#parse_transaction) – Parses transaction boc into a JSON

[parse\_account](mod_boc.md#parse_account) – Parses account boc into a JSON

[parse\_block](mod_boc.md#parse_block) – Parses block boc into a JSON

[parse\_shardstate](mod_boc.md#parse_shardstate) – Parses shardstate boc into a JSON

[get\_blockchain\_config](mod_boc.md#get_blockchain_config) – Extract blockchain configuration from key block and also from zerostate.

[get\_boc\_hash](mod_boc.md#get_boc_hash) – Calculates BOC root hash

[get\_code\_from\_tvc](mod_boc.md#get_code_from_tvc) – Extracts code from TVC contract image

[cache\_get](mod_boc.md#cache_get) – Get BOC from cache

[cache\_set](mod_boc.md#cache_set) – Save BOC into cache

[cache\_unpin](mod_boc.md#cache_unpin) – Unpin BOCs with specified pin.

[encode\_boc](mod_boc.md#encode_boc) – Encodes bag of cells \(BOC\) with builder operations. This method provides the same functionality as Solidity TvmBuilder. Resulting BOC of this method can be passed into Solidity and C++ contracts as TvmCell type

### Types

[BocCacheType](mod_boc.md#BocCacheType)

[BocErrorCode](mod_boc.md#BocErrorCode)

[ParamsOfParse](mod_boc.md#ParamsOfParse)

[ResultOfParse](mod_boc.md#ResultOfParse)

[ParamsOfParseShardstate](mod_boc.md#ParamsOfParseShardstate)

[ParamsOfGetBlockchainConfig](mod_boc.md#ParamsOfGetBlockchainConfig)

[ResultOfGetBlockchainConfig](mod_boc.md#ResultOfGetBlockchainConfig)

[ParamsOfGetBocHash](mod_boc.md#ParamsOfGetBocHash)

[ResultOfGetBocHash](mod_boc.md#ResultOfGetBocHash)

[ParamsOfGetCodeFromTvc](mod_boc.md#ParamsOfGetCodeFromTvc)

[ResultOfGetCodeFromTvc](mod_boc.md#ResultOfGetCodeFromTvc)

[ParamsOfBocCacheGet](mod_boc.md#ParamsOfBocCacheGet)

[ResultOfBocCacheGet](mod_boc.md#ResultOfBocCacheGet)

[ParamsOfBocCacheSet](mod_boc.md#ParamsOfBocCacheSet)

[ResultOfBocCacheSet](mod_boc.md#ResultOfBocCacheSet)

[ParamsOfBocCacheUnpin](mod_boc.md#ParamsOfBocCacheUnpin)

[BuilderOp](mod_boc.md#BuilderOp) – Cell builder operation.

[ParamsOfEncodeBoc](mod_boc.md#ParamsOfEncodeBoc)

[ResultOfEncodeBoc](mod_boc.md#ResultOfEncodeBoc)

## Functions

### parse\_message

Parses message boc into a JSON

JSON structure is compatible with GraphQL API message object

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64

#### Result

* `parsed`: _any_ – JSON containing parsed BOC

### parse\_transaction

Parses transaction boc into a JSON

JSON structure is compatible with GraphQL API transaction object

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64

#### Result

* `parsed`: _any_ – JSON containing parsed BOC

### parse\_account

Parses account boc into a JSON

JSON structure is compatible with GraphQL API account object

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64

#### Result

* `parsed`: _any_ – JSON containing parsed BOC

### parse\_block

Parses block boc into a JSON

JSON structure is compatible with GraphQL API block object

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64

#### Result

* `parsed`: _any_ – JSON containing parsed BOC

### parse\_shardstate

Parses shardstate boc into a JSON

JSON structure is compatible with GraphQL API shardstate object

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64
* `id`: _string_ – Shardstate identificator
* `workchain_id`: _number_ – Workchain shardstate belongs to

#### Result

* `parsed`: _any_ – JSON containing parsed BOC

### get\_blockchain\_config

Extract blockchain configuration from key block and also from zerostate.

```typescript
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

#### Parameters

* `block_boc`: _string_ – Key block BOC or zerostate BOC encoded as base64

#### Result

* `config_boc`: _string_ – Blockchain config BOC encoded as base64

### get\_boc\_hash

Calculates BOC root hash

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64

#### Result

* `hash`: _string_ – BOC root hash encoded with hex

### get\_code\_from\_tvc

Extracts code from TVC contract image

```typescript
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

#### Parameters

* `tvc`: _string_ – Contract TVC image encoded as base64

#### Result

* `code`: _string_ – Contract code encoded as base64

### cache\_get

Get BOC from cache

```typescript
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

#### Parameters

* `boc_ref`: _string_ – Reference to the cached BOC

#### Result

* `boc`?: _string_ – BOC encoded as base64.

### cache\_set

Save BOC into cache

```typescript
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

#### Parameters

* `boc`: _string_ – BOC encoded as base64 or BOC reference
* `cache_type`: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type

#### Result

* `boc_ref`: _string_ – Reference to the cached BOC

### cache\_unpin

Unpin BOCs with specified pin.

BOCs which don't have another pins will be removed from cache

```typescript
type ParamsOfBocCacheUnpin = {
    pin: string,
    boc_ref?: string
}

function cache_unpin(
    params: ParamsOfBocCacheUnpin,
): Promise<void>;
```

#### Parameters

* `pin`: _string_ – Pinned name
* `boc_ref`?: _string_ – Reference to the cached BOC.

  
  If it is provided then only referenced BOC is unpinned

### encode\_boc

Encodes bag of cells \(BOC\) with builder operations. This method provides the same functionality as Solidity TvmBuilder. Resulting BOC of this method can be passed into Solidity and C++ contracts as TvmCell type

```typescript
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
```

#### Parameters

* `builder`: [_BuilderOp_](mod_boc.md#BuilderOp)_\[\]_ – Cell builder operations.
* `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result. The BOC itself returned if no cache type provided.

#### Result

* `boc`: _string_ – Encoded cell BOC or BOC cache key.

## Types

### BocCacheType

```typescript
type BocCacheType = {
    type: 'Pinned'
    pin: string
} | {
    type: 'Unpinned'
}
```

Depends on value of the `type` field.

When _type_ is _'Pinned'_

Pin the BOC with `pin` name.

Such BOC will not be removed from cache until it is unpinned

* `pin`: _string_

When _type_ is _'Unpinned'_

Variant constructors:

```typescript
function bocCacheTypePinned(pin: string): BocCacheType;
function bocCacheTypeUnpinned(): BocCacheType;
```

### BocErrorCode

```typescript
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

* `InvalidBoc = 201`
* `SerializationError = 202`
* `InappropriateBlock = 203`
* `MissingSourceBoc = 204`
* `InsufficientCacheSize = 205`
* `BocRefNotFound = 206`
* `InvalidBocRef = 207`

### ParamsOfParse

```typescript
type ParamsOfParse = {
    boc: string
}
```

* `boc`: _string_ – BOC encoded as base64

### ResultOfParse

```typescript
type ResultOfParse = {
    parsed: any
}
```

* `parsed`: _any_ – JSON containing parsed BOC

### ParamsOfParseShardstate

```typescript
type ParamsOfParseShardstate = {
    boc: string,
    id: string,
    workchain_id: number
}
```

* `boc`: _string_ – BOC encoded as base64
* `id`: _string_ – Shardstate identificator
* `workchain_id`: _number_ – Workchain shardstate belongs to

### ParamsOfGetBlockchainConfig

```typescript
type ParamsOfGetBlockchainConfig = {
    block_boc: string
}
```

* `block_boc`: _string_ – Key block BOC or zerostate BOC encoded as base64

### ResultOfGetBlockchainConfig

```typescript
type ResultOfGetBlockchainConfig = {
    config_boc: string
}
```

* `config_boc`: _string_ – Blockchain config BOC encoded as base64

### ParamsOfGetBocHash

```typescript
type ParamsOfGetBocHash = {
    boc: string
}
```

* `boc`: _string_ – BOC encoded as base64

### ResultOfGetBocHash

```typescript
type ResultOfGetBocHash = {
    hash: string
}
```

* `hash`: _string_ – BOC root hash encoded with hex

### ParamsOfGetCodeFromTvc

```typescript
type ParamsOfGetCodeFromTvc = {
    tvc: string
}
```

* `tvc`: _string_ – Contract TVC image encoded as base64

### ResultOfGetCodeFromTvc

```typescript
type ResultOfGetCodeFromTvc = {
    code: string
}
```

* `code`: _string_ – Contract code encoded as base64

### ParamsOfBocCacheGet

```typescript
type ParamsOfBocCacheGet = {
    boc_ref: string
}
```

* `boc_ref`: _string_ – Reference to the cached BOC

### ResultOfBocCacheGet

```typescript
type ResultOfBocCacheGet = {
    boc?: string
}
```

* `boc`?: _string_ – BOC encoded as base64.

### ParamsOfBocCacheSet

```typescript
type ParamsOfBocCacheSet = {
    boc: string,
    cache_type: BocCacheType
}
```

* `boc`: _string_ – BOC encoded as base64 or BOC reference
* `cache_type`: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type

### ResultOfBocCacheSet

```typescript
type ResultOfBocCacheSet = {
    boc_ref: string
}
```

* `boc_ref`: _string_ – Reference to the cached BOC

### ParamsOfBocCacheUnpin

```typescript
type ParamsOfBocCacheUnpin = {
    pin: string,
    boc_ref?: string
}
```

* `pin`: _string_ – Pinned name
* `boc_ref`?: _string_ – Reference to the cached BOC.

  
  If it is provided then only referenced BOC is unpinned

### BuilderOp

Cell builder operation.

```typescript
type BuilderOp = {
    type: 'Integer'
    size: number,
    value: any
} | {
    type: 'BitString'
    value: string
} | {
    type: 'Cell'
    builder: BuilderOp[]
} | {
    type: 'CellBoc'
    boc: string
}
```

Depends on value of the `type` field.

When _type_ is _'Integer'_

Append integer to cell data.

* `size`: _number_ – Bit size of the value.
* `value`: _any_ – Value: - `Number` containing integer number.

  
  e.g. `123`, `-123`. - Decimal string. e.g. `"123"`, `"-123"`.  
  - `0x` prefixed hexadecimal string.  
    e.g `0x123`, `0X123`, `-0x123`.

When _type_ is _'BitString'_

Append bit string to cell data.

* `value`: _string_ – Bit string content using bitstring notation. See `TON VM specification` 1.0.

  
  Contains hexadecimal string representation:  
  - Can end with `_` tag.  
  - Can be prefixed with `x` or `X`.  
  - Can be prefixed with `x{` or `X{` and ended with `}`.  
  
  Contains binary string represented as a sequence  
  of `0` and `1` prefixed with `n` or `N`.  
  
  Examples:  
  `1AB`, `x1ab`, `X1AB`, `x{1abc}`, `X{1ABC}`  
  `2D9_`, `x2D9_`, `X2D9_`, `x{2D9_}`, `X{2D9_}`  
  `n00101101100`, `N00101101100`

When _type_ is _'Cell'_

Append ref to nested cells

* `builder`: [_BuilderOp_](mod_boc.md#BuilderOp)_\[\]_ – Nested cell builder

When _type_ is _'CellBoc'_

Append ref to nested cell

* `boc`: _string_ – Nested cell BOC encoded with `base64` or BOC cache key.

Variant constructors:

```typescript
function builderOpInteger(size: number, value: any): BuilderOp;
function builderOpBitString(value: string): BuilderOp;
function builderOpCell(builder: BuilderOp[]): BuilderOp;
function builderOpCellBoc(boc: string): BuilderOp;
```

### ParamsOfEncodeBoc

```typescript
type ParamsOfEncodeBoc = {
    builder: BuilderOp[],
    boc_cache?: BocCacheType
}
```

* `builder`: [_BuilderOp_](mod_boc.md#BuilderOp)_\[\]_ – Cell builder operations.
* `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result. The BOC itself returned if no cache type provided.

### ResultOfEncodeBoc

```typescript
type ResultOfEncodeBoc = {
    boc: string
}
```

* `boc`: _string_ – Encoded cell BOC or BOC cache key.


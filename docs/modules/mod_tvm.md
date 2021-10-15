# Module tvm

## Module tvm

### Functions

[run_executor](mod_tvm.md#run_executor) – Emulates all the phases of contract execution locally

[run_tvm](mod_tvm.md#run_tvm) – Executes get-methods of ABI-compatible contracts

[run_get](mod_tvm.md#run_get) – Executes a get-method of FIFT contract

### Types

[TvmErrorCode](mod_tvm.md#TvmErrorCode)

[ExecutionOptions](mod_tvm.md#ExecutionOptions)

[AccountForExecutor](mod_tvm.md#AccountForExecutor)

[TransactionFees](mod_tvm.md#TransactionFees)

[ParamsOfRunExecutor](mod_tvm.md#ParamsOfRunExecutor)

[ResultOfRunExecutor](mod_tvm.md#ResultOfRunExecutor)

[ParamsOfRunTvm](mod_tvm.md#ParamsOfRunTvm)

[ResultOfRunTvm](mod_tvm.md#ResultOfRunTvm)

[ParamsOfRunGet](mod_tvm.md#ParamsOfRunGet)

[ResultOfRunGet](mod_tvm.md#ResultOfRunGet)

## Functions

### run_executor

Emulates all the phases of contract execution locally

Performs all the phases of contract execution on Transaction Executor - the same component that is used on Validator Nodes.

Can be used for contract debugging, to find out the reason why a message was not delivered successfully. Validators throw away the failed external inbound messages (if they failed bedore `ACCEPT`) in the real network. This is why these messages are impossible to debug in the real network. With the help of run_executor you can do that. In fact, `process_message` function performs local check with `run_executor` if there was no transaction as a result of processing and returns the error, if there is one.

Another use case to use `run_executor` is to estimate fees for message execution. Set `AccountForExecutor::Account.unlimited_balance` to `true` so that emulation will not depend on the actual balance. This may be needed to calculate deploy fees for an account that does not exist yet. JSON with fees is in `fees` field of the result.

One more use case - you can produce the sequence of operations, thus emulating the sequential contract calls locally. And so on.

Transaction executor requires account BOC (bag of cells) as a parameter. To get the account BOC - use `net.query` method to download it from GraphQL API (field `boc` of `account`) or generate it with `abi.encode_account` method.

Also it requires message BOC. To get the message BOC - use `abi.encode_message` or `abi.encode_internal_message`.

If you need this emulation to be as precise as possible (for instance - emulate transaction with particular lt in particular block or use particular blockchain config, downloaded from a particular key block - then specify `execution_options` parameter.

If you need to see the aborted transaction as a result, not as an error, set `skip_transaction_check` to `true`.

```typescript
type ParamsOfRunExecutor = {
    message: string,
    account: AccountForExecutor,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    skip_transaction_check?: boolean,
    boc_cache?: BocCacheType,
    return_updated_account?: boolean
}

type ResultOfRunExecutor = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string,
    fees: TransactionFees
}

function run_executor(
    params: ParamsOfRunExecutor,
): Promise<ResultOfRunExecutor>;
```

#### Parameters

*   `message`: _string_ – Input message BOC.

    \
    Must be encoded as base64.
* `account`: [_AccountForExecutor_](mod_tvm.md#AccountForExecutor) – Account to run on executor
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options.
* `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI for decoding output messages
* `skip_transaction_check`?: _boolean_ – Skip transaction check flag
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided
*   `return_updated_account`?: _boolean_ – Return updated account flag.

    \
    Empty string is returned if the flag is `false`

#### Result

*   `transaction`: _any_ – Parsed transaction.

    \
    In addition to the regular transaction fields there is a\
    `boc` field encoded with `base64` which contains source\
    transaction BOC.
*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
*   `account`: _string_ – Updated account state BOC.

    \
    Encoded as `base64`
* `fees`: [_TransactionFees_](mod_tvm.md#TransactionFees) – Transaction fees

### run_tvm

Executes get-methods of ABI-compatible contracts

Performs only a part of compute phase of transaction execution that is used to run get-methods of ABI-compatible contracts.

If you try to run get-methods with `run_executor` you will get an error, because it checks ACCEPT and exits if there is none, which is actually true for get-methods.

To get the account BOC (bag of cells) - use `net.query` method to download it from GraphQL API (field `boc` of `account`) or generate it with `abi.encode_account method`. To get the message BOC - use `abi.encode_message` or prepare it any other way, for instance, with FIFT script.

Attention! Updated account state is produces as well, but only `account_state.storage.state.data` part of the BOC is updated.

```typescript
type ParamsOfRunTvm = {
    message: string,
    account: string,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    boc_cache?: BocCacheType,
    return_updated_account?: boolean
}

type ResultOfRunTvm = {
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string
}

function run_tvm(
    params: ParamsOfRunTvm,
): Promise<ResultOfRunTvm>;
```

#### Parameters

*   `message`: _string_ – Input message BOC.

    \
    Must be encoded as base64.
*   `account`: _string_ – Account BOC.

    \
    Must be encoded as base64.
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options.
* `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI for decoding output messages
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided
*   `return_updated_account`?: _boolean_ – Return updated account flag.

    \
    Empty string is returned if the flag is `false`

#### Result

*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
*   `account`: _string_ – Updated account state BOC.

    \
    Encoded as `base64`. Attention! Only `account_state.storage.state.data` part of the BOC is updated.

### run_get

Executes a get-method of FIFT contract

Executes a get-method of FIFT contract that fulfills the smc-guidelines [https://test.ton.org/smc-guidelines.txt](https://test.ton.org/smc-guidelines.txt) and returns the result data from TVM's stack

```typescript
type ParamsOfRunGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions,
    tuple_list_as_array?: boolean
}

type ResultOfRunGet = {
    output: any
}

function run_get(
    params: ParamsOfRunGet,
): Promise<ResultOfRunGet>;
```

#### Parameters

* `account`: _string_ – Account BOC in `base64`
* `function_name`: _string_ – Function name
* `input`?: _any_ – Input parameters
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options
*   `tuple_list_as_array`?: _boolean_ – Convert lists based on nested tuples in the **result** into plain arrays.

    \
    Default is `false`. Input parameters may use any of lists representations\
    If you receive this error on Web: "Runtime error. Unreachable code should not be executed...",\
    set this flag to true.\
    This may happen, for example, when elector contract contains too many participants

#### Result

* `output`: _any_ – Values returned by get-method on stack

## Types

### TvmErrorCode

```typescript
enum TvmErrorCode {
    CanNotReadTransaction = 401,
    CanNotReadBlockchainConfig = 402,
    TransactionAborted = 403,
    InternalError = 404,
    ActionPhaseFailed = 405,
    AccountCodeMissing = 406,
    LowBalance = 407,
    AccountFrozenOrDeleted = 408,
    AccountMissing = 409,
    UnknownExecutionError = 410,
    InvalidInputStack = 411,
    InvalidAccountBoc = 412,
    InvalidMessageType = 413,
    ContractExecutionError = 414
}
```

One of the following value:

* `CanNotReadTransaction = 401`
* `CanNotReadBlockchainConfig = 402`
* `TransactionAborted = 403`
* `InternalError = 404`
* `ActionPhaseFailed = 405`
* `AccountCodeMissing = 406`
* `LowBalance = 407`
* `AccountFrozenOrDeleted = 408`
* `AccountMissing = 409`
* `UnknownExecutionError = 410`
* `InvalidInputStack = 411`
* `InvalidAccountBoc = 412`
* `InvalidMessageType = 413`
* `ContractExecutionError = 414`

### ExecutionOptions

```typescript
type ExecutionOptions = {
    blockchain_config?: string,
    block_time?: number,
    block_lt?: bigint,
    transaction_lt?: bigint
}
```

* `blockchain_config`?: _string_ – boc with config
* `block_time`?: _number_ – time that is used as transaction time
* `block_lt`?: _bigint_ – block logical time
* `transaction_lt`?: _bigint_ – transaction logical time

### AccountForExecutor

```typescript
type AccountForExecutor = {
    type: 'None'
} | {
    type: 'Uninit'
} | {
    type: 'Account'
    boc: string,
    unlimited_balance?: boolean
}
```

Depends on value of the `type` field.

When _type_ is _'None'_

Non-existing account to run a creation internal message. Should be used with `skip_transaction_check = true` if the message has no deploy data since transactions on the uninitialized account are always aborted

When _type_ is _'Uninit'_

Emulate uninitialized account to run deploy message

When _type_ is _'Account'_

Account state to run message

*   `boc`: _string_ – Account BOC.

    \
    Encoded as base64.
*   `unlimited_balance`?: _boolean_ – Flag for running account with the unlimited balance.

    \
    Can be used to calculate transaction fees without balance check

Variant constructors:

```typescript
function accountForExecutorNone(): AccountForExecutor;
function accountForExecutorUninit(): AccountForExecutor;
function accountForExecutorAccount(boc: string, unlimited_balance?: boolean): AccountForExecutor;
```

### TransactionFees

```typescript
type TransactionFees = {
    in_msg_fwd_fee: bigint,
    storage_fee: bigint,
    gas_fee: bigint,
    out_msgs_fwd_fee: bigint,
    total_account_fees: bigint,
    total_output: bigint
}
```

* `in_msg_fwd_fee`: _bigint_
* `storage_fee`: _bigint_
* `gas_fee`: _bigint_
* `out_msgs_fwd_fee`: _bigint_
* `total_account_fees`: _bigint_
* `total_output`: _bigint_

### ParamsOfRunExecutor

```typescript
type ParamsOfRunExecutor = {
    message: string,
    account: AccountForExecutor,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    skip_transaction_check?: boolean,
    boc_cache?: BocCacheType,
    return_updated_account?: boolean
}
```

*   `message`: _string_ – Input message BOC.

    \
    Must be encoded as base64.
* `account`: [_AccountForExecutor_](mod_tvm.md#AccountForExecutor) – Account to run on executor
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options.
* `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI for decoding output messages
* `skip_transaction_check`?: _boolean_ – Skip transaction check flag
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided
*   `return_updated_account`?: _boolean_ – Return updated account flag.

    \
    Empty string is returned if the flag is `false`

### ResultOfRunExecutor

```typescript
type ResultOfRunExecutor = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string,
    fees: TransactionFees
}
```

*   `transaction`: _any_ – Parsed transaction.

    \
    In addition to the regular transaction fields there is a\
    `boc` field encoded with `base64` which contains source\
    transaction BOC.
*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
*   `account`: _string_ – Updated account state BOC.

    \
    Encoded as `base64`
* `fees`: [_TransactionFees_](mod_tvm.md#TransactionFees) – Transaction fees

### ParamsOfRunTvm

```typescript
type ParamsOfRunTvm = {
    message: string,
    account: string,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    boc_cache?: BocCacheType,
    return_updated_account?: boolean
}
```

*   `message`: _string_ – Input message BOC.

    \
    Must be encoded as base64.
*   `account`: _string_ – Account BOC.

    \
    Must be encoded as base64.
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options.
* `abi`?: [_Abi_](mod_abi.md#Abi) – Contract ABI for decoding output messages
*   `boc_cache`?: [_BocCacheType_](mod_boc.md#BocCacheType) – Cache type to put the result.

    \
    The BOC itself returned if no cache type provided
*   `return_updated_account`?: _boolean_ – Return updated account flag.

    \
    Empty string is returned if the flag is `false`

### ResultOfRunTvm

```typescript
type ResultOfRunTvm = {
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string
}
```

*   `out_messages`: _string\[]_ – List of output messages' BOCs.

    \
    Encoded as `base64`
* `decoded`?: [_DecodedOutput_](mod_processing.md#DecodedOutput) – Optional decoded message bodies according to the optional `abi` parameter.
*   `account`: _string_ – Updated account state BOC.

    \
    Encoded as `base64`. Attention! Only `account_state.storage.state.data` part of the BOC is updated.

### ParamsOfRunGet

```typescript
type ParamsOfRunGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions,
    tuple_list_as_array?: boolean
}
```

* `account`: _string_ – Account BOC in `base64`
* `function_name`: _string_ – Function name
* `input`?: _any_ – Input parameters
* `execution_options`?: [_ExecutionOptions_](mod_tvm.md#ExecutionOptions) – Execution options
*   `tuple_list_as_array`?: _boolean_ – Convert lists based on nested tuples in the **result** into plain arrays.

    \
    Default is `false`. Input parameters may use any of lists representations\
    If you receive this error on Web: "Runtime error. Unreachable code should not be executed...",\
    set this flag to true.\
    This may happen, for example, when elector contract contains too many participants

### ResultOfRunGet

```typescript
type ResultOfRunGet = {
    output: any
}
```

* `output`: _any_ – Values returned by get-method on stack

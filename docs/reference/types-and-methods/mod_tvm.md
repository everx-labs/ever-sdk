# Module tvm


## Functions
[run_executor](mod\_tvm.md#run_executor) – Emulates all the phases of contract execution locally

[run_tvm](mod\_tvm.md#run_tvm) – Executes get-methods of ABI-compatible contracts

[run_get](mod\_tvm.md#run_get) – Executes a get-method of FIFT contract

## Types
[TvmErrorCode](mod\_tvm.md#tvmerrorcode)

[ExecutionOptions](mod\_tvm.md#executionoptions)

[AccountForExecutorNoneVariant](mod\_tvm.md#accountforexecutornonevariant) – Non-existing account to run a creation internal message. Should be used with `skip_transaction_check = true` if the message has no deploy data since transactions on the uninitialized account are always aborted

[AccountForExecutorUninitVariant](mod\_tvm.md#accountforexecutoruninitvariant) – Emulate uninitialized account to run deploy message

[AccountForExecutorAccountVariant](mod\_tvm.md#accountforexecutoraccountvariant) – Account state to run message

[AccountForExecutor](mod\_tvm.md#accountforexecutor)

[TransactionFees](mod\_tvm.md#transactionfees)

[ParamsOfRunExecutor](mod\_tvm.md#paramsofrunexecutor)

[ResultOfRunExecutor](mod\_tvm.md#resultofrunexecutor)

[ParamsOfRunTvm](mod\_tvm.md#paramsofruntvm)

[ResultOfRunTvm](mod\_tvm.md#resultofruntvm)

[ParamsOfRunGet](mod\_tvm.md#paramsofrunget)

[ResultOfRunGet](mod\_tvm.md#resultofrunget)


# Functions
## run_executor

Emulates all the phases of contract execution locally

Performs all the phases of contract execution on Transaction Executor -
the same component that is used on Validator Nodes.

Can be used for contract debugging, to find out the reason why a message was not delivered successfully.
Validators throw away the failed external inbound messages (if they failed bedore `ACCEPT`) in the real network.
This is why these messages are impossible to debug in the real network.
With the help of run_executor you can do that. In fact, `process_message` function
performs local check with `run_executor` if there was no transaction as a result of processing
and returns the error, if there is one.

Another use case to use `run_executor` is to estimate fees for message execution.
Set  `AccountForExecutor::Account.unlimited_balance`
to `true` so that emulation will not depend on the actual balance.
This may be needed to calculate deploy fees for an account that does not exist yet.
JSON with fees is in `fees` field of the result.

One more use case - you can produce the sequence of operations,
thus emulating the sequential contract calls locally.
And so on.

Transaction executor requires account BOC (bag of cells) as a parameter.
To get the account BOC - use `net.query` method to download it from GraphQL API
(field `boc` of `account`) or generate it with `abi.encode_account` method.

Also it requires message BOC. To get the message BOC - use `abi.encode_message` or `abi.encode_internal_message`.

If you need this emulation to be as precise as possible (for instance - emulate transaction
with particular lt in particular block or use particular blockchain config,
downloaded from a particular key block - then specify `execution_options` parameter.

If you need to see the aborted transaction as a result, not as an error, set `skip_transaction_check` to `true`.

```ts
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
### Parameters
- `message`: _string_ – Input message BOC.
<br>Must be encoded as base64.
- `account`: _[AccountForExecutor](mod\_tvm.md#accountforexecutor)_ – Account to run on executor
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI for decoding output messages
- `skip_transaction_check`?: _boolean_ – Skip transaction check flag
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided
- `return_updated_account`?: _boolean_ – Return updated account flag.
<br>Empty string is returned if the flag is `false`


### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC.
<br>Encoded as `base64`
- `fees`: _[TransactionFees](mod\_tvm.md#transactionfees)_ – Transaction fees


## run_tvm

Executes get-methods of ABI-compatible contracts

Performs only a part of compute phase of transaction execution
that is used to run get-methods of ABI-compatible contracts.

If you try to run get-methods with `run_executor` you will get an error, because it checks ACCEPT and exits
if there is none, which is actually true for get-methods.

 To get the account BOC (bag of cells) - use `net.query` method to download it from GraphQL API
(field `boc` of `account`) or generate it with `abi.encode_account method`.
To get the message BOC - use `abi.encode_message` or prepare it any other way, for instance, with FIFT script.

Attention! Updated account state is produces as well, but only
`account_state.storage.state.data`  part of the BOC is updated.

```ts
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
### Parameters
- `message`: _string_ – Input message BOC.
<br>Must be encoded as base64.
- `account`: _string_ – Account BOC.
<br>Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI for decoding output messages
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided
- `return_updated_account`?: _boolean_ – Return updated account flag.
<br>Empty string is returned if the flag is `false`


### Result

- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC.
<br>Encoded as `base64`. Attention! Only `account_state.storage.state.data` part of the BOC is updated.


## run_get

Executes a get-method of FIFT contract

Executes a get-method of FIFT contract that fulfills the smc-guidelines https://test.ton.org/smc-guidelines.txt
and returns the result data from TVM's stack

```ts
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
### Parameters
- `account`: _string_ – Account BOC in `base64`
- `function_name`: _string_ – Function name
- `input`?: _any_ – Input parameters
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options
- `tuple_list_as_array`?: _boolean_ – Convert lists based on nested tuples in the **result** into plain arrays.
<br>Default is `false`. Input parameters may use any of lists representations<br>If you receive this error on Web: "Runtime error. Unreachable code should not be executed...",<br>set this flag to true.<br>This may happen, for example, when elector contract contains too many participants


### Result

- `output`: _any_ – Values returned by get-method on stack


# Types
## TvmErrorCode
```ts
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

- `CanNotReadTransaction = 401`
- `CanNotReadBlockchainConfig = 402`
- `TransactionAborted = 403`
- `InternalError = 404`
- `ActionPhaseFailed = 405`
- `AccountCodeMissing = 406`
- `LowBalance = 407`
- `AccountFrozenOrDeleted = 408`
- `AccountMissing = 409`
- `UnknownExecutionError = 410`
- `InvalidInputStack = 411`
- `InvalidAccountBoc = 412`
- `InvalidMessageType = 413`
- `ContractExecutionError = 414`


## ExecutionOptions
```ts
type ExecutionOptions = {
    blockchain_config?: string,
    block_time?: number,
    block_lt?: bigint,
    transaction_lt?: bigint
}
```
- `blockchain_config`?: _string_ – boc with config
- `block_time`?: _number_ – time that is used as transaction time
- `block_lt`?: _bigint_ – block logical time
- `transaction_lt`?: _bigint_ – transaction logical time


## AccountForExecutorNoneVariant
Non-existing account to run a creation internal message. Should be used with `skip_transaction_check = true` if the message has no deploy data since transactions on the uninitialized account are always aborted

```ts
type AccountForExecutorNoneVariant = {

}
```


## AccountForExecutorUninitVariant
Emulate uninitialized account to run deploy message

```ts
type AccountForExecutorUninitVariant = {

}
```


## AccountForExecutorAccountVariant
Account state to run message

```ts
type AccountForExecutorAccountVariant = {
    boc: string,
    unlimited_balance?: boolean
}
```
- `boc`: _string_ – Account BOC.
<br>Encoded as base64.
- `unlimited_balance`?: _boolean_ – Flag for running account with the unlimited balance.
<br>Can be used to calculate transaction fees without balance check


## AccountForExecutor
```ts
type AccountForExecutor = ({
    type: 'None'
} & AccountForExecutorNoneVariant) | ({
    type: 'Uninit'
} & AccountForExecutorUninitVariant) | ({
    type: 'Account'
} & AccountForExecutorAccountVariant)
```
Depends on value of the  `type` field.

When _type_ is _'None'_

Non-existing account to run a creation internal message. Should be used with `skip_transaction_check = true` if the message has no deploy data since transactions on the uninitialized account are always aborted


When _type_ is _'Uninit'_

Emulate uninitialized account to run deploy message


When _type_ is _'Account'_

Account state to run message

- `boc`: _string_ – Account BOC.
<br>Encoded as base64.
- `unlimited_balance`?: _boolean_ – Flag for running account with the unlimited balance.
<br>Can be used to calculate transaction fees without balance check


Variant constructors:

```ts
function accountForExecutorNone(): AccountForExecutor;
function accountForExecutorUninit(): AccountForExecutor;
function accountForExecutorAccount(boc: string, unlimited_balance?: boolean): AccountForExecutor;
```

## TransactionFees
```ts
type TransactionFees = {
    in_msg_fwd_fee: bigint,
    storage_fee: bigint,
    gas_fee: bigint,
    out_msgs_fwd_fee: bigint,
    total_account_fees: bigint,
    total_output: bigint,
    ext_in_msg_fee: bigint,
    total_fwd_fees: bigint,
    account_fees: bigint
}
```
- `in_msg_fwd_fee`: _bigint_ – Deprecated.
<br>Left for backward compatibility. Does not participate in account transaction fees calculation.
- `storage_fee`: _bigint_ – Fee for account storage
- `gas_fee`: _bigint_ – Fee for processing
- `out_msgs_fwd_fee`: _bigint_ – Deprecated.
<br>Contains the same data as total_fwd_fees field. Deprecated because of its confusing name, that is not the same with GraphQL API Transaction type's field.
- `total_account_fees`: _bigint_ – Deprecated.
<br>This is the field that is named as `total_fees` in GraphQL API Transaction type. `total_account_fees` name is misleading, because it does not mean account fees, instead it means<br>validators total fees received for the transaction execution. It does not include some forward fees that account<br>actually pays now, but validators will receive later during value delivery to another account (not even in the receiving<br>transaction).<br>Because of all of this, this field is not interesting for those who wants to understand<br>the real account fees, this is why it is deprecated and left for backward compatibility.
- `total_output`: _bigint_ – Deprecated because it means total value sent in the transaction, which does not relate to any fees.
- `ext_in_msg_fee`: _bigint_ – Fee for inbound external message import.
- `total_fwd_fees`: _bigint_ – Total fees the account pays for message forwarding
- `account_fees`: _bigint_ – Total account fees for the transaction execution. Compounds of storage_fee + gas_fee + ext_in_msg_fee + total_fwd_fees


## ParamsOfRunExecutor
```ts
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
- `message`: _string_ – Input message BOC.
<br>Must be encoded as base64.
- `account`: _[AccountForExecutor](mod\_tvm.md#accountforexecutor)_ – Account to run on executor
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI for decoding output messages
- `skip_transaction_check`?: _boolean_ – Skip transaction check flag
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided
- `return_updated_account`?: _boolean_ – Return updated account flag.
<br>Empty string is returned if the flag is `false`


## ResultOfRunExecutor
```ts
type ResultOfRunExecutor = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string,
    fees: TransactionFees
}
```
- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC.
<br>Encoded as `base64`
- `fees`: _[TransactionFees](mod\_tvm.md#transactionfees)_ – Transaction fees


## ParamsOfRunTvm
```ts
type ParamsOfRunTvm = {
    message: string,
    account: string,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    boc_cache?: BocCacheType,
    return_updated_account?: boolean
}
```
- `message`: _string_ – Input message BOC.
<br>Must be encoded as base64.
- `account`: _string_ – Account BOC.
<br>Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Contract ABI for decoding output messages
- `boc_cache`?: _[BocCacheType](mod\_boc.md#boccachetype)_ – Cache type to put the result.
<br>The BOC itself returned if no cache type provided
- `return_updated_account`?: _boolean_ – Return updated account flag.
<br>Empty string is returned if the flag is `false`


## ResultOfRunTvm
```ts
type ResultOfRunTvm = {
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string
}
```
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC.
<br>Encoded as `base64`. Attention! Only `account_state.storage.state.data` part of the BOC is updated.


## ParamsOfRunGet
```ts
type ParamsOfRunGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions,
    tuple_list_as_array?: boolean
}
```
- `account`: _string_ – Account BOC in `base64`
- `function_name`: _string_ – Function name
- `input`?: _any_ – Input parameters
- `execution_options`?: _[ExecutionOptions](mod\_tvm.md#executionoptions)_ – Execution options
- `tuple_list_as_array`?: _boolean_ – Convert lists based on nested tuples in the **result** into plain arrays.
<br>Default is `false`. Input parameters may use any of lists representations<br>If you receive this error on Web: "Runtime error. Unreachable code should not be executed...",<br>set this flag to true.<br>This may happen, for example, when elector contract contains too many participants


## ResultOfRunGet
```ts
type ResultOfRunGet = {
    output: any
}
```
- `output`: _any_ – Values returned by get-method on stack



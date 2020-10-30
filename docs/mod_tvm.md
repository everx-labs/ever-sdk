# Module tvm

null
## Functions
[run_executor](#run_executor)

[run_tvm](#run_tvm)

[run_get](#run_get) – Executes getmethod and returns data from TVM stack

## Types
[ExecutionOptions](#ExecutionOptions)

[AccountForExecutor](#AccountForExecutor)

[ParamsOfRunExecutor](#ParamsOfRunExecutor)

[ResultOfRunExecutor](#ResultOfRunExecutor)

[ParamsOfRunTvm](#ParamsOfRunTvm)

[ResultOfRunTvm](#ResultOfRunTvm)

[ParamsOfRunGet](#ParamsOfRunGet)

[ResultOfRunGet](#ResultOfRunGet)


# Functions
## run_executor

```ts
type ParamsOfRunExecutor = {
    message: string,
    account: AccountForExecutor,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    skip_transaction_check?: boolean
};

type ResultOfRunExecutor = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string,
    fees: TransactionFees
};

function run_executor(
    params: ParamsOfRunExecutor,
): Promise<ResultOfRunExecutor>;
```
### Parameters
- `message`: _string_ – Input message BOC. Must be encoded as base64.
- `account`: _[AccountForExecutor](mod_tvm.md#AccountForExecutor)_ – Account to run on executor
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for decoding output messages
- `skip_transaction_check`?: _boolean_ – Skip transaction check flag
### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC. Encoded as `base64`
- `fees`: _TransactionFees_ – Transaction fees


## run_tvm

```ts
type ParamsOfRunTvm = {
    message: string,
    account: string,
    execution_options?: ExecutionOptions,
    abi?: Abi
};

type ResultOfRunTvm = {
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string
};

function run_tvm(
    params: ParamsOfRunTvm,
): Promise<ResultOfRunTvm>;
```
### Parameters
- `message`: _string_ – Input message BOC. Must be encoded as base64.
- `account`: _string_ – Account BOC. Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for dedcoding output messages
### Result

- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC. Encoded as `base64`. Attention! Only data in account state is updated.


## run_get

Executes getmethod and returns data from TVM stack

```ts
type ParamsOfRunGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions
};

type ResultOfRunGet = {
    output: any
};

function run_get(
    params: ParamsOfRunGet,
): Promise<ResultOfRunGet>;
```
### Parameters
- `account`: _string_ – Account BOC in `base64`
- `function_name`: _string_ – Function name
- `input`?: _any_ – Input parameters
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_
### Result

- `output`: _any_ – Values returned by getmethod on stack


# Types
## ExecutionOptions
```ts
type ExecutionOptions = {
    blockchain_config?: string,
    block_time?: number,
    block_lt?: bigint,
    transaction_lt?: bigint
};
```
- `blockchain_config`?: _string_ – boc with config
- `block_time`?: _number_ – time that is used as transaction time
- `block_lt`?: _bigint_ – block logical time
- `transaction_lt`?: _bigint_ – transaction logical time


## AccountForExecutor
```ts
type AccountForExecutor = {
    type: 'None'
} | {
    type: 'Uninit'
} | {
    type: 'Account'
    boc: string,
    unlimited_balance?: boolean
};
```
Depends on value of the  `type` field.

When _type_ is _'None'_

Non-existing account to run a creation internal message. Should be used with `skip_transaction_check = true` if the message has no deploy data since transaction on the unitialized account are always aborted


When _type_ is _'Uninit'_

Emulate unitialized account to run deploy message


When _type_ is _'Account'_

Account state to run message


- `boc`: _string_ – Account BOC. Encoded as base64.
- `unlimited_balance`?: _boolean_ – Flag for running account with the unlimited balance. Can be used to calculate transaction fees without balance check


## ParamsOfRunExecutor
```ts
type ParamsOfRunExecutor = {
    message: string,
    account: AccountForExecutor,
    execution_options?: ExecutionOptions,
    abi?: Abi,
    skip_transaction_check?: boolean
};
```
- `message`: _string_ – Input message BOC. Must be encoded as base64.
- `account`: _[AccountForExecutor](mod_tvm.md#AccountForExecutor)_ – Account to run on executor
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for decoding output messages
- `skip_transaction_check`?: _boolean_ – Skip transaction check flag


## ResultOfRunExecutor
```ts
type ResultOfRunExecutor = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string,
    fees: TransactionFees
};
```
- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC. Encoded as `base64`
- `fees`: _TransactionFees_ – Transaction fees


## ParamsOfRunTvm
```ts
type ParamsOfRunTvm = {
    message: string,
    account: string,
    execution_options?: ExecutionOptions,
    abi?: Abi
};
```
- `message`: _string_ – Input message BOC. Must be encoded as base64.
- `account`: _string_ – Account BOC. Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for dedcoding output messages


## ResultOfRunTvm
```ts
type ResultOfRunTvm = {
    out_messages: string[],
    decoded?: DecodedOutput,
    account: string
};
```
- `out_messages`: _string[]_ – List of output messages' BOCs. Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `account`: _string_ – Updated account state BOC. Encoded as `base64`. Attention! Only data in account state is updated.


## ParamsOfRunGet
```ts
type ParamsOfRunGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions
};
```
- `account`: _string_ – Account BOC in `base64`
- `function_name`: _string_ – Function name
- `input`?: _any_ – Input parameters
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_


## ResultOfRunGet
```ts
type ResultOfRunGet = {
    output: any
};
```
- `output`: _any_ – Values returned by getmethod on stack



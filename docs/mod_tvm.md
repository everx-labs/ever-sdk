# Module tvm

null
## Functions
[run_executor](#run_executor)

[run_tvm](#run_tvm)

[run_get](#run_get) – Executes getmethod and returns data from TVM stack

## Types
[ExecutionOptions](#ExecutionOptions)

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
    account?: string,
    execution_options?: ExecutionOptions,
    abi?: Abi
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
- `account`?: _string_ – Account BOC. Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for dedcoding output messages
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


## ParamsOfRunExecutor
```ts
type ParamsOfRunExecutor = {
    message: string,
    account?: string,
    execution_options?: ExecutionOptions,
    abi?: Abi
};
```
- `message`: _string_ – Input message BOC. Must be encoded as base64.
- `account`?: _string_ – Account BOC. Must be encoded as base64.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ – Execution options.
- `abi`?: _[Abi](mod_abi.md#Abi)_ – Contract ABI for dedcoding output messages


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



# Module tvm

null
## Functions
[execute_message](#execute_message)

[execute_get](#execute_get)

## Types
[ExecutionMode](#ExecutionMode)

[ExecutionOptions](#ExecutionOptions)

[ParamsOfExecuteMessage](#ParamsOfExecuteMessage)

[ResultOfExecuteMessage](#ResultOfExecuteMessage)

[ParamsOfExecuteGet](#ParamsOfExecuteGet)

[ResultOfExecuteGet](#ResultOfExecuteGet)


# Functions
## execute_message

```ts
type ParamsOfExecuteMessage = {
    message: MessageSource,
    account: string,
    mode: ExecutionMode,
    execution_options?: ExecutionOptions
};

type ResultOfExecuteMessage = {
    transaction?: any,
    out_messages: any[],
    decoded?: DecodedOutput,
    account?: any
};

function execute_message(
    params: ParamsOfExecuteMessage,
): Promise<ResultOfExecuteMessage>;
```
### Parameters
- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Input message.
- `account`: _string_ –  Account BOC. Must be encoded as base64.
- `mode`: _[ExecutionMode](mod_tvm.md#ExecutionMode)_ –  Execution mode.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ –  Execution options.
### Result

- `transaction`?: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional
- `account`?: _any_ –  JSON with parsed updated account state. Attention! When used in


## execute_get

```ts
type ParamsOfExecuteGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions
};

type ResultOfExecuteGet = {
    output: any
};

function execute_get(
    params: ParamsOfExecuteGet,
): Promise<ResultOfExecuteGet>;
```
### Parameters
- `account`: _string_
- `function_name`: _string_
- `input`?: _any_
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_
### Result

- `output`: _any_


# Types
## ExecutionMode

```ts
type ExecutionMode = 'Full' | 'TvmOnly';
```
One of the following value:

- `Full` –  Executes all phases and performs all checks
- `TvmOnly` –  Executes contract only on TVM (part of compute phase)


## ExecutionOptions

```ts
type ExecutionOptions = {
    blockchain_config?: string,
    block_time?: number,
    block_lt?: bigint,
    transaction_lt?: bigint
};
```
- `blockchain_config`?: _string_ –  boc with config
- `block_time`?: _number_ –  time that is used as transaction time
- `block_lt`?: _bigint_ –  block logical time
- `transaction_lt`?: _bigint_ –  transaction logical time


## ParamsOfExecuteMessage

```ts
type ParamsOfExecuteMessage = {
    message: MessageSource,
    account: string,
    mode: ExecutionMode,
    execution_options?: ExecutionOptions
};
```
- `message`: _[MessageSource](mod_processing.md#MessageSource)_ –  Input message.
- `account`: _string_ –  Account BOC. Must be encoded as base64.
- `mode`: _[ExecutionMode](mod_tvm.md#ExecutionMode)_ –  Execution mode.
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_ –  Execution options.


## ResultOfExecuteMessage

```ts
type ResultOfExecuteMessage = {
    transaction?: any,
    out_messages: any[],
    decoded?: DecodedOutput,
    account?: any
};
```
- `transaction`?: _any_ –  Parsed transaction.
- `out_messages`: _any[]_ –  List of parsed output messages.
- `decoded`?: _[DecodedOutput](mod_processing.md#DecodedOutput)_ –  Optional decoded message bodies according to the optional
- `account`?: _any_ –  JSON with parsed updated account state. Attention! When used in


## ParamsOfExecuteGet

```ts
type ParamsOfExecuteGet = {
    account: string,
    function_name: string,
    input?: any,
    execution_options?: ExecutionOptions
};
```
- `account`: _string_
- `function_name`: _string_
- `input`?: _any_
- `execution_options`?: _[ExecutionOptions](mod_tvm.md#ExecutionOptions)_


## ResultOfExecuteGet

```ts
type ResultOfExecuteGet = {
    output: any
};
```
- `output`: _any_



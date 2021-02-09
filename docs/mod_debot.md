# Module debot

[UNSTABLE](UNSTABLE.md) Module for working with debot.


## Functions
[start](#start) – [UNSTABLE](UNSTABLE.md) Starts an instance of debot.

[fetch](#fetch) – [UNSTABLE](UNSTABLE.md) Fetches debot from blockchain.

[execute](#execute) – [UNSTABLE](UNSTABLE.md) Executes debot action.

[send](#send) – [UNSTABLE](UNSTABLE.md) Sends message to Debot.

[remove](#remove) – [UNSTABLE](UNSTABLE.md) Destroys debot handle.

## Types
[DebotErrorCode](#DebotErrorCode)

[DebotHandle](#DebotHandle) – [UNSTABLE](UNSTABLE.md) Handle of registered in SDK debot

[DebotAction](#DebotAction) – [UNSTABLE](UNSTABLE.md) Describes a debot action in a Debot Context.

[ParamsOfStart](#ParamsOfStart) – [UNSTABLE](UNSTABLE.md) Parameters to start debot.

[RegisteredDebot](#RegisteredDebot) – [UNSTABLE](UNSTABLE.md) Structure for storing debot handle returned from `start` and `fetch` functions.

[ParamsOfAppDebotBrowser](#ParamsOfAppDebotBrowser) – [UNSTABLE](UNSTABLE.md) Debot Browser callbacks

[ResultOfAppDebotBrowser](#ResultOfAppDebotBrowser) – [UNSTABLE](UNSTABLE.md) Returning values from Debot Browser callbacks.

[ParamsOfFetch](#ParamsOfFetch) – [UNSTABLE](UNSTABLE.md) Parameters to fetch debot.

[ParamsOfExecute](#ParamsOfExecute) – [UNSTABLE](UNSTABLE.md) Parameters for executing debot action.

[ParamsOfSend](#ParamsOfSend) – [UNSTABLE](UNSTABLE.md) Parameters of `send` function.


# Functions
## start

[UNSTABLE](UNSTABLE.md) Starts an instance of debot.

Downloads debot smart contract from blockchain and switches it to
context zero.
Returns a debot handle which can be used later in `execute` function.
This function must be used by Debot Browser to start a dialog with debot.
While the function is executing, several Browser Callbacks can be called,
since the debot tries to display all actions from the context 0 to the user.

# Remarks
`start` is equivalent to `fetch` + switch to context 0.

```ts
type ParamsOfStart = {
    address: string
}

type RegisteredDebot = {
    debot_handle: DebotHandle
}

function start(
    params: ParamsOfStart,
    obj: AppDebotBrowser,
): Promise<RegisteredDebot>;
```
### Parameters
- `address`: _string_ – Debot smart contract address


### Result

- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.


## fetch

[UNSTABLE](UNSTABLE.md) Fetches debot from blockchain.

Downloads debot smart contract (code and data) from blockchain and creates
an instance of Debot Engine for it.

# Remarks
It does not switch debot to context 0. Browser Callbacks are not called.

```ts
type ParamsOfFetch = {
    address: string
}

type RegisteredDebot = {
    debot_handle: DebotHandle
}

function fetch(
    params: ParamsOfFetch,
    obj: AppDebotBrowser,
): Promise<RegisteredDebot>;
```
### Parameters
- `address`: _string_ – Debot smart contract address


### Result

- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.


## execute

[UNSTABLE](UNSTABLE.md) Executes debot action.

Calls debot engine referenced by debot handle to execute input action.
Calls Debot Browser Callbacks if needed.

# Remarks
Chain of actions can be executed if input action generates a list of subactions.

```ts
type ParamsOfExecute = {
    debot_handle: DebotHandle,
    action: DebotAction
}

function execute(
    params: ParamsOfExecute,
): Promise<void>;
```
### Parameters
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.
- `action`: _[DebotAction](mod_debot.md#DebotAction)_ – Debot Action that must be executed.


### Result



## send

[UNSTABLE](UNSTABLE.md) Sends message to Debot.

Used by Debot Browser to send response on Dinterface call or from other Debots.

```ts
type ParamsOfSend = {
    debot_handle: DebotHandle,
    source: string,
    func_id: number,
    params: string
}

function send(
    params: ParamsOfSend,
): Promise<void>;
```
### Parameters
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.
- `source`: _string_ – Std address of interface or debot.
- `func_id`: _number_ – Function Id to call
- `params`: _string_ – Json string with parameters


### Result



## remove

[UNSTABLE](UNSTABLE.md) Destroys debot handle.

Removes handle from Client Context and drops debot engine referenced by that handle.

```ts
type RegisteredDebot = {
    debot_handle: DebotHandle
}

function remove(
    params: RegisteredDebot,
): Promise<void>;
```
### Parameters
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.


### Result



# Types
## DebotErrorCode
```ts
enum DebotErrorCode {
    DebotStartFailed = 801,
    DebotFetchFailed = 802,
    DebotExecutionFailed = 803,
    DebotInvalidHandle = 804,
    DebotInvalidJsonParams = 805,
    DebotInvalidFunctionId = 806,
    DebotInvalidAbi = 807,
    DebotGetMethodFailed = 808,
    DebotInvalidMsg = 809,
    DebotExternaCallFailed = 810
}
```
One of the following value:

- `DebotStartFailed = 801`
- `DebotFetchFailed = 802`
- `DebotExecutionFailed = 803`
- `DebotInvalidHandle = 804`
- `DebotInvalidJsonParams = 805`
- `DebotInvalidFunctionId = 806`
- `DebotInvalidAbi = 807`
- `DebotGetMethodFailed = 808`
- `DebotInvalidMsg = 809`
- `DebotExternaCallFailed = 810`


## DebotHandle
[UNSTABLE](UNSTABLE.md) Handle of registered in SDK debot

```ts
type DebotHandle = number
```


## DebotAction
[UNSTABLE](UNSTABLE.md) Describes a debot action in a Debot Context.

```ts
type DebotAction = {
    description: string,
    name: string,
    action_type: number,
    to: number,
    attributes: string,
    misc: string
}
```
- `description`: _string_ – A short action description.
<br>Should be used by Debot Browser as name of menu item.
- `name`: _string_ – Depends on action type.
<br>Can be a debot function name or a print string (for Print Action).
- `action_type`: _number_ – Action type.
- `to`: _number_ – ID of debot context to switch after action execution.
- `attributes`: _string_ – Action attributes.
<br>In the form of "param=value,flag". attribute example: instant, args, fargs, sign.
- `misc`: _string_ – Some internal action data.
<br>Used by debot only.


## ParamsOfStart
[UNSTABLE](UNSTABLE.md) Parameters to start debot.

```ts
type ParamsOfStart = {
    address: string
}
```
- `address`: _string_ – Debot smart contract address


## RegisteredDebot
[UNSTABLE](UNSTABLE.md) Structure for storing debot handle returned from `start` and `fetch` functions.

```ts
type RegisteredDebot = {
    debot_handle: DebotHandle
}
```
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.


## ParamsOfAppDebotBrowser
[UNSTABLE](UNSTABLE.md) Debot Browser callbacks

Called by debot engine to communicate with debot browser.

```ts
type ParamsOfAppDebotBrowser = {
    type: 'Log'
    msg: string
} | {
    type: 'Switch'
    context_id: number
} | {
    type: 'SwitchCompleted'
} | {
    type: 'ShowAction'
    action: DebotAction
} | {
    type: 'Input'
    prompt: string
} | {
    type: 'GetSigningBox'
} | {
    type: 'InvokeDebot'
    debot_addr: string,
    action: DebotAction
} | {
    type: 'Send'
    message: string
}
```
Depends on value of the  `type` field.

When _type_ is _'Log'_

Print message to user.


- `msg`: _string_ – A string that must be printed to user.

When _type_ is _'Switch'_

Switch debot to another context (menu).


- `context_id`: _number_ – Debot context ID to which debot is switched.

When _type_ is _'SwitchCompleted'_

Notify browser that all context actions are shown.


When _type_ is _'ShowAction'_

Show action to the user. Called after `switch` for each action in context.


- `action`: _[DebotAction](mod_debot.md#DebotAction)_ – Debot action that must be shown to user as menu item. At least `description` property must be shown from [DebotAction] structure.

When _type_ is _'Input'_

Request user input.


- `prompt`: _string_ – A prompt string that must be printed to user before input request.

When _type_ is _'GetSigningBox'_

Get signing box to sign data.

Signing box returned is owned and disposed by debot engine


When _type_ is _'InvokeDebot'_

Execute action of another debot.


- `debot_addr`: _string_ – Address of debot in blockchain.
- `action`: _[DebotAction](mod_debot.md#DebotAction)_ – Debot action to execute.

When _type_ is _'Send'_

Used by Debot to call DInterface implemented by Debot Browser.


- `message`: _string_ – Internal message to DInterface address.
<br>Message body contains interface function and parameters.


## ResultOfAppDebotBrowser
[UNSTABLE](UNSTABLE.md) Returning values from Debot Browser callbacks.

```ts
type ResultOfAppDebotBrowser = {
    type: 'Input'
    value: string
} | {
    type: 'GetSigningBox'
    signing_box: SigningBoxHandle
} | {
    type: 'InvokeDebot'
}
```
Depends on value of the  `type` field.

When _type_ is _'Input'_

Result of user input.


- `value`: _string_ – String entered by user.

When _type_ is _'GetSigningBox'_

Result of getting signing box.


- `signing_box`: _[SigningBoxHandle](mod_crypto.md#SigningBoxHandle)_ – Signing box for signing data requested by debot engine.
<br>Signing box is owned and disposed by debot engine

When _type_ is _'InvokeDebot'_

Result of debot invoking.



## ParamsOfFetch
[UNSTABLE](UNSTABLE.md) Parameters to fetch debot.

```ts
type ParamsOfFetch = {
    address: string
}
```
- `address`: _string_ – Debot smart contract address


## ParamsOfExecute
[UNSTABLE](UNSTABLE.md) Parameters for executing debot action.

```ts
type ParamsOfExecute = {
    debot_handle: DebotHandle,
    action: DebotAction
}
```
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.
- `action`: _[DebotAction](mod_debot.md#DebotAction)_ – Debot Action that must be executed.


## ParamsOfSend
[UNSTABLE](UNSTABLE.md) Parameters of `send` function.

```ts
type ParamsOfSend = {
    debot_handle: DebotHandle,
    source: string,
    func_id: number,
    params: string
}
```
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_ – Debot handle which references an instance of debot engine.
- `source`: _string_ – Std address of interface or debot.
- `func_id`: _number_ – Function Id to call
- `params`: _string_ – Json string with parameters



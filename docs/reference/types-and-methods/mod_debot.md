# Module debot

## Module debot

[UNSTABLE](UNSTABLE.md) Module for working with debot.

### Functions

[init](mod\_debot.md#init) – [UNSTABLE](UNSTABLE.md) Creates and instance of DeBot.

[start](mod\_debot.md#start) – [UNSTABLE](UNSTABLE.md) Starts the DeBot.

[fetch](mod\_debot.md#fetch) – [UNSTABLE](UNSTABLE.md) Fetches DeBot metadata from blockchain.

[execute](mod\_debot.md#execute) – [UNSTABLE](UNSTABLE.md) Executes debot action.

[send](mod\_debot.md#send) – [UNSTABLE](UNSTABLE.md) Sends message to Debot.

[remove](mod\_debot.md#remove) – [UNSTABLE](UNSTABLE.md) Destroys debot handle.

### Types

[DebotErrorCode](mod\_debot.md#deboterrorcode)

[DebotHandle](mod\_debot.md#debothandle) – [UNSTABLE](UNSTABLE.md) Handle of registered in SDK debot

[DebotAction](mod\_debot.md#debotaction) – [UNSTABLE](UNSTABLE.md) Describes a debot action in a Debot Context.

[DebotInfo](mod\_debot.md#debotinfo) – [UNSTABLE](UNSTABLE.md) Describes DeBot metadata.

[DebotActivityTransactionVariant](mod\_debot.md#debotactivitytransactionvariant) – DeBot wants to create new transaction in blockchain.

[DebotActivity](mod\_debot.md#debotactivity) – [UNSTABLE](UNSTABLE.md) Describes the operation that the DeBot wants to perform.

[Spending](mod\_debot.md#spending) – [UNSTABLE](UNSTABLE.md) Describes how much funds will be debited from the target contract balance as a result of the transaction.

[ParamsOfInit](mod\_debot.md#paramsofinit) – [UNSTABLE](UNSTABLE.md) Parameters to init DeBot.

[RegisteredDebot](mod\_debot.md#registereddebot) – [UNSTABLE](UNSTABLE.md) Structure for storing debot handle returned from `init` function.

[ParamsOfAppDebotBrowserLogVariant](mod\_debot.md#paramsofappdebotbrowserlogvariant) – Print message to user.

[ParamsOfAppDebotBrowserSwitchVariant](mod\_debot.md#paramsofappdebotbrowserswitchvariant) – Switch debot to another context (menu).

[ParamsOfAppDebotBrowserSwitchCompletedVariant](mod\_debot.md#paramsofappdebotbrowserswitchcompletedvariant) – Notify browser that all context actions are shown.

[ParamsOfAppDebotBrowserShowActionVariant](mod\_debot.md#paramsofappdebotbrowsershowactionvariant) – Show action to the user. Called after `switch` for each action in context.

[ParamsOfAppDebotBrowserInputVariant](mod\_debot.md#paramsofappdebotbrowserinputvariant) – Request user input.

[ParamsOfAppDebotBrowserGetSigningBoxVariant](mod\_debot.md#paramsofappdebotbrowsergetsigningboxvariant) – Get signing box to sign data.

[ParamsOfAppDebotBrowserInvokeDebotVariant](mod\_debot.md#paramsofappdebotbrowserinvokedebotvariant) – Execute action of another debot.

[ParamsOfAppDebotBrowserSendVariant](mod\_debot.md#paramsofappdebotbrowsersendvariant) – Used by Debot to call DInterface implemented by Debot Browser.

[ParamsOfAppDebotBrowserApproveVariant](mod\_debot.md#paramsofappdebotbrowserapprovevariant) – Requests permission from DeBot Browser to execute DeBot operation.

[ParamsOfAppDebotBrowser](mod\_debot.md#paramsofappdebotbrowser) – [UNSTABLE](UNSTABLE.md) Debot Browser callbacks

[ResultOfAppDebotBrowserInputVariant](mod\_debot.md#resultofappdebotbrowserinputvariant) – Result of user input.

[ResultOfAppDebotBrowserGetSigningBoxVariant](mod\_debot.md#resultofappdebotbrowsergetsigningboxvariant) – Result of getting signing box.

[ResultOfAppDebotBrowserInvokeDebotVariant](mod\_debot.md#resultofappdebotbrowserinvokedebotvariant) – Result of debot invoking.

[ResultOfAppDebotBrowserApproveVariant](mod\_debot.md#resultofappdebotbrowserapprovevariant) – Result of `approve` callback.

[ResultOfAppDebotBrowser](mod\_debot.md#resultofappdebotbrowser) – [UNSTABLE](UNSTABLE.md) Returning values from Debot Browser callbacks.

[ParamsOfStart](mod\_debot.md#paramsofstart) – [UNSTABLE](UNSTABLE.md) Parameters to start DeBot. DeBot must be already initialized with init() function.

[ParamsOfFetch](mod\_debot.md#paramsoffetch) – [UNSTABLE](UNSTABLE.md) Parameters to fetch DeBot metadata.

[ResultOfFetch](mod\_debot.md#resultoffetch) – [UNSTABLE](UNSTABLE.md)

[ParamsOfExecute](mod\_debot.md#paramsofexecute) – [UNSTABLE](UNSTABLE.md) Parameters for executing debot action.

[ParamsOfSend](mod\_debot.md#paramsofsend) – [UNSTABLE](UNSTABLE.md) Parameters of `send` function.

[ParamsOfRemove](mod\_debot.md#paramsofremove) – [UNSTABLE](UNSTABLE.md)

[AppDebotBrowser](mod\_debot.md#appdebotbrowser) – [UNSTABLE](UNSTABLE.md) Debot Browser callbacks

## Functions

### init

[UNSTABLE](UNSTABLE.md) Creates and instance of DeBot.

Downloads debot smart contract (code and data) from blockchain and creates an instance of Debot Engine for it.

## Remarks

It does not switch debot to context 0. Browser Callbacks are not called.

```ts
type ParamsOfInit = {
    address: string
}

type RegisteredDebot = {
    debot_handle: DebotHandle,
    debot_abi: string,
    info: DebotInfo
}

function init(
    params: ParamsOfInit,
    obj: AppDebotBrowser,
): Promise<RegisteredDebot>;
```

#### Parameters

* `address`: _string_ – Debot smart contract address
* `obj`: [AppDebotBrowser](mod\_AppDebotBrowser.md#appdebotbrowser) – [UNSTABLE](UNSTABLE.md) Debot Browser callbacks

#### Result

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `debot_abi`: _string_ – Debot abi as json string.
* `info`: [_DebotInfo_](mod\_debot.md#debotinfo) – Debot metadata.

### start

[UNSTABLE](UNSTABLE.md) Starts the DeBot.

Downloads debot smart contract from blockchain and switches it to context zero.

This function must be used by Debot Browser to start a dialog with debot. While the function is executing, several Browser Callbacks can be called, since the debot tries to display all actions from the context 0 to the user.

When the debot starts SDK registers `BrowserCallbacks` AppObject. Therefore when `debote.remove` is called the debot is being deleted and the callback is called with `finish`=`true` which indicates that it will never be used again.

```ts
type ParamsOfStart = {
    debot_handle: DebotHandle
}

function start(
    params: ParamsOfStart,
): Promise<void>;
```

#### Parameters

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.

### fetch

[UNSTABLE](UNSTABLE.md) Fetches DeBot metadata from blockchain.

Downloads DeBot from blockchain and creates and fetches its metadata.

```ts
type ParamsOfFetch = {
    address: string
}

type ResultOfFetch = {
    info: DebotInfo
}

function fetch(
    params: ParamsOfFetch,
): Promise<ResultOfFetch>;
```

#### Parameters

* `address`: _string_ – Debot smart contract address.

#### Result

* `info`: [_DebotInfo_](mod\_debot.md#debotinfo) – Debot metadata.

### execute

[UNSTABLE](UNSTABLE.md) Executes debot action.

Calls debot engine referenced by debot handle to execute input action. Calls Debot Browser Callbacks if needed.

## Remarks

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

#### Parameters

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot Action that must be executed.

### send

[UNSTABLE](UNSTABLE.md) Sends message to Debot.

Used by Debot Browser to send response on Dinterface call or from other Debots.

```ts
type ParamsOfSend = {
    debot_handle: DebotHandle,
    message: string
}

function send(
    params: ParamsOfSend,
): Promise<void>;
```

#### Parameters

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `message`: _string_ – BOC of internal message to debot encoded in base64 format.

### remove

[UNSTABLE](UNSTABLE.md) Destroys debot handle.

Removes handle from Client Context and drops debot engine referenced by that handle.

```ts
type ParamsOfRemove = {
    debot_handle: DebotHandle
}

function remove(
    params: ParamsOfRemove,
): Promise<void>;
```

#### Parameters

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.

## Types

### DebotErrorCode

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
    DebotExternalCallFailed = 810,
    DebotBrowserCallbackFailed = 811,
    DebotOperationRejected = 812,
    DebotNoCode = 813
}
```

One of the following value:

* `DebotStartFailed = 801`
* `DebotFetchFailed = 802`
* `DebotExecutionFailed = 803`
* `DebotInvalidHandle = 804`
* `DebotInvalidJsonParams = 805`
* `DebotInvalidFunctionId = 806`
* `DebotInvalidAbi = 807`
* `DebotGetMethodFailed = 808`
* `DebotInvalidMsg = 809`
* `DebotExternalCallFailed = 810`
* `DebotBrowserCallbackFailed = 811`
* `DebotOperationRejected = 812`
* `DebotNoCode = 813`

### DebotHandle

[UNSTABLE](UNSTABLE.md) Handle of registered in SDK debot

```ts
type DebotHandle = number
```

### DebotAction

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

* `description`: _string_ – A short action description.\
  Should be used by Debot Browser as name of menu item.
* `name`: _string_ – Depends on action type.\
  Can be a debot function name or a print string (for Print Action).
* `action_type`: _number_ – Action type.
* `to`: _number_ – ID of debot context to switch after action execution.
* `attributes`: _string_ – Action attributes.\
  In the form of "param=value,flag". attribute example: instant, args, fargs, sign.
* `misc`: _string_ – Some internal action data.\
  Used by debot only.

### DebotInfo

[UNSTABLE](UNSTABLE.md) Describes DeBot metadata.

```ts
type DebotInfo = {
    name?: string,
    version?: string,
    publisher?: string,
    caption?: string,
    author?: string,
    support?: string,
    hello?: string,
    language?: string,
    dabi?: string,
    icon?: string,
    interfaces: string[],
    dabiVersion: string
}
```

* `name`?: _string_ – DeBot short name.
* `version`?: _string_ – DeBot semantic version.
* `publisher`?: _string_ – The name of DeBot deployer.
* `caption`?: _string_ – Short info about DeBot.
* `author`?: _string_ – The name of DeBot developer.
* `support`?: _string_ – TON address of author for questions and donations.
* `hello`?: _string_ – String with the first messsage from DeBot.
* `language`?: _string_ – String with DeBot interface language (ISO-639).
* `dabi`?: _string_ – String with DeBot ABI.
* `icon`?: _string_ – DeBot icon.
* `interfaces`: _string\[]_ – Vector with IDs of DInterfaces used by DeBot.
* `dabiVersion`: _string_ – ABI version ("x.y") supported by DeBot

### DebotActivityTransactionVariant

DeBot wants to create new transaction in blockchain.

```ts
type DebotActivityTransactionVariant = {
    msg: string,
    dst: string,
    out: Spending[],
    fee: bigint,
    setcode: boolean,
    signkey: string,
    signing_box_handle: number
}
```

* `msg`: _string_ – External inbound message BOC.
* `dst`: _string_ – Target smart contract address.
* `out`: [_Spending_](mod\_debot.md#spending)_\[]_ – List of spendings as a result of transaction.
* `fee`: _bigint_ – Transaction total fee.
* `setcode`: _boolean_ – Indicates if target smart contract updates its code.
* `signkey`: _string_ – Public key from keypair that was used to sign external message.
* `signing_box_handle`: _number_ – Signing box handle used to sign external message.

### DebotActivity

[UNSTABLE](UNSTABLE.md) Describes the operation that the DeBot wants to perform.

```ts
type DebotActivity = ({
    type: 'Transaction'
} & DebotActivityTransactionVariant)
```

Depends on value of the `type` field.

When _type_ is _'Transaction'_

DeBot wants to create new transaction in blockchain.

* `msg`: _string_ – External inbound message BOC.
* `dst`: _string_ – Target smart contract address.
* `out`: [_Spending_](mod\_debot.md#spending)_\[]_ – List of spendings as a result of transaction.
* `fee`: _bigint_ – Transaction total fee.
* `setcode`: _boolean_ – Indicates if target smart contract updates its code.
* `signkey`: _string_ – Public key from keypair that was used to sign external message.
* `signing_box_handle`: _number_ – Signing box handle used to sign external message.

Variant constructors:

```ts
function debotActivityTransaction(msg: string, dst: string, out: Spending[], fee: bigint, setcode: boolean, signkey: string, signing_box_handle: number): DebotActivity;
```

### Spending

[UNSTABLE](UNSTABLE.md) Describes how much funds will be debited from the target contract balance as a result of the transaction.

```ts
type Spending = {
    amount: bigint,
    dst: string
}
```

* `amount`: _bigint_ – Amount of nanotokens that will be sent to `dst` address.
* `dst`: _string_ – Destination address of recipient of funds.

### ParamsOfInit

[UNSTABLE](UNSTABLE.md) Parameters to init DeBot.

```ts
type ParamsOfInit = {
    address: string
}
```

* `address`: _string_ – Debot smart contract address

### RegisteredDebot

[UNSTABLE](UNSTABLE.md) Structure for storing debot handle returned from `init` function.

```ts
type RegisteredDebot = {
    debot_handle: DebotHandle,
    debot_abi: string,
    info: DebotInfo
}
```

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `debot_abi`: _string_ – Debot abi as json string.
* `info`: [_DebotInfo_](mod\_debot.md#debotinfo) – Debot metadata.

### ParamsOfAppDebotBrowserLogVariant

Print message to user.

```ts
type ParamsOfAppDebotBrowserLogVariant = {
    msg: string
}
```

* `msg`: _string_ – A string that must be printed to user.

### ParamsOfAppDebotBrowserSwitchVariant

Switch debot to another context (menu).

```ts
type ParamsOfAppDebotBrowserSwitchVariant = {
    context_id: number
}
```

* `context_id`: _number_ – Debot context ID to which debot is switched.

### ParamsOfAppDebotBrowserSwitchCompletedVariant

Notify browser that all context actions are shown.

```ts
type ParamsOfAppDebotBrowserSwitchCompletedVariant = {

}
```

### ParamsOfAppDebotBrowserShowActionVariant

Show action to the user. Called after `switch` for each action in context.

```ts
type ParamsOfAppDebotBrowserShowActionVariant = {
    action: DebotAction
}
```

* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action that must be shown to user as menu item. At least `description` property must be shown from \[DebotAction] structure.

### ParamsOfAppDebotBrowserInputVariant

Request user input.

```ts
type ParamsOfAppDebotBrowserInputVariant = {
    prompt: string
}
```

* `prompt`: _string_ – A prompt string that must be printed to user before input request.

### ParamsOfAppDebotBrowserGetSigningBoxVariant

Get signing box to sign data.

Signing box returned is owned and disposed by debot engine

```ts
type ParamsOfAppDebotBrowserGetSigningBoxVariant = {

}
```

### ParamsOfAppDebotBrowserInvokeDebotVariant

Execute action of another debot.

```ts
type ParamsOfAppDebotBrowserInvokeDebotVariant = {
    debot_addr: string,
    action: DebotAction
}
```

* `debot_addr`: _string_ – Address of debot in blockchain.
* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action to execute.

### ParamsOfAppDebotBrowserSendVariant

Used by Debot to call DInterface implemented by Debot Browser.

```ts
type ParamsOfAppDebotBrowserSendVariant = {
    message: string
}
```

* `message`: _string_ – Internal message to DInterface address.\
  Message body contains interface function and parameters.

### ParamsOfAppDebotBrowserApproveVariant

Requests permission from DeBot Browser to execute DeBot operation.

```ts
type ParamsOfAppDebotBrowserApproveVariant = {
    activity: DebotActivity
}
```

* `activity`: [_DebotActivity_](mod\_debot.md#debotactivity) – DeBot activity details.

### ParamsOfAppDebotBrowser

[UNSTABLE](UNSTABLE.md) Debot Browser callbacks

Called by debot engine to communicate with debot browser.

```ts
type ParamsOfAppDebotBrowser = ({
    type: 'Log'
} & ParamsOfAppDebotBrowserLogVariant) | ({
    type: 'Switch'
} & ParamsOfAppDebotBrowserSwitchVariant) | ({
    type: 'SwitchCompleted'
} & ParamsOfAppDebotBrowserSwitchCompletedVariant) | ({
    type: 'ShowAction'
} & ParamsOfAppDebotBrowserShowActionVariant) | ({
    type: 'Input'
} & ParamsOfAppDebotBrowserInputVariant) | ({
    type: 'GetSigningBox'
} & ParamsOfAppDebotBrowserGetSigningBoxVariant) | ({
    type: 'InvokeDebot'
} & ParamsOfAppDebotBrowserInvokeDebotVariant) | ({
    type: 'Send'
} & ParamsOfAppDebotBrowserSendVariant) | ({
    type: 'Approve'
} & ParamsOfAppDebotBrowserApproveVariant)
```

Depends on value of the `type` field.

When _type_ is _'Log'_

Print message to user.

* `msg`: _string_ – A string that must be printed to user.

When _type_ is _'Switch'_

Switch debot to another context (menu).

* `context_id`: _number_ – Debot context ID to which debot is switched.

When _type_ is _'SwitchCompleted'_

Notify browser that all context actions are shown.

When _type_ is _'ShowAction'_

Show action to the user. Called after `switch` for each action in context.

* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action that must be shown to user as menu item. At least `description` property must be shown from \[DebotAction] structure.

When _type_ is _'Input'_

Request user input.

* `prompt`: _string_ – A prompt string that must be printed to user before input request.

When _type_ is _'GetSigningBox'_

Get signing box to sign data.

Signing box returned is owned and disposed by debot engine

When _type_ is _'InvokeDebot'_

Execute action of another debot.

* `debot_addr`: _string_ – Address of debot in blockchain.
* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action to execute.

When _type_ is _'Send'_

Used by Debot to call DInterface implemented by Debot Browser.

* `message`: _string_ – Internal message to DInterface address.\
  Message body contains interface function and parameters.

When _type_ is _'Approve'_

Requests permission from DeBot Browser to execute DeBot operation.

* `activity`: [_DebotActivity_](mod\_debot.md#debotactivity) – DeBot activity details.

Variant constructors:

```ts
function paramsOfAppDebotBrowserLog(msg: string): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserSwitch(context_id: number): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserSwitchCompleted(): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserShowAction(action: DebotAction): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserInput(prompt: string): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserGetSigningBox(): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserInvokeDebot(debot_addr: string, action: DebotAction): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserSend(message: string): ParamsOfAppDebotBrowser;
function paramsOfAppDebotBrowserApprove(activity: DebotActivity): ParamsOfAppDebotBrowser;
```

### ResultOfAppDebotBrowserInputVariant

Result of user input.

```ts
type ResultOfAppDebotBrowserInputVariant = {
    value: string
}
```

* `value`: _string_ – String entered by user.

### ResultOfAppDebotBrowserGetSigningBoxVariant

Result of getting signing box.

```ts
type ResultOfAppDebotBrowserGetSigningBoxVariant = {
    signing_box: SigningBoxHandle
}
```

* `signing_box`: [_SigningBoxHandle_](mod\_crypto.md#signingboxhandle) – Signing box for signing data requested by debot engine.\
  Signing box is owned and disposed by debot engine

### ResultOfAppDebotBrowserInvokeDebotVariant

Result of debot invoking.

```ts
type ResultOfAppDebotBrowserInvokeDebotVariant = {

}
```

### ResultOfAppDebotBrowserApproveVariant

Result of `approve` callback.

```ts
type ResultOfAppDebotBrowserApproveVariant = {
    approved: boolean
}
```

* `approved`: _boolean_ – Indicates whether the DeBot is allowed to perform the specified operation.

### ResultOfAppDebotBrowser

[UNSTABLE](UNSTABLE.md) Returning values from Debot Browser callbacks.

```ts
type ResultOfAppDebotBrowser = ({
    type: 'Input'
} & ResultOfAppDebotBrowserInputVariant) | ({
    type: 'GetSigningBox'
} & ResultOfAppDebotBrowserGetSigningBoxVariant) | ({
    type: 'InvokeDebot'
} & ResultOfAppDebotBrowserInvokeDebotVariant) | ({
    type: 'Approve'
} & ResultOfAppDebotBrowserApproveVariant)
```

Depends on value of the `type` field.

When _type_ is _'Input'_

Result of user input.

* `value`: _string_ – String entered by user.

When _type_ is _'GetSigningBox'_

Result of getting signing box.

* `signing_box`: [_SigningBoxHandle_](mod\_crypto.md#signingboxhandle) – Signing box for signing data requested by debot engine.\
  Signing box is owned and disposed by debot engine

When _type_ is _'InvokeDebot'_

Result of debot invoking.

When _type_ is _'Approve'_

Result of `approve` callback.

* `approved`: _boolean_ – Indicates whether the DeBot is allowed to perform the specified operation.

Variant constructors:

```ts
function resultOfAppDebotBrowserInput(value: string): ResultOfAppDebotBrowser;
function resultOfAppDebotBrowserGetSigningBox(signing_box: SigningBoxHandle): ResultOfAppDebotBrowser;
function resultOfAppDebotBrowserInvokeDebot(): ResultOfAppDebotBrowser;
function resultOfAppDebotBrowserApprove(approved: boolean): ResultOfAppDebotBrowser;
```

### ParamsOfStart

[UNSTABLE](UNSTABLE.md) Parameters to start DeBot. DeBot must be already initialized with init() function.

```ts
type ParamsOfStart = {
    debot_handle: DebotHandle
}
```

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.

### ParamsOfFetch

[UNSTABLE](UNSTABLE.md) Parameters to fetch DeBot metadata.

```ts
type ParamsOfFetch = {
    address: string
}
```

* `address`: _string_ – Debot smart contract address.

### ResultOfFetch

[UNSTABLE](UNSTABLE.md)

```ts
type ResultOfFetch = {
    info: DebotInfo
}
```

* `info`: [_DebotInfo_](mod\_debot.md#debotinfo) – Debot metadata.

### ParamsOfExecute

[UNSTABLE](UNSTABLE.md) Parameters for executing debot action.

```ts
type ParamsOfExecute = {
    debot_handle: DebotHandle,
    action: DebotAction
}
```

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot Action that must be executed.

### ParamsOfSend

[UNSTABLE](UNSTABLE.md) Parameters of `send` function.

```ts
type ParamsOfSend = {
    debot_handle: DebotHandle,
    message: string
}
```

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.
* `message`: _string_ – BOC of internal message to debot encoded in base64 format.

### ParamsOfRemove

[UNSTABLE](UNSTABLE.md)

```ts
type ParamsOfRemove = {
    debot_handle: DebotHandle
}
```

* `debot_handle`: [_DebotHandle_](mod\_debot.md#debothandle) – Debot handle which references an instance of debot engine.

### AppDebotBrowser

[UNSTABLE](UNSTABLE.md) Debot Browser callbacks

Called by debot engine to communicate with debot browser.

```ts
export interface AppDebotBrowser {
    log(params: ParamsOfAppDebotBrowserLogVariant): void,
    switch(params: ParamsOfAppDebotBrowserSwitchVariant): void,
    switch_completed(): void,
    show_action(params: ParamsOfAppDebotBrowserShowActionVariant): void,
    input(params: ParamsOfAppDebotBrowserInputVariant): Promise<ResultOfAppDebotBrowserInputVariant>,
    get_signing_box(): Promise<ResultOfAppDebotBrowserGetSigningBoxVariant>,
    invoke_debot(params: ParamsOfAppDebotBrowserInvokeDebotVariant): Promise<void>,
    send(params: ParamsOfAppDebotBrowserSendVariant): void,
    approve(params: ParamsOfAppDebotBrowserApproveVariant): Promise<ResultOfAppDebotBrowserApproveVariant>,
}
```

### log

Print message to user.

```ts
type ParamsOfAppDebotBrowserLogVariant = ParamsOfAppDebotBrowserLogVariant

function log(
    params: ParamsOfAppDebotBrowserLogVariant,
): Promise<>;
```

#### Parameters

* `msg`: _string_ – A string that must be printed to user.

### switch

Switch debot to another context (menu).

```ts
type ParamsOfAppDebotBrowserSwitchVariant = ParamsOfAppDebotBrowserSwitchVariant

function switch(
    params: ParamsOfAppDebotBrowserSwitchVariant,
): Promise<>;
```

#### Parameters

* `context_id`: _number_ – Debot context ID to which debot is switched.

### switch\_completed

Notify browser that all context actions are shown.

```ts
function switch_completed(): Promise<>;
```

### show\_action

Show action to the user. Called after `switch` for each action in context.

```ts
type ParamsOfAppDebotBrowserShowActionVariant = ParamsOfAppDebotBrowserShowActionVariant

function show_action(
    params: ParamsOfAppDebotBrowserShowActionVariant,
): Promise<>;
```

#### Parameters

* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action that must be shown to user as menu item. At least `description` property must be shown from \[DebotAction] structure.

### input

Request user input.

```ts
type ParamsOfAppDebotBrowserInputVariant = ParamsOfAppDebotBrowserInputVariant

type ResultOfAppDebotBrowserInputVariant = ResultOfAppDebotBrowserInputVariant

function input(
    params: ParamsOfAppDebotBrowserInputVariant,
): Promise<ResultOfAppDebotBrowserInputVariant>;
```

#### Parameters

* `prompt`: _string_ – A prompt string that must be printed to user before input request.

#### Result

* `value`: _string_ – String entered by user.

### get\_signing\_box

Get signing box to sign data.

Signing box returned is owned and disposed by debot engine

```ts
type ResultOfAppDebotBrowserGetSigningBoxVariant = ResultOfAppDebotBrowserGetSigningBoxVariant

function get_signing_box(): Promise<ResultOfAppDebotBrowserGetSigningBoxVariant>;
```

#### Result

* `signing_box`: [_SigningBoxHandle_](mod\_crypto.md#signingboxhandle) – Signing box for signing data requested by debot engine.\
  Signing box is owned and disposed by debot engine

### invoke\_debot

Execute action of another debot.

```ts
type ParamsOfAppDebotBrowserInvokeDebotVariant = ParamsOfAppDebotBrowserInvokeDebotVariant

function invoke_debot(
    params: ParamsOfAppDebotBrowserInvokeDebotVariant,
): Promise<void>;
```

#### Parameters

* `debot_addr`: _string_ – Address of debot in blockchain.
* `action`: [_DebotAction_](mod\_debot.md#debotaction) – Debot action to execute.

### send

Used by Debot to call DInterface implemented by Debot Browser.

```ts
type ParamsOfAppDebotBrowserSendVariant = ParamsOfAppDebotBrowserSendVariant

function send(
    params: ParamsOfAppDebotBrowserSendVariant,
): Promise<>;
```

#### Parameters

* `message`: _string_ – Internal message to DInterface address.\
  Message body contains interface function and parameters.

### approve

Requests permission from DeBot Browser to execute DeBot operation.

```ts
type ParamsOfAppDebotBrowserApproveVariant = ParamsOfAppDebotBrowserApproveVariant

type ResultOfAppDebotBrowserApproveVariant = ResultOfAppDebotBrowserApproveVariant

function approve(
    params: ParamsOfAppDebotBrowserApproveVariant,
): Promise<ResultOfAppDebotBrowserApproveVariant>;
```

#### Parameters

* `activity`: [_DebotActivity_](mod\_debot.md#debotactivity) – DeBot activity details.

#### Result

* `approved`: _boolean_ – Indicates whether the DeBot is allowed to perform the specified operation.

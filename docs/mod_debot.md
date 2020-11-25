# Module debot

 Module for working with debot.
## Functions
[start](#start)

[fetch](#fetch)

[execute](#execute)

[remove](#remove)

## Types
[DebotHandle](#DebotHandle)

[DebotAction](#DebotAction)

[ParamsOfStart](#ParamsOfStart)

[RegisteredDebot](#RegisteredDebot)

[ParamsOfAppDebotBrowser](#ParamsOfAppDebotBrowser)

[ResultOfAppDebotBrowser](#ResultOfAppDebotBrowser)

[ParamsOfFetch](#ParamsOfFetch)

[ParamsOfExecute](#ParamsOfExecute)


# Functions
## start

```ts
type ParamsOfStart = {
    address: string
};

type RegisteredDebot = {
    debot_handle: DebotHandle
};

function start(
    params: ParamsOfStart,
    obj: AppDebotBrowser,
): Promise<RegisteredDebot>;
```
### Parameters
- `address`: _string_
### Result

- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_


## fetch

```ts
type ParamsOfFetch = {
    address: string
};

type RegisteredDebot = {
    debot_handle: DebotHandle
};

function fetch(
    params: ParamsOfFetch,
    obj: AppDebotBrowser,
): Promise<RegisteredDebot>;
```
### Parameters
- `address`: _string_
### Result

- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_


## execute

```ts
type ParamsOfExecute = {
    debot_handle: DebotHandle,
    action: DebotAction
};

function execute(
    params: ParamsOfExecute,
): Promise<void>;
```
### Parameters
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_
- `action`: _[DebotAction](mod_debot.md#DebotAction)_
### Result



## remove

```ts
type RegisteredDebot = {
    debot_handle: DebotHandle
};

function remove(
    params: RegisteredDebot,
): Promise<void>;
```
### Parameters
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_
### Result



# Types
## DebotHandle
```ts
type DebotHandle = number;
```


## DebotAction
```ts
type DebotAction = {
    description: string,
    name: string,
    action_type: number,
    to: number,
    attributes: string,
    misc: string
};
```
- `description`: _string_
- `name`: _string_
- `action_type`: _number_
- `to`: _number_
- `attributes`: _string_
- `misc`: _string_


## ParamsOfStart
```ts
type ParamsOfStart = {
    address: string
};
```
- `address`: _string_


## RegisteredDebot
```ts
type RegisteredDebot = {
    debot_handle: DebotHandle
};
```
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_


## ParamsOfAppDebotBrowser
```ts
type ParamsOfAppDebotBrowser = {
    type: 'Log'
    msg: string
} | {
    type: 'Switch'
    context_id: number
} | {
    type: 'ShowAction'
    action: DebotAction
} | {
    type: 'Input'
    prefix: string
} | {
    type: 'LoadKey'
} | {
    type: 'InvokeDebot'
    debot_addr: string,
    action: DebotAction
};
```
Depends on value of the  `type` field.

When _type_ is _'Log'_


- `msg`: _string_

When _type_ is _'Switch'_


- `context_id`: _number_

When _type_ is _'ShowAction'_


- `action`: _[DebotAction](mod_debot.md#DebotAction)_

When _type_ is _'Input'_


- `prefix`: _string_

When _type_ is _'LoadKey'_


When _type_ is _'InvokeDebot'_


- `debot_addr`: _string_
- `action`: _[DebotAction](mod_debot.md#DebotAction)_


## ResultOfAppDebotBrowser
```ts
type ResultOfAppDebotBrowser = {
    type: 'Input'
    value: string
} | {
    type: 'LoadKey'
    keys: KeyPair
} | {
    type: 'InvokeDebot'
};
```
Depends on value of the  `type` field.

When _type_ is _'Input'_


- `value`: _string_

When _type_ is _'LoadKey'_


- `keys`: _[KeyPair](mod_crypto.md#KeyPair)_

When _type_ is _'InvokeDebot'_



## ParamsOfFetch
```ts
type ParamsOfFetch = {
    address: string
};
```
- `address`: _string_


## ParamsOfExecute
```ts
type ParamsOfExecute = {
    debot_handle: DebotHandle,
    action: DebotAction
};
```
- `debot_handle`: _[DebotHandle](mod_debot.md#DebotHandle)_
- `action`: _[DebotAction](mod_debot.md#DebotAction)_



# Module net

 Network access.
## Functions
[query_collection](#query_collection)

[wait_for_collection](#wait_for_collection)

[unsubscribe](#unsubscribe)

[subscribe_collection](#subscribe_collection)

## Types
[OrderBy](#OrderBy)

[SortDirection](#SortDirection)

[ParamsOfQueryCollection](#ParamsOfQueryCollection)

[ResultOfQueryCollection](#ResultOfQueryCollection)

[ParamsOfWaitForCollection](#ParamsOfWaitForCollection)

[ResultOfWaitForCollection](#ResultOfWaitForCollection)

[ResultOfSubscribeCollection](#ResultOfSubscribeCollection)

[unit](#unit)

[ParamsOfSubscribeCollection](#ParamsOfSubscribeCollection)


# Functions
## query_collection

```ts
type ParamsOfQueryCollection = {
    collection: String,
    filter?: any,
    result: String,
    order?: OrderBy[],
    limit?: Number
};

type ResultOfQueryCollection = {
    result: any[]
};

function query_collection(
    params: ParamsOfQueryCollection,
): Promise<ResultOfQueryCollection>;
```
### Parameters
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ –  sorting order
- `limit`?: _number_ –  number of documents to return
### Result

- `result`: _any[]_ –  objects that match provided criteria


## wait_for_collection

```ts
type ParamsOfWaitForCollection = {
    collection: String,
    filter?: any,
    result: String,
    timeout?: Number
};

type ResultOfWaitForCollection = {
    result: any
};

function wait_for_collection(
    params: ParamsOfWaitForCollection,
): Promise<ResultOfWaitForCollection>;
```
### Parameters
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `timeout`?: _number_ –  query timeout
### Result

- `result`: _any_ –  first found object that match provided criteria


## unsubscribe

```ts
type ResultOfSubscribeCollection = {
    handle: Number
};

function unsubscribe(
    params: ResultOfSubscribeCollection,
): Promise<void>;
```
### Parameters
- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function
### Result



## subscribe_collection

```ts
type ParamsOfSubscribeCollection = {
    collection: String,
    filter?: any,
    result: String
};

type ResultOfSubscribeCollection = {
    handle: Number
};

function subscribe_collection(
    params: ParamsOfSubscribeCollection,
    responseHandler?: ResponseHandler,
): Promise<ResultOfSubscribeCollection>;
```
### Parameters
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function


# Types
## OrderBy

```ts
type OrderBy = {
    path: String,
    direction: SortDirection
};
```
- `path`: _string_
- `direction`: _[SortDirection](mod_net.md#SortDirection)_


## SortDirection

```ts
type SortDirection = 'ASC' | 'DESC';
```
One of the following value:

- `ASC`
- `DESC`


## ParamsOfQueryCollection

```ts
type ParamsOfQueryCollection = {
    collection: String,
    filter?: any,
    result: String,
    order?: OrderBy[],
    limit?: Number
};
```
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ –  sorting order
- `limit`?: _number_ –  number of documents to return


## ResultOfQueryCollection

```ts
type ResultOfQueryCollection = {
    result: any[]
};
```
- `result`: _any[]_ –  objects that match provided criteria


## ParamsOfWaitForCollection

```ts
type ParamsOfWaitForCollection = {
    collection: String,
    filter?: any,
    result: String,
    timeout?: Number
};
```
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `timeout`?: _number_ –  query timeout


## ResultOfWaitForCollection

```ts
type ResultOfWaitForCollection = {
    result: any
};
```
- `result`: _any_ –  first found object that match provided criteria


## ResultOfSubscribeCollection

```ts
type ResultOfSubscribeCollection = {
    handle: Number
};
```
- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function


## unit

```ts
type unit = void;
```


## ParamsOfSubscribeCollection

```ts
type ParamsOfSubscribeCollection = {
    collection: String,
    filter?: any,
    result: String
};
```
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string



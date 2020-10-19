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
    collection: string,
    filter?: any,
    result: string,
    order?: OrderBy[],
    limit?: number
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
    collection: string,
    filter?: any,
    result: string,
    timeout?: number
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
    handle: number
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
    collection: string,
    filter?: any,
    result: string
};

type ResultOfSubscribeCollection = {
    handle: number
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
    path: string,
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
    collection: string,
    filter?: any,
    result: string,
    order?: OrderBy[],
    limit?: number
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
    collection: string,
    filter?: any,
    result: string,
    timeout?: number
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
    handle: number
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
    collection: string,
    filter?: any,
    result: string
};
```
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string



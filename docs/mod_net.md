# Module net

 Network access.
## Functions
[query](#query) – Performs DAppServer GraphQL query.

[query_collection](#query_collection) – Queries collection data

[wait_for_collection](#wait_for_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](#unsubscribe) – Cancels a subscription

[subscribe_collection](#subscribe_collection) – Creates a subscription

## Types
[OrderBy](#OrderBy)

[SortDirection](#SortDirection)

[ParamsOfQuery](#ParamsOfQuery)

[ResultOfQuery](#ResultOfQuery)

[ParamsOfQueryCollection](#ParamsOfQueryCollection)

[ResultOfQueryCollection](#ResultOfQueryCollection)

[ParamsOfWaitForCollection](#ParamsOfWaitForCollection)

[ResultOfWaitForCollection](#ResultOfWaitForCollection)

[ResultOfSubscribeCollection](#ResultOfSubscribeCollection)

[ParamsOfSubscribeCollection](#ParamsOfSubscribeCollection)


# Functions
## query

Performs DAppServer GraphQL query.

```ts
type ParamsOfQuery = {
    query: string,
    variables?: any
};

type ResultOfQuery = {
    result: any
};

function query(
    params: ParamsOfQuery,
): Promise<ResultOfQuery>;
```
### Parameters
- `query`: _string_ – GraphQL query text.
- `variables`?: _any_ – Variables used in query. Must be a map with named values that can be used in query.
### Result

- `result`: _any_ – Result provided by DAppServer.


## query_collection

Queries collection data

Queries data that satisfies the `filter` conditions,
limits the number of returned records and orders them.
The projection fields are limited to `result` fields

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
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ – Sorting order
- `limit`?: _number_ – Number of documents to return
### Result

- `result`: _any[]_ – Objects that match the provided criteria


## wait_for_collection

Returns an object that fulfills the conditions or waits for its appearance

Triggers only once.
If object that satisfies the `filter` conditions
already exists - returns it immediately.
If not - waits for insert/update of data within the specified `timeout`,
and returns it.
The projection fields are limited to `result` fields

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
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `timeout`?: _number_ – Query timeout
### Result

- `result`: _any_ – First found object that matches the provided criteria


## unsubscribe

Cancels a subscription

Cancels a subscription specified by its handle.

```ts
type ResultOfSubscribeCollection = {
    handle: number
};

function unsubscribe(
    params: ResultOfSubscribeCollection,
): Promise<void>;
```
### Parameters
- `handle`: _number_ – Subscription handle. Must be closed with `unsubscribe`
### Result



## subscribe_collection

Creates a subscription

Triggers for each insert/update of data
that satisfies the `filter` conditions.
The projection fields are limited to `result` fields.

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
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `responseHandler`?: _ResponseHandler_ – additional responses handler.### Result

- `handle`: _number_ – Subscription handle. Must be closed with `unsubscribe`


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


## ParamsOfQuery
```ts
type ParamsOfQuery = {
    query: string,
    variables?: any
};
```
- `query`: _string_ – GraphQL query text.
- `variables`?: _any_ – Variables used in query. Must be a map with named values that can be used in query.


## ResultOfQuery
```ts
type ResultOfQuery = {
    result: any
};
```
- `result`: _any_ – Result provided by DAppServer.


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
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ – Sorting order
- `limit`?: _number_ – Number of documents to return


## ResultOfQueryCollection
```ts
type ResultOfQueryCollection = {
    result: any[]
};
```
- `result`: _any[]_ – Objects that match the provided criteria


## ParamsOfWaitForCollection
```ts
type ParamsOfWaitForCollection = {
    collection: string,
    filter?: any,
    result: string,
    timeout?: number
};
```
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `timeout`?: _number_ – Query timeout


## ResultOfWaitForCollection
```ts
type ResultOfWaitForCollection = {
    result: any
};
```
- `result`: _any_ – First found object that matches the provided criteria


## ResultOfSubscribeCollection
```ts
type ResultOfSubscribeCollection = {
    handle: number
};
```
- `handle`: _number_ – Subscription handle. Must be closed with `unsubscribe`


## ParamsOfSubscribeCollection
```ts
type ParamsOfSubscribeCollection = {
    collection: string,
    filter?: any,
    result: string
};
```
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string



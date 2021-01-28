# Module net

null
## Functions
[query](#query) – Performs DAppServer GraphQL query.

[batch_query](#batch_query) – Performs multiple queries per single fetch.

[query_collection](#query_collection) – Queries collection data

[aggregate_collection](#aggregate_collection) – Aggregates collection data.

[wait_for_collection](#wait_for_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](#unsubscribe) – Cancels a subscription

[subscribe_collection](#subscribe_collection) – Creates a subscription

[suspend](#suspend) – Suspends network module to stop any network activity

[resume](#resume) – Resumes network module to enable network activity

[find_last_shard_block](#find_last_shard_block) – Returns ID of the last block in a specified account shard

[fetch_endpoints](#fetch_endpoints) – Requests the list of alternative endpoints from server

[set_endpoints](#set_endpoints) – Sets the list of endpoints to use on reinit

## Types
[NetErrorCode](#NetErrorCode)

[OrderBy](#OrderBy)

[SortDirection](#SortDirection)

[ParamsOfQuery](#ParamsOfQuery)

[ResultOfQuery](#ResultOfQuery)

[ParamsOfBatchQuery](#ParamsOfBatchQuery)

[ResultOfBatchQuery](#ResultOfBatchQuery)

[ParamsOfQueryCollection](#ParamsOfQueryCollection)

[ResultOfQueryCollection](#ResultOfQueryCollection)

[ParamsOfAggregateCollection](#ParamsOfAggregateCollection)

[ResultOfAggregateCollection](#ResultOfAggregateCollection)

[ParamsOfWaitForCollection](#ParamsOfWaitForCollection)

[ResultOfWaitForCollection](#ResultOfWaitForCollection)

[ResultOfSubscribeCollection](#ResultOfSubscribeCollection)

[ParamsOfSubscribeCollection](#ParamsOfSubscribeCollection)

[ParamsOfFindLastShardBlock](#ParamsOfFindLastShardBlock)

[ResultOfFindLastShardBlock](#ResultOfFindLastShardBlock)

[EndpointsSet](#EndpointsSet)


# Functions
## query

Performs DAppServer GraphQL query.

```ts
type ParamsOfQuery = {
    query: string,
    variables?: any
}

type ResultOfQuery = {
    result: any
}

function query(
    params: ParamsOfQuery,
): Promise<ResultOfQuery>;
```
### Parameters
- `query`: _string_ – GraphQL query text.
- `variables`?: _any_ – Variables used in query.
<br>Must be a map with named values that can be used in query.
### Result

- `result`: _any_ – Result provided by DAppServer.


## batch_query

Performs multiple queries per single fetch.

```ts
type ParamsOfBatchQuery = {
    operations: ParamsOfQueryOperation[]
}

type ResultOfBatchQuery = {
    results: any[]
}

function batch_query(
    params: ParamsOfBatchQuery,
): Promise<ResultOfBatchQuery>;
```
### Parameters
- `operations`: _ParamsOfQueryOperation[]_ – List of query operations that must be performed per single fetch.
### Result

- `results`: _any[]_ – Result values for batched queries.
<br>Returns an array of values. Each value corresponds to `queries` item.


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
}

type ResultOfQueryCollection = {
    result: any[]
}

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


## aggregate_collection

Aggregates collection data.

Aggregates values from the specified `fields` for records
that satisfies the `filter` conditions,

```ts
type ParamsOfAggregateCollection = {
    collection: string,
    filter?: any,
    fields?: FieldAggregation[]
}

type ResultOfAggregateCollection = {
    values: any
}

function aggregate_collection(
    params: ParamsOfAggregateCollection,
): Promise<ResultOfAggregateCollection>;
```
### Parameters
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter.
- `fields`?: _FieldAggregation[]_ – Projection (result) string
### Result

- `values`: _any_ – Values for requested fields.
<br>Returns an array of strings. Each string refers to the corresponding `fields` item.<br>Numeric values is returned as a decimal string representations.


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
}

type ResultOfWaitForCollection = {
    result: any
}

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
}

function unsubscribe(
    params: ResultOfSubscribeCollection,
): Promise<void>;
```
### Parameters
- `handle`: _number_ – Subscription handle.
<br>Must be closed with `unsubscribe`
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
}

type ResultOfSubscribeCollection = {
    handle: number
}

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

- `handle`: _number_ – Subscription handle.
<br>Must be closed with `unsubscribe`


## suspend

Suspends network module to stop any network activity

```ts
function suspend(): Promise<void>;
```
### Result



## resume

Resumes network module to enable network activity

```ts
function resume(): Promise<void>;
```
### Result



## find_last_shard_block

Returns ID of the last block in a specified account shard

```ts
type ParamsOfFindLastShardBlock = {
    address: string
}

type ResultOfFindLastShardBlock = {
    block_id: string
}

function find_last_shard_block(
    params: ParamsOfFindLastShardBlock,
): Promise<ResultOfFindLastShardBlock>;
```
### Parameters
- `address`: _string_ – Account address
### Result

- `block_id`: _string_ – Account shard last block ID


## fetch_endpoints

Requests the list of alternative endpoints from server

```ts
type EndpointsSet = {
    endpoints: string[]
}

function fetch_endpoints(): Promise<EndpointsSet>;
```
### Result

- `endpoints`: _string[]_ – List of endpoints provided by server


## set_endpoints

Sets the list of endpoints to use on reinit

```ts
type EndpointsSet = {
    endpoints: string[]
}

function set_endpoints(
    params: EndpointsSet,
): Promise<void>;
```
### Parameters
- `endpoints`: _string[]_ – List of endpoints provided by server
### Result



# Types
## NetErrorCode
```ts
enum NetErrorCode {
    QueryFailed = 601,
    SubscribeFailed = 602,
    WaitForFailed = 603,
    GetSubscriptionResultFailed = 604,
    InvalidServerResponse = 605,
    ClockOutOfSync = 606,
    WaitForTimeout = 607,
    GraphqlError = 608,
    NetworkModuleSuspended = 609,
    WebsocketDisconnected = 610,
    NotSupported = 611,
    NoEndpointsProvided = 612,
    GraphqlWebsocketInitError = 613,
    NetworkModuleResumed = 614
}
```
One of the following value:

- `QueryFailed = 601`
- `SubscribeFailed = 602`
- `WaitForFailed = 603`
- `GetSubscriptionResultFailed = 604`
- `InvalidServerResponse = 605`
- `ClockOutOfSync = 606`
- `WaitForTimeout = 607`
- `GraphqlError = 608`
- `NetworkModuleSuspended = 609`
- `WebsocketDisconnected = 610`
- `NotSupported = 611`
- `NoEndpointsProvided = 612`
- `GraphqlWebsocketInitError = 613`
- `NetworkModuleResumed = 614`


## OrderBy
```ts
type OrderBy = {
    path: string,
    direction: SortDirection
}
```
- `path`: _string_
- `direction`: _[SortDirection](mod_net.md#SortDirection)_


## SortDirection
```ts
enum SortDirection {
    ASC = "ASC",
    DESC = "DESC"
}
```
One of the following value:

- `ASC = "ASC"`
- `DESC = "DESC"`


## ParamsOfQuery
```ts
type ParamsOfQuery = {
    query: string,
    variables?: any
}
```
- `query`: _string_ – GraphQL query text.
- `variables`?: _any_ – Variables used in query.
<br>Must be a map with named values that can be used in query.


## ResultOfQuery
```ts
type ResultOfQuery = {
    result: any
}
```
- `result`: _any_ – Result provided by DAppServer.


## ParamsOfBatchQuery
```ts
type ParamsOfBatchQuery = {
    operations: ParamsOfQueryOperation[]
}
```
- `operations`: _ParamsOfQueryOperation[]_ – List of query operations that must be performed per single fetch.


## ResultOfBatchQuery
```ts
type ResultOfBatchQuery = {
    results: any[]
}
```
- `results`: _any[]_ – Result values for batched queries.
<br>Returns an array of values. Each value corresponds to `queries` item.


## ParamsOfQueryCollection
```ts
type ParamsOfQueryCollection = {
    collection: string,
    filter?: any,
    result: string,
    order?: OrderBy[],
    limit?: number
}
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
}
```
- `result`: _any[]_ – Objects that match the provided criteria


## ParamsOfAggregateCollection
```ts
type ParamsOfAggregateCollection = {
    collection: string,
    filter?: any,
    fields?: FieldAggregation[]
}
```
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter.
- `fields`?: _FieldAggregation[]_ – Projection (result) string


## ResultOfAggregateCollection
```ts
type ResultOfAggregateCollection = {
    values: any
}
```
- `values`: _any_ – Values for requested fields.
<br>Returns an array of strings. Each string refers to the corresponding `fields` item.<br>Numeric values is returned as a decimal string representations.


## ParamsOfWaitForCollection
```ts
type ParamsOfWaitForCollection = {
    collection: string,
    filter?: any,
    result: string,
    timeout?: number
}
```
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `timeout`?: _number_ – Query timeout


## ResultOfWaitForCollection
```ts
type ResultOfWaitForCollection = {
    result: any
}
```
- `result`: _any_ – First found object that matches the provided criteria


## ResultOfSubscribeCollection
```ts
type ResultOfSubscribeCollection = {
    handle: number
}
```
- `handle`: _number_ – Subscription handle.
<br>Must be closed with `unsubscribe`


## ParamsOfSubscribeCollection
```ts
type ParamsOfSubscribeCollection = {
    collection: string,
    filter?: any,
    result: string
}
```
- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string


## ParamsOfFindLastShardBlock
```ts
type ParamsOfFindLastShardBlock = {
    address: string
}
```
- `address`: _string_ – Account address


## ResultOfFindLastShardBlock
```ts
type ResultOfFindLastShardBlock = {
    block_id: string
}
```
- `block_id`: _string_ – Account shard last block ID


## EndpointsSet
```ts
type EndpointsSet = {
    endpoints: string[]
}
```
- `endpoints`: _string[]_ – List of endpoints provided by server



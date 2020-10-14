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

function queryCollection(
    params: ParamsOfQueryCollection,
    responseHandler: ResponseHandler | null,
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

function waitForCollection(
    params: ParamsOfWaitForCollection,
    responseHandler: ResponseHandler | null,
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

function unsubscribe(
    params: ResultOfSubscribeCollection,
    responseHandler: ResponseHandler | null,
): Promise<void>;

```
### Parameters
- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function
### Result



## subscribe_collection

```ts

function subscribeCollection(
    params: ParamsOfSubscribeCollection,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfSubscribeCollection>;

```
### Parameters
- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
### Result

- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function


# Types
## OrderBy

- `path`: _string_
- `direction`: _[SortDirection](mod_net.md#SortDirection)_


## SortDirection



## ParamsOfQueryCollection

- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ –  sorting order
- `limit`?: _number_ –  number of documents to return


## ResultOfQueryCollection

- `result`: _any[]_ –  objects that match provided criteria


## ParamsOfWaitForCollection

- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string
- `timeout`?: _number_ –  query timeout


## ResultOfWaitForCollection

- `result`: _any_ –  first found object that match provided criteria


## ResultOfSubscribeCollection

- `handle`: _number_ –  handle to subscription. It then can be used in `get_next_subscription_data` function


## unit



## ParamsOfSubscribeCollection

- `collection`: _string_ –  collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ –  collection filter
- `result`: _string_ –  projection (result) string



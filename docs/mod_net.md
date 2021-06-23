# Module net

Network access.


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

[get_endpoints](#get_endpoints) – Requests the list of alternative endpoints from server

[query_counterparties](#query_counterparties) – Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

[query_transaction_tree](#query_transaction_tree) – Returns transactions tree for specific message.

[create_block_iterator](#create_block_iterator) – Creates block iterator.

[resume_block_iterator](#resume_block_iterator) – Resumes block iterator.

[create_transaction_iterator](#create_transaction_iterator) – Creates transaction iterator.

[resume_transaction_iterator](#resume_transaction_iterator) – Resumes transaction iterator.

[iterator_next](#iterator_next) – Returns next available items.

[remove_iterator](#remove_iterator) – Removes an iterator

## Types
[NetErrorCode](#NetErrorCode)

[OrderBy](#OrderBy)

[SortDirection](#SortDirection)

[ParamsOfQueryOperation](#ParamsOfQueryOperation)

[FieldAggregation](#FieldAggregation)

[AggregationFn](#AggregationFn)

[TransactionNode](#TransactionNode)

[MessageNode](#MessageNode)

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

[ResultOfGetEndpoints](#ResultOfGetEndpoints)

[ParamsOfQueryCounterparties](#ParamsOfQueryCounterparties)

[ParamsOfQueryTransactionTree](#ParamsOfQueryTransactionTree)

[ResultOfQueryTransactionTree](#ResultOfQueryTransactionTree)

[ParamsOfCreateBlockIterator](#ParamsOfCreateBlockIterator)

[RegisteredIterator](#RegisteredIterator)

[ParamsOfResumeBlockIterator](#ParamsOfResumeBlockIterator)

[ParamsOfCreateTransactionIterator](#ParamsOfCreateTransactionIterator)

[ParamsOfResumeTransactionIterator](#ParamsOfResumeTransactionIterator)

[ParamsOfIteratorNext](#ParamsOfIteratorNext)

[ResultOfIteratorNext](#ResultOfIteratorNext)


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
- `operations`: _[ParamsOfQueryOperation](mod_net.md#ParamsOfQueryOperation)[]_ – List of query operations that must be performed per single fetch.


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
- `filter`?: _any_ – Collection filter
- `fields`?: _[FieldAggregation](mod_net.md#FieldAggregation)[]_ – Projection (result) string


### Result

- `values`: _any_ – Values for requested fields.
<br>Returns an array of strings. Each string refers to the corresponding `fields` item.<br>Numeric value is returned as a decimal string representations.


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


## subscribe_collection

Creates a subscription

Triggers for each insert/update of data that satisfies
the `filter` conditions.
The projection fields are limited to `result` fields.

The subscription is a persistent communication channel between
client and Free TON Network.
All changes in the blockchain will be reflected in realtime.
Changes means inserts and updates of the blockchain entities.

### Important Notes on Subscriptions

Unfortunately sometimes the connection with the network brakes down.
In this situation the library attempts to reconnect to the network.
This reconnection sequence can take significant time.
All of this time the client is disconnected from the network.

Bad news is that all blockchain changes that happened while
the client was disconnected are lost.

Good news is that the client report errors to the callback when
it loses and resumes connection.

So, if the lost changes are important to the application then
the application must handle these error reports.

Library reports errors with `responseType` == 101
and the error object passed via `params`.

When the library has successfully reconnected
the application receives callback with
`responseType` == 101 and `params.code` == 614 (NetworkModuleResumed).

Application can use several ways to handle this situation:
- If application monitors changes for the single blockchain
object (for example specific account):  application
can perform a query for this object and handle actual data as a
regular data from the subscription.
- If application monitors sequence of some blockchain objects
(for example transactions of the specific account): application must
refresh all cached (or visible to user) lists where this sequences presents.

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
- `responseHandler`?: _[ResponseHandler](modules.md#ResponseHandler)_ – additional responses handler.

### Result

- `handle`: _number_ – Subscription handle.
<br>Must be closed with `unsubscribe`


## suspend

Suspends network module to stop any network activity

```ts
function suspend(): Promise<void>;
```


## resume

Resumes network module to enable network activity

```ts
function resume(): Promise<void>;
```


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


## get_endpoints

Requests the list of alternative endpoints from server

```ts
type ResultOfGetEndpoints = {
    query: string,
    endpoints: string[]
}

function get_endpoints(): Promise<ResultOfGetEndpoints>;
```


### Result

- `query`: _string_ – Current query endpoint
- `endpoints`: _string[]_ – List of all endpoints used by client


## query_counterparties

Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

*Attention* this query retrieves data from 'Counterparties' service which is not supported in
the opensource version of DApp Server (and will not be supported) as well as in TON OS SE (will be supported in SE in future),
but is always accessible via [TON OS Devnet/Mainnet Clouds](https://docs.ton.dev/86757ecb2/p/85c869-networks)

```ts
type ParamsOfQueryCounterparties = {
    account: string,
    result: string,
    first?: number,
    after?: string
}

type ResultOfQueryCollection = {
    result: any[]
}

function query_counterparties(
    params: ParamsOfQueryCounterparties,
): Promise<ResultOfQueryCollection>;
```
### Parameters
- `account`: _string_ – Account address
- `result`: _string_ – Projection (result) string
- `first`?: _number_ – Number of counterparties to return
- `after`?: _string_ – `cursor` field of the last received result


### Result

- `result`: _any[]_ – Objects that match the provided criteria


## query_transaction_tree

Returns transactions tree for specific message.

Performs recursive retrieval of the transactions tree produced by the specific message:
in_msg -> dst_transaction -> out_messages -> dst_transaction -> ...

All retrieved messages and transactions will be included
into `result.messages` and `result.transactions` respectively.

The retrieval process will stop when the retrieved transaction count is more than 50.

It is guaranteed that each message in `result.messages` has the corresponding transaction
in the `result.transactions`.

But there are no guaranties that all messages from transactions `out_msgs` are
presented in `result.messages`.
So the application have to continue retrieval for missing messages if it requires.

```ts
type ParamsOfQueryTransactionTree = {
    in_msg: string,
    abi_registry?: Abi[],
    timeout?: number
}

type ResultOfQueryTransactionTree = {
    messages: MessageNode[],
    transactions: TransactionNode[]
}

function query_transaction_tree(
    params: ParamsOfQueryTransactionTree,
): Promise<ResultOfQueryTransactionTree>;
```
### Parameters
- `in_msg`: _string_ – Input message id.
- `abi_registry`?: _[Abi](mod_abi.md#Abi)[]_ – List of contract ABIs that will be used to decode message bodies. Library will try to decode each returned message body using any ABI from the registry.
- `timeout`?: _number_ – Timeout used to limit waiting time for the missing messages and transaction.
<br>If some of the following messages and transactions are missing yet<br>The maximum waiting time is regulated by this option.<br><br>Default value is 60000 (1 min).


### Result

- `messages`: _[MessageNode](mod_net.md#MessageNode)[]_ – Messages.
- `transactions`: _[TransactionNode](mod_net.md#TransactionNode)[]_ – Transactions.


## create_block_iterator

Creates block iterator.

Block iterator uses robust iteration methods that guaranties that every
block in the specified range isn't missed or iterated twice.

Iterated range can be reduced with some filters:
- `start_time` – the bottom time range. Only blocks with `gen_utime`
more or equal to this value is iterated. If this parameter is omitted then there is
no bottom time edge, so all blocks since zero state is iterated.
- `end_time` – the upper time range. Only blocks with `gen_utime`
less then this value is iterated. If this parameter is omitted then there is
no upper time edge, so iterator never finishes.
- `shard_filter` – workchains and shard prefixes that reduces set of interesting
blocks. Block conforms to the shard filter if it belongs to the filter workchain
and the first bits of block's `shard` fields matches to the shard prefix.
Only blocks with suitable shard are iterated.

Items iterated is a JSON objects with block data. The minimal set of returned
fields is:

   id
   gen_utime
   workchain_id
   shard
   after_split
   after_merge
   prev_ref {
       root_hash
   }
   prev_alt_ref {
       root_hash
   }

Application can request an additional fields in the `result` parameter.

Application should call the `remove_iterator` when iterator is no longer required.

```ts
type ParamsOfCreateBlockIterator = {
    start_time?: number,
    end_time?: number,
    shard_filter?: string[],
    result?: string
}

type RegisteredIterator = {
    handle: number
}

function create_block_iterator(
    params: ParamsOfCreateBlockIterator,
): Promise<RegisteredIterator>;
```
### Parameters
- `start_time`?: _number_ – Starting time to iterate from.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` >= `start_time`.<br>Otherwise the iteration starts from zero state.<br><br>Must be specified in seconds.
- `end_time`?: _number_ – Optional end time to iterate for.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` < `end_time`.<br>Otherwise the iteration never stops.<br><br>Must be specified in seconds.
- `shard_filter`?: _string[]_ – Shard prefix filter.
<br>If the application specifies this parameter and it is not the empty array<br>then the iteration will include items related to accounts that belongs to<br>the specified shard prefixes.<br>Shard prefix must be represented as a string "workchain:prefix".<br>Where `workchain` is a signed integer and the `prefix` if a hexadecimal<br>representation if the 64-bit unsigned integer with tagged shard prefix.<br>For example: "0:3800000000000000".
- `result`?: _string_ – Projection (result) string.
<br>List of the fields that must be returned for iterated items.<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.


### Result

- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


## resume_block_iterator

Resumes block iterator.

The iterator stays exactly at the same position where the `resume_state` was catch.

Application should call the `remove_iterator` when iterator is no longer required.

```ts
type ParamsOfResumeBlockIterator = {
    resume_state: any
}

type RegisteredIterator = {
    handle: number
}

function resume_block_iterator(
    params: ParamsOfResumeBlockIterator,
): Promise<RegisteredIterator>;
```
### Parameters
- `resume_state`: _any_ – Iterator state from which to resume.
<br>Same as value returned from `iterator_next`.


### Result

- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


## create_transaction_iterator

Creates transaction iterator.

Transaction iterator uses robust iteration methods that guaranties that every
transaction in the specified range isn't missed or iterated twice.

Iterated range can be reduced with some filters:
- `start_time` – the bottom time range. Only transactions with `now`
more or equal to this value is iterated. If this parameter is omitted then there is
no bottom time edge, so all transactions since zero state is iterated.
- `end_time` – the upper time range. Only transactions with `now`
less then this value is iterated. If this parameter is omitted then there is
no upper time edge, so iterator never finishes.
- `shard_filter` – workchains and shard prefixes that reduces set of interesting
accounts. Account address conforms to the shard filter if
it belongs to the filter workchain and the first bits of address matches to
the shard prefix. Only transactions with suitable account addresses are iterated.
- `accounts_filter` – set of account addresses whose transactions must be iterated.
Note that accounts filter can conflict with shard filter so application must combine
thees filters carefully.

Items iterated is a JSON objects with transaction data. The minimal set of returned
fields is:

    id
    account_addr
    now
    balance_delta(format:DEC)
    bounce { bounce_type }
    in_message {
        id
        value(format:DEC)
        msg_type
        src
    }
    out_messages {
        id
        value(format:DEC)
        msg_type
        dst
    }

Application can request an additional fields in the `result` parameter.

Another parameter that affects on the returned fields is the `include_transfers`.
When this parameter is `true` the iterator computes and adds `transfer` field containing
list of the useful `TransactionTransfer` objects.
Each transfer is calculated from the particular message related to the transaction
and has the following structure:
- message – source message identifier.
- isBounced – indicates that the transaction is bounced.
- isDeposit – indicates that this transfer is the deposit or withdraw.
- counterparty – account address of the transfer source or destination depending on `isDeposit`.
- value – amount of nano tokens transferred. The value is represented as a decimal string
because the actual value can be more precise than the JSON number can represent. Application
must use this string carefully – conversion to number can follow to loose of precision.

Application should call the `remove_iterator` when iterator is no longer required.

```ts
type ParamsOfCreateTransactionIterator = {
    start_time?: number,
    end_time?: number,
    shard_filter?: string[],
    accounts_filter?: string[],
    result?: string,
    include_transfers?: boolean
}

type RegisteredIterator = {
    handle: number
}

function create_transaction_iterator(
    params: ParamsOfCreateTransactionIterator,
): Promise<RegisteredIterator>;
```
### Parameters
- `start_time`?: _number_ – Starting time to iterate from.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` >= `start_time`.<br>Otherwise the iteration starts from zero state.<br><br>Must be specified in seconds.
- `end_time`?: _number_ – Optional end time to iterate for.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` < `end_time`.<br>Otherwise the iteration never stops.<br><br>Must be specified in seconds.
- `shard_filter`?: _string[]_ – Shard prefix filters.
<br>If the application specifies this parameter and it is not the empty array<br>then the iteration will include items related to accounts that belongs to<br>the specified shard prefixes.<br>Shard prefix must be represented as a string "workchain:prefix".<br>Where `workchain` is a signed integer and the `prefix` if a hexadecimal<br>representation if the 64-bit unsigned integer with tagged shard prefix.<br>For example: "0:3800000000000000".
- `accounts_filter`?: _string[]_ – Account address filter.
<br>Application can specify the list of accounts for which<br>it wants to iterate transactions.<br><br>If this parameter is missing or an empty list then the library iterates<br>transactions for all accounts that passes the shard filter.<br><br>Note that the library doesn't detect conflicts between the account filter and the shard filter<br>if both is specified.<br>So it is an application responsibility to specify the correct filter combination.
- `result`?: _string_ – Projection (result) string.
<br>List of the fields that must be returned for iterated items.<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.
- `include_transfers`?: _boolean_ – Include `transfers` field in iterated transactions.
<br>If this parameter is `true` then each transaction contains field<br>`transfers` with list of transfer<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.


### Result

- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


## resume_transaction_iterator

Resumes transaction iterator.

The iterator stays exactly at the same position where the `resume_state` was catch.
Note that `resume_state` doesn't store the account filter. If the application requires
to use the same account filter as it was when the iterator was created then the application
must account filter again in `accounts_filter` parameter.

Application should call the `remove_iterator` when iterator is no longer required.

```ts
type ParamsOfResumeTransactionIterator = {
    resume_state: any,
    accounts_filter?: string[]
}

type RegisteredIterator = {
    handle: number
}

function resume_transaction_iterator(
    params: ParamsOfResumeTransactionIterator,
): Promise<RegisteredIterator>;
```
### Parameters
- `resume_state`: _any_ – Iterator state from which to resume.
<br>Same as value returned from `iterator_next`.
- `accounts_filter`?: _string[]_ – Account address filter.
<br>Application can specify the list of accounts for which<br>it wants to iterate transactions.<br><br>If this parameter is missing or an empty list then the library iterates<br>transactions for all accounts that passes the shard filter.<br><br>Note that the library doesn't detect conflicts between the account filter and the shard filter<br>if both is specified.<br>So it is an application responsibility to specify the correct filter combination.


### Result

- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


## iterator_next

Returns next available items.

In addition to available items this function returns the `has_more` flag
indicating that the iterator isn't reach the end of the iterated range yet.

This function can return the empty list of available items but
indicates that there are more items is available.
This situation appears when the iterator doesn't reach iterated range
but database doesn't contains available items yet.

If application requests resume state in `return_resume_state` parameter
then this function returns `resume_state` that can be used later to
resume the iteration from the position after returned items.

The structure of the items returned depends on the iterator used.
See the description to the appropriated iterator creation function.

```ts
type ParamsOfIteratorNext = {
    iterator: number,
    limit?: number,
    return_resume_state?: boolean
}

type ResultOfIteratorNext = {
    items: any[],
    has_more: boolean,
    resume_state?: any
}

function iterator_next(
    params: ParamsOfIteratorNext,
): Promise<ResultOfIteratorNext>;
```
### Parameters
- `iterator`: _number_ – Iterator handle
- `limit`?: _number_ – Maximum count of the returned items.
<br>If value is missing or is less than 1 the library uses 1.
- `return_resume_state`?: _boolean_ – Indicates that function must return the iterator state that can be used for resuming iteration.


### Result

- `items`: _any[]_ – Next available items.
<br>Note that `iterator_next` can return an empty items and `has_more` equals to `true`.<br>In this case the application have to continue iteration.<br>Such situation can take place when there is no data yet but<br>the requested `end_time` is not reached.
- `has_more`: _boolean_ – Indicates that there are more available items in iterated range.
- `resume_state`?: _any_ – Optional iterator state that can be used for resuming iteration.
<br>This field is returned only if the `return_resume_state` parameter<br>is specified.<br><br>Note that `resume_state` corresponds to the iteration position<br>after the returned items.


## remove_iterator

Removes an iterator

Frees all resources allocated in library to serve iterator.

Application always should call the `remove_iterator` when iterator
is no longer required.

```ts
type RegisteredIterator = {
    handle: number
}

function remove_iterator(
    params: RegisteredIterator,
): Promise<void>;
```
### Parameters
- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


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


## ParamsOfQueryOperation
```ts
type ParamsOfQueryOperation = ({
    type: 'QueryCollection'
} & ParamsOfQueryCollection) | ({
    type: 'WaitForCollection'
} & ParamsOfWaitForCollection) | ({
    type: 'AggregateCollection'
} & ParamsOfAggregateCollection) | ({
    type: 'QueryCounterparties'
} & ParamsOfQueryCounterparties)
```
Depends on value of the  `type` field.

When _type_ is _'QueryCollection'_

- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `order`?: _[OrderBy](mod_net.md#OrderBy)[]_ – Sorting order
- `limit`?: _number_ – Number of documents to return

When _type_ is _'WaitForCollection'_

- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `result`: _string_ – Projection (result) string
- `timeout`?: _number_ – Query timeout

When _type_ is _'AggregateCollection'_

- `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block_signatures)
- `filter`?: _any_ – Collection filter
- `fields`?: _[FieldAggregation](mod_net.md#FieldAggregation)[]_ – Projection (result) string

When _type_ is _'QueryCounterparties'_

- `account`: _string_ – Account address
- `result`: _string_ – Projection (result) string
- `first`?: _number_ – Number of counterparties to return
- `after`?: _string_ – `cursor` field of the last received result


Variant constructors:

```ts
function paramsOfQueryOperationQueryCollection(params: ParamsOfQueryCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationWaitForCollection(params: ParamsOfWaitForCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationAggregateCollection(params: ParamsOfAggregateCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationQueryCounterparties(params: ParamsOfQueryCounterparties): ParamsOfQueryOperation;
```

## FieldAggregation
```ts
type FieldAggregation = {
    field: string,
    fn: AggregationFn
}
```
- `field`: _string_ – Dot separated path to the field
- `fn`: _[AggregationFn](mod_net.md#AggregationFn)_ – Aggregation function that must be applied to field values


## AggregationFn
```ts
enum AggregationFn {
    COUNT = "COUNT",
    MIN = "MIN",
    MAX = "MAX",
    SUM = "SUM",
    AVERAGE = "AVERAGE"
}
```
One of the following value:

- `COUNT = "COUNT"` – Returns count of filtered record
- `MIN = "MIN"` – Returns the minimal value for a field in filtered records
- `MAX = "MAX"` – Returns the maximal value for a field in filtered records
- `SUM = "SUM"` – Returns a sum of values for a field in filtered records
- `AVERAGE = "AVERAGE"` – Returns an average value for a field in filtered records


## TransactionNode
```ts
type TransactionNode = {
    id: string,
    in_msg: string,
    out_msgs: string[],
    account_addr: string,
    total_fees: string,
    aborted: boolean,
    exit_code?: number
}
```
- `id`: _string_ – Transaction id.
- `in_msg`: _string_ – In message id.
- `out_msgs`: _string[]_ – Out message ids.
- `account_addr`: _string_ – Account address.
- `total_fees`: _string_ – Transactions total fees.
- `aborted`: _boolean_ – Aborted flag.
- `exit_code`?: _number_ – Compute phase exit code.


## MessageNode
```ts
type MessageNode = {
    id: string,
    src_transaction_id?: string,
    dst_transaction_id?: string,
    src?: string,
    dst?: string,
    value?: string,
    bounce: boolean,
    decoded_body?: DecodedMessageBody
}
```
- `id`: _string_ – Message id.
- `src_transaction_id`?: _string_ – Source transaction id.
<br>This field is missing for an external inbound messages.
- `dst_transaction_id`?: _string_ – Destination transaction id.
<br>This field is missing for an external outbound messages.
- `src`?: _string_ – Source address.
- `dst`?: _string_ – Destination address.
- `value`?: _string_ – Transferred tokens value.
- `bounce`: _boolean_ – Bounce flag.
- `decoded_body`?: _[DecodedMessageBody](mod_abi.md#DecodedMessageBody)_ – Decoded body.
<br>Library tries to decode message body using provided `params.abi_registry`.<br>This field will be missing if none of the provided abi can be used to decode.


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
- `operations`: _[ParamsOfQueryOperation](mod_net.md#ParamsOfQueryOperation)[]_ – List of query operations that must be performed per single fetch.


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
- `filter`?: _any_ – Collection filter
- `fields`?: _[FieldAggregation](mod_net.md#FieldAggregation)[]_ – Projection (result) string


## ResultOfAggregateCollection
```ts
type ResultOfAggregateCollection = {
    values: any
}
```
- `values`: _any_ – Values for requested fields.
<br>Returns an array of strings. Each string refers to the corresponding `fields` item.<br>Numeric value is returned as a decimal string representations.


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


## ResultOfGetEndpoints
```ts
type ResultOfGetEndpoints = {
    query: string,
    endpoints: string[]
}
```
- `query`: _string_ – Current query endpoint
- `endpoints`: _string[]_ – List of all endpoints used by client


## ParamsOfQueryCounterparties
```ts
type ParamsOfQueryCounterparties = {
    account: string,
    result: string,
    first?: number,
    after?: string
}
```
- `account`: _string_ – Account address
- `result`: _string_ – Projection (result) string
- `first`?: _number_ – Number of counterparties to return
- `after`?: _string_ – `cursor` field of the last received result


## ParamsOfQueryTransactionTree
```ts
type ParamsOfQueryTransactionTree = {
    in_msg: string,
    abi_registry?: Abi[],
    timeout?: number
}
```
- `in_msg`: _string_ – Input message id.
- `abi_registry`?: _[Abi](mod_abi.md#Abi)[]_ – List of contract ABIs that will be used to decode message bodies. Library will try to decode each returned message body using any ABI from the registry.
- `timeout`?: _number_ – Timeout used to limit waiting time for the missing messages and transaction.
<br>If some of the following messages and transactions are missing yet<br>The maximum waiting time is regulated by this option.<br><br>Default value is 60000 (1 min).


## ResultOfQueryTransactionTree
```ts
type ResultOfQueryTransactionTree = {
    messages: MessageNode[],
    transactions: TransactionNode[]
}
```
- `messages`: _[MessageNode](mod_net.md#MessageNode)[]_ – Messages.
- `transactions`: _[TransactionNode](mod_net.md#TransactionNode)[]_ – Transactions.


## ParamsOfCreateBlockIterator
```ts
type ParamsOfCreateBlockIterator = {
    start_time?: number,
    end_time?: number,
    shard_filter?: string[],
    result?: string
}
```
- `start_time`?: _number_ – Starting time to iterate from.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` >= `start_time`.<br>Otherwise the iteration starts from zero state.<br><br>Must be specified in seconds.
- `end_time`?: _number_ – Optional end time to iterate for.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` < `end_time`.<br>Otherwise the iteration never stops.<br><br>Must be specified in seconds.
- `shard_filter`?: _string[]_ – Shard prefix filter.
<br>If the application specifies this parameter and it is not the empty array<br>then the iteration will include items related to accounts that belongs to<br>the specified shard prefixes.<br>Shard prefix must be represented as a string "workchain:prefix".<br>Where `workchain` is a signed integer and the `prefix` if a hexadecimal<br>representation if the 64-bit unsigned integer with tagged shard prefix.<br>For example: "0:3800000000000000".
- `result`?: _string_ – Projection (result) string.
<br>List of the fields that must be returned for iterated items.<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.


## RegisteredIterator
```ts
type RegisteredIterator = {
    handle: number
}
```
- `handle`: _number_ – Iterator handle.
<br>Must be removed using `remove_iterator`<br>when it is no more needed for the application.


## ParamsOfResumeBlockIterator
```ts
type ParamsOfResumeBlockIterator = {
    resume_state: any
}
```
- `resume_state`: _any_ – Iterator state from which to resume.
<br>Same as value returned from `iterator_next`.


## ParamsOfCreateTransactionIterator
```ts
type ParamsOfCreateTransactionIterator = {
    start_time?: number,
    end_time?: number,
    shard_filter?: string[],
    accounts_filter?: string[],
    result?: string,
    include_transfers?: boolean
}
```
- `start_time`?: _number_ – Starting time to iterate from.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` >= `start_time`.<br>Otherwise the iteration starts from zero state.<br><br>Must be specified in seconds.
- `end_time`?: _number_ – Optional end time to iterate for.
<br>If the application specifies this parameter then the iteration<br>includes blocks with `gen_utime` < `end_time`.<br>Otherwise the iteration never stops.<br><br>Must be specified in seconds.
- `shard_filter`?: _string[]_ – Shard prefix filters.
<br>If the application specifies this parameter and it is not the empty array<br>then the iteration will include items related to accounts that belongs to<br>the specified shard prefixes.<br>Shard prefix must be represented as a string "workchain:prefix".<br>Where `workchain` is a signed integer and the `prefix` if a hexadecimal<br>representation if the 64-bit unsigned integer with tagged shard prefix.<br>For example: "0:3800000000000000".
- `accounts_filter`?: _string[]_ – Account address filter.
<br>Application can specify the list of accounts for which<br>it wants to iterate transactions.<br><br>If this parameter is missing or an empty list then the library iterates<br>transactions for all accounts that passes the shard filter.<br><br>Note that the library doesn't detect conflicts between the account filter and the shard filter<br>if both is specified.<br>So it is an application responsibility to specify the correct filter combination.
- `result`?: _string_ – Projection (result) string.
<br>List of the fields that must be returned for iterated items.<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.
- `include_transfers`?: _boolean_ – Include `transfers` field in iterated transactions.
<br>If this parameter is `true` then each transaction contains field<br>`transfers` with list of transfer<br>This field is the same as the `result` parameter of<br>the `query_collection` function.<br>Note that iterated items can contains additional fields that is<br>not requested in the `result`.


## ParamsOfResumeTransactionIterator
```ts
type ParamsOfResumeTransactionIterator = {
    resume_state: any,
    accounts_filter?: string[]
}
```
- `resume_state`: _any_ – Iterator state from which to resume.
<br>Same as value returned from `iterator_next`.
- `accounts_filter`?: _string[]_ – Account address filter.
<br>Application can specify the list of accounts for which<br>it wants to iterate transactions.<br><br>If this parameter is missing or an empty list then the library iterates<br>transactions for all accounts that passes the shard filter.<br><br>Note that the library doesn't detect conflicts between the account filter and the shard filter<br>if both is specified.<br>So it is an application responsibility to specify the correct filter combination.


## ParamsOfIteratorNext
```ts
type ParamsOfIteratorNext = {
    iterator: number,
    limit?: number,
    return_resume_state?: boolean
}
```
- `iterator`: _number_ – Iterator handle
- `limit`?: _number_ – Maximum count of the returned items.
<br>If value is missing or is less than 1 the library uses 1.
- `return_resume_state`?: _boolean_ – Indicates that function must return the iterator state that can be used for resuming iteration.


## ResultOfIteratorNext
```ts
type ResultOfIteratorNext = {
    items: any[],
    has_more: boolean,
    resume_state?: any
}
```
- `items`: _any[]_ – Next available items.
<br>Note that `iterator_next` can return an empty items and `has_more` equals to `true`.<br>In this case the application have to continue iteration.<br>Such situation can take place when there is no data yet but<br>the requested `end_time` is not reached.
- `has_more`: _boolean_ – Indicates that there are more available items in iterated range.
- `resume_state`?: _any_ – Optional iterator state that can be used for resuming iteration.
<br>This field is returned only if the `return_resume_state` parameter<br>is specified.<br><br>Note that `resume_state` corresponds to the iteration position<br>after the returned items.



# Module net

## Module net

Network access.

### Functions

[query](mod\_net.md#query) – Performs DAppServer GraphQL query.

[batch\_query](mod\_net.md#batch\_query) – Performs multiple queries per single fetch.

[query\_collection](mod\_net.md#query\_collection) – Queries collection data

[aggregate\_collection](mod\_net.md#aggregate\_collection) – Aggregates collection data.

[wait\_for\_collection](mod\_net.md#wait\_for\_collection) – Returns an object that fulfills the conditions or waits for its appearance

[unsubscribe](mod\_net.md#unsubscribe) – Cancels a subscription

[subscribe\_collection](mod\_net.md#subscribe\_collection) – Creates a collection subscription

[subscribe](mod\_net.md#subscribe) – Creates a subscription

[suspend](mod\_net.md#suspend) – Suspends network module to stop any network activity

[resume](mod\_net.md#resume) – Resumes network module to enable network activity

[find\_last\_shard\_block](mod\_net.md#find\_last\_shard\_block) – Returns ID of the last block in a specified account shard

[fetch\_endpoints](mod\_net.md#fetch\_endpoints) – Requests the list of alternative endpoints from server

[set\_endpoints](mod\_net.md#set\_endpoints) – Sets the list of endpoints to use on reinit

[get\_endpoints](mod\_net.md#get\_endpoints) – Requests the list of alternative endpoints from server

[query\_counterparties](mod\_net.md#query\_counterparties) – Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

[query\_transaction\_tree](mod\_net.md#query\_transaction\_tree) – Returns a tree of transactions triggered by a specific message.

[create\_block\_iterator](mod\_net.md#create\_block\_iterator) – Creates block iterator.

[resume\_block\_iterator](mod\_net.md#resume\_block\_iterator) – Resumes block iterator.

[create\_transaction\_iterator](mod\_net.md#create\_transaction\_iterator) – Creates transaction iterator.

[resume\_transaction\_iterator](mod\_net.md#resume\_transaction\_iterator) – Resumes transaction iterator.

[iterator\_next](mod\_net.md#iterator\_next) – Returns next available items.

[remove\_iterator](mod\_net.md#remove\_iterator) – Removes an iterator

### Types

[NetErrorCode](mod\_net.md#neterrorcode)

[OrderBy](mod\_net.md#orderby)

[SortDirection](mod\_net.md#sortdirection)

[ParamsOfQueryOperation](mod\_net.md#paramsofqueryoperation)

[FieldAggregation](mod\_net.md#fieldaggregation)

[AggregationFn](mod\_net.md#aggregationfn)

[TransactionNode](mod\_net.md#transactionnode)

[MessageNode](mod\_net.md#messagenode)

[ParamsOfQuery](mod\_net.md#paramsofquery)

[ResultOfQuery](mod\_net.md#resultofquery)

[ParamsOfBatchQuery](mod\_net.md#paramsofbatchquery)

[ResultOfBatchQuery](mod\_net.md#resultofbatchquery)

[ParamsOfQueryCollection](mod\_net.md#paramsofquerycollection)

[ResultOfQueryCollection](mod\_net.md#resultofquerycollection)

[ParamsOfAggregateCollection](mod\_net.md#paramsofaggregatecollection)

[ResultOfAggregateCollection](mod\_net.md#resultofaggregatecollection)

[ParamsOfWaitForCollection](mod\_net.md#paramsofwaitforcollection)

[ResultOfWaitForCollection](mod\_net.md#resultofwaitforcollection)

[ResultOfSubscribeCollection](mod\_net.md#resultofsubscribecollection)

[ParamsOfSubscribeCollection](mod\_net.md#paramsofsubscribecollection)

[ParamsOfSubscribe](mod\_net.md#paramsofsubscribe)

[ParamsOfFindLastShardBlock](mod\_net.md#paramsoffindlastshardblock)

[ResultOfFindLastShardBlock](mod\_net.md#resultoffindlastshardblock)

[EndpointsSet](mod\_net.md#endpointsset)

[ResultOfGetEndpoints](mod\_net.md#resultofgetendpoints)

[ParamsOfQueryCounterparties](mod\_net.md#paramsofquerycounterparties)

[ParamsOfQueryTransactionTree](mod\_net.md#paramsofquerytransactiontree)

[ResultOfQueryTransactionTree](mod\_net.md#resultofquerytransactiontree)

[ParamsOfCreateBlockIterator](mod\_net.md#paramsofcreateblockiterator)

[RegisteredIterator](mod\_net.md#registerediterator)

[ParamsOfResumeBlockIterator](mod\_net.md#paramsofresumeblockiterator)

[ParamsOfCreateTransactionIterator](mod\_net.md#paramsofcreatetransactioniterator)

[ParamsOfResumeTransactionIterator](mod\_net.md#paramsofresumetransactioniterator)

[ParamsOfIteratorNext](mod\_net.md#paramsofiteratornext)

[ResultOfIteratorNext](mod\_net.md#resultofiteratornext)

## Functions

### query

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

#### Parameters

* `query`: _string_ – GraphQL query text.
* `variables`?: _any_ – Variables used in query.\
  Must be a map with named values that can be used in query.

#### Result

* `result`: _any_ – Result provided by DAppServer.

### batch\_query

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

#### Parameters

* `operations`: [_ParamsOfQueryOperation_](mod\_net.md#paramsofqueryoperation)_\[]_ – List of query operations that must be performed per single fetch.

#### Result

* `results`: _any\[]_ – Result values for batched queries.\
  Returns an array of values. Each value corresponds to `queries` item.

### query\_collection

Queries collection data

Queries data that satisfies the `filter` conditions, limits the number of returned records and orders them. The projection fields are limited to `result` fields

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

#### Parameters

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `order`?: [_OrderBy_](mod\_net.md#orderby)_\[]_ – Sorting order
* `limit`?: _number_ – Number of documents to return

#### Result

* `result`: _any\[]_ – Objects that match the provided criteria

### aggregate\_collection

Aggregates collection data.

Aggregates values from the specified `fields` for records that satisfies the `filter` conditions,

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

#### Parameters

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `fields`?: [_FieldAggregation_](mod\_net.md#fieldaggregation)_\[]_ – Projection (result) string

#### Result

* `values`: _any_ – Values for requested fields.\
  Returns an array of strings. Each string refers to the corresponding `fields` item.\
  Numeric value is returned as a decimal string representations.

### wait\_for\_collection

Returns an object that fulfills the conditions or waits for its appearance

Triggers only once. If object that satisfies the `filter` conditions already exists - returns it immediately. If not - waits for insert/update of data within the specified `timeout`, and returns it. The projection fields are limited to `result` fields

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

#### Parameters

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `timeout`?: _number_ – Query timeout

#### Result

* `result`: _any_ – First found object that matches the provided criteria

### unsubscribe

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

#### Parameters

* `handle`: _number_ – Subscription handle.\
  Must be closed with `unsubscribe`

### subscribe\_collection

Creates a collection subscription

Triggers for each insert/update of data that satisfies the `filter` conditions. The projection fields are limited to `result` fields.

The subscription is a persistent communication channel between client and Free TON Network. All changes in the blockchain will be reflected in realtime. Changes means inserts and updates of the blockchain entities.

#### Important Notes on Subscriptions

Unfortunately sometimes the connection with the network brakes down. In this situation the library attempts to reconnect to the network. This reconnection sequence can take significant time. All of this time the client is disconnected from the network.

Bad news is that all blockchain changes that happened while the client was disconnected are lost.

Good news is that the client report errors to the callback when it loses and resumes connection.

So, if the lost changes are important to the application then the application must handle these error reports.

Library reports errors with `responseType` == 101 and the error object passed via `params`.

When the library has successfully reconnected the application receives callback with `responseType` == 101 and `params.code` == 614 (NetworkModuleResumed).

Application can use several ways to handle this situation:

* If application monitors changes for the single blockchain object (for example specific account): application can perform a query for this object and handle actual data as a regular data from the subscription.
* If application monitors sequence of some blockchain objects (for example transactions of the specific account): application must refresh all cached (or visible to user) lists where this sequences presents.

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

#### Parameters

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `responseHandler`?: [_ResponseHandler_](modules.md#responsehandler) – additional responses handler.

#### Result

* `handle`: _number_ – Subscription handle.\
  Must be closed with `unsubscribe`

### subscribe

Creates a subscription

The subscription is a persistent communication channel between client and Everscale Network.

#### Important Notes on Subscriptions

Unfortunately sometimes the connection with the network brakes down. In this situation the library attempts to reconnect to the network. This reconnection sequence can take significant time. All of this time the client is disconnected from the network.

Bad news is that all changes that happened while the client was disconnected are lost.

Good news is that the client report errors to the callback when it loses and resumes connection.

So, if the lost changes are important to the application then the application must handle these error reports.

Library reports errors with `responseType` == 101 and the error object passed via `params`.

When the library has successfully reconnected the application receives callback with `responseType` == 101 and `params.code` == 614 (NetworkModuleResumed).

Application can use several ways to handle this situation:

* If application monitors changes for the single object (for example specific account): application can perform a query for this object and handle actual data as a regular data from the subscription.
* If application monitors sequence of some objects (for example transactions of the specific account): application must refresh all cached (or visible to user) lists where this sequences presents.

```ts
type ParamsOfSubscribe = {
    subscription: string,
    variables?: any
}

type ResultOfSubscribeCollection = {
    handle: number
}

function subscribe(
    params: ParamsOfSubscribe,
    responseHandler?: ResponseHandler,
): Promise<ResultOfSubscribeCollection>;
```

#### Parameters

* `subscription`: _string_ – GraphQL subscription text.
* `variables`?: _any_ – Variables used in subscription.\
  Must be a map with named values that can be used in query.
* `responseHandler`?: [_ResponseHandler_](modules.md#responsehandler) – additional responses handler.

#### Result

* `handle`: _number_ – Subscription handle.\
  Must be closed with `unsubscribe`

### suspend

Suspends network module to stop any network activity

```ts
function suspend(): Promise<void>;
```

### resume

Resumes network module to enable network activity

```ts
function resume(): Promise<void>;
```

### find\_last\_shard\_block

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

#### Parameters

* `address`: _string_ – Account address

#### Result

* `block_id`: _string_ – Account shard last block ID

### fetch\_endpoints

Requests the list of alternative endpoints from server

```ts
type EndpointsSet = {
    endpoints: string[]
}

function fetch_endpoints(): Promise<EndpointsSet>;
```

#### Result

* `endpoints`: _string\[]_ – List of endpoints provided by server

### set\_endpoints

Sets the list of endpoints to use on reinit

```ts
type EndpointsSet = {
    endpoints: string[]
}

function set_endpoints(
    params: EndpointsSet,
): Promise<void>;
```

#### Parameters

* `endpoints`: _string\[]_ – List of endpoints provided by server

### get\_endpoints

Requests the list of alternative endpoints from server

```ts
type ResultOfGetEndpoints = {
    query: string,
    endpoints: string[]
}

function get_endpoints(): Promise<ResultOfGetEndpoints>;
```

#### Result

* `query`: _string_ – Current query endpoint
* `endpoints`: _string\[]_ – List of all endpoints used by client

### query\_counterparties

Allows to query and paginate through the list of accounts that the specified account has interacted with, sorted by the time of the last internal message between accounts

_Attention_ this query retrieves data from 'Counterparties' service which is not supported in the opensource version of DApp Server (and will not be supported) as well as in Evernode SE (will be supported in SE in future), but is always accessible via [EVER OS Clouds](../ever-os-api/networks.md)

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

#### Parameters

* `account`: _string_ – Account address
* `result`: _string_ – Projection (result) string
* `first`?: _number_ – Number of counterparties to return
* `after`?: _string_ – `cursor` field of the last received result

#### Result

* `result`: _any\[]_ – Objects that match the provided criteria

### query\_transaction\_tree

Returns a tree of transactions triggered by a specific message.

Performs recursive retrieval of a transactions tree produced by a specific message: in\_msg -> dst\_transaction -> out\_messages -> dst\_transaction -> ... If the chain of transactions execution is in progress while the function is running, it will wait for the next transactions to appear until the full tree or more than 50 transactions are received.

All the retrieved messages and transactions are included into `result.messages` and `result.transactions` respectively.

Function reads transactions layer by layer, by pages of 20 transactions.

The retrieval prosess goes like this: Let's assume we have an infinite chain of transactions and each transaction generates 5 messages.

1. Retrieve 1st message (input parameter) and corresponding transaction - put it into result. It is the first level of the tree of transactions - its root. Retrieve 5 out message ids from the transaction for next steps.
2. Retrieve 5 messages and corresponding transactions on the 2nd layer. Put them into result. Retrieve 5\*5 out message ids from these transactions for next steps
3. Retrieve 20 (size of the page) messages and transactions (3rd layer) and 20\*5=100 message ids (4th layer).
4. Retrieve the last 5 messages and 5 transactions on the 3rd layer + 15 messages and transactions (of 100) from the 4th layer

* 25 message ids of the 4th layer + 75 message ids of the 5th layer.

1. Retrieve 20 more messages and 20 more transactions of the 4th layer + 100 more message ids of the 5th layer.
2. Now we have 1+5+20+20+20 = 66 transactions, which is more than 50. Function exits with the tree of 1m->1t->5m->5t->25m->25t->35m->35t. If we see any message ids in the last transactions out\_msgs, which don't have corresponding messages in the function result, it means that the full tree was not received and we need to continue iteration.

To summarize, it is guaranteed that each message in `result.messages` has the corresponding transaction in the `result.transactions`. But there is no guarantee that all messages from transactions `out_msgs` are presented in `result.messages`. So the application has to continue retrieval for missing messages if it requires.

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

#### Parameters

* `in_msg`: _string_ – Input message id.
* `abi_registry`?: [_Abi_](mod\_abi.md#abi)_\[]_ – List of contract ABIs that will be used to decode message bodies. Library will try to decode each returned message body using any ABI from the registry.
* `timeout`?: _number_ – Timeout used to limit waiting time for the missing messages and transaction.\
  If some of the following messages and transactions are missing yet\
  The maximum waiting time is regulated by this option.\
  \
  Default value is 60000 (1 min).

#### Result

* `messages`: [_MessageNode_](mod\_net.md#messagenode)_\[]_ – Messages.
* `transactions`: [_TransactionNode_](mod\_net.md#transactionnode)_\[]_ – Transactions.

### create\_block\_iterator

Creates block iterator.

Block iterator uses robust iteration methods that guaranties that every block in the specified range isn't missed or iterated twice.

Iterated range can be reduced with some filters:

* `start_time` – the bottom time range. Only blocks with `gen_utime` more or equal to this value is iterated. If this parameter is omitted then there is no bottom time edge, so all blocks since zero state is iterated.
* `end_time` – the upper time range. Only blocks with `gen_utime` less then this value is iterated. If this parameter is omitted then there is no upper time edge, so iterator never finishes.
* `shard_filter` – workchains and shard prefixes that reduce the set of interesting blocks. Block conforms to the shard filter if it belongs to the filter workchain and the first bits of block's `shard` fields matches to the shard prefix. Only blocks with suitable shard are iterated.

Items iterated is a JSON objects with block data. The minimal set of returned fields is:

```
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
```

Application can request additional fields in the `result` parameter.

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

#### Parameters

* `start_time`?: _number_ – Starting time to iterate from.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` >= `start_time`.\
  Otherwise the iteration starts from zero state.\
  \
  Must be specified in seconds.
* `end_time`?: _number_ – Optional end time to iterate for.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` < `end_time`.\
  Otherwise the iteration never stops.\
  \
  Must be specified in seconds.
* `shard_filter`?: _string\[]_ – Shard prefix filter.\
  If the application specifies this parameter and it is not the empty array\
  then the iteration will include items related to accounts that belongs to\
  the specified shard prefixes.\
  Shard prefix must be represented as a string "workchain:prefix".\
  Where `workchain` is a signed integer and the `prefix` if a hexadecimal\
  representation if the 64-bit unsigned integer with tagged shard prefix.\
  For example: "0:3800000000000000".
* `result`?: _string_ – Projection (result) string.\
  List of the fields that must be returned for iterated items.\
  This field is the same as the `result` parameter of\
  the `query_collection` function.\
  Note that iterated items can contains additional fields that are\
  not requested in the `result`.

#### Result

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

### resume\_block\_iterator

Resumes block iterator.

The iterator stays exactly at the same position where the `resume_state` was catched.

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

#### Parameters

* `resume_state`: _any_ – Iterator state from which to resume.\
  Same as value returned from `iterator_next`.

#### Result

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

### create\_transaction\_iterator

Creates transaction iterator.

Transaction iterator uses robust iteration methods that guaranty that every transaction in the specified range isn't missed or iterated twice.

Iterated range can be reduced with some filters:

* `start_time` – the bottom time range. Only transactions with `now` more or equal to this value are iterated. If this parameter is omitted then there is no bottom time edge, so all the transactions since zero state are iterated.
* `end_time` – the upper time range. Only transactions with `now` less then this value are iterated. If this parameter is omitted then there is no upper time edge, so iterator never finishes.
* `shard_filter` – workchains and shard prefixes that reduce the set of interesting accounts. Account address conforms to the shard filter if it belongs to the filter workchain and the first bits of address match to the shard prefix. Only transactions with suitable account addresses are iterated.
* `accounts_filter` – set of account addresses whose transactions must be iterated. Note that accounts filter can conflict with shard filter so application must combine these filters carefully.

Iterated item is a JSON objects with transaction data. The minimal set of returned fields is:

```
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
```

Application can request an additional fields in the `result` parameter.

Another parameter that affects on the returned fields is the `include_transfers`. When this parameter is `true` the iterator computes and adds `transfer` field containing list of the useful `TransactionTransfer` objects. Each transfer is calculated from the particular message related to the transaction and has the following structure:

* message – source message identifier.
* isBounced – indicates that the transaction is bounced, which means the value will be returned back to the sender.
* isDeposit – indicates that this transfer is the deposit (true) or withdraw (false).
* counterparty – account address of the transfer source or destination depending on `isDeposit`.
* value – amount of nano tokens transferred. The value is represented as a decimal string because the actual value can be more precise than the JSON number can represent. Application must use this string carefully – conversion to number can follow to loose of precision.

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

#### Parameters

* `start_time`?: _number_ – Starting time to iterate from.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` >= `start_time`.\
  Otherwise the iteration starts from zero state.\
  \
  Must be specified in seconds.
* `end_time`?: _number_ – Optional end time to iterate for.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` < `end_time`.\
  Otherwise the iteration never stops.\
  \
  Must be specified in seconds.
* `shard_filter`?: _string\[]_ – Shard prefix filters.\
  If the application specifies this parameter and it is not an empty array\
  then the iteration will include items related to accounts that belongs to\
  the specified shard prefixes.\
  Shard prefix must be represented as a string "workchain:prefix".\
  Where `workchain` is a signed integer and the `prefix` if a hexadecimal\
  representation if the 64-bit unsigned integer with tagged shard prefix.\
  For example: "0:3800000000000000".\
  Account address conforms to the shard filter if\
  it belongs to the filter workchain and the first bits of address match to\
  the shard prefix. Only transactions with suitable account addresses are iterated.
* `accounts_filter`?: _string\[]_ – Account address filter.\
  Application can specify the list of accounts for which\
  it wants to iterate transactions.\
  \
  If this parameter is missing or an empty list then the library iterates\
  transactions for all accounts that pass the shard filter.\
  \
  Note that the library doesn't detect conflicts between the account filter and the shard filter\
  if both are specified.\
  So it is an application responsibility to specify the correct filter combination.
* `result`?: _string_ – Projection (result) string.\
  List of the fields that must be returned for iterated items.\
  This field is the same as the `result` parameter of\
  the `query_collection` function.\
  Note that iterated items can contain additional fields that are\
  not requested in the `result`.
* `include_transfers`?: _boolean_ – Include `transfers` field in iterated transactions.\
  If this parameter is `true` then each transaction contains field\
  `transfers` with list of transfer. See more about this structure in function description.

#### Result

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

### resume\_transaction\_iterator

Resumes transaction iterator.

The iterator stays exactly at the same position where the `resume_state` was caught. Note that `resume_state` doesn't store the account filter. If the application requires to use the same account filter as it was when the iterator was created then the application must pass the account filter again in `accounts_filter` parameter.

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

#### Parameters

* `resume_state`: _any_ – Iterator state from which to resume.\
  Same as value returned from `iterator_next`.
* `accounts_filter`?: _string\[]_ – Account address filter.\
  Application can specify the list of accounts for which\
  it wants to iterate transactions.\
  \
  If this parameter is missing or an empty list then the library iterates\
  transactions for all accounts that passes the shard filter.\
  \
  Note that the library doesn't detect conflicts between the account filter and the shard filter\
  if both are specified.\
  So it is the application's responsibility to specify the correct filter combination.

#### Result

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

### iterator\_next

Returns next available items.

In addition to available items this function returns the `has_more` flag indicating that the iterator isn't reach the end of the iterated range yet.

This function can return the empty list of available items but indicates that there are more items is available. This situation appears when the iterator doesn't reach iterated range but database doesn't contains available items yet.

If application requests resume state in `return_resume_state` parameter then this function returns `resume_state` that can be used later to resume the iteration from the position after returned items.

The structure of the items returned depends on the iterator used. See the description to the appropriated iterator creation function.

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

#### Parameters

* `iterator`: _number_ – Iterator handle
* `limit`?: _number_ – Maximum count of the returned items.\
  If value is missing or is less than 1 the library uses 1.
* `return_resume_state`?: _boolean_ – Indicates that function must return the iterator state that can be used for resuming iteration.

#### Result

* `items`: _any\[]_ – Next available items.\
  Note that `iterator_next` can return an empty items and `has_more` equals to `true`.\
  In this case the application have to continue iteration.\
  Such situation can take place when there is no data yet but\
  the requested `end_time` is not reached.
* `has_more`: _boolean_ – Indicates that there are more available items in iterated range.
* `resume_state`?: _any_ – Optional iterator state that can be used for resuming iteration.\
  This field is returned only if the `return_resume_state` parameter\
  is specified.\
  \
  Note that `resume_state` corresponds to the iteration position\
  after the returned items.

### remove\_iterator

Removes an iterator

Frees all resources allocated in library to serve iterator.

Application always should call the `remove_iterator` when iterator is no longer required.

```ts
type RegisteredIterator = {
    handle: number
}

function remove_iterator(
    params: RegisteredIterator,
): Promise<void>;
```

#### Parameters

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

## Types

### NetErrorCode

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

* `QueryFailed = 601`
* `SubscribeFailed = 602`
* `WaitForFailed = 603`
* `GetSubscriptionResultFailed = 604`
* `InvalidServerResponse = 605`
* `ClockOutOfSync = 606`
* `WaitForTimeout = 607`
* `GraphqlError = 608`
* `NetworkModuleSuspended = 609`
* `WebsocketDisconnected = 610`
* `NotSupported = 611`
* `NoEndpointsProvided = 612`
* `GraphqlWebsocketInitError = 613`
* `NetworkModuleResumed = 614`

### OrderBy

```ts
type OrderBy = {
    path: string,
    direction: SortDirection
}
```

* `path`: _string_
* `direction`: [_SortDirection_](mod\_net.md#sortdirection)

### SortDirection

```ts
enum SortDirection {
    ASC = "ASC",
    DESC = "DESC"
}
```

One of the following value:

* `ASC = "ASC"`
* `DESC = "DESC"`

### ParamsOfQueryOperation

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

Depends on value of the `type` field.

When _type_ is _'QueryCollection'_

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `order`?: [_OrderBy_](mod\_net.md#orderby)_\[]_ – Sorting order
* `limit`?: _number_ – Number of documents to return

When _type_ is _'WaitForCollection'_

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `timeout`?: _number_ – Query timeout

When _type_ is _'AggregateCollection'_

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `fields`?: [_FieldAggregation_](mod\_net.md#fieldaggregation)_\[]_ – Projection (result) string

When _type_ is _'QueryCounterparties'_

* `account`: _string_ – Account address
* `result`: _string_ – Projection (result) string
* `first`?: _number_ – Number of counterparties to return
* `after`?: _string_ – `cursor` field of the last received result

Variant constructors:

```ts
function paramsOfQueryOperationQueryCollection(params: ParamsOfQueryCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationWaitForCollection(params: ParamsOfWaitForCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationAggregateCollection(params: ParamsOfAggregateCollection): ParamsOfQueryOperation;
function paramsOfQueryOperationQueryCounterparties(params: ParamsOfQueryCounterparties): ParamsOfQueryOperation;
```

### FieldAggregation

```ts
type FieldAggregation = {
    field: string,
    fn: AggregationFn
}
```

* `field`: _string_ – Dot separated path to the field
* `fn`: [_AggregationFn_](mod\_net.md#aggregationfn) – Aggregation function that must be applied to field values

### AggregationFn

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

* `COUNT = "COUNT"` – Returns count of filtered record
* `MIN = "MIN"` – Returns the minimal value for a field in filtered records
* `MAX = "MAX"` – Returns the maximal value for a field in filtered records
* `SUM = "SUM"` – Returns a sum of values for a field in filtered records
* `AVERAGE = "AVERAGE"` – Returns an average value for a field in filtered records

### TransactionNode

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

* `id`: _string_ – Transaction id.
* `in_msg`: _string_ – In message id.
* `out_msgs`: _string\[]_ – Out message ids.
* `account_addr`: _string_ – Account address.
* `total_fees`: _string_ – Transactions total fees.
* `aborted`: _boolean_ – Aborted flag.
* `exit_code`?: _number_ – Compute phase exit code.

### MessageNode

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

* `id`: _string_ – Message id.
* `src_transaction_id`?: _string_ – Source transaction id.\
  This field is missing for an external inbound messages.
* `dst_transaction_id`?: _string_ – Destination transaction id.\
  This field is missing for an external outbound messages.
* `src`?: _string_ – Source address.
* `dst`?: _string_ – Destination address.
* `value`?: _string_ – Transferred tokens value.
* `bounce`: _boolean_ – Bounce flag.
* `decoded_body`?: [_DecodedMessageBody_](mod\_abi.md#decodedmessagebody) – Decoded body.\
  Library tries to decode message body using provided `params.abi_registry`.\
  This field will be missing if none of the provided abi can be used to decode.

### ParamsOfQuery

```ts
type ParamsOfQuery = {
    query: string,
    variables?: any
}
```

* `query`: _string_ – GraphQL query text.
* `variables`?: _any_ – Variables used in query.\
  Must be a map with named values that can be used in query.

### ResultOfQuery

```ts
type ResultOfQuery = {
    result: any
}
```

* `result`: _any_ – Result provided by DAppServer.

### ParamsOfBatchQuery

```ts
type ParamsOfBatchQuery = {
    operations: ParamsOfQueryOperation[]
}
```

* `operations`: [_ParamsOfQueryOperation_](mod\_net.md#paramsofqueryoperation)_\[]_ – List of query operations that must be performed per single fetch.

### ResultOfBatchQuery

```ts
type ResultOfBatchQuery = {
    results: any[]
}
```

* `results`: _any\[]_ – Result values for batched queries.\
  Returns an array of values. Each value corresponds to `queries` item.

### ParamsOfQueryCollection

```ts
type ParamsOfQueryCollection = {
    collection: string,
    filter?: any,
    result: string,
    order?: OrderBy[],
    limit?: number
}
```

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `order`?: [_OrderBy_](mod\_net.md#orderby)_\[]_ – Sorting order
* `limit`?: _number_ – Number of documents to return

### ResultOfQueryCollection

```ts
type ResultOfQueryCollection = {
    result: any[]
}
```

* `result`: _any\[]_ – Objects that match the provided criteria

### ParamsOfAggregateCollection

```ts
type ParamsOfAggregateCollection = {
    collection: string,
    filter?: any,
    fields?: FieldAggregation[]
}
```

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `fields`?: [_FieldAggregation_](mod\_net.md#fieldaggregation)_\[]_ – Projection (result) string

### ResultOfAggregateCollection

```ts
type ResultOfAggregateCollection = {
    values: any
}
```

* `values`: _any_ – Values for requested fields.\
  Returns an array of strings. Each string refers to the corresponding `fields` item.\
  Numeric value is returned as a decimal string representations.

### ParamsOfWaitForCollection

```ts
type ParamsOfWaitForCollection = {
    collection: string,
    filter?: any,
    result: string,
    timeout?: number
}
```

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string
* `timeout`?: _number_ – Query timeout

### ResultOfWaitForCollection

```ts
type ResultOfWaitForCollection = {
    result: any
}
```

* `result`: _any_ – First found object that matches the provided criteria

### ResultOfSubscribeCollection

```ts
type ResultOfSubscribeCollection = {
    handle: number
}
```

* `handle`: _number_ – Subscription handle.\
  Must be closed with `unsubscribe`

### ParamsOfSubscribeCollection

```ts
type ParamsOfSubscribeCollection = {
    collection: string,
    filter?: any,
    result: string
}
```

* `collection`: _string_ – Collection name (accounts, blocks, transactions, messages, block\_signatures)
* `filter`?: _any_ – Collection filter
* `result`: _string_ – Projection (result) string

### ParamsOfSubscribe

```ts
type ParamsOfSubscribe = {
    subscription: string,
    variables?: any
}
```

* `subscription`: _string_ – GraphQL subscription text.
* `variables`?: _any_ – Variables used in subscription.\
  Must be a map with named values that can be used in query.

### ParamsOfFindLastShardBlock

```ts
type ParamsOfFindLastShardBlock = {
    address: string
}
```

* `address`: _string_ – Account address

### ResultOfFindLastShardBlock

```ts
type ResultOfFindLastShardBlock = {
    block_id: string
}
```

* `block_id`: _string_ – Account shard last block ID

### EndpointsSet

```ts
type EndpointsSet = {
    endpoints: string[]
}
```

* `endpoints`: _string\[]_ – List of endpoints provided by server

### ResultOfGetEndpoints

```ts
type ResultOfGetEndpoints = {
    query: string,
    endpoints: string[]
}
```

* `query`: _string_ – Current query endpoint
* `endpoints`: _string\[]_ – List of all endpoints used by client

### ParamsOfQueryCounterparties

```ts
type ParamsOfQueryCounterparties = {
    account: string,
    result: string,
    first?: number,
    after?: string
}
```

* `account`: _string_ – Account address
* `result`: _string_ – Projection (result) string
* `first`?: _number_ – Number of counterparties to return
* `after`?: _string_ – `cursor` field of the last received result

### ParamsOfQueryTransactionTree

```ts
type ParamsOfQueryTransactionTree = {
    in_msg: string,
    abi_registry?: Abi[],
    timeout?: number
}
```

* `in_msg`: _string_ – Input message id.
* `abi_registry`?: [_Abi_](mod\_abi.md#abi)_\[]_ – List of contract ABIs that will be used to decode message bodies. Library will try to decode each returned message body using any ABI from the registry.
* `timeout`?: _number_ – Timeout used to limit waiting time for the missing messages and transaction.\
  If some of the following messages and transactions are missing yet\
  The maximum waiting time is regulated by this option.\
  \
  Default value is 60000 (1 min).

### ResultOfQueryTransactionTree

```ts
type ResultOfQueryTransactionTree = {
    messages: MessageNode[],
    transactions: TransactionNode[]
}
```

* `messages`: [_MessageNode_](mod\_net.md#messagenode)_\[]_ – Messages.
* `transactions`: [_TransactionNode_](mod\_net.md#transactionnode)_\[]_ – Transactions.

### ParamsOfCreateBlockIterator

```ts
type ParamsOfCreateBlockIterator = {
    start_time?: number,
    end_time?: number,
    shard_filter?: string[],
    result?: string
}
```

* `start_time`?: _number_ – Starting time to iterate from.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` >= `start_time`.\
  Otherwise the iteration starts from zero state.\
  \
  Must be specified in seconds.
* `end_time`?: _number_ – Optional end time to iterate for.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` < `end_time`.\
  Otherwise the iteration never stops.\
  \
  Must be specified in seconds.
* `shard_filter`?: _string\[]_ – Shard prefix filter.\
  If the application specifies this parameter and it is not the empty array\
  then the iteration will include items related to accounts that belongs to\
  the specified shard prefixes.\
  Shard prefix must be represented as a string "workchain:prefix".\
  Where `workchain` is a signed integer and the `prefix` if a hexadecimal\
  representation if the 64-bit unsigned integer with tagged shard prefix.\
  For example: "0:3800000000000000".
* `result`?: _string_ – Projection (result) string.\
  List of the fields that must be returned for iterated items.\
  This field is the same as the `result` parameter of\
  the `query_collection` function.\
  Note that iterated items can contains additional fields that are\
  not requested in the `result`.

### RegisteredIterator

```ts
type RegisteredIterator = {
    handle: number
}
```

* `handle`: _number_ – Iterator handle.\
  Must be removed using `remove_iterator`\
  when it is no more needed for the application.

### ParamsOfResumeBlockIterator

```ts
type ParamsOfResumeBlockIterator = {
    resume_state: any
}
```

* `resume_state`: _any_ – Iterator state from which to resume.\
  Same as value returned from `iterator_next`.

### ParamsOfCreateTransactionIterator

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

* `start_time`?: _number_ – Starting time to iterate from.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` >= `start_time`.\
  Otherwise the iteration starts from zero state.\
  \
  Must be specified in seconds.
* `end_time`?: _number_ – Optional end time to iterate for.\
  If the application specifies this parameter then the iteration\
  includes blocks with `gen_utime` < `end_time`.\
  Otherwise the iteration never stops.\
  \
  Must be specified in seconds.
* `shard_filter`?: _string\[]_ – Shard prefix filters.\
  If the application specifies this parameter and it is not an empty array\
  then the iteration will include items related to accounts that belongs to\
  the specified shard prefixes.\
  Shard prefix must be represented as a string "workchain:prefix".\
  Where `workchain` is a signed integer and the `prefix` if a hexadecimal\
  representation if the 64-bit unsigned integer with tagged shard prefix.\
  For example: "0:3800000000000000".\
  Account address conforms to the shard filter if\
  it belongs to the filter workchain and the first bits of address match to\
  the shard prefix. Only transactions with suitable account addresses are iterated.
* `accounts_filter`?: _string\[]_ – Account address filter.\
  Application can specify the list of accounts for which\
  it wants to iterate transactions.\
  \
  If this parameter is missing or an empty list then the library iterates\
  transactions for all accounts that pass the shard filter.\
  \
  Note that the library doesn't detect conflicts between the account filter and the shard filter\
  if both are specified.\
  So it is an application responsibility to specify the correct filter combination.
* `result`?: _string_ – Projection (result) string.\
  List of the fields that must be returned for iterated items.\
  This field is the same as the `result` parameter of\
  the `query_collection` function.\
  Note that iterated items can contain additional fields that are\
  not requested in the `result`.
* `include_transfers`?: _boolean_ – Include `transfers` field in iterated transactions.\
  If this parameter is `true` then each transaction contains field\
  `transfers` with list of transfer. See more about this structure in function description.

### ParamsOfResumeTransactionIterator

```ts
type ParamsOfResumeTransactionIterator = {
    resume_state: any,
    accounts_filter?: string[]
}
```

* `resume_state`: _any_ – Iterator state from which to resume.\
  Same as value returned from `iterator_next`.
* `accounts_filter`?: _string\[]_ – Account address filter.\
  Application can specify the list of accounts for which\
  it wants to iterate transactions.\
  \
  If this parameter is missing or an empty list then the library iterates\
  transactions for all accounts that passes the shard filter.\
  \
  Note that the library doesn't detect conflicts between the account filter and the shard filter\
  if both are specified.\
  So it is the application's responsibility to specify the correct filter combination.

### ParamsOfIteratorNext

```ts
type ParamsOfIteratorNext = {
    iterator: number,
    limit?: number,
    return_resume_state?: boolean
}
```

* `iterator`: _number_ – Iterator handle
* `limit`?: _number_ – Maximum count of the returned items.\
  If value is missing or is less than 1 the library uses 1.
* `return_resume_state`?: _boolean_ – Indicates that function must return the iterator state that can be used for resuming iteration.

### ResultOfIteratorNext

```ts
type ResultOfIteratorNext = {
    items: any[],
    has_more: boolean,
    resume_state?: any
}
```

* `items`: _any\[]_ – Next available items.\
  Note that `iterator_next` can return an empty items and `has_more` equals to `true`.\
  In this case the application have to continue iteration.\
  Such situation can take place when there is no data yet but\
  the requested `end_time` is not reached.
* `has_more`: _boolean_ – Indicates that there are more available items in iterated range.
* `resume_state`?: _any_ – Optional iterator state that can be used for resuming iteration.\
  This field is returned only if the `return_resume_state` parameter\
  is specified.\
  \
  Note that `resume_state` corresponds to the iteration position\
  after the returned items.

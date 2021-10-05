# Schema

A schema defines a type system of GraphQL API. It describes the complete set of possible data \(objects, fields, relationships, everything\) that a client can access.

* [Root types](3_schema.md#root-types)
* [Non-root types](3_schema.md#non-root-types)
* [Query types](3_schema.md#query-types)
* [Subscription types](3_schema.md#subscription-types)
* [Mutation types](3_schema.md#mutation-types)
* [Syntax](3_schema.md#syntax)

## Root types

TON Labs GraphQL schema has three root types:

* query
* mutation
* subscription

The [query type](https://graphql.github.io/graphql-spec/June2018/#sec-Type-System) defines GraphQL operations that retrieve data from the server.

The [mutation type](https://graphql.github.io/graphql-spec/June2018/#sec-Type-System) defines GraphQL operations that change data on the server. It is analogous to performing HTTP verbs such as `POST`, `PATCH`, and `DELETE`. Mutations are used to send messages to the blockchain. We recommend to do it only via SDK, not directly.

The **subscription** root type – a long‐lived request that fetches data in response to source events.

Check out [TON Labs SDK query module](../mod_net.md) - the official TON Labs wrapper over GraphQL API for root queries and subscriptions.

## Non-root types

See non-root type descriptions in [Field descriptions](5_field_descriptions.md) section.

## Query types

**Collection type queries:**

| name | description |
| :--- | :--- |
| [blocks](5_field_descriptions.md#block-type) | Query blocks data. Blocks include masterchain and shardchain blocks. |
| [accounts](5_field_descriptions.md#account-type) | Query the latest account data: includes such information as address, balance, code, data, etc. |
| [messages](5_field_descriptions.md#message-type) | Query messages data. |
| [transaction](5_field_descriptions.md#transaction-type) | Query transactions data. |
| blocks\_signatures | Query data about block signatures. |

**Aggregation queries:**

| name | description |
| :--- | :--- |
| aggregateBlocks | Get aggregation info about blocks: COUNT, SUM, MAX, MIN, AVERAGE functions over blocks data. |
| aggregateTransactions | Get aggregation info about transactions: COUNT, SUM, MAX, MIN, AVERAGE functions over transactions data. |
| aggregateMessages | Get aggregation info about messages: COUNT, SUM, MAX, MIN, AVERAGE functions over messages data. |
| aggregateAccounts | Get aggregation info about accounts: COUNT, SUM, MAX, MIN, AVERAGE functions over accounts data. |
| aggregateBlockSignatures | Get aggregation info about block signaturess: COUNT, SUM, MAX, MIN, AVERAGE functions over block signatures data. |

**Other queries**

* getAccountsCount - number of accounts. **will be deprecated soon**. 
* getTransactionsCount - number of transactions. **will be deprecated soon**. 
* getAccountsTotalBalance - total balance of accounts. **will be deprecated soon**. 
* info - get information about the active GraphQL API version.

## Subscription types

* blocks
* accounts
* messages
* transaction 
* blocks\_signatures

## Mutation types

* postRequests - used to send messages in SDK libraries. 

## Syntax

Below you can see an example of a query for 5 random accounts filtered by balance range from 50 to 100 coins \(in hex\) and ordered in descending direction.

```text
query {
  accounts(
    filter:{
      balance:{
        gt: "0xBA43B7400" 
        lt: "0x174876E800"
      }
    }
    limit: 5
    orderBy:{
      path:"balance"
      direction:DESC
    }
  ) {
    id,
    last_trans_lt,
    last_paid
    balance
  }
}
```

Here you can see a request for accounts with the fields

`id`, `last_trans_lt` , `last_paid` and `balance`

forming a selection set \(also called a 'projection'\).

`Filter`, `Orderby` and `Limit` arguments are used.

A selection set must contain only scalar fields, otherwise you will get an error. A scalar field describes one discrete piece of information available to a request within a selection set.

Read more in the next sections.


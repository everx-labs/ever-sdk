# Schema

A schema defines a type system of GraphQL API. It describes the complete set of possible data (objects, fields, relationships, everything) that a client can access.

* [Root types](schema.md#root-types)
* [Non-root types](schema.md#non-root-types)
* [Query types](schema.md#query-types)
* [Subscription types](schema.md#subscription-types)
* [Mutation types](schema.md#mutation-types)
* [Syntax](schema.md#syntax)

## Root types

TON Labs GraphQL schema has three root types:

* query
* mutation
* subscription

The [query type](https://graphql.github.io/graphql-spec/June2018/#sec-Type-System) defines GraphQL operations that retrieve data from the server.

The [mutation type](https://graphql.github.io/graphql-spec/June2018/#sec-Type-System) defines GraphQL operations that change data on the server. It is analogous to performing HTTP verbs such as `POST`, `PATCH`, and `DELETE`. Mutations are used to send messages to the blockchain. We recommend to do it only via SDK, not directly.

The **subscription** root type – a long‐lived request that fetches data in response to source events.

Check out [TON Labs SDK net module](../types-and-methods/mod\_net.md) - the official TON Labs wrapper over GraphQL API for root queries and subscriptions.

## Non-root types

See non-root type descriptions in [Field descriptions](field\_descriptions.md) section.

## Query types

**Collection type queries:**

| name                                                   | description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| blockchain                                             | New API that includes a set of functions for pagination of `blocks`, `key_blocks`, `transactions` and account’s transactions via blockchain-based cursor that stays the same for all the endpoints, compared to an approach with an artificial database cursor - like timestamp or sequential index - that may vary from instance to instance. May be useful for Integrators and DApps who needs to sequentially read all blocks or transactions from API, due to inefficiency of simple collection pagination by timestamps or `seq_no` in multithreaded Everscale environment, also such simple pagination may not work when there are too many objects with the same timestamp. |
| [blocks](field\_descriptions.md#block-type)            | Query blocks data. Blocks include masterchain and shardchain blocks.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| [accounts](field\_descriptions.md#account-type)        | Query the latest account data: includes such information as address, balance, code, data, etc.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| [messages](field\_descriptions.md#message-type)        | Query messages data.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| [transaction](field\_descriptions.md#transaction-type) | Query transactions data.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| blocks\_signatures                                     | Query data about block signatures.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| statistics                                             | <p>General Everscale Network statistics related to accounts, transactions, messages and blocks. And also some essential statistics about validators and depools.<br><br>Available only in Cloud API.</p>                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| counterparties                                         | <p>Returns a list of addresses the specified account interacted with, sorted by the latest interaction time (the latest message time between 2 accounts) DESC. Feature may be useful for wallet applications or for chat-based DApps to show the list of counterparties in descending order.<br><br>Available only in Cloud API.</p>                                                                                                                                                                                                                                                                                                                                               |

**Aggregation queries:**

| name                     | description                                                                                                       |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| aggregateBlocks          | Get aggregation info about blocks: COUNT, SUM, MAX, MIN, AVERAGE functions over blocks data.                      |
| aggregateTransactions    | Get aggregation info about transactions: COUNT, SUM, MAX, MIN, AVERAGE functions over transactions data.          |
| aggregateMessages        | Get aggregation info about messages: COUNT, SUM, MAX, MIN, AVERAGE functions over messages data.                  |
| aggregateAccounts        | Get aggregation info about accounts: COUNT, SUM, MAX, MIN, AVERAGE functions over accounts data.                  |
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
* counterparties

## Mutation types

* postRequests - used to send messages to blockchain.

## Syntax

Read about GraphQL syntax in its [`official documentation`](https://graphql.org). In this example we query account info:

```graphql
query {
  blockchain{
   account(address:"0:653b9a6452c7a982c6dc92b2da9eba832ade1c467699ebb3b43dca6d77b780dd"){
    info{
      address
      acc_type
      balance
      last_paid
      last_trans_lt
      boc
      data
      code
      library
      data_hash
      code_hash
      library_hash
    }
  }
  }
}
```

Here you can see a request for account's  fields `address`, `acc_type`, etc, forming a selection set (also called a 'projection').

A selection set must contain only scalar fields, otherwise you will get an error. A scalar field describes one discrete piece of information available to a request within a selection set. If field is an object, you need to specify the fields of this object.

Read more in the next sections.

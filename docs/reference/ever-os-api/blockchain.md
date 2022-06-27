---
description: Learn about basic blockchain API
---

# Blockchain

`blockchain` root type is API that includes such basic real-time data as:

* [blocks](../../graphql-samples/blocks.md)
* [key blocks](../../graphql-samples/blocks.md#key-blocks-pagination)
* [transactions](../../graphql-samples/transactions.md)
* [account](../../graphql-samples/accounts.md)&#x20;
  * [account info](../../graphql-samples/accounts.md#get-account-info)&#x20;
  * [account transactions](../../graphql-samples/accounts.md#pagination-of-account-transactions)
  * [account messages](../../graphql-samples/accounts.md#pagination-of-accounts-messages)
* (coming soon) accounts - allows to paginate accounts by deploy time or init time, filtering by code\_hash

This API is truly a Graph-oriented API .&#x20;

We followed GraphQL best practices and implemented Relay Cursor Connections Specification for pagination. You can read more here [https://relay.dev/graphql/connections.htm](https://relay.dev/graphql/connections.htm)&#x20;

&#x20;&#x20;

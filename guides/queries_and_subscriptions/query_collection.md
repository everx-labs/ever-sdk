# Query Collection

Query blockchain data collections: filter, sort

## What is a collection?

There are a few collections with blockchain data:

* _accounts_: blockchain account data;
* _transactions_: transactions related to accounts;
* _messages_: input and output messages related to transactions;
* _blocks_: blockchain blocks.
* _block_signatures_ : validator block signatures

The JS Client Library contains the Query Module called `net` - a wrapper for [TON Labs GraphQL API](../../docs/ton_os_api/) - to perform GraphQL queries over a blockchain.

[Use `query_collection` method to query data that can be filtered and sorted](../../docs/modules/mod_net.md#query_collection).

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query)

**AppKit**

[https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query](https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query)

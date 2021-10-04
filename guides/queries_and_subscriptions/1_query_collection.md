# Query collection

Query blockchain data collections: filter, sort

# What is a collection?

There are a few collections with blockchain data: 

- *accounts*: blockchain account data;
- *transactions*: transactions related to accounts;
- *messages*: input and output messages related to transactions;
- *blocks*: blockchain blocks.
- *block_signatures* : validator block signatures

The JS Client Library contains the Query Module called `net` - a wrapper for [TON Labs GraphQL API](../../docs/ton_os_api.md) - to perform GraphQL queries over a blockchain. 

[Use `query_collection` method to query data that can be filtered and sorted](../../docs/mod_net.md#query_collection).



# Sample source code

**Core**

https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query

**AppKit**

https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query

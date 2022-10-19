---
description: About GraphQL API
---

# GraphQL API

GraphQL API is shared by all [Evernode Platform](https://docs.everos.dev/evernode-platform) products.

Due to the GraphQL ability to stitch its schemas, GraphQL API is in fact a set of "stitched" APIs behind it. Meanwhile all Evernode Platform products share the same core API that allows to communicate with Everscale network, [each product has its own subset](https://docs.everos.dev/evernode-platform/products/functionality-comparison) of APIs corresponding to the Product use-cases.

## Use-cases

* Send a prepared message to blockchain
* Query account data
  * state
  * messages (including events)
  * transactions
* Query block data
* Query transaction data
* Query message (including events) data
* Get list of
  * all blocks
  * all transaction
* Subscribe to
  * account updates
  * contract events(external outbound messages)
  * new blocks
  * new transactions
  * new messages

Read more in the next sections.

[Quick Start](samples.md#quick-start)

## SDK

GraphQL API goes along with [SDK](https://github.com/tonlabs/ever-sdk) that helps one to create messages, handle network issues and implement any possible use-case over this API.

SDK is supported for 14 programming languages for all the popular platforms.

## More about GraphQL protocol

Read more about GraphQL on the official GraphQL Foundation website [https://graphql.org/](https://graphql.org)

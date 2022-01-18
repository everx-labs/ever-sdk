# Quick Start

Go to net.ton.dev/graphql (or [choose another network GraphQL endpoint](networks.md)) in your browser.

You will see the GraphQL playground.

In this picture we query the GraphQL API version on the left and see the result on the right.

```graphql
query{
  info{
    version
  }
}
```

![scr1.png](../../.gitbook/assets/scr1.png)

Check out [TON Labs SDK net module](../types-and-methods/mod_net.md) - the official TON Labs wrapper over GraphQL API for root queries and subscriptions.

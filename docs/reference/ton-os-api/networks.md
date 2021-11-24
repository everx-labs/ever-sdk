# Networks

Each TON Operating System instance has a single GraphQL API endpoint. Each Cloud has several TON OS instances running for reliability.

## Networks

> **Attention!**
>
> **You need to specify ALL the endpoints in your configuration, not just one from the list. We do not guarantee availability of each endpoint all the time, but we guarantee that at least 1 endpoint is operational at the moment.**

| TON OS                                                        | Description                            | Web Playground URLs                                                                                                                                                                          | Endpoint URLs                                                                                                                                   |
| ------------------------------------------------------------- | -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| TON OS Cloud for Free TON                                     | Access to Free TON network             | [https://main2.ton.dev/graphql](https://main2.ton.dev/graphql) [https://main3.ton.dev/graphql](https://main3.ton.dev/graphql) [https://main4.ton.dev/graphql](https://main4.ton.dev/graphql) | [https://main2.ton.dev/](https://main2.ton.dev) [https://main3.ton.dev/](https://main3.ton.dev) [https://main4.ton.dev/](https://main4.ton.dev) |
| TON OS Cloud for Developer network                            | Access to TON Labs Development Network | [https://net1.ton.dev/graphql](https://net1.ton.dev/graphql) [https://net5.ton.dev/graphql](https://net5.ton.dev/graphql)                                                                    | [https://net1.ton.dev/](https://net1.ton.dev) [https://net5.ton.dev/](https://net5.ton.dev)                                                     |
| [TON OS Startup Edition](https://github.com/tonlabs/tonos-se) | Access to TON OS SE for local testing  | [http://0.0.0.0/graphql](http://0.0.0.0/graphql) (For Windows, use [http://127.0.0.1/graphql](http://127.0.0.1/graphql) or [http://localhost/graphql](http://localhost/graphql))             | [http://0.0.0.0/](http://0.0.0.0) [http://127.0.0.1/](http://127.0.0.1) [http://localhost/](http://localhost)                                   |

## Connect your application to TON OS

Find out how to [connect your JS application to TON OS](../../guides/installation/configure_sdk.md).

## Connect TONOS-CLI to TON OS

Find out how to [connect TONOS-CLI to TON OS](https://github.com/tonlabs/tonos-cli#21-set-the-network-and-parameter-values).

## Other Clients

If you use another language check the official GraphQL documentation how to connect:

* via [other GraphQL Clients](https://graphql.org/code/)
* via [HTTP requests](https://graphql.org/learn/serving-over-http/)

In the next section find out how to work with GraphQL Web playground and easily explore blockchain data with it.

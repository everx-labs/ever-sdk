# Networks

Each Everscale Operating System (EVER OS) instance has a single GraphQL API endpoint. Each Cloud has several EVER OS instances running for reliability.

## Networks

> **Attention!**
>
> **You need to specify ALL the endpoints in your configuration, not just one from the list. We do not guarantee availability of each endpoint all the time, but we guarantee that at least 1 endpoint is operational at the moment.**

| EVER OS                                                        | Description                            | Web Playground URLs                                                                                                                                                                                                                                                                                                                                                                                                            | Endpoint URLs                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| EVER OS Cloud for Everscale                                    | Access to Everscale network            | [https://eri01.main.everos.dev/graphql](https://eri01.main.everos.dev/graphql) <br> [https://gra01.main.everos.dev/graphql](https://gra01.main.everos.dev/graphql) <br> [https://gra02.main.everos.dev/graphql](https://gra02.main.everos.dev/graphql) <br> [https://lim01.main.everos.dev/graphql](https://lim01.main.everos.dev/graphql) <br> [https://rbx01.main.everos.dev/graphql](https://rbx01.main.everos.dev/graphql) | [https://eri01.main.everos.dev](https://eri01.main.everos.dev) <br> [https://gra01.main.everos.dev](https://gra01.main.everos.dev) <br> [https://gra02.main.everos.dev](https://gra02.main.everos.dev) <br> [https://lim01.main.everos.dev](https://lim01.main.everos.dev) <br> [https://rbx01.main.everos.dev](https://rbx01.main.everos.dev) |
| EVER OS Cloud for Developer network                            | Access to TON Labs Development Network | [https://eri01.net.everos.dev/graphql](https://eri01.net.everos.dev/graphql) <br> [https://rbx01.net.everos.dev/graphql](https://rbx01.net.everos.dev/graphql) <br> [https://gra01.net.everos.dev/graphql](https://gra01.net.everos.dev/graphql)                                                                                                                                                                               | [https://eri01.net.everos.dev](https://eri01.net.everos.dev) <br> [https://rbx01.net.everos.dev](https://rbx01.net.everos.dev) <br> [https://gra01.net.everos.dev](https://gra01.net.everos.dev)                                                                                                                                               |
| [EVER OS Startup Edition](https://github.com/tonlabs/tonos-se) | Access to EVER OS SE for local testing | [http://localhost/graphql](http://localhost/graphql) <br> [http://127.0.0.1/graphql](http://127.0.0.1/graphql) <br> [http://0.0.0.0/graphql](http://0.0.0.0/graphql) (*nix only)                                                                                                                                                                                                                                               | [http://localhost](http://localhost) <br> [http://127.0.0.1](http://127.0.0.1) <br> [http://0.0.0.0](http://0.0.0.0)                                                                                                                                                                                                                           |

## Connect your application to EVER OS

Find out how to [connect your JS application to EVER OS](../../guides/installation/configure_sdk.md).

## Connect TONOS-CLI to EVER OS

Find out how to [connect TONOS-CLI to EVER OS](https://github.com/tonlabs/tonos-cli#21-set-the-network-and-parameter-values).

## Other Clients

If you use another language check the official GraphQL documentation how to connect:

* via [other GraphQL Clients](https://graphql.org/code/)
* via [HTTP requests](https://graphql.org/learn/serving-over-http/)

In the next section find out how to work with GraphQL Web playground and easily explore blockchain data with it.

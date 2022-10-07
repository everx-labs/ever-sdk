---
description: Get TIP3 tokens info, balances, transfers
---

# FT (Fungible Token) API

``![](<../../.gitbook/assets/image (10).png>)``

`ft` root type is API that provides data about fungible tokens, transfers, holders.

Now API indexes only TIP3.2 standard.&#x20;

* version - API version
* [tokens](../../samples/graphql-samples/ft-fungible-token-api.md#list-of-tokens) - returns the list of tokens
* [token](../../samples/graphql-samples/ft-fungible-token-api.md#token-info) - returns information about the token
* [holder](../../samples/graphql-samples/ft-fungible-token-api.md#holder-info) - returns information about token holder: her wallets, transfers
* [wallet](../../samples/graphql-samples/ft-fungible-token-api.md#wallet-info) - returns information about a particular wallet: its balance, owner, transfers, balance percentage(related to the total token supply)
* holdersByAddresses - returns an array of holders by their addresses
* transfersByMessageIds - returns an array of transfers by their message ids
* [account.holderInfo](../../samples/graphql-samples/ft-fungible-token-api.md#account)
* [transaction.transferInfo](../../samples/graphql-samples/ft-fungible-token-api.md#transaction)
* [message.transferInfo](../../samples/graphql-samples/ft-fungible-token-api.md#message)

We followed GraphQL best practices and implemented Relay Cursor Connections Specification for pagination for all list types. You can read more here [https://relay.dev/graphql/connections.htm](https://relay.dev/graphql/connections.htm)&#x20;

---
description: Get TIP3 tokens info, balances, transfers
---

# FT (Fungible Token) API

``![](<../../.gitbook/assets/image (10).png>)``

`ft` root type is API that provides data about fungible tokens, transfers, holders.

Now API indexes only TIP3.2 standard.&#x20;

* version - API version
* tokens - returns the list of tokens
* token - returns information about the token
* holder - returns information about token holder: her wallets, transfers
* wallet - returns information about a particular wallet: its balance, owner, transfers, balance percentage(related to the total token supply)
* holdersByAddresses - returns an array of holders by their addresses
* transfersByMessageIds - returns an array of transfers by their message ids

We followed GraphQL best practices and implemented Relay Cursor Connections Specification for pagination for all list types. You can read more here [https://relay.dev/graphql/connections.htm](https://relay.dev/graphql/connections.htm)&#x20;

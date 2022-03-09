# Accounts

## Get account info

To get account info use the following GraphQL query:

```graphql
query{
  accounts(
  filter:{
        id:{
            eq:"-1:3333333333333333333333333333333333333333333333333333333333333333"
      }
  }){
    acc_type
    last_paid
    due_payment
    last_trans_lt
    balance
    data_hash
    code_hash
    library_hash
    boc    
  }
}
```

Result:

```graphql
{
  "data": {
    "accounts": [
      {
        "acc_type": 1,
        "last_paid": 0,
        "due_payment": null,
        "last_trans_lt": "0x4905d5a4a03",
        "balance": "0x906201ebb43418",
        "data_hash": "3a09512d6a5c469d1ea182d080761a01bac0897bc67a4cf510a1911020b96104",
        "code_hash": "e48892fa8be43954a2923d668ff9e8d68931c82d8dc80be1c8848b8ae8fe366a",
        "library_hash": null,
        "boc": "<...>"
      }
    ]
  }
}
```

where `id` (full address) consists of workchainID:address (Note: smart contract and an account are the same thing in the context of the Everscale Blockchain. A large smart-contract may employ several accounts lying in different shardchains of the same workchain for load balancing purposes.)

fields:

* acc\_type
  * 0 – uninit (Account has balance but no code)
  * 1 – active (Account has balance and code)
  * 2 – frozen(Account has been frozen for some reasons)
* last\_paid - unixtime of the most recent storage payment or
* balance - tokens on account (Note: to deploy smart contract code you need to have non-zero balance)
* last\_trans\_lt - logical time of last account transaction
* data\_hash - data field hash
* code\_hash - code field hash
* library - If present, contains library code used in smart-contract.
* library\_hash - library field hash
* boc - Bag of cells with the account struct encoded as base64.

## Pagination of account transactions

<mark style="color:orange;">**Attention! Pagination with cursor functionality is new and not yet supported in Evernode-DS. But will be soon!**</mark>&#x20;

If you want to paginate all account transactions from the very first one, use this query

```graphql
query{
  account_transactions(
    account_address:"0:27da7884e032ede0f7d5758bb58bcab9942dfb6c1b764a38bb6377a47a0822de"
  ){
    edges{
      node{
        id
        lt
        now
      }
      cursor
    }
    pageInfo{
      endCursor
      hasNextPage
    }
  }
}
```

Use `endCursor` field for further pagination and `hasNextCursor` for identifying if more records exist.&#x20;

If you want to paginate within some time range, you can use masterchain seq\_no or time range filter. Also you can use additional handy pagination parameters such as `after`, `first`, `before`, `last`. Read more about it in [blocks pagination section](blocks.md#about-cursor).

## Get the list of account's counterparties

Returns the paginable list of accounts the account has ever interacted with, with the last message info attached, sorted by the last message time. Useful for applications that want to show a screen with dialog list sorted by the last interaction time.

<mark style="color:orange;">**Attention! Available only in public API. Is not available in Evernode-DS**</mark>**.**[ **See functionality comparison section.** ](https://tonlabs.gitbook.io/evernode-platform/products/functionality-comparison)****

```graphql
query{
  counterparties(
    account:"0:27da7884e032ede0f7d5758bb58bcab9942dfb6c1b764a38bb6377a47a0822de"
  ){
    last_message_id
    last_message_value
    last_message_at
    last_message_is_reverse
  }
}
```



## Transaction count

To get count of account transactions use `aggregateTransactions`:

**Query**:

```graphql
query {
  aggregateTransactions(
    filter:{
      account_addr : {
        eq: "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"
      }
    },
    fields:[
      { fn:COUNT }
    ]
  )
}
```

**Result**:

```graphql
{
  "data": {
    "aggregateTransactions": [
      "10071"
    ]
  }
}
```

## Get all messages of a specified account

<mark style="color:orange;">**Attention! At the moment we are developing a handy method for account messages pagination. Please use the approach below as temporary, it will be deprecated soon.**</mark>&#x20;

By default query result is limited to 50 records. To get next 50 records, you need to use created\_lt (creation logical time) of the last record in the next query.

In the next example we limit the number of results returned to 2.

**Query**:

```graphql
{
  messages(
      filter:{ 
        src:{eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
        created_lt:{gt:"0x0"}
        OR:{
            dst:{eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
            created_lt:{gt:"0x0"}
        }
        }
      limit:2
        orderBy:[{path:"created_at",direction:ASC},{path:"created_lt"}]
  )
  {
    id
    src
    dst
    created_at
    created_lt
  }
}
```

**Result**:

```graphql
{
  "data": {
    "messages": [
      {
        "id": "2e80b1b06a8a5340d06627dd3e37f6b2b8436af643c24ff80a2f7840899d8e0e",
        "src": "-1:7777777777777777777777777777777777777777777777777777777777777777",
        "dst": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
        "created_at": 1593681809,
        "created_lt": "0x451307782"
      },
      {
        "id": "2a0d4e79d8f2d22fc6f3226cf1c972202b5ce8cb709a7d5539b3e7a0e90729ea",
        "src": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
        "dst": "-1:7777777777777777777777777777777777777777777777777777777777777777",
        "created_at": 1593681814,
        "created_lt": "0x4514efc02"
      }
    ]
  }
}
```

There are two records returned and that may mean that there is another page. We take last record `created_lt` and repeat query with "greater than created\_lt" condition:

**Query**:

```graphql
{
  messages(
      filter:{ 
            src:{eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
            created_lt:{gt:"0x4514efc02"}
        OR:{
            dst:{eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
            created_lt:{gt:"0x4514efc02"}
        }
      }
      limit:2
        orderBy:[{path:"created_at",direction:ASC},{path:"created_lt"}]
  )
  {
    id
    src
    dst
    created_at
    created_lt
  }
}
```

**Result**:

```graphql
{
  "data": {
    "messages": [
      {
        "id": "b0989c38a1b3be613be6dab1c78072fc915737be771445bb9f718db000727532",
        "src": "-1:7777777777777777777777777777777777777777777777777777777777777777",
        "dst": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
        "created_at": 1593682159,
        "created_lt": "0x458948e82"
      },
      {
        "id": "57df11539055f8bd1f4fddcb794fa6af66aa2e3f1c3daca934c4e6c1d684c5fd",
        "src": "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13",
        "dst": "0:d259a644903ead3ea5d8124d47b3c28a9e1bdebbf576402503798477ace117d0",
        "created_at": 1593682274,
        "created_lt": "0x45ac921c2"
      }
    ]
  }
}
```

To receive count of all account messages (both in and out) use `aggregateMessages`.

**Query**:

```graphql
query{ aggregateMessages( filter:
  { src : { eq: "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
    OR:{dst: {eq: "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}}
  }
, fields:[ { fn:COUNT } ] ) }
```

Result:

```graphql
{
  "data": {
    "aggregateMessages": [
      "24772"
    ]
  }
}
```

## Aggregate transfers between 2 accounts

Determine min, max and sum values of transferred tokens and number of transfers between two accounts

```graphql
query{
  aggregateMessages(
    filter:{
        src:{eq:"0:797f32a15bbe5213a07cafe4c80e5e28f2662e865e95b23694f4bd36f2b42ff8"}
        dst:{eq:"0:7d667fed88b9edb82eb6a116b48052b6a7765577ad341b35acb118451c7aa625"}

          OR:{
        src:{eq:"0:7d667fed88b9edb82eb6a116b48052b6a7765577ad341b35acb118451c7aa625"}
        dst:{eq:"0:797f32a15bbe5213a07cafe4c80e5e28f2662e865e95b23694f4bd36f2b42ff8"}
        }
    }
    fields:[
        { field: "value", fn: MIN},
        { field: "value", fn: MAX },
        { field: "value", fn: SUM },
          { fn: COUNT}
    ]
  )
}
```

Result:

```graphql
{
  "data": {
    "aggregateMessages": [
      "10000000",
      "10000000",
      "30000000",
      "3"
    ]
  }
}
```

## Aggregate gas consumption

Determine min, max and sum value for the gas\_used of a transactions compute phase.

You can use a dot separated path as a field name to use fields resided deep in a JSON structure of a transaction record:

```graphql
query{
  aggregateTransactions(
    filter:{
     account_addr : 
            {eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" }
    },
    fields:[
      { field: "compute.gas_used", fn:MIN },
      { field: "compute.gas_used", fn:MAX },
      { field: "compute.gas_used", fn:SUM },
    ]
  )
}
```

Result:

```graphql
{
  "data": {
    "aggregateTransactions": [
      "1434",
      "45221",
      "32578614"
    ]
  }
}
```

- [Account queries](#account-queries)
  - [Get account info](#get-account-info)
  - [Pagination of account transactions](#pagination-of-account-transactions)
  - [Get the last message from user's account to a specified destination address](#get-the-last-message-from-users-account-to-a-specified-destination-address)
  - [Get all transactions of a specified account](#get-all-transactions-of-a-specified-account)
  - [Get all messages of a specified account](#get-all-messages-of-a-specified-account)
  - [Get the number of account transactions](#get-the-number-of-account-transactions)
  - [Aggregate transfers between 2 accounts](#aggregate-transfers-between-2-accounts)
  - [Aggregate gas consumption](#aggregate-gas-consumption)

# Account queries

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

* acc_type
  * 0 – uninit (Account has balance but no code)
  * 1 – active (Account has balance and code)
  * 2 – frozen(Account has been frozen for some reasons)
* last_paid - unixtime of the most recent storage payment or
* balance - tokens on account (Note: to deploy smart contract code you need to have non-zero balance)
* last_trans_lt - logical time of last account transaction
* data_hash - data field hash
* code_hash - code field hash
* library - If present, contains library code used in smart-contract.
* library_hash - library field hash
* boc - Bag of cells with the account struct encoded as base64.

In case account not found result will be empty:

```graphql
{
  "data": {
    "accounts": []
  }
}
```

## Pagination of account transactions

To implement account transactions pagination in descending order do the following steps:

1. Get the last account transaction logical time. You will receive it in hexadecimal string format. This is the start point of your pagination range.

```graphql
query{
  accounts(filter:{
    id:{
      eq:"-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af"
    }
  })
  {
    last_trans_lt
  }
}
```

Result:

```graphql
{
  "data": {
    "accounts": [
      {
        "last_trans_lt": "0x448c10cd4c2"
      }
    ]
  }
}
```

1. Now lets fetch the first batch of transactions. We need to filter them by the account, specify the start point `lt` and sort by block time now and logical time in descending order. Limit the result by 25 records.

We use `lt` (less than) operator instead of `le` (less or equal) even for the first step because the `last_trans_lt` field of account is always equal to the (last transaction lt +1).

```graphql
query{
    transactions(
    filter:{
      account_addr:{
        eq:"-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af"
      }    

      lt:{
        lt:"0x448c10cd4c2"
      }
    }
    orderBy:[
            { path:"now",direction:DESC },
              { path:"lt", direction:DESC }
    ]
    limit:25
  ){
    id
    lt
    now
    account_addr
  }
}
```

the result shows us 25 records. These are the last 2 records:

```graphql
{
        "id": "a7c9296961d0c105c060dbb1c7d4a92afbfc6a4606982407ab56a88ebc06cefb",
        "lt": "0x3b5608087c1",
        "now": 1598369415,
        "account_addr": "-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af"
      },
      {
        "id": "7457721794359aec848f9ac26e0759125d6d53def115f702d8c31c955c229d5a",
        "lt": "0x3b55ff73381",
        "now": 1598369395,
        "account_addr": "-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af"
      }
    ]
  }
}
```

1. Take the `lt` of the last retrieved transaction and use it as the start point for the next batch. Make sure to use `lt` operator, not `le`, so that you will not get the same transaction in 2 batches:

```graphql
query{
    transactions(
    filter:{
      account_addr:{
        eq:"-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af"
      }    

      lt:{
        lt:"0x3b55ff73381"
      }
    }
    orderBy:[
            { path:"now",direction:DESC },
              { path:"lt", direction:DESC }
    ]
    limit:25
  ){
    id
    lt
    now
    account_addr
  }
}
```

And so on.

## Get the last message from user's account to a specified destination address

Use the following query:

```graphql
{
  messages( 
    filter: { 
      src:{eq:"-1:7777777777777777777777777777777777777777777777777777777777777777"}
        dst:{eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"}
    }
    orderBy:{ path:"created_at", direction: DESC}
    limit: 1
  ) 
  { 
    id 
    msg_type
    body
    status
  } 
}
```

Result:

```graphql
{
  "data": {
    "messages": [
      {
        "id": "b0989c38a1b3be613be6dab1c78072fc915737be771445bb9f718db000727532",
        "msg_type": 0,
        "body": null,
        "status": 5
      }
    ]
  }
}
```

In the above query:

* src - User's account (source address).
* dst - destination account (destination address).

## Get all transactions of a specified account

By default query result is limited to 50 records. To implement pagination you need to use creation lt (creation logical time) of the last record in the next query.

Please note that we do not sort by `created_at` here because the data for one account is unequivocally sorted by lt (to be precise - all the data inside one shard).

In the next example we limit the number of results returned to 2.

**Query**:

```graphql
{
  transactions(
    filter: { 
          account_addr: { 
              eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"
          }
    }
    orderBy:[
          {path:"lt",direction:ASC}
        ]
    limit:2
  )
  {
    id
    now
    lt
  }
}
```

**Result**:

```graphql
{
  "data": {
    "transactions": [
      {
        "id": "35394c0caae38b34e17d9dbbbfd068a02c65c133583fb516a65ca11431e8b9ff",
        "now": 1593681814,
        "lt": "0x4514efc01"
      },
      {
        "id": "72b89951ebbe819a82fc0c1d3bc3af49096e7aa6f76d8dc3069addc8b993858c",
        "now": 1593682165,
        "lt": "0x458b31301"
      }
    ]
  }
}
```

In the above query:

* account_addr - address of an account to filter by.
* now - time of the transaction.
* lt - Logical time. A component of the Everscale Blockchain that also plays an important role in message delivery is the logical time, usually denoted by Lt. It is a non-negative 64-bit integer, assigned to certain events. For more details, see the Everscale blockchain specification.

**Pagination Query:**

We take last record `lt` retrieved from the initial query and repeat query with "greater than lt" condition:

```graphql
{
  transactions(
    filter: { 
          account_addr:{
              eq:"0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13"
          }
          lt:    {gt:"0x458b31301"}
    }
    orderBy:[{path:"lt",direction:ASC}]
    limit:2
  )
  {
    id
    now
    lt
  }
}
```

**Result**:

```graphql
{
  "data": {
    "transactions": [
      {
        "id": "76fa008118645ce5376d5be728ed79fbb86466fa32ce72e4786a2fce3bb9e629",
        "now": 1593682244,
        "lt": "0x45a5e5201"
      },
      {
        "id": "d5f8e3b6ca81b84a1288979ca291df567eb9da4d2027010532d658d825b998c6",
        "now": 1593682274,
        "lt": "0x45ac921c1"
      }
    ]
  }
}
```

You need to repeat it `count of transactions`/ 50 times.

To get count of transactions use `aggregateTransactions`:

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

By default query result is limited to 50 records. To get next 50 records, you need to use created_lt (creation logical time) of the last record in the next query.

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

There are two records returned and that may mean that there is another page. We take last record `created_lt` and repeat query with "greater than created_lt" condition:

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

## Get the number of account transactions

Get COUNT of the transactions of a specified account

```graphql
query{
  aggregateTransactions(
    filter:{
     account_addr : { 
        eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" }
    },
    fields:[
      {
        fn:COUNT
      }
    ]
  )
}
```

Result:

```graphql
{
  "data": {
    "aggregateTransactions": [
      "1444"
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

Determine min, max and sum value for the gas_used of a transactions compute phase.

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

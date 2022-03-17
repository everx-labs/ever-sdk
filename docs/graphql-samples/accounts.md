# Accounts

## Get account info

To get account info use the following GraphQL query:

```graphql
query {
  blockchain{
   account(address:"0:653b9a6452c7a982c6dc92b2da9eba832ade1c467699ebb3b43dca6d77b780dd"){
    info{
      address
      acc_type
      balance
    	last_paid
      last_trans_lt
      boc
      data
      code
      library
      data_hash
      code_hash
      library_hash
    }
  }
  }
}

```

Result:

```graphql
{
  "data": {
    "blockchain": {
      "account": {
        "info": {
          "address": "0:653b9a6452c7a982c6dc92b2da9eba832ade1c467699ebb3b43dca6d77b780dd",
          "acc_type": 1,
          "balance": "0x223e8b8cef379b",
          "last_paid": 1647425518,
          "last_trans_lt": "0x2a9059e77c4",
          "boc": "te6ccgECDwEAApkAAnXABlO5pkUsepgsbckrLanrqDKt4cRnaZ67O0Pcptd7eA3SHoR9QxGNv3AAAAqkFnnfEciPouM7zebTQAIBAJNniOOihCJZNr2ArCaziee6VYr6JdmUdNs82Mlm2VJbMQAAAX+SNgQJwAJMlgo4O1jZEmyBymkHd/cTX5Y2hWW2OCWru/YnrscYSAIm/wD0pCAiwAGS9KDhiu1TWDD0oQUDAQr0pCD0oQQAAAIBIAgGAez/fyHtRNAg10nCAY4R0//TP9MA+Gp/+GH4Zvhj+GKOPvQFjQhgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE+GpwAYBA9A7yvdcL//hicPhjcPhmf/hh4tMAAZ+BAgDXGCD5AVj4QvkQ8qje0z8BBwBajh74QyG5IJ8wIPgjgQPoqIIIG3dAoLnekvhj4IA08jTY0x8B8AH4R26S8jzeAgEgDgkCAnULCgDVtF1VjXwgt0cKdqJoaf/pn+mAfDU//DD8M3wx/DFvfSBo/AB8JRDjgscSwQwLpDt0ABC4ZGfCwGUAOeegZwD9AUA056BnwOfA5Ln9gBB8NW+YfCFkZf/8IeeFn/wjZ4WAfCUA52T2qj/8M8ABCbRar5/ADAH++EFujlztRNAg10nCAY4R0//TP9MA+Gp/+GH4Zvhj+GKOPvQFjQhgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE+GpwAYBA9A7yvdcL//hicPhjcPhmf/hh4t74RvJzcfhm0fgA+ELIy//4Q88LP/hGzwsA+EoBzg0ADMntVH/4ZwBq3nAi0NYCMdIAMNwhxwCQ4CHXDR+S8jzhUxGQ4cEEIoIQ/////byxkvI84AHwAfhHbpLyPN4=",
          "data": "te6ccgEBAQEATAAAk2eI46KEIlk2vYCsJrOJ57pVivol2ZR02zzYyWbZUlsxAAABf5I2BAnAAkyWCjg7WNkSbIHKaQd39xNfljaFZbY4Jau79ieuxxhI",
          "code": "te6ccgECDQEAAg4AAib/APSkICLAAZL0oOGK7VNYMPShAwEBCvSkIPShAgAAAgEgBgQB7P9/Ie1E0CDXScIBjhHT/9M/0wD4an/4Yfhm+GP4Yo4+9AWNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4anABgED0DvK91wv/+GJw+GNw+GZ/+GHi0wABn4ECANcYIPkBWPhC+RDyqN7TPwEFAFqOHvhDIbkgnzAg+COBA+iogggbd0Cgud6S+GPggDTyNNjTHwHwAfhHbpLyPN4CASAMBwICdQkIANW0XVWNfCC3Rwp2omhp/+mf6YB8NT/8MPwzfDH8MW99IGj8AHwlEOOCxxLBDAukO3QAELhkZ8LAZQA556BnAP0BQDTnoGfA58Dkuf2AEHw1b5h8IWRl//wh54Wf/CNnhYB8JQDnZPaqP/wzwAEJtFqvn8AKAf74QW6OXO1E0CDXScIBjhHT/9M/0wD4an/4Yfhm+GP4Yo4+9AWNCGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAT4anABgED0DvK91wv/+GJw+GNw+GZ/+GHi3vhG8nNx+GbR+AD4QsjL//hDzws/+EbPCwD4SgHOCwAMye1Uf/hnAGrecCLQ1gIx0gAw3CHHAJDgIdcNH5LyPOFTEZDhwQQighD////9vLGS8jzgAfAB+EdukvI83g==",
          "library": null,
          "data_hash": "a34d868df79e09b2c3af67c1a9e210c1afef27f2376ee4ea5b00d20e7e55c058",
          "code_hash": "59ba6d164798169031c8ca18fa10c7038e7ad73b8d64f4c990e029a5dcfa59c3",
          "library_hash": null
        }
      }
    }
  }
}
```

fields:

* `address` is full account address that consists of workchainID:address
* `acc_type`
  * 0 – uninit (Account has balance but no code)
  * 1 – active (Account has balance and code)
  * 2 – frozen(Account has been frozen for some reasons)
  * 3 - nonExist (Account was deleted)
* `last_paid` - unixtime of the most recent storage payment (happens each transaction execution)
* `balance` - tokens on account (Note: to deploy smart contract code you need to have non-zero balance)
* `last_trans_lt` - logical time of last account transaction
* `boc` - Bag of cells with the account struct encoded as base64 (contains code, data, library and other header information).
* `data` - bag of cells with the account's data
* `code` - bag of cells with the account's code
* `library` - If present, contains library code used in smart-contract.
* `data_hash` - hash of account data&#x20;
* `code_hash` - hash of account code
* `library_hash` - library field hash

## Pagination of account transactions

<mark style="color:orange;">**Attention! Pagination with cursor functionality is new and not yet supported in Evernode-DS. But will be soon!**</mark>&#x20;

If you want to paginate all account transactions from the very first one, use this query

```graphql
query {
  blockchain{
   account(address:"0:653b9a6452c7a982c6dc92b2da9eba832ade1c467699ebb3b43dca6d77b780dd"){
    transactions{
      edges{
        node{
          id
          hash
          
        }
      }
      pageInfo{
        endCursor
        hasNextPage
      }
    }
  }
  }
}

```

Result

```graphql
{
  "data": {
    "blockchain": {
      "account": {
        "transactions": {
          "edges": [
            {
              "node": {
                "id": "transaction/172880ec68742d85cbbae19cda7bf900d2701c65847b8e11158142fc4af89099",
                "hash": "172880ec68742d85cbbae19cda7bf900d2701c65847b8e11158142fc4af89099"
              }
            },
            {
              "node": {
                "id": "transaction/c5ec73599e55d9257ca9e072ce867ab996c579b81ebc003acca121f7fb4797f6",
                "hash": "c5ec73599e55d9257ca9e072ce867ab996c579b81ebc003acca121f7fb4797f6"
              }
            },
            ...
          ],
          "pageInfo": {
            "endCursor": "5286af50052a33e50104",
            "hasNextPage": true
          }
        }
      }
    }
  }
}
```

Use `endCursor` field for further pagination and `hasNextCursor` for identifying if more records exist.&#x20;

If you want to paginate within some time range, you can use masterchain seq\_no or time range filter. You can paginate backwards as well.  Also you can use additional handy pagination parameters such as `after`, `first`, `before`, `last`. Read more about it in [blocks pagination section](blocks.md#about-cursor).&#x20;

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

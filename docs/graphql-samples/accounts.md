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

Use-cases:

* Collect account operations with detailed fees information
* Collect account balance history by pre-processing `balance_delta` changes on your side
* Query new account transactions to trigger some logic on your side
* Optionally filter transactions by `Aborted` type or `balance_delta` value

Let's paginate some account transactions from the very first one:

```graphql
query {
  blockchain{
   account(address:"0:653b9a6452c7a982c6dc92b2da9eba832ade1c467699ebb3b43dca6d77b780dd", first: 50){
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

## Pagination of account's messages

Use-cases:

* see transfers that some account sent or received
* monitor account's events
* monitor external calls of an account
* retrieve transfers between an account and some counterparty account
* optionally filter messages by value amount

In all these cases you need to paginate account messages with some filters applied. Lets see how to do it.

### Account transfers&#x20;

Lets get first 2  transfers some account received or sent. So we need to get incoming and outcoming internal messages. We separated `internal` message type into 2 types: `IntIn` and `IntOut` for search convenience. This way it is possible also to get only deposits, and only withdrawals.&#x20;

```graphql
query{
  blockchain{
    account(address:"-1:99392dea1c5035feddb1bb3db9e71138d82868f7460c6da3dca26f0520798ebd"){
      messages(msg_type:[IntIn, IntOut],first:2){
        edges{
          node{
            src
            dst
            id
            hash
            value(format:DEC)
            msg_type
            created_at_string
          }
          cursor
        }
        pageInfo{
          hasNextPage
        }
      }
    }
  }
}
```

Result. We see that the next page exists, we can continue pagination.

```graphql
{
  "data": {
    "blockchain": {
      "account": {
        "messages": {
          "edges": [
            {
              "node": {
                "src": "0:7db5e456a7c41306c23c588fb0561fe63443a6f17d7e2a08672369636980678f",
                "dst": "-1:99392dea1c5035feddb1bb3db9e71138d82868f7460c6da3dca26f0520798ebd",
                "id": "message/a74d826adf7f00153e034e1ee4de4f6e5a38843ee8d14c744bfcbf3c0df9f73d",
                "hash": "a74d826adf7f00153e034e1ee4de4f6e5a38843ee8d14c744bfcbf3c0df9f73d",
                "value": "1090000000",
                "msg_type": 0,
                "created_at_string": "2021-07-17 21:08:16.000"
              },
              "cursor": "59876bem0400"
            },
            {
              "node": {
                "src": "-1:99392dea1c5035feddb1bb3db9e71138d82868f7460c6da3dca26f0520798ebd",
                "dst": "-1:3333333333333333333333333333333333333333333333333333333333333333",
                "id": "message/ead06f194b988c1658215e178e68522f27cc018df1830bcfe779d9b9ce7fee93",
                "hash": "ead06f194b988c1658215e178e68522f27cc018df1830bcfe779d9b9ce7fee93",
                "value": "1000000000",
                "msg_type": 0,
                "created_at_string": "2021-07-17 21:08:24.000"
              },
              "cursor": "59876bem0401"
            }
          ],
          "pageInfo": {
            "hasNextPage": true
          }
        }
      }
    }
  }
}gra
```

### Account events

To get account events, we need to get Account's external outbound message. Their type is `ExtOut.` `Body` field contains ABI-encoded information with Event data.  You can parse it with SDK function [`abi.decode_message_body`](../reference/types-and-methods/mod\_abi.md#decode\_message\_body).

```graphql
query{
  blockchain{
    account(address:"0:454abb3c7db044603a9fb0802d3c6507b08d6b04855baa9a60802d9ecd34edad"){
      messages(msg_type:[ExtOut],first:2){
        edges{
          node{
            hash
            body
            created_at_string
          }
          cursor
        }
        pageInfo{
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
        "messages": {
          "edges": [
            {
              "node": {
                "hash": "315ca96ae9ded116b98692491d8accf1e01acd48e85b1db53b63615cd37f433b",
                "body": "te6ccgEBAQEAeQAA7VuEb3wAAAAAAAAAyWDwxNxg8kTcAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABg78TcCAUAAT1bfCW7sAABPW5DHEo+AAAAAAAAAAAAAAAIs3ySDgAAAAEAAHA+pB7Z9IAAAAAAAAAAAAAAAAAAAABA",
                "created_at_string": "2021-07-17 03:00:34.000"
              },
              "cursor": "59838dc005df77c11120003"
            },
            {
              "node": {
                "hash": "b54d2053c965cf2e41e265fd67a0b71a896fd06df8c305d1cb95da6780947113",
                "body": "te6ccgEBAQEALAAAU0UWNxJg8sTcn/M7F8rhuMCstb3zywTdpGh9GrmjYzToX9gcm8mXZJpVcA==",
                "created_at_string": "2021-07-17 03:00:34.000"
              },
              "cursor": "59838dc005df77c11120d01"
            }
          ],
          "pageInfo": {
            "hasNextPage": true
          }
        }
      }
    }
  }
}
```

### Account external calls

If you want to collect external calls of an account, filter by msg\_type = `ExtIn`. `Body` field contains ABI-encoded information with Event data.  You can parse it with SDK function [`abi.decode_message_body`](../reference/types-and-methods/mod\_abi.md#decode\_message\_body). Lets get the last external call:

```graphql
query{
  blockchain{
    account(address:"0:3d10c4d6dfc5d3cf6f8ac3d7468b792b91385c087da8f59669569493c7c0e28e"){
      messages(msg_type:[ExtIn],last:1){
        edges{
          node{
            hash
            body
            created_at_string
          }
          cursor
        }
        pageInfo{
          hasPreviousPage
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
        "messages": {
          "edges": [
            {
              "node": {
                "hash": "3ebc5a30f598825a015b99048b3f9baeb1d60818aa77ec6ceb3b84254e649723",
                "body": "te6ccgEBAQEAewAA8cb04wBQrr+dL/xBeDVKUIHJpF+ixQ9vsl7rIu8BtyRr72MIA9l87nY/maACAjMiwTkNeYlx+Vm3AtMvU000ZYXOo+U6Dh8fZKMrMO68do6VqlWYBXM3BEnQiVL3dDmtSMAAAGABANbS2JO2i4ap0DtYk7XgW5WzcGA=",
                "created_at_string": "2022-04-07 12:32:53.000"
              },
              "cursor": "5f7bcee00615d8d7711c0000"
            }
          ],
          "pageInfo": {
            "hasPreviousPage": true
          }
        }
      }
    }
  }
}кф
```

### Transfers between 2 accounts

In this example we retrieve last 30 messages between elector contract and some validator wallet with value more than some number:

```graphql
query{
  blockchain{
    account(address:"-1:3333333333333333333333333333333333333333333333333333333333333333"){
      messages(last:30, counterparties:["-1:99392dea1c5035feddb1bb3db9e71138d82868f7460c6da3dca26f0520798ebd"],
       min_value:"58579566000" ){
        edges{
          node{
            src
            dst
            id
            hash
            value(format:DEC)
            msg_type
            created_at_string
          }
          cursor
        }
        pageInfo{
          hasPreviousPage
        }
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
        "messages": {
          "edges": [
            {
              "node": {
                "src": "-1:3333333333333333333333333333333333333333333333333333333333333333",
                "dst": "-1:99392dea1c5035feddb1bb3db9e71138d82868f7460c6da3dca26f0520798ebd",
                "id": "message/958ee60bb2233e9e94d6c36465c0941632535d9dd1f9cb8e6b67616f1d33959e",
                "hash": "958ee60bb2233e9e94d6c36465c0941632535d9dd1f9cb8e6b67616f1d33959e",
                "value": "1625586165251876",
                "msg_type": 0,
                "created_at_string": "2022-03-15 18:12:16.000"
              },
              "cursor": "5edf8c1m0e01"
            },
          .....
                    ],
          "pageInfo": {
            "hasPreviousPage": true
          }
        }
      }
    }
  }
}
```

We see that previous page exists and can continue pagination.&#x20;

## Get the list of account's counterparties

Returns the paginable list of accounts the account has ever interacted with, with the last message info attached, sorted by the last message time. Useful for applications that want to show a screen with dialog list sorted by the last interaction time.

<mark style="color:orange;">**Attention! Available only in public API. Not available in Evernode-DS**</mark>**.**[ **See functionality comparison section.** ](https://tonlabs.gitbook.io/evernode-platform/products/functionality-comparison)****

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



## Account transactions count

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

## Account messages count

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

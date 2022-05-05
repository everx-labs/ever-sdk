# Transactions

## Get transaction info by hash

```graphql
query{
  blockchain{
    transaction(hash:"b0e26c42164ec0137913fdcd754aa819323a6a4b9ef5188863b021c3801e7ae4"){
      id
      hash
      balance_delta
      aborted
      lt
      now
    }
  }
}
```

Result:&#x20;

```graphql
{
  "data": {
    "blockchain": {
      "transaction": {
        "id": "transaction/b0e26c42164ec0137913fdcd754aa819323a6a4b9ef5188863b021c3801e7ae4",
        "hash": "b0e26c42164ec0137913fdcd754aa819323a6a4b9ef5188863b021c3801e7ae4",
        "balance_delta": "0x0",
        "aborted": false,
        "lt": "0x15bb39a23783",
        "now": 1645453010
      }
    }
  }
}
```

## Calculate account fees for transaction

```graphql
query{
  blockchain{
    transaction(hash:"998fee062e8daad96e88ce43adb52832b2e653d9a824912bc83051060932aceb"){
      ext_in_msg_fee(format:DEC)
      storage{
        storage_fees_collected(format:DEC)
      }
      compute{
        gas_fees(format:DEC)
      }
      action{
        total_fwd_fees(format:DEC)
      }      
    }
  }
}
```

You need to sum up these values to get the total fee the account paid for the transaction

```graphql
{
  "data": {
    "blockchain": {
      "transaction": {
        "ext_in_msg_fee": "2062000",
        "storage": {
          "storage_fees_collected": "270"
        },
        "compute": {
          "gas_fees": "10741000"
        },
        "action": {
          "total_fwd_fees": null
        }
      }
    }
  }
```

## Paginate blockchain transactions

Sometimes it is needed to  paginate all the network transactions. &#x20;

Due to the fact that Everscale is a multi-chain multi-threaded blockchain, pagination is not a straightforward operation and requires some special cursor and order of blocks and transactions inside them.

We provide our users with such functionality.  &#x20;

Read more about the used cursor-based approach in [blocks pagination section.](blocks.md#blocks-pagination)

You can either paginate all workchains' transactions (if you omit `workchain` parameter) or only the specified workchain's transactions.

If you do not specify time range you will start pagination from the start (`first` parameter) or the end (`last` parameter) of the network.

Here is a simple query how to get last 3 transactions of the network and paginate backwards.

You can paginate from the beginning, specify time period and number of returned items. Check how to implement different pagination use-cases in[ blocks pagination section.](blocks.md#paginate\_by\_seqno)

```graphql
query{
  blockchain{
      transactions(
        last:3
      ){
        edges{
          node{
            id
            now
          }
          cursor
        }
        pageInfo{
          startCursor
          hasPreviousPage
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
      "transactions": {
        "edges": [
          {
            "node": {
              "id": "transaction/d5ec03df890727a90eb80960560664ca49d9842e0a9f6f3c188ad1a49fca7264",
              "now": 1647429363
            },
            "cursor": "528cc96m05"
          },
          {
            "node": {
              "id": "transaction/39c2c3174aa414003bc55e68b6a08710cbcf6ce1b71dec1a0c8962fdcef5fc76",
              "now": 1647429363
            },
            "cursor": "528cc96m06"
          },
          {
            "node": {
              "id": "transaction/fa674f4c537aeaac951ebe4fd9eb2a1d094aae26d611f126652bee92827f3ed2",
              "now": 1647429363
            },
            "cursor": "528cc96m07"
          }
        ],
        "pageInfo": {
          "startCursor": "528cc96m05",
          "hasPreviousPage": true
        }
      }
    }
  }
}
```

Use `startCursor` and `hasPreviousPage` == true condition to paginate backwards like this:

```graphql
query{
  blockchain{
      transactions(
        last:3
        before: "528cc96m05"
      ){
        edges{
          node{
            id
            now
          }
          cursor
        }
        pageInfo{
          startCursor
          hasPreviousPage
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
      "transactions": {
        "edges": [
          {
            "node": {
              "id": "transaction/2ddaa133fb753c34d72bbae1b9040b773d2b9360612874da5fce19bcdd4b37a1",
              "now": 1647429363
            },
            "cursor": "528cc96m02"
          },
          {
            "node": {
              "id": "transaction/051363b194ebf6cf0ecab31643b34254aef7ec859d027211c0701df9046ad0bc",
              "now": 1647429363
            },
            "cursor": "528cc96m03"
          },
          {
            "node": {
              "id": "transaction/d7b764cd13e5b945aff381e45d1b2b855bf81444950f6b83eb355ac9b2d53ef5",
              "now": 1647429363
            },
            "cursor": "528cc96m04"
          }
        ],
        "pageInfo": {
          "startCursor": "528cc96m02",
          "hasPreviousPage": true
        }
      }
    }
  }
}
```

## Query transaction data

```graphql
query{
  transactions(filter:{
    id:{
      eq:"29948390d177f9ab10e48e3aad4d64af6f20441954d7e689cca5116e6ccbf5bf"
    }
  }){
    id
    tr_type
    tr_type_name
    aborted # if the transaction was executed successfully
    block_id
    account_addr
    balance_delta # how balance of account_addr has changed after transaction
    workchain_id
    lt # transaction logical time
    prev_trans_lt
    now # block time that contains transaction. transaction execution time
    outmsg_cnt # number of external messages, generated by the transaction
    orig_status_name
    end_status_name
    in_msg # message that produced the transaction
    in_message{
      msg_type_name
      src # account that sent the message
      dst # same as account_addr
      value # value attached to the message
    }
  }
}
```

The result:

```graphql
{
  "data": {
    "transactions": [
      {
        "id": "29948390d177f9ab10e48e3aad4d64af6f20441954d7e689cca5116e6ccbf5bf",
        "tr_type": 0,
        "tr_type_name": "Ordinary",
        "aborted": false,
        "block_id": "1c93bad89390b71ceb537707cff49167aabe6fbaa65e28c8357e12570c67c644",
        "account_addr": "-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af",
        "balance_delta": "0x9520368f50aab",
        "workchain_id": -1,
        "lt": "0x448c10cd4c1",
        "prev_trans_lt": "0x4093667f781",
        "now": 1599752024,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": "0e82f3c1c5f7e0414257954370c02890ad9046f5f2c0c9e1d7119591be7ae93f",
        "in_message": {
          "msg_type_name": "Internal",
          "src": "0:d1938ecc8a53ae58a6bf4416f9045a80b533227d79463b25424a8fd7c6de88fe",
          "dst": "-1:f6967e2ce65843a5cc450362b898e87a0fab3925bdc507195fa5003465cd62af",
          "value": "0x95203900d92ee"
        }
      }
    ]
  }
}
```

## Get the number of transactions in a specified shard over a period of time.

Here we specify the only shard of "-1" workchain and time from 18.43 till 19.43. You can do the same for any shard of "0" workchain.

```graphql
query{
  aggregateBlocks(filter:{
    workchain_id:{
      eq:-1
    }
    shard:{
      eq:"8000000000000000"
    }
    gen_utime:{
      gt:1596393798 # Date and time (GMT): Sunday, August 2, 2020 18:43:18
      lt:1596397406 # Date and time (GMT): Sunday, August 2, 2020 19:43:18

    }
  },
    fields:[
      { field: "tr_count", fn:SUM },
    ]
  )
}
```

Result:

```graphql
{
  "data": {
    "aggregateBlocks": [
      "4447"
    ]
  }
}
```

- [Block queries](#block-queries)
  - [Get the block hash by seq_no](#get-the-block-hash-by-seq_no)
  - [Query the latest masterchain block height](#query-the-latest-masterchain-block-height)
  - [Get the current list of shards](#get-the-current-list-of-shards)
  - [Query the latest shardchain block height](#query-the-latest-shardchain-block-height)
  - [Get block transactions](#get-block-transactions)

# Block queries

## Get the block hash by seq_no

Specify the workchain_id, shard and seq_no:

```graphql
query{
  blocks(filter:{
    workchain_id:{
      eq:-1
    }   
    shard:{
      eq:"8000000000000000"
    }
    seq_no:{
      eq:1418523
    }
  }
    orderBy:{
      path:"seq_no"
      direction:DESC
    }
    limit: 1
  )
  {
    id
    workchain_id
    shard
    seq_no
  }
}
```

The block hash is `11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef`:

```graphql
{
  "data": {
    "blocks": [
      {
        "id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "workchain_id": -1,
        "shard": "8000000000000000",
        "seq_no": 1418523
      }
    ]
  }
}
```

## Query the latest masterchain block height

Masterchain has only 1 shard `8000000000000000`.

So, to get its last block height we sort its blocks by `seq_no` in DESC order and get the newest one

```graphql
query{
  blocks(filter:{
    workchain_id:{
      eq:-1
    }   
  }
    orderBy:{
      path:"seq_no"
      direction:DESC
    }
    limit: 1
  )
  {
    id
    workchain_id
    shard
    seq_no
  }
}
```

The the latest masterchain block height is `1418096`:

```graphql
{
  "data": {
    "blocks": [
      {
        "id": "8d2a104aeaf7ce6dc96859c6476d6977bf83af5cc1198fb78fd5efb48e52a8bf",
        "workchain_id": -1,
        "shard": "8000000000000000",
        "seq_no": 1418096
      }
    ]
  }
}
```

See below how to query the list of shards and get the block height for every shard.

## Get the current list of shards

Workchain shard list can change dynamically depending on the network load.

To get the list of shards for Zero workchain for the current moment run this query. Here we sort the main workchain blocks by `seq_no`, get the newest one and extract the list of active shards of Zero workchain.

```graphql
query{
  blocks(filter:{
    workchain_id:{
      eq:-1
    }   
  }
    orderBy:{
      path:"seq_no"
      direction:DESC
    }
    limit: 1
  )
  {
    master{
      shard_hashes{
        shard
      }
    }
  }
}
```

Result:

```graphql
{
  "data": {
    "blocks": [
      {
        "master": {
          "shard_hashes": [
            {
              "shard": "0800000000000000"
            },
            {
              "shard": "1800000000000000"
            },
            {
              "shard": "2800000000000000"
            },
            {
              "shard": "3800000000000000"
            },
            {
              "shard": "4800000000000000"
            },
            {
              "shard": "5800000000000000"
            },
            {
              "shard": "6800000000000000"
            },
            {
              "shard": "7800000000000000"
            },
            {
              "shard": "8800000000000000"
            },
            {
              "shard": "9800000000000000"
            },
            {
              "shard": "a800000000000000"
            },
            {
              "shard": "b800000000000000"
            },
            {
              "shard": "c800000000000000"
            },
            {
              "shard": "d800000000000000"
            },
            {
              "shard": "e800000000000000"
            },
            {
              "shard": "f800000000000000"
            }
          ]
        }
      }
    ]
  }
}
```

## Query the latest shardchain block height

Let's get the latest shardchain 0800000000000000 of zero workchain block height (see how to get the list of shardchains at the previous step).

```graphql
query{
  blocks(filter:{
    workchain_id:{
      eq:0
    }   
    shard:{
      eq:"0800000000000000"
    }
  }
    orderBy:{
      path:"seq_no"
      direction:DESC
    }
    limit: 1
  )
  {
    id
    workchain_id
    shard
    seq_no
  }
}
```

The block height is `1948985`:

```graphql
{
  "data": {
    "blocks": [
      {
        "id": "8c06adeebfab1491ec532f2c40785b04a16ef31085f7e1ed3015c7de424ea953",
        "workchain_id": 0,
        "shard": "0800000000000000",
        "seq_no": 1948985
      }
    ]
  }
}
```

## Get block transactions

```graphql
query{
  transactions(filter:{
    block_id:{
      eq:"11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef"
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

Result:

```graphql
{
  "data": {
    "transactions": [
      {
        "id": "d90fe1b788a14d511ae3261b21bbea3623e7fd5dd79dfe2ee0edee24095f6bc7",
        "tr_type": 3,
        "tr_type_name": "Tock",
        "aborted": true,
        "block_id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "account_addr": "-1:04f64c6afbff3dd10d8ba6707790ac9670d540f37a9448b0337baa6a5a92acac",
        "balance_delta": "0x0",
        "workchain_id": -1,
        "lt": "0x22602dc3a03",
        "prev_trans_lt": "0x22602dc3a01",
        "now": 1600420927,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": null,
        "in_message": null
      },
      {
        "id": "411a8e30d8ee1793de7af7f271520efac8896faf9a397ed71920861e1da8bf28",
        "tr_type": 3,
        "tr_type_name": "Tock",
        "aborted": false,
        "block_id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "account_addr": "-1:5555555555555555555555555555555555555555555555555555555555555555",
        "balance_delta": "0x0",
        "workchain_id": -1,
        "lt": "0x22602dc3a03",
        "prev_trans_lt": "0x22602ccf7c3",
        "now": 1600420927,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": null,
        "in_message": null
      },
      {
        "id": "a3e5b32206db9d102d5861f31bb6ba0afcdf8ae962533265dd5f36d66ff31246",
        "tr_type": 2,
        "tr_type_name": "Tick",
        "aborted": true,
        "block_id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "account_addr": "-1:04f64c6afbff3dd10d8ba6707790ac9670d540f37a9448b0337baa6a5a92acac",
        "balance_delta": "0x0",
        "workchain_id": -1,
        "lt": "0x22602dc3a01",
        "prev_trans_lt": "0x22602ccf7c3",
        "now": 1600420927,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": null,
        "in_message": null
      },
      {
        "id": "74766cfd1a6b6efa2476a6e5440da8d6e0f52ec98034f00b4e1461c861a5f50c",
        "tr_type": 0,
        "tr_type_name": "Ordinary",
        "aborted": false,
        "block_id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "account_addr": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "balance_delta": "0xca2356bc",
        "workchain_id": -1,
        "lt": "0x22602dc3a02",
        "prev_trans_lt": "0x22602dc3a01",
        "now": 1600420927,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": "6972723200285f3940a1413dc3c7d2a2311e3722d62de4f700a8a5019eb8ef38",
        "in_message": {
          "msg_type_name": "Internal",
          "src": "-1:0000000000000000000000000000000000000000000000000000000000000000",
          "dst": "-1:3333333333333333333333333333333333333333333333333333333333333333",
          "value": "0xca2356bc"
        }
      },
      {
        "id": "b37dc576b2aeab6c7445153292946c4a8e2698df469dbe74666c53f3363f8a97",
        "tr_type": 2,
        "tr_type_name": "Tick",
        "aborted": false,
        "block_id": "11d663227777659a9f90a4098281cedfd50b929daa7093876b061d6915c90bef",
        "account_addr": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "balance_delta": "0x0",
        "workchain_id": -1,
        "lt": "0x22602dc3a01",
        "prev_trans_lt": "0x22602ccf7c2",
        "now": 1600420927,
        "outmsg_cnt": 0,
        "orig_status_name": "Active",
        "end_status_name": "Active",
        "in_msg": null,
        "in_message": null
      }
    ]
  }
}
```

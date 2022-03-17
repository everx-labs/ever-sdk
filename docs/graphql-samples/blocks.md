# Blocks

## Blocks pagination

### About cursor

<mark style="color:orange;">**Attention! Pagination functionality is new and not yet supported in Evernode-DS. But will be soon!**</mark>

As Everscale is a multi-chain multi-threaded blockchain, to paginate blocks we need to paginate several parallel workchains each having several parallel threads, which can split and merge over time, simultaneously. This differs Everscale from other blockchains where you simply paginate one chain with one thread and that has one height in a single moment in time.

This was really a challenge for us to solve this task. We needed to construct such a cursor that will be calculated from blockchain data and be the same across all Evernode Platform instances inside one network. So that if you switch from one endpoint to another - you keep the order. And the story stays really decentralized. And we did it.&#x20;

We formed this cursor based on the fact that all workchain blocks are commited into masterchain blocks in some specific order. And masterchain is ordered by seq\_no and has 1 thread. In simple words, this cursor splits all blockchain blocks into ranges between masterchain blocks and sort them inside that range in an ambiguos order. And, by the way, we derived the cursor for all blockchain transactions - from cursor for blocks, so that it is possible to paginate them too (check transactions pagination section).&#x20;

### Paginate by masterchain blocks seq\_no range <a href="#paginate_by_seqno" id="paginate_by_seqno"></a>

Query:

We specify **masterchain** blocks seq\_no range and that we want to paginate only 0 workchain blocks. If workchain parameter is omitted - you will get all workchains blocks. You can specify -1 to get masterchain blocks.

```graphql
query {
  blockchain{
    blocks(
      master_seq_no_range: {
        start: 2660661
        end: 2670661
      }
          workchain:0
    ) {
      edges {
        node {
          workchain_id
          id
          shard
          seq_no
          hash
          file_hash
        }
            cursor
      }
          pageInfo{
            endCursor
          }
    }
  }
}
```

Result:

You can see cursor field in each edge object, which can be passed over to the next query for pagination. Or you can get the latest cursor for the result set in `PageInfo.endCursor` field.

```graphql
{
  "data": {
    "blockchain": {
      "blocks": {
        "edges": [
          {
            "node": {
              "workchain_id": 0,
              "id": "block/279dd285a1a73dfcdd0b3739751232be6d6669821e53dd1b450c67e8f59d651d",
              "shard": "8000000000000000",
              "seq_no": 2773451,
              "hash": "279dd285a1a73dfcdd0b3739751232be6d6669821e53dd1b450c67e8f59d651d",
              "file_hash": "f7d7f2a506413c736eb3cb6ca3b071c15dc953f6d5e1996244fca68a2594fecf"
            },
            "cursor": "52899360052a51cb01"
          },
          {
            "node": {
              "workchain_id": 0,
              "id": "block/b146ba0ce2fd140b18f42199de57128d2a4de65e38676ca007b68ff0ab4f4b60",
              "shard": "8000000000000000",
              "seq_no": 2773452,
              "hash": "b146ba0ce2fd140b18f42199de57128d2a4de65e38676ca007b68ff0ab4f4b60",
              "file_hash": "c35b121b5edfa3389e64fb44d937553e6b3928877b14505559a168670953055c"
            },
            "cursor": "52899370052a51cc01"
          },
        ...
        ],
        "pageInfo": {
          "endCursor": "52899800052a51fc01"
        }
      }
    }
  }
}
```

Next page:

Let's check other available parameters for pagination.

`after/first` - Show `first` number of items `after` (not including) cursor.

`before/last`- Show `last` number of items `before` (not including) cursor. Used for backward pagination.

To check if the next page exists - we ask for `pageInfo.hasNextPage` parameter. If no next page exists, we can move `seq_no` range forward.  If you implement backward pagination - use `pageInfo.hasPreviousPage.`

Check other available parameters in GraphQL schema in playground.&#x20;

Here we continue pagination within the same `seq_no` range, and ask for the next 10 blocks after the last cursor in the previous query.  We see that the next page exists so we can continue paginating whithin the same `seq_no` range.

```graphql
query {
  blockchain{
    workchain_blocks(
      master_seq_no: {
        start: 2660661
        end: 2670661
      }
          after:"52899800052a51fc01"
          first:10
          workchain:0
    ) {
      edges {
        node {
          workchain_id
          id
          shard
          seq_no
          hash
          file_hash
        }
            cursor
      }
          pageInfo{
            endCursor
            hasNextPage
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
      "blocks": {
        "edges": [
          {
            "node": {
              "workchain_id": 0,
              "id": "block/279dd285a1a73dfcdd0b3739751232be6d6669821e53dd1b450c67e8f59d651d",
              "shard": "8000000000000000",
              "seq_no": 2773451,
              "hash": "279dd285a1a73dfcdd0b3739751232be6d6669821e53dd1b450c67e8f59d651d",
              "file_hash": "f7d7f2a506413c736eb3cb6ca3b071c15dc953f6d5e1996244fca68a2594fecf"
            },
            "cursor": "52899360052a51cb01"
          },
          {
            "node": {
              "workchain_id": 0,
              "id": "block/b146ba0ce2fd140b18f42199de57128d2a4de65e38676ca007b68ff0ab4f4b60",
              "shard": "8000000000000000",
              "seq_no": 2773452,
              "hash": "b146ba0ce2fd140b18f42199de57128d2a4de65e38676ca007b68ff0ab4f4b60",
              "file_hash": "c35b121b5edfa3389e64fb44d937553e6b3928877b14505559a168670953055c"
            },
            "cursor": "52899370052a51cc01"
          },
      ..... other blocks
        ],
        "pageInfo": {
          "endCursor": "52899800052a51fc01"
          "hasNextPage": true
        }
      }
    }
  }
}
```

### Paginate by masterchain blocks time range <a href="#paginate_by_timerange" id="paginate_by_timerange"></a>

If you do not know the `seq_no` of masterchain blocks  to create a range you can first obtain it by the time range, and then implement pagination the same way as described above.

To get the `seq_no` range by time rage do this query:

```graphql
query{
  blockchain{
    master_seq_no_range(time_start: 1647421084 time_end: 1647422084){
      start
      end
    }
  }
}
```

&#x20;In the result you will get the required seq\_no range.

<mark style="color:orange;">**Attention! Specifying timestamp range does not mean that there will be no blocks outside this range in the result set: this happens due to the fact that some thread blocks that were generated outside this time range were committed to masterchain block generated within this time range. But anyway, this pagination allows us to get all blocks in a handy manner, these time deltas are very small and not significant and can be ignored.**</mark>

```graphql
{
  "data": {
    "blockchain": {
      "master_seq_no_range": {
        "start": 2670769,
        "end": 2671143
      }
    }
  }
}
```

## Key blocks pagination

Sometimes it may be needed to paginate key blocks - for instance, it is used for proofs calculations.&#x20;

Or you can get blockchain config with this simple query:

```graphql
query {
  blockchain{
   key_blocks( last:1){
          edges {
           node {
                   workchain_id
                   id
                   shard
                   hash
                   file_hash
                   gen_utime
                   gen_utime_string
                    master{
                     config{
                       p34{
                         total_weight
                       }
                       #...any other config params
                       # check graphql schema for available fields
                     }
                    }
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
      "key_blocks": {
        "edges": [
          {
            "node": {
              "workchain_id": -1,
              "id": "block/45f416ea991e22ff01309ea517155e6ca61f13a7d367478ad8dce4a6af84692f",
              "shard": "8000000000000000",
              "hash": "45f416ea991e22ff01309ea517155e6ca61f13a7d367478ad8dce4a6af84692f",
              "file_hash": "fe1c5cf34b2ab632431f21048e5b200b0a6690a1acf3882721319ef9cd01e0c6",
              "gen_utime": 1647413431,
              "gen_utime_string": "2022-03-16 06:50:31.000",
              "master": {
                "config": {
                  "p34": {
                    "total_weight": "0xfffffffffffffff"
                  }
                }
              }
            }
          }
        ]
      }
    }
  }
}
```

Implement Pagination  the same way as described above:)

## Get the block hash by seq\_no

Specify the workchain\_id, shard and seq\_no:

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
query {
  blockchain{
   blocks( workchain:-1  thread: "8000000000000000"  last:1){
          edges {
           node {
            seq_no
           }
          }
   }
  }
}

```

The the latest masterchain block height is `1418096`:

```graphql
{
  "data": {
    "blockchain": {
      "blocks": {
        "edges": [
          {
            "node": {
              "seq_no": 2671859
            }
          }
        ]
      }
    }
  }
}
```

See below how to query the list of shards and get the block height for every shard.

## Get the current list of shards

Workchain shard list can change dynamically depending on the network load.

To get the list of shards for Zero workchain for the current moment run this query. Here we sort the main workchain blocks by `seq_no`, get the newest one and extract the list of active shards of Zero workchain.

```graphql
query {
  blockchain{
   blocks( last:1){
          edges {
           node {
              master{
               shard_hashes{
                shard
              }
              }
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
      "blocks": {
        "edges": [
          {
            "node": {
              "master": {
                "shard_hashes": [
                  {
                    "shard": "8000000000000000"
                  }
                ]
              }
            }
          }
        ]
      }
    }
  }
}
```

## Query the latest shardchain block height

Let's get the latest shardchain 0800000000000000 of zero workchain block height (see how to get the list of shardchains at the previous step).

```graphql
query {
  blockchain{
   blocks( workchain:0  thread: "8000000000000000"  last:1){
          edges {
           node {
            seq_no
           }
          }
   }
  }
}


```

The block height is `1948985`:

```graphql
{
  "data": {
    "blockchain": {
      "blocks": {
        "edges": [
          {
            "node": {
              "seq_no": 2780615
            }
          }
        ]
      }
    }
  }
}
```

# Block Queries

* [Block queries](blocks.md#block-queries)
  * [Get the block hash by seq\_no](blocks.md#get-the-block-hash-by-seq\_no)
  * [Query the latest masterchain block height](blocks.md#query-the-latest-masterchain-block-height)
  * [Get the current list of shards](blocks.md#get-the-current-list-of-shards)
  * [Query the latest shardchain block height](blocks.md#query-the-latest-shardchain-block-height)
  * [Get block transactions](blocks.md#get-block-transactions)

## Blocks pagination

### About cursor

As Everscale is a multi-chain multi-threaded blockchain, to paginate blocks we need to paginate several parallel workchains each having several parallel threads, which can split and merge over time, simultaneously. This differs Everscale from other blockchains where you simply paginate one chain with one thread and that has one height in a single moment in time.

This was really a challenge for us to solve this task. We needed to construct such a cursor that will be calculated from blockchain data and be the same across all Evernode Platform instances inside one network. So that if you switch from one endpoint to another - you keep the order. And the story stays really decentralized. And we did it.&#x20;

We formed this cursor based on the fact that all workchain blocks are commited into masterchain blocks in some specific order. And masterchain is ordered by seq\_no and has 1 thread. In simple words, this cursor splits all blockchain blocks into ranges between masterchain blocks and sort them inside that range in an ambiguos order. And, by the way, we derived the cursor for all blockchain transactions - from cursor for blocks, so that it is possible to paginate them too (check transactions pagination section).&#x20;

### Paginate by masterchain blocks seq\_no range

Query:

We specify **masterchain** blocks seq\_no range and that we want to paginate only 0 workchain blocks. If workchain parameter is omitted - you will get all workchains blocks. You can specify -1 to get masterchain blocks.

```graphql
query {
workchain_blocks(
	master_seq_no: {
		start: 13928620
		end: 13928625
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
```

Result:

You can see cursor field in each edge object, which can be passed over to the next query for pagination. Or you can get the latest cursor for the result set in `PageInfo.endCursor` field.

```graphql
{
  "data": {
    "workchain_blocks": {
      "edges": [
        {
          "node": {
            "workchain_id": 0,
            "id": "block/1a23b2d52c5030b98c685ab3fa400507e8f68454f34b17584b30e73f14fec445",
            "shard": "1800000000000000",
            "seq_no": 20272673,
            "hash": "1a23b2d52c5030b98c685ab3fa400507e8f68454f34b17584b30e73f14fec445",
            "file_hash": "f3b745078896e2457f828f0a8ed88c81ddf973cf0ec5f7e6f8605156646cbf9f"
          },
          "cursor": "5d488ac0061355621118"
        },
        {
          "node": {
            "workchain_id": 0,
            "id": "block/1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
            "shard": "1800000000000000",
            "seq_no": 20272674,
            "hash": "1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
            "file_hash": "eba0d9ebe14b7205088b5c4ebdf6616e5724abf9649d87617e47b77f927f0489"
          },
          "cursor": "5d488ac0061355622118"
        },
        ...
      ],
      "pageInfo": {
        "endCursor": "5d488ad0061355625118"
      }
    }
  }
}
```

Next page:

Let's check some more available parameters for pagination.

See `after`, `first` parameters, starting from `after` = `endCursor` from the previous page. Notice that we stay within the same seq\_no range - because we did not read it all yet.&#x20;

To check if the next page exist - we ask for `pageInfo.nextPage` parameter. If no `nextPage` exist, we can increase `seq_no` range.&#x20;

Check other available parameters in GraphQL schema in playground.&#x20;

```graphql
query {
workchain_blocks(
	master_seq_no: {
		start: 13928620
		end: 13928625
	}
      after:"5d488ac00613657a3117"
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
```

Result:

```graphql
{
  "data": {
    "workchain_blocks": {
      "edges": [
        {
          "node": {
            "workchain_id": 0,
            "id": "block/a7afaf3fb3300b14efbbc8c4b14c90905ec031df9b792d261d139ba046c4b62c",
            "shard": "6800000000000000",
            "seq_no": 20312063,
            "hash": "a7afaf3fb3300b14efbbc8c4b14c90905ec031df9b792d261d139ba046c4b62c",
            "file_hash": "0ff74ab2aac270efecf202fd15e7912aa0755eaf1fcd8db46261dd96268ec20e"
          },
          "cursor": "5d488ad006135efff116"
        },
        {
          "node": {
            "workchain_id": 0,
            "id": "block/5e33bef24b0bf36a333592083e5622bc0b7e2940873815d7074b8066b8df262c",
            "shard": "6800000000000000",
            "seq_no": 20312064,
            "hash": "5e33bef24b0bf36a333592083e5622bc0b7e2940873815d7074b8066b8df262c",
            "file_hash": "88d780bf277a0ec073fe1b2457fab2ab1121eaac8f083cea6c74e28194106148"
          },
          "cursor": "5d488ad006135f000116"
        },
      ..... other blocks
      ],
      "pageInfo": {
        "endCursor": "5d488ad0061362a04110",
        "hasNextPage": true
      }
    }
  }
}
```

### Paginate by masterchain blocks time range

If you do not know the `seq_no` of masterchain blocks  to create a range you can first obtain it by the time range, and then implement pagination the same way as described above.

To get the `seq_no` range by time rage do this query:

```graphql
query{
  master_seq_no_range(time_start: 1642144982 time_end: 1642145982){
    start
    end
  }
}
```

&#x20;In the result you will get the required seq\_no range.

<mark style="color:purple;">Attention! Specifying timestamp range does not guarantee that there will be no thread blocks outside this range in the result set: this happens due to the fact that some thread blocks are commited to masterchain block generated within this time range but were generated outside of it. But anyway, this pagination allows us to get all blocks in a handy manner, these time deltas are very small and not significant.</mark>

```graphql
{
  "data": {
    "master_seq_no_range": {
      "start": 13928618,
      "end": 13928913
    }
  }
}
```

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

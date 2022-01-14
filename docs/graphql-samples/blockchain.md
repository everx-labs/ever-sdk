- [Blockchain queries](#blockchain-queries)
  - [Key blocks](#key-blocks)
  - [Workchain blocks](#workchain-blocks)
  - [Workchain transactions](#workchain-transactions)
  - [Account transactions](#account-transactions)
  - [Pagination](#pagination)

# Blockchain queries

## Key blocks

Get the last key-block info before the given masterchain `seq_no` = 10,000,000.

```graphql
query {
	blockchain {
		key_blocks(
			seq_no: { end: 100000000 }
			last: 1
		) {
			edges {
				node {
					id
					seq_no
		  			hash
		  			file_hash
				}
			}
		}
	}
}
```

## Workchain blocks

Get all shardchain block chains for materchain block with `seq_no` = 13,928,620.

```graphql
query {
	blockchain {
		workchain_blocks(
			master_seq_no: {
				start: 13928620
				end: 13928621
			}
		) {
			edges {
				node {
					id
					shard
					seq_no
					hash
					file_hash
				}
			}
		}
	}
}
```

Result:
```json
{
  "data": {
    "blockchain": {
      "workchain_blocks": {
        "edges": [
          {
            "node": {
              "id": "block/1a23b2d52c5030b98c685ab3fa400507e8f68454f34b17584b30e73f14fec445",
              "shard": "1800000000000000",
              "seq_no": 20272673,
              "hash": "1a23b2d52c5030b98c685ab3fa400507e8f68454f34b17584b30e73f14fec445",
              "file_hash": "f3b745078896e2457f828f0a8ed88c81ddf973cf0ec5f7e6f8605156646cbf9f"
            }
          },
          {
            "node": {
              "id": "block/1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
              "shard": "1800000000000000",
              "seq_no": 20272674,
              "hash": "1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
              "file_hash": "eba0d9ebe14b7205088b5c4ebdf6616e5724abf9649d87617e47b77f927f0489"
            }
          },
          {
            "node": {
              "id": "block/40ae559adaea29153f8280ebc323c636379745ccf195a4edfb9f68019a463817",
              "shard": "1800000000000000",
              "seq_no": 20272675,
              "hash": "40ae559adaea29153f8280ebc323c636379745ccf195a4edfb9f68019a463817",
              "file_hash": "2d21ffc832a63003feb1e2dffa4be4c4322169b016e4c20cf67a50bbab0be13b"
            }
          },
          {
            "node": {
              "id": "block/46b70eef43b10e9646c4a258209af76b4eb13634f12345d11fe4f338476a6e18",
              "shard": "5800000000000000",
              "seq_no": 20280978,
              "hash": "46b70eef43b10e9646c4a258209af76b4eb13634f12345d11fe4f338476a6e18",
              "file_hash": "ec72ee3f63ae1389723726bf46fb48c0fe3b52223dfd06af5773918527cc759b"
            }
          },
          {
            "node": {
              "id": "block/dd8767c2caa15eb927acfeb7fc4e32483dbc949bee55a7d0530dd072a73c5710",
              "shard": "5800000000000000",
              "seq_no": 20280979,
              "hash": "dd8767c2caa15eb927acfeb7fc4e32483dbc949bee55a7d0530dd072a73c5710",
              "file_hash": "27f58f37f2153bd5112c8655ff28e58561d29419d9272c9aab42e734facc5da8"
            }
          },
          {
            "node": {
              "id": "block/2482353e3a42fe1bd4463c77b681bb5e5b6a388006df4e5c04bfccafa7df492f",
              "shard": "5800000000000000",
              "seq_no": 20280980,
              "hash": "2482353e3a42fe1bd4463c77b681bb5e5b6a388006df4e5c04bfccafa7df492f",
              "file_hash": "adfa84d337af9ae7b1ca2133cdc395391d25cd95f985a5ca4d2bcd1dc2dea189"
            }
          },
          {
            "node": {
              "id": "block/a5b98dc4740b49a069ada6bb9800139f1e3fbe95e027ca674346b021cb770a1b",
              "shard": "5800000000000000",
              "seq_no": 20280981,
              "hash": "a5b98dc4740b49a069ada6bb9800139f1e3fbe95e027ca674346b021cb770a1b",
              "file_hash": "10f1f0afdc5c783ffd49ae0f0cf662a2bf3671893b1d443ad9a575b783e01959"
            }
          },
          
          // ... other shards are omitted ...
        ]
      }
    }
  }
}
```

## Workchain transactions

Get all transactions with balance delta 1 token and more:

```graphql
query {
	blockchain {
		workchain_transactions(
			master_seq_no: {
				start: 13928620
				end: 13928621
			}
			min_balance_delta: 1000000000
		) {
			edges {
				node {
					id
					balance_delta(
						format: DEC
					)
					aborted
				}
			}
		}
	}
}
```

Result:
```json
{
  "data": {
    "blockchain": {
      "workchain_transactions": {
        "edges": [
          {
            "node": {
              "id": "transaction/8d940d3b1e02b78482c8a2f10e361ad62851049d945f28b8364b35b2b16f60ad",
              "balance_delta": "3728493497",
              "aborted": false
            }
          },
          {
            "node": {
              "id": "transaction/0befda171606156d509c11085a22e30414918f206948228525594dbbea7e9074",
              "balance_delta": "3944405127",
              "aborted": false
            }
          },
          {
            "node": {
              "id": "transaction/44e5f7caeb9100d21f5b4e0f72f6ea658d71891f56cfc7150ae9a12978cd775b",
              "balance_delta": "5229395000",
              "aborted": false
            }
          },
          {
            "node": {
              "id": "transaction/585c21311dc7490e813c99d2c56b366d57d96da565a426bafe1a3e2aada0ba64",
              "balance_delta": "3929265640",
              "aborted": false
            }
          },
          {
            "node": {
              "id": "transaction/2fa254acf247295f196af361377210e66fb62e4959d49ae024e3ddd4c6fd2111",
              "balance_delta": "4785149908",
              "aborted": false
            }
          }
        ]
      }
    }
  }
}
```

## Account transactions

Get info about the latest 50 transactions for the Elector account in masterchain `seq_no` range from 1 million to 2 millions.

```graphql
query {
	blockchain {
		account_transactions(
			master_seq_no: {
				start: 1000000
				end: 2000000
			}
			account_address: "-1:3333333333333333333333333333333333333333333333333333333333333333"
			last: 50
		) {
			edges {
				node {
					id
					account_addr
					balance_delta(
						format: DEC
					)
				}
			}
		}
	}
}
```

## Pagination

By default, record counts of all results of GraphQL queries are limited by 50. In order to obtain more records you need to do additional requests using `cursor` from the first or last previously queried record in `before` or `after`, respectivelly, filters. This will help you to organize pagination of queries:

```graphql
query {
	blockchain {
		account_transactions(
			master_seq_no: {
				start: 1000000
				end: 2000000
			}
			account_address: "-1:3333333333333333333333333333333333333333333333333333333333333333"
			after: "4f4240m01"
		) {
			edges {
				node {
					id
					account_addr
					balance_delta(
						format: DEC
					)
				}
				cursor
			}
			pageInfo {
				startCursor
				hasPreviousPage
				endCursor
				hasNextPage
			}
		}
	}
}
```

Result:
```json
{
  "data": {
    "blockchain": {
      "account_transactions": {
        "edges": [
          {
            "node": {
              "id": "transaction/29794987cf5a757da69a9ad5c2729cb6cc054bae909d4524288da1613e109556",
              "account_addr": "-1:3333333333333333333333333333333333333333333333333333333333333333",
              "balance_delta": "2887500000"
            },
            "cursor": "4f4240m02"
          },
          {
            "node": {
              "id": "transaction/5c735470092ca5be7093d3a68b49487c19f595363e3081d9da41bd9be8522996",
              "account_addr": "-1:3333333333333333333333333333333333333333333333333333333333333333",
              "balance_delta": "0"
            },
            "cursor": "4f4241m01"
          },

         // ... 47 other transactions are omitted ...

          {
            "node": {
              "id": "transaction/d49308ede91a4c048d86d46fe3a5768405f982143018dc4dac6cd13c46b33c31",
              "account_addr": "-1:3333333333333333333333333333333333333333333333333333333333333333",
              "balance_delta": "0"
            },
            "cursor": "4f4259m01"
          }
        ],
        "pageInfo": {
          "startCursor": "4f4240m02",
          "hasPreviousPage": false,
          "endCursor": "4f4259m01",
          "hasNextPage": true
        }
      }
    }
  }
}
```

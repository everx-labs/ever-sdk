- [Blockchain queries](#blockchain-queries)
  - [Key blocks](#key-blocks)
  - [Workchain blocks](#workchain-blocks)
  - [Workchain transactions](#workchain-transactions)
  - [Account transactions](#account-transactions)
  - [Pagination](#pagination)
    - [Key blocks pagination](#key-blocks-pagination)
    - [Workchain blocks pagination](#workchain-blocks-pagination)
    - [Workchain transactions pagination](#workchain-transactions-pagination)
    - [Account transactions pagination](#account-transactions-pagination)

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

Result:
```json
{
  "data": {
    "blockchain": {
      "key_blocks": {
        "edges": [
          {
            "node": {
              "id": "block/f5d26eaab04ac9f2de4eb5a7f47500acb255147bec3fa966fdd865b169ecfcb6",
              "seq_no": 14057800,
              "hash": "f5d26eaab04ac9f2de4eb5a7f47500acb255147bec3fa966fdd865b169ecfcb6",
              "file_hash": "c1e0c74a777e7a255834e362b68ed9871eca23c74118ab0f34b9be762067503c"
            }
          }
        ]
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

By default, record counts of all results of GraphQL queries are limited by 50. In order to obtain more records you need to do additional requests using `cursor` from the first or last previously queried record in `before` or `after`, respectivelly, filters. This will help you to organize pagination of queries.

### Key blocks pagination

```graphql
query {
	blockchain {
		key_blocks(
			seq_no: { end: 100000000 }
			after: "46de1fm" # put the last `cursor` value here
		) {
			edges {
				node {
					id
					seq_no
					hash
					file_hash
				}
				cursor
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
      "key_blocks": {
        "edges": [
          {
            "node": {
              "id": "block/5b2b1d4345e2788e81cb79d3ce36d1e2c03565781119c3cd8ba1f662cdfa373f",
              "seq_no": 465273,
              "hash": "5b2b1d4345e2788e81cb79d3ce36d1e2c03565781119c3cd8ba1f662cdfa373f",
              "file_hash": null
            },
            "cursor": "471979m"
          },
          {
            "node": {
              "id": "block/9eca6848865c353d99e3692b77bf665812239f8e49616d3a5b64f3334fa365db",
              "seq_no": 467518,
              "hash": "9eca6848865c353d99e3692b77bf665812239f8e49616d3a5b64f3334fa365db",
              "file_hash": null
            },
            "cursor": "47223em"
          },
          
          // ... 47 other key blocks are omitted ...

          {
            "node": {
              "id": "block/b7a9fe5a41d89ecff41822bbc9ad51e985b5de02078bfca6402314874bb9844a",
              "seq_no": 898804,
              "hash": "b7a9fe5a41d89ecff41822bbc9ad51e985b5de02078bfca6402314874bb9844a",
              "file_hash": null
            },
            "cursor": "4db6f4m" // Use this value in the `after` argument in the next query in order to get next page
          }
        ]
      }
    }
  }
}
```

### Workchain blocks pagination

```graphql
query {
	blockchain {
		workchain_blocks(
			master_seq_no: {
				start: 13928620
				end: 13928621
			}
	  	after: "5d488ac0061355621118" # put the last `cursor` value here
		) {
			edges {
				node {
					id
					shard
					seq_no
					hash
					file_hash
				}
				cursor
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
              "id": "block/1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
              "shard": "1800000000000000",
              "seq_no": 20272674,
              "hash": "1d1c37dc1f074f8a1e84b7e16f3517ba6fdae2fe92cb52de90cbafef17ce672b",
              "file_hash": "eba0d9ebe14b7205088b5c4ebdf6616e5724abf9649d87617e47b77f927f0489"
            },
            "cursor": "5d488ac0061355622118"
          },
          
          // ... 48 other workchain blocks are omitted ...

          {
            "node": {
              "id": "block/fba2f95642bfc6baa0f71232073945db267795a2918f0625ca77cd5991323a2b",
              "shard": "8000000000000000",
              "seq_no": 13928620,
              "hash": "fba2f95642bfc6baa0f71232073945db267795a2918f0625ca77cd5991323a2b",
              "file_hash": "13f7d8dfe64109a69a13fc6ba038a6224b326565d8b1a9ccc1b7d697887a8107"
            },
            "cursor": "5d488acm"
          }
        ]
      }
    }
  }
}
```

### Workchain transactions pagination

```graphql
query {
	blockchain {
		workchain_transactions(
			master_seq_no: {
				start: 13928620
			}
			min_balance_delta: 1000000000
			after: "5d488ac00613627c111500" # put the last `cursor` value here
		) {
			edges {
				node {
					id
					balance_delta(
						format: DEC
					)
					aborted
				}
				cursor
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
              "id": "transaction/0befda171606156d509c11085a22e30414918f206948228525594dbbea7e9074",
              "balance_delta": "3944405127",
              "aborted": false
            },
            "cursor": "5d488ac006136342111100"
          },
          {
            "node": {
              "id": "transaction/44e5f7caeb9100d21f5b4e0f72f6ea658d71891f56cfc7150ae9a12978cd775b",
              "balance_delta": "5229395000",
              "aborted": false
            },
            "cursor": "5d488ac00613635c411c00"
          },
          
          // ... 47 other transactions are omitted ...

          {
            "node": {
              "id": "transaction/b269e0c50ba50c6083963f27ccf8be281e4f6761f1e2420f3aa2da96506ce5f5",
              "balance_delta": "480139674427885",
              "aborted": false
            },
            "cursor": "5d488d4m04" // Use this value in the `after` argument in the next query in order to get next page
          }
        ]
      }
    }
  }
}
```

### Account transactions pagination

```graphql
query {
	blockchain {
		account_transactions(
			master_seq_no: {
				start: 1000000
				end: 2000000
			}
			account_address: "-1:3333333333333333333333333333333333333333333333333333333333333333"
			after: "4f4240m01" # put the last `cursor` value here
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
            "cursor": "4f4259m01" // Use this value in the `after` argument in the next query in order to get next page
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

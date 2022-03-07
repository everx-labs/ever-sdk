# Counterparties

Returns a list of addresses the specified account interacted with, sorted by the latest interaction time (the latest message time between 2 accounts) DESC. Feature may be useful for wallet applications or for chat-based DApps to show the list of counterparties in descending order.\
\
Available only in Cloud API.

* [Counterparties queries](counterparties.md#counterparties-queries)
  * [All counterparties for specific account](counterparties.md#all-counterparties-for-specific-account)
  * [Pagination](counterparties.md#pagination)

## Counterparties query

### All counterparties for specific account

Query information about the last value flows of all counterparties of the elector account:

```graphql
query {
	counterparties(
		account: "-1:3333333333333333333333333333333333333333333333333333333333333333"
	) {
		account
		counterparty
		last_message_id
		last_message_at
		last_message_is_reverse
		last_message_value(format: DEC)
	}
}
```

Result:

```json
{
  "data": {
    "counterparties": [
      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:0000000000000000000000000000000000000000000000000000000000000000",
        "last_message_id": "14d6f332c520da3101497e875ef4662048276415c8c200c1b744f7c47fb3ea21",
        "last_message_at": 1642155198,
        "last_message_is_reverse": true,
        "last_message_value": "1825000000"
      },
      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:f18a64fa0ba3c6eeaf40934aae5319890253e5a41ebcd28268831c8ef1601efd",
        "last_message_id": "acce2971f2e51046d52d18dbcd808a1e7481cf8f216986053aa8c3a6784327d3",
        "last_message_at": 1642149742,
        "last_message_is_reverse": false,
        "last_message_value": "45958252455702"
      },
      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:5555555555555555555555555555555555555555555555555555555555555555",
        "last_message_id": "ea45dc94450d41915c2839ca2758dd68e893a802e16caf002ecb26438f24a27b",
        "last_message_at": 1642149490,
        "last_message_is_reverse": false,
        "last_message_value": "1073741824"
      },

      // ... all other 47 records are omitted...
    ]
  }
}
```

### Pagination

By default, record counts of all results of GraphQL queries are limited by 50. In order to obtain more records you need to do additional requests using `cursor` from the first or last previously queried record in `before` or `after`, respectivelly, filters. This will help you to organize pagination of counterparites queries:

```graphql
query {
	counterparties(
		account: "-1:3333333333333333333333333333333333333333333333333333333333333333"
		after: "1642131731/-1:97331a562c2de1d03798fc55d2b3fb6377e144bcfec22b13a9e0fc39948661c8"
	) {
		account
		counterparty
		last_message_id
		last_message_at
		last_message_is_reverse
		last_message_value(format: DEC)
		cursor
	}
}
```

Result:

```json
{
  "data": {
    "counterparties": [
      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:b35240ac9d6c542bb9e08c969bf85d33ee7601a2eb633d831f9fc67fcc12106f",
        "last_message_id": "3c85978e5d74f67cb160016af146238d73254fcec400e04d30d0b6f82db99834",
        "last_message_at": 1642131722,
        "last_message_is_reverse": false,
        "last_message_value": "1000000000",
        "cursor": "1642131722/-1:b35240ac9d6c542bb9e08c969bf85d33ee7601a2eb633d831f9fc67fcc12106f"
      },
      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:0d19ad161fb5adc3e7e46ff55021fc0d0c7d6599fcda9606af78f0f907af8fec",
        "last_message_id": "2804027eb2826a5adf2c7dc680faa2a78bb39e498a64dbc2b22ae1794cd7b5a2",
        "last_message_at": 1642131709,
        "last_message_is_reverse": false,
        "last_message_value": "1000000000",
        "cursor": "1642131709/-1:0d19ad161fb5adc3e7e46ff55021fc0d0c7d6599fcda9606af78f0f907af8fec"
      },
      
      // ... other 47 records are omitted ...

      {
        "account": "-1:3333333333333333333333333333333333333333333333333333333333333333",
        "counterparty": "-1:a6639baeeb0842e54b8695cc71343435ffeb76cdd03f5c6e6860bc8c2cf1ec6f",
        "last_message_id": "9aa8b010f59f927c861cafbf1ca18811774342b493ccd32a2dca453c4bbfc538",
        "last_message_at": 1642126820,
        "last_message_is_reverse": false,
        "last_message_value": "1000000000",
        "cursor": "1642126820/-1:a6639baeeb0842e54b8695cc71343435ffeb76cdd03f5c6e6860bc8c2cf1ec6f"
      }
    ]
  }
}
```

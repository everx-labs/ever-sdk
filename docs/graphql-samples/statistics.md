# Statistics Queries

General Everscale Network statistics related to accounts, transactions, messages and blocks. And also some essential statistics about validators and depools.\
\
Available only in Cloud API.



Get all blockchain statistics:

```graphql
query {
	statistics {
		version

		blocks {
			totalCount
			countByCurrentValidators
			ratePerSecond
			ratePerSecond
		}
		messages {
			totalCount
			ratePerSecond
		}
		transactions {
			totalOrdinaryCount
			lastDayOrdinaryCount
			ratePerSecond
		}
		accounts {
			totalCount
			totalSupply
			amountOnGivers
			circulatingSupply
			lastDayCount
			accountTypesCount
		}
		validators {
			totalCount
			lastCycleCountDelta
			totalStaked
			rewardsPer30Days
			apr
		}
		depools {
			activeDepoolCount
			activeParticipantsCount
			totalRewards
			totalStaked
		}
	}
}
```

Result:

```json
{
  "data": {
    "statistics": {
      "version": "0.1.0",
      "blocks": {
        "totalCount": 339156267,
        "countByCurrentValidators": 369007,
        "ratePerSecond": 3.0956521739130434
      },
      "messages": {
        "totalCount": 26712014,
        "ratePerSecond": 0.16666666666666666
      },
      "transactions": {
        "totalOrdinaryCount": 25062879,
        "lastDayOrdinaryCount": 60120,
        "ratePerSecond": 0.8333333333333334
      },
      "accounts": {
        "totalCount": 537712,
        "totalSupply": "2044010756821730838",
        "amountOnGivers": "1023416009272518819",
        "circulatingSupply": "1020594747549212019",
        "lastDayCount": 0,
        "accountTypesCount": 4029
      },
      "validators": {
        "totalCount": 441,
        "lastCycleCountDelta": 7,
        "totalStaked": "447109714691325382",
        "rewardsPer30Days": "2146598319706806",
        "apr": 0.06233074698652623
      },
      "depools": {
        "activeDepoolCount": 451,
        "activeParticipantsCount": 3655,
        "totalRewards": "15498449926018398",
        "totalStaked": "416906687487291516"
      }
    }
  }
}
```

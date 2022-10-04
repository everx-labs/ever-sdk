# FT (Fungible Token) API

## List of tokens

Shows the list of tokens, sorted by token creation timestamp (DESC) - fresh ones go first.&#x20;

#### Filter

You can optionally filter by symbol substring.&#x20;

#### Pagination

Use `cursor`, {`first`, `after`} or  {`last`, `before`} filters for pagination.

```graphql
query{
  ft{
    tokens(symbolSubstring:"EVER", first:4){
      edges{
        node{
          address
          symbol
          standard
          name
          decimals
          rootOwner
          totalSupply
        }
        cursor
      }
    }
  }
}
```

## Token info

Returns info about the token by its root contract address

```graphql
query{
  ft{
    token(address:"0:a49cd4e158a9a15555e624759e2e4e766d22600b7800d891e46f9291f044a93d"){
      address
      symbol
      name
      decimals
      rootOwner
      totalSupply
    }
  }
}
```

## Holder info

Returns info about token owner by its address.

#### Pagination

Use `cursor`, {`first`, `after`} or  {`last`, `before`} filters for pagination of list fields.

```graphql
query{
  ft{
    holder(address:"0:3666ef0af863317eafcea173b9a9fdd6d1e4aa6dd080a1f472b2ad217215e5c9"){
      address
      wallets(first:2){
        edges{
          node{
            balance
            token{
              symbol
            }
            percentage
          }
          cursor
        }
      }
      transfers(first:1){
        edges{
          node{
            messageId
            token{
              symbol
            }
            fromWallet{
              address
            }
            toWallet{
              address
            }
            fromHolder{
              address
            }
            toHolder{
              address
            }
            transferType
            timestamp
            value
          }
          cursor
        }
      }
    }
    }
}
```

## Wallet info

Returns the information related to the tip3 token wallet.&#x20;

Explore schema for all available fields.

#### Pagination

Use `cursor`, {`first`, `after`} or  {`last`, `before`} filters for pagination of list fields.

```graphql
query{
  ft{
    wallet(address:"0:fc9316349bdd2b961dc0a89fc9cf4cb95a26aff662c042945a6a533a401db3f3"){
      address
      token{
        address
        symbol
      }
      holder{
        address
      }
      balance
      percentage
      transfers(first:10){
        edges{
          node{
            value
          }
        }
      }
    }
  }
}
```

## Account

Field `tokenHolder` was added to `BlockchainAccount`

```graphql
query{
blockchain{
    account(address:"0:3666ef0af863317eafcea173b9a9fdd6d1e4aa6dd080a1f472b2ad217215e5c9"){
      info{
        address
        balance
        tokenHolder{
          wallets{
            edges{
              node{
                balance
                token{
                  symbol
                }
                transfers{
                  edges{
                    node{
                      value
                      messageId
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
```

## Transaction

Field `tokenTransfer` was added to `BlockchainTransaction`

```graphql
query{
  blockchain{
    transaction(hash:"fda25eec114d7da8d6ebd31ec2b2ca07eed0db8749501d6c1a128f646f91e350"){
      hash
      tokenTransfer{
        value
        transferType 
      }
    }
  }
}
```

#### About `transferType`

In source transaction transferType for simple transfer will be `Transfer`

In destination transaction transferType for simple transfer will be `Receive`

## Message

Field `tokenTransfer` was added to `BlockchainMessage.`

```graphql
query{
blockchain{
    message(hash:"39af972085108d71ba439dab66cd4cbdb010ab018a078eeed7bf2f05df237c68"){
      hash
      tokenTransfer{
        value
        transferType
        token{
          symbol
        }
      }
    }
  }
}
```

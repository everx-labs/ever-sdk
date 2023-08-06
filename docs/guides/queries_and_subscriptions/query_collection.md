# Query Collection

{% hint style="warning" %}
<mark style="color:red;">**Collections is an analytics API  (not real-time, though it may look like one).**</mark>&#x20;

<mark style="color:red;">**Not all filters and sortings are working now. Data is provided only for the past 7 days.**</mark>
{% endhint %}

## When you may need collections?&#x20;

If you want to apply some custom filters and sortings on the data.

## About collections

There are a few collections with blockchain data:

* _accounts_: blockchain account data;
* _transactions_: transactions related to accounts;
* _messages_: input and output messages related to transactions;
* _blocks_: blockchain blocks.
* _block\_signatures_ : validator block signatures

[Use `query_collection` method to query data that can be filtered and sorted](../../reference/types-and-methods/mod\_net.md#query\_collection).

## Usage

```javascript
await client.net.query_collection({
    collection: 'accounts',
    filter: {
        id: {
            eq: wallet1Address
        }
    },
    result: 'balance'
})
```

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query)

**AppKit**

[https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query](https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query)

# Query Collection

{% hint style="warning" %}
<mark style="color:red;">**Please avoid using collection queries as much as possible. Soon we will restrict their execution time by 5 seconds. Use**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**`net.query`**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**+**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**`blockchain`**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**API  instead.**</mark>&#x20;
{% endhint %}

`Collections` is the old API we had started providing when we didn't know what use-cases might be needed for developers and how to allow pagination of the sharded data the best way.&#x20;

We created API that allowed to query basically anything which led to impossible data optimizations on our end to provide good quality of such functionality.

Now we analyzed what use-cases users need and created a new API called `blockchain`. You can find it in root `query` next to all collections. Use it with [net.query function](raw\_query.md).&#x20;

What is a collection?

There are a few collections with blockchain data:

* _accounts_: blockchain account data;
* _transactions_: transactions related to accounts;
* _messages_: input and output messages related to transactions;
* _blocks_: blockchain blocks.
* _block\_signatures_ : validator block signatures

[Use `query_collection` method to query data that can be filtered and sorted](../../reference/types-and-methods/mod\_net.md#query\_collection).

<mark style="color:red;">**Attention! Avoid using collections if possible. Use**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**`net.query`**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**+**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**`blockchain`**</mark><mark style="color:red;">** **</mark><mark style="color:red;">**api instead.**</mark>

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/query)

**AppKit**

[https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query](https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/query)

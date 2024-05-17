---
description: >-
  Write your graphql query in playground, copy it and insert into SDK's
  net.query function.  Define variables and execute it.
---

# net.query syntax

## About `net.query` function

Whenever you need to poll realtime data from GraphQL API with SDK - use [net.query](../../reference/types-and-methods/mod\_net.md#query) function.&#x20;

Write your graphql query in playground, copy it and insert into SDK's net.query function.  Define variables and execute it.&#x20;

See All the functions available in API here&#x20;

## Usage

### Pass parameters via `variables` object

You can pass variables via a separate parameter (graphql-style). You just copy the query from playground and replace the param values with $param\_name and then pass parameters via additional object like this:

{% hint style="success" %}
If you use variables object, you need to wrap your query in \
query MyQuery(params){$param1: Param1Type}.&#x20;
{% endhint %}

{% hint style="danger" %}
If the parameter is mandatory you must specify its type with exclamation mark on the end like this:\
query MyQuery(params){$param1: Param1Type!}
{% endhint %}

```javascript
await client.net.query({
    query: `query MyQuery($utime: Int){
    blockchain {
        master_seq_no_range(time_end: $utime) { end }
    }
}`,
    variables: { utime },
})
```

### Pass parameters inline

```javascript
await client.net.query({
    query: `
    blockchain {
        master_seq_no_range(time_end: ${utime}) { end }
    }`
})
```

See this sample to understand how to pass variables to the queries

{% embed url="https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/pagination" %}

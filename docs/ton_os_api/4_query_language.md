# Query Language

## Query language

This document provides query language syntax examples and documentation

Check out \(../mod\_net.md\) - the official TON Labs wrapper over GraphQL API for root queries and subscriptions.

* [Query a collection](4_query_language.md#query-a-collection)
* [Aggregate a collection](4_query_language.md#aggregate-a-collection)
* [Subscription](4_query_language.md#subscription)
* [Filtration](4_query_language.md#filtration)
  * [Scalar filters](4_query_language.md#scalar-filters)
  * [Array filters](4_query_language.md#array-filters)
  * [Structure filters](4_query_language.md#structure-filters)
* [OR operator](4_query_language.md#or-operator)
* [Joins](4_query_language.md#joins)
* [Sorting and limiting](4_query_language.md#sorting-and-limiting)
* [Working with u64 and u128 numbers](4_query_language.md#working-with-u64-and-u128-numbers)
  * [U64String](4_query_language.md#u64string)
  * [U1024String](4_query_language.md#u1024string)
  * [GraphQL interaction](4_query_language.md#graphql-interaction)

## Query a collection

Account collection query sample that returns the specified account's balance

```text
query {
  accounts(
    filter: {
      id: {eq: "-1:6666666666666666666666666666666666666666666666666666666666666666"}
    }
  ) {
    id
    balance
  }
}
```

To perform a query over a collection, choose a collection and result projection. **Optionally** specify a filter, sorting order and the maximum number of items in the results list.

```text
query {
  transactions(
    filter: {
      now: {gt: 1567601735}
    }
    orderBy: {path:"now",direction:DESC}
    limit :5
  )
  {
    id
    now
  }
}
```

The example above demonstrates a query to the `transactions` collection with the following parameters:

* `filter`: a JSON object matching the internal collection structure. It supports additional conditions for particular fields. In the example above, the `now` field of a transaction must be greater than 1567601735.
* `orderby`: sort by "now" field in DESC order
* `limit`: show only top 5 objects
* `result`: is a result projection that determines structural subset used for returning items. In the example above the request is limited to two fields: `id` and `now`. Note that results have to follow GraphQL rules.

Read more about filtration, sorting and limiting below in this section.

## Aggregate a collection

You can collect aggregated data with aggregation root queries:

* aggregateBlocks
* aggregateTransactions
* aggregateMessages
* aggregateAccounts
* aggregateBlockSignatures

Data is aggregated by `filter` and `fields`.

`filter` - read more about it below in this section

`fields` is an array of tuples `{field, fn}` - field names and aggregation functions. Aggregation function is a predefined string constant and determines the value that should be collected for a corresponding field.

**Example 1**

Get COUNT of the transactions of a specified account \(note that in case of `COUNT` you can omit `field` in `fields` \):

```text
query{
  aggregateTransactions(
    filter:{
     account_addr : { 
        eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" }
    },
    fields:[
      {
        fn:COUNT
      }
    ]
  )
}
```

Result:

```text
{
  "data": {
    "aggregateTransactions": [
      "1444"
    ]
  }
}
```

**Example 2**

Determine min, max and sum values of transferred coins and number of transfers between two accounts:

```text
query{
  aggregateMessages(
    filter:{
        src:{eq:"0:797f32a15bbe5213a07cafe4c80e5e28f2662e865e95b23694f4bd36f2b42ff8"}
        dst:{eq:"0:7d667fed88b9edb82eb6a116b48052b6a7765577ad341b35acb118451c7aa625"}

          OR:{
            src:{eq:"0:7d667fed88b9edb82eb6a116b48052b6a7765577ad341b35acb118451c7aa625"}
            dst:{eq:"0:797f32a15bbe5213a07cafe4c80e5e28f2662e865e95b23694f4bd36f2b42ff8"}
        }
    }
    fields:[
        { field: "value", fn: MIN},
        { field: "value", fn: MAX },
        { field: "value", fn: SUM },
          { fn: COUNT}
    ]
  )
}
```

Result:

```text
{
  "data": {
    "aggregateMessages": [
      "10000000",
      "10000000",
      "30000000",
      "3"
    ]
  }
}
```

**Example 3**

Determine `min`, `max` and `sum` value for the gas\_used of a transactions compute phase \(you can use a dot separated path as a field name to use fields resided deep in a JSON structure of a transaction record\):

```text
query{
  aggregateTransactions(
    filter:{
     account_addr : 
            {eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" }
    },
    fields:[
      { field: "compute.gas_used", fn:MIN },
      { field: "compute.gas_used", fn:MAX },
      { field: "compute.gas_used", fn:SUM },
    ]
  )
}
```

Result:

```text
{
  "data": {
    "aggregateTransactions": [
      "1434",
      "45221",
      "32578614"
    ]
  }
}
```

Video tutorial - GraphQL: Joined Blocks, OR Operator, Aggregations

{% embed url="https://www.youtube.com/watch?v=8dNAv5vsYRI" %}



## Subscription

In this example, we start a subscription and get a result whenever a block is inserted or updated in the blockchain.

```text
subscription{
  blocks{
    id
  }
}
```

The `filter` and `result` parameters are the same as in the `query` method. The `filter` parameter narrows the action down to a subset of monitored items. In this case, the filter is empty: all items are included into monitoring.

## Filtration

> Filtration applies only to collection query types

Filters applied to query functions are data structures matching collection item with several extra features:

* The value for scalar fields \(e.g. strings, numbers etc.\) is a structure with the `scalar filter`.
* The value for array fields is a structure with an `array filter`.
* The value for nested structures is a filter for `nested structure`.

These filter types will be described in more details below in this section.

### Scalar filters

Scalar filter is a structure with one or more predefined fields. Each field defines a specific scalar operation and a reference value:

* `eq`:  item value must be equal to the specified value;
* `ne`: item value must not be equal to the specified value;
* `gt`:  item value must be greater than the specified value;
* `lt`:  item value must be less than specified value;
* `ge`: item value must be greater than or equal to the specified value;
* `le`: item value must be less than or equal to the specified value;
* `in`:  item value must be contained in the specified array of values;
* `notIn`: item value must not be contained within the specified array of values.

Scalar filter example 1

```text
filter: {
    id: { eq: 'e19948d53c4fc8d405fbb8bde4af83039f37ce6bc9d0fc07bbd47a1cf59a8465'},
    status: { in: [0, 1, 2] }
}
```

Scalar filter example 2

```text
filter: {
    now: { gt: 1563449, lt: 2063449 }
}
```

The logic from the above snippet can be expressed in the following way:

```text
(transaction.now > 1563449) && (transaction.now < 2063449)
```

### Array filters

Array filters are used for array \(list\) fields. Each has to contain at least one of the predefined operators:

* `any`: used when at least one array item matches the nested filter;
* `all`: used when all items matches the nested filter.

The `any` or `all` must contain a nested filter for an array item.

Array operators are mutually exclusive and can not be combined. For empty arrays, the array filter is assumed to be false.

### Structure filters

If an item is a structure, then a filter has to contain fields named as fields of this item. Each nested filter field contains a condition for the appropriate field of an item. The `AND` operator is used to combine conditions for several fields.

## OR operator

You can combine several struct filters over collection with logical OR in a single query. Just specify `OR` field in collection struct filter.

Determine all messages related to the specified account:

```text
query {
  messages(
  filter:{
    src: { eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" },
    OR: 
    {
       dst: { eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" }
    }
})
  {
    id
    src
    dst
    value
  }
}
```

Request messages of myAcc or messages with value more than 10000 nG \(combine several `OR` operators\) :

```text
query {
  messages(
  filter:{
    src: { eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" },
    OR: 
    {
        dst: { eq: "0:a52f6a7ea6bc7279728cbff01ad1e8b1dfc386098cfac1f381ae3959bf2ae9db" },
        OR: 
        {
               value: { gt: "10000" }
        }
    }
})
  {
    id
    src
    dst
    value
  }
}
```

## Joins

A NoSQL database contains additional fields that work as cross-references for related collections. For example, the _transactions_ collection has the `in_message` field that stores the relevant message item. The message item exists in _messages_ collection and has the `id` value equal to the `in_msg` value in _transactions_. Block join is present in Messages and Transactions collections.

Joined items are represented as nested structures in a filter and in the result projection.

## Sorting and limiting

> Sorting and limiting applies only to collection query types

By default, retrieval order for several items is not defined. To specify it, use the `orderBy` parameter of `query` method.

The sort order is represented by an array or sort descriptors. These structures contain two fields: `path` and `direction`:

* `path` specifies a path from a root item of the collection to the field that determines the order of return items. The path includes field names separated by dot.
* `direction` specifies the sorting order: ASC or DESC \(ascending and descending\).

You can specify more than one field to define an order. If two items have equal values for the first sort descriptor, then second descriptor is used for comparison, etc. If values of sorting fields are the same for several items, then the order of these items is not defined.

The `limit` parameter determines the maximum number of items returned. This parameter has a default value of 50 and can not exceed it. If specified limit exceeds 50, 50 is used.

## Working with u64 and u128 numbers

All the numbers larger than 2^32 are stored as hexadecimal strings with a string length prefix as defined below.

### U64String

All number types in range \(2^32 ... 2^64\) are encoded as a string using the following format:

```text
"MN...N"
```

where:

* `M` – one char with hex \(length-1\) of hexadecimal representation of a number.
* `N...N` – hexadecimal lowercased representation of a number.

Number examples:

* `11` – 1
* `12` – 2
* `1a` – 10
* `2ff` – 255
* `fffffffffffffffff` - 0xffffffffffffffff = 2^\(2 \* 16\)-1 = 2^32-1

### U1024String

All number types in range \(2^64 ... 2^1024\] are encoded as a string using the following format:

```text
"MMN...N"
```

where:

* `MM` – two chars with hex \(length-1\) of hexadecimal representation of a number.
* `N...N` – hexadecimal lowercased representation of a number.

Number examples:

* `011` – 1
* `012` – 2
* `01a` – 10
* `02ff` – 255
* `ffff..ff` - 2^\(2 \*256\) - 1 = 2^512 - 1

### GraphQL interaction

Within the GraphQL filter fields these numbers can be represented as follows:

1. Hexadecimal number string starting with a `0x` prefix for example `0x10f0345ae`. Note that you can specify characters for hexadecimal numbers in any letter case, for example `0xa4b` is the same as a `0xA4B`.
2. Decimal number representation, for example `100034012`.

GraphQL always returns large numbers as a hexadecimal number string starting with a `0x` prefix; for example `0xa34ff`. Note that GraphQL always returns characters in lower case.

To interact with large numbers in GraphQl one needs to use `BigInt(value)` where `value` can be both hexadecimal with `0x` prefix or a decimal number.


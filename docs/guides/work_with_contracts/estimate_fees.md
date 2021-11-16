# Estimate Fees

Find out how to estimate fees before executing contracts on-chain

## When you may need to estimate fees?

* Before deploy to find out how much it will cost
* Before contract execution to find out how much will it cost
* To calculate how much will it cost to store a contract for a period of time

## Estimate deploy and run fees

The steps you need to complete to estimate fees of deploy and run:

1. Encode a message you will locally execute with [encode\_message](../../reference/types-and-methods/mod\_abi.md#encode\_message)
2. Download account boc the same way you did in [Run ABI Get method](run\_abi\_get\_method.md)
3. Run the message on the downloaded state with [run\_executor](../../reference/types-and-methods/mod\_tvm.md#run\_executor) function
4. Retrieve `result.fees` object

## Fees object detailed explanation

Here is the structure of fees object.

Basically here you need only 3 fields:

* `total_account_fees` - in fact, this name is misleading. it is actually total validator fees collected from the transaction. But for backward compatibility we do not rename it. It includes `storage_fee`, `in_msg_fwd_fee`, `gas_fee`, and other fees that are being delivered to the validators of this block and not visible here (like fees for external message, `action_fees`, etc - but we will add them in future).
* `total_output` - sum of all values of internal messages being sent.
* `out_msgs_fwd_fee` - sum of all fees, that will be collected by the validators of the destination shard. In originating transaction, they are just charged from the account balance or from messages' values (during message creation - depending on the contract logic), and are not included in `total_account_fees` for the current block validators.

So we can sum it up in one formula:

```graphql
{
  "in_msg_fwd_fee": 42881000,
  "storage_fee": 1,
  "gas_fee": 11813000,
  "out_msgs_fwd_fee": 0,
  "total_account_fees": 54694001,
  "total_output": 0
}
```

## Estimate storage fee

Use [calc\_storage\_fee](../../reference/types-and-methods/mod\_utils.md#calc\_storage\_fee) method.

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run\_executor](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run\_executor)

**AppKit**

[https://github.com/tonlabs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js](https://github.com/tonlabs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js)

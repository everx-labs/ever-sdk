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

* `in_msg_fwd_fee`: _bigint_ – Deprecated.\
  Left for backward compatibility. Does not participate in account transaction fees calculation.
* `storage_fee`: _bigint_ – Fee for account storage
* `gas_fee`: _bigint_ – Fee for processing
* `out_msgs_fwd_fee`: _bigint_ – Deprecated.\
  Contains the same data as total\_fwd\_fees field. Deprecated because of its confusing name, that is not the same with GraphQL API Transaction type's field.
* `total_account_fees`: _bigint_ – Deprecated.\
  This is the field that is named as `total_fees` in GraphQL API Transaction type. `total_account_fees` name is misleading, because it does not mean account fees, instead it means\
  validators total fees received for the transaction execution. It does not include some forward fees that account\
  actually pays now, but validators will receive later during value delivery to another account (not even in the receiving\
  transaction).\
  Because of all of this, this field is not interesting for those who wants to understand\
  the real account fees, this is why it is deprecated and left for backward compatibility.
* `total_output`: _bigint_ – Deprecated because it means total value sent in the transaction, which does not relate to any fees.
* `ext_in_msg_fee`: _bigint_ – Fee for inbound external message import.
* `total_fwd_fees`: _bigint_ – Total fees the account pays for message forwarding
* `account_fees`: _bigint_ – Total account fees for the transaction execution. Compounds of storage\_fee + gas\_fee + ext\_in\_msg\_fee + total\_fwd\_fees

```graphql
type TransactionFees = {
    in_msg_fwd_fee: bigint,
    storage_fee: bigint,
    gas_fee: bigint,
    out_msgs_fwd_fee: bigint,
    total_account_fees: bigint,
    total_output: bigint,
    ext_in_msg_fee: bigint,
    total_fwd_fees: bigint,
    account_fees: bigint
}
```

## Estimate storage fee

Use [calc\_storage\_fee](../../reference/types-and-methods/mod\_utils.md#calc\_storage\_fee) method.

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run\_executor](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run\_executor)

**AppKit**

[https://github.com/tonlabs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js](https://github.com/tonlabs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js)

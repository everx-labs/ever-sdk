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
4. Retrieve `result.fees` object. You need **`result.fees.account_fees`** for total account fees value.

## Fees object detailed explanation

Here is the structure of fees object.

* `storage_fee`: _bigint_ – Fee for account storage
* `gas_fee`: _bigint_ – Fee for processing
* `in_msg_fwd_fee`: _bigint_ – Deprecated. Left for backward compatibility.&#x20;
* `ext_in_msg_fee`: _bigint_ – Fee for inbound external message import.
* `total_fwd_fees`: _bigint_ – Total fees the account pays for message forwarding
* **`account_fees`: **_**bigint**_** – Total account fees for the transaction execution. Compounds of storage\_fee + gas\_fee + ext\_in\_msg\_fee + total\_fwd\_fees**

#### Deprecated fields. Left for backward compatibility.

* `out_msgs_fwd_fee`: _bigint_ – Deprecated. Left for backward compatibility.
* `total_account_fees`: _bigint_ – Deprecated.  Left for backward compatibility.\
  This is the field that is named as `total_fees` in GraphQL API Transaction type. `total_account_fees` name is misleading, because it does not mean account fees, instead it means validators total fees received for the transaction execution. It does not include some forward fees that account actually pays now, but validators will receive later during value delivery to another account (not even in the receiving transaction but along the way of a chain of transactions processing).\
  Because of all of this, this field is not interesting for those who want to understand\
  the real account fees, this is why it is deprecated and left for backward compatibility.
* `total_output`: _bigint_ – Deprecated. Left for backward compatibility.&#x20;

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

[https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/run\_executor](https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/run\_executor)

**AppKit**

[https://github.com/everx-labs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js](https://github.com/everx-labs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js)

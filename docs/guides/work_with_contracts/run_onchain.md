# Run on-chain

Learn how to run methods of a contract on-chain

* [About run](run\_onchain.md#about-run)
* [Run on-chain](run\_onchain.md#run-on-chain-1)
  * [Pattern 1. Run in 1 step: `process_message` method](run\_onchain.md#pattern-1-run-in-1-step-process\_message-method)
  * [Pattern 2. Run in 3 steps: `encode_message` -> `send_message` -> `wait_for_transaction`](run\_onchain.md#pattern-2--run-in-3-steps-encode\_message----send\_message---wait\_for\_transaction)

> [See the API reference](../../reference/types-and-methods/modules.md).

Core api is more flexible than [AppKit](https://github.com/everx-labs/ever-appkit-js) and you can perform a lot of complex logic using it. But you will need to write more code with it as well :)

You need to [define the contract in your node.js](add\_contract\_to\_your\_app.md) application before running its methods.

Full sample: [https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/hello-wallet](https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/hello-wallet)

## About run

Run operation consists of few steps:

1. Creating a message;
2. Sending a message;
3. Receiving the message completion transaction;
4. Receiving external messages created by `return` function;
5. Decoding the messages bodies according to the ABI.

## Run on-chain

### Pattern 1. Run in 1 step: `process_message` method

The [process\_message](../../reference/types-and-methods/mod\_processing.md#process\_message) method runs ABI-compatible contract method on-chain and returns the result transaction along with the external messages bocs, and decoded external messages bodies.

`process_message` performs all the run operation steps inside.

> **Note**: This is an easy to implement pattern to run, but we do not recommend to use it because of network inconsistency and application crash probability. For more reliable approach - use the Pattern 2.

For example, a contract has such method:

```javascript
// Function setting set value to state variable timestamp and returning it
function touchMe() public alwaysAccept returns (uint32) {
    timestamp = uint32(now);
    return timestamp;
}
```

Client method call:

```javascript
// Encode the message with `touch` function call
const params = {
    send_events: false,
    message_encode_params: {
        address,
        abi,
        call_set: {
            function_name: 'touch',
            input: {}
        },
        // There is no pubkey key check in the contract
        // so we can leave it empty. Never use this approach in production
        // because anyone can call this function
        signer: { type: 'None' }
    }
}
// Call `touch` function
let response = await client.processing.process_message(params);
console.log(`Сontract run transaction with output ${response.decoded.output}, ${response.transaction.id}`);
```

See the full sample in the repository with sdk samples:

[https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/hello-wallet](https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/hello-wallet)

### Pattern 2. Run in 3 steps: `encode_message` -> `send_message` -> `wait_for_transaction`

To run a contract method, first you need to create a run message:

```javascript
// Encode the message with `touch` function call
const params = {
        abi = {
            type: 'Contract',
            value: contract.abi
        },
        address,
        call_set: {
            function_name: 'touch',
            input: {}
        },
        // There is no pubkey key check in the contract
        // so we can leave it empty. Never use this approach in production
        // because anyone can call this function
        signer: { type: 'None' }
}

// Create external inbound message with `touch` function call
const encode_touch_result = await client.abi.encode_message(params);
```

Now the message should be sent. `sendMessage` method returns the the last block created in the shard before the message was sent.

```javascript
// Send `touch` call message to the network
// See more info about `send_message` here  
// https://github.com/everx-labs/ever-sdk/blob/master/docs/mod_processing.md#send_message
shard_block_id = (await client.processing.send_message({
    message: encode_touch_result.message,
    send_events: true
    },logEvents
)).shard_block_id;
console.log(`Touch message was sent.`);
```

After the message was sent we need to wait for the transaction starting from the last shard block:

```javascript
// Monitor message delivery. 
// See more info about `wait_for_transaction` here  
// https://github.com/everx-labs/ever-sdk/blob/master/docs/mod_processing.md#wait_for_transaction
const touch_processing_result = await client.processing.wait_for_transaction({
    abi = {
        type: 'Contract',
        value: contract.abi
    },
    message: encode_touch_result.message,
    shard_block_id: shard_block_id,
    send_events:true
    },
    logEvents
)
console.log(`Touch transaction: ${JSON.stringify(touch_processing_result.transaction,null,2)}`);
console.log(`Touch fees: ${JSON.stringify(touch_processing_result.fees,null,2)}`);
```



## Calculate account fees

Retrieve `result.fees` object of `process_message` or `wait_for_transaction`. You need **`result.fees.account_fees`** for total account fees value.

### Fees object detailed explanation

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

See the full example in sdk samples repository: [https://github.com/everx-labs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index\_pattern2.js](https://github.com/everx-labs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index\_pattern2.js)

Check out [AppKit documentation](https://docs.everos.dev/appkit-js/guides/run\_onchain) for this use case.

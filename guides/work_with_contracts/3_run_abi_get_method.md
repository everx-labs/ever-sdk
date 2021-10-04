# Run ABI Get Method

## Run ABI Get Method

Run ABI compatible get methods

* [Run get method](3_run_abi_get_method.md#run-get-method)
* [Source code](3_run_abi_get_method.md#source-code)

## Run get method

With low level SDK get method is executed in 3 steps:

1. Download the latest Account State \(BOC\)
2. Encode message that calls the method
3. Execute the message locally on the downloaded state:

Here is the sample that executes the get method `getTimestamp` on the latest account's state.

1. account boc is downloaded with `query_collection` function
2. message that calls contract's function `getTimestamp` is encoded with `encode_message` function
3. message is executed on local TVM with `run_local` method

```text
    // Execute the get method `getTimestamp` on the latest account's state
    // This can be managed in 3 steps:
    // 1. Download the latest Account State (BOC)
    // 2. Encode message
    // 3. Execute the message locally on the downloaded state

    const [account, message] = await Promise.all([
        // Download the latest state (BOC)
        // See more info about query method here 
        // https://github.com/tonlabs/TON-SDK/blob/master/docs/mod_net.md#query_collection
        client.net.query_collection({
            collection: 'accounts',
            filter: { id: { eq: address } },
            result: 'boc'
        })
        .then(({ result }) => result[0].boc)
        .catch(() => {
            throw Error(`Failed to fetch account data`)
        }),
        // Encode the message with `getTimestamp` call
        client.abi.encode_message({
            abi,
            address,
            call_set: {
                function_name: 'getTimestamp',
                input: {}
            },
            signer: { type: 'None' }
        }).then(({ message }) => message)
    ]);

    // Execute `getTimestamp` get method  (execute the message locally on TVM)
    // See more info about run_tvm method here 
    // https://github.com/tonlabs/TON-SDK/blob/master/docs/mod_tvm.md#run_tvm
    response = await client.tvm.run_tvm({ message, account, abi });
    console.log('Contract reacted to your getTimestamp:', response.decoded.output);
```

## Source code

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index.js)

Check out [AppKit documentation](https://github.com/tonlabs/appkit-js/blob/main/docs/guides/3_run_abi_get_method_with_appkit.md) for this use case.


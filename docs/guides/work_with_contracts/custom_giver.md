# Using custom Giver

Working with DevNet is similar to working with SE except you usually don't have any predeployed giver in DevNet. So you need to fund your contracts manually or deploy your own giver, which you will be able to use the same way as in SE. Deploying your own giver can be useful, if you need to deploy many contracts or need to frequently redeploy and test contract after subsequent midification.

You can find how to deploy your own Giver [here](github.com/tonlabs/tondev/blob/main/docs/work_with_devnet.md).

In this guide we will show you how you can use your own Giver in the core SDK. 

First of all, let's initialize giver's address, keys and ABI. All the data in this example are used from TON OS SE giver, substitute them by your Giver's data.

```javascript
// Giver address TON OS SE
const giverAddress = '0:b5e9240fc2d2f1ff8cbb1d1dee7fb7cae155e5f6320e585fcc685698994a19a5';
// Giver keys on TON OS SE
const giverSigner = signerKeys({
    "public": "2ada2e65ab8eeab09490e3521415f45b6e42df9c760a639bcf53957550b25a16",
    "secret": "172af540e43a524763dd53b26a066d472a97c4de37d5498170564510608250c3"
});
// Giver ABI on TON OS SE
const giverAbi = abiContract({
    'ABI version': 2,
    header: ['time', 'expire'],
    functions: [
        {
            name: 'sendTransaction',
            inputs: [
                { 'name': 'dest', 'type': 'address' },
                { 'name': 'value', 'type': 'uint128' },
                { 'name': 'bounce', 'type': 'bool' }
            ],
            outputs: []
        },
        {
            name: 'getMessages',
            inputs: [],
            outputs: [
                {
                    components: [
                        { name: 'hash', type: 'uint256' },
                        { name: 'expireAt', type: 'uint64' }
                    ],
                    name: 'messages',
                    type: 'tuple[]'
                }
            ]
        },
        {
            name: 'upgrade',
            inputs: [
                { name: 'newcode', type: 'cell' }
            ],
            outputs: []
        },
        {
            name: 'constructor',
            inputs: [],
            outputs: []
        }
    ],
    data: [],
    events: []
});
```

Next, implement a function to request tokens from the Giver:

```javascript
// Requesting test tokens from the Giver
async function getTokensFromGiver(client, account, amount) {
    // Execute method `sendTransaction` of the TON OS SE Giver v2 contract:
    const processingResult = await client.processing.process_message({
        send_events: false,
        message_encode_params: {
            address: giverAddress,
            abi: giverAbi,
            call_set: {
                function_name: 'sendTransaction',
                input: {
                    dest: account,
                    value: amount,
                    bounce: false
                }
            },
            signer: giverSigner,
        },
    });

    // Wait until all messages have been produced:
    const transactionTree = await client.net.query_transaction_tree({
        in_msg: processingResult.transaction.in_msg,
    });

    // Make additional checks to ensure success:
    if (transactionTree.transactions.length !== 2) {
        throw `Something went wrong during requesting funds from the Giver: there must be 2 transactions, but actual count is ${transactionTree.transactions.length}`;
    }

    if (transactionTree.transactions[1].account_addr !== account) {
        throw "Something went wrong during requesting funds from the Giver: 2nd transaction's account address must be " + account;
    }
}
```

Now you can use your new function for funding your contracts before deployment:

```javascript
async function main(client) {
    // Define contract ABI in the Application 
    // See more info about ABI type here https://github.com/tonlabs/TON-SDK/blob/master/docs/mod_abi.md#abi
    const abi = {
        type: 'Contract',
        value: HelloWallet.abi
    }
    // Generate an ed25519 key pair
    const helloKeys = await client.crypto.generate_random_sign_keys();
    
    // Prepare parameters for deploy message encoding
    // See more info about `encode_message` method parameters here https://github.com/tonlabs/TON-SDK/blob/master/docs/mod_abi.md#encode_message
    const deployOptions = {
        abi,
        deploy_set: {
            tvc: HelloWallet.tvc,
            initial_data: {}
        },
        call_set: {
            function_name: 'constructor',
            input: {}
        },
        signer: {
            type: 'Keys',
            keys: helloKeys
        }
    }

    // Encode deploy message
    // Get future `Hello` contract address from `encode_message` result
    // to sponsor it with tokens before deploy
    const { address } = await client.abi.encode_message(deployOptions);
    console.log(`Future address of the contract will be: ${address}`);

    // Request contract deployment funds form the Giver
    await getTokensFromGiver(client, address, 1_000_000_000);
    console.log(`Tokens were transfered from giver to ${address}`);

    // Deploy `hello` contract
    // See more info about `process_message` here  
    // https://github.com/tonlabs/TON-SDK/blob/master/docs/mod_processing.md#process_message
    await client.processing.process_message({
        send_events: false,
        message_encode_params: deployOptions
    });

    console.log(`Hello contract was deployed at address: ${address}`);
}
```

## Sample Source Code

Full sample: [https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/custom-giver/](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/custom-giver/)

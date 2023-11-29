# Quick start (JavaScript)

Create your first DApp and run it on local blockchain

* [Prerequisites](quick\_start.md#prerequisites)
* [Prepare development environment](quick\_start.md#prepare-development-environment)
* [Start local node (SE)](quick\_start.md#start-local-node-se)
* [Install demo application](quick\_start.md#install-demo-application)
* [What the script does](quick\_start.md#what-the-script-does)
* [Sample code](quick\_start.md#sample-code)
* [Run it!](quick\_start.md#run-it)
* [Source code](quick\_start.md#source-code)

## Prerequisites

Node.js latest version installed [Docker](https://www.docker.com/get-started) latest version installed

## Prepare development environment

Install [EVERDEV CLI](https://github.com/tonlabs/everdev) that will help you easily start local node, compile your contracts, install demo projects and create new empty projects.

```shell
$ npm install -g everdev
```

## Start local node (SE)

We will run our test on local blockchain for testing ([Evernode SE](https://github.com/tonlabs/evernode-se), start it with this command (docker should be launched).

```
$ everdev se start
```

## Install demo application

Create a working folder. Then create a node.js demo project with EVERDEV

```
$ everdev js demo hello-wallet
$ cd hello-wallet
$ npm i
```

## What the script does

The script implements the following logic:

1. Links the project with Node.js [Ever-SDK](https://github.com/tonlabs/ever-sdk) binary. If you plan to use JS SDK in Web, link it with Wasm binary. Read more [here](https://github.com/tonlabs/ever-sdk-js).
2. `TONClient` instance is created and initialized with [Evernode SE](https://github.com/tonlabs/evernode-se) ("[http://localhost](http://localhost)", local blockchain) endpoint. See the list of other available [endpoints](https://docs.everplatform.dev/reference/graphql-api/networks).
3. Future address is calculated from the code and data of the contract (data includes signing keys)
4. &#x20;Flag `useGiver: true` allows to sponsor deploy with Evernode SE giver that is hard coded as the default Account giver. [You can re-assign it to your own giver](guides/work\_with\_contracts/deploy.md#transfer-funds-to-the-future-address).

## Sample code

{% tabs %}
{% tab title="Core API Implementation" %}
```javascript
(async () => {
    try {
        // Generate an ed25519 key pair
        const walletKeys = await client.crypto.generate_random_sign_keys();

        // Calculate future wallet address.
        const walletAddress = await calcWalletAddress(walletKeys);

        // Send some tokens to `walletAddress` before deploy
        await getTokensFromGiver(walletAddress, 1_000_000_000);

        // Deploy wallet
        await deployWallet(walletKeys);

        // Get wallet's account info and print balance
        const accountState = await getAccount(walletAddress);
        console.log("Hello wallet balance is", accountState.balance)

        // Run account's get method `getTimestamp`
        let walletTimestamp = await runGetMethod('getTimestamp', walletAddress, accountState.boc );
        console.log("`timestamp` value is", walletTimestamp)

        // Perform 2 seconds sleep, so that we receive an updated timestamp
        await new Promise(r => setTimeout(r, 2000));
        // Execute `touch` method for newly deployed Hello wallet contract
        // Remember the logical time of the generated transaction
        let transLt = await runOnChain(walletAddress, "touch");

        // Run contract's get method locally after account is updated
        walletTimestamp = await runGetMethodAfterLt('getTimestamp', walletAddress, transLt);
        console.log("Updated `timestamp` value is", walletTimestamp)

        // Send some tokens from Hello wallet to a random account
        // Remember the logical time of the generated transaction
        const destAddress = await genRandomAddress();
        transLt = await sendValue(walletAddress, destAddress, 100_000_000, walletKeys);

        console.log('Normal exit');
        process.exit(0);
    } catch (error) {
        if (error.code === 504) {
            console.error(
                [
                    'Network is inaccessible. You have to start Evernode SE using `everdev se start`',
                    'If you run SE on another port or ip, replace http://localhost endpoint with',
                    'http://localhost:port or http://ip:port in index.js file.',
                ].join('\n'),
            );
        } else {
            console.error(error);
            process.exit(1);
        }
    }
})();

async function calcWalletAddress(keys) {
    // Get future `Hello`Wallet contract address from `encode_message` result
    const { address } = await client.abi.encode_message(buildDeployOptions(keys));
    console.log(`Future address of Hello wallet contract is: ${address}`);
    return address;
}

function buildDeployOptions(keys) {
    // Prepare parameters for deploy message encoding
    // See more info about `encode_message` method parameters here:
    // https://github.com/tonlabs/ever-sdk/blob/master/docs/reference/types-and-methods/mod_abi.md#encode_message
    const deployOptions = {
        abi: {
            type: 'Contract',
            value: HelloWallet.abi,
        },
        deploy_set: {
            tvc: HelloWallet.tvc,
            initial_data: {},
        },
        call_set: {
            function_name: 'constructor',
            input: {},
        },
        signer: {
            type: 'Keys',
            keys,
        },
    };
    return deployOptions;
}

// Request funds from Giver contract
async function getTokensFromGiver(dest, value) {
    console.log(`Transfering ${value} tokens from giver to ${dest}`);

    const params = {
        send_events: false,
        message_encode_params: {
            address: GIVER_ADDRESS,
            abi: abiContract(GIVER_ABI),
            call_set: {
                function_name: 'sendTransaction',
                input: {
                    dest,
                    value,
                    bounce: false,
                },
            },
            signer: {
                type: 'Keys',
                keys: GIVER_KEYS,
            },
        },
    };
    await client.processing.process_message(params);
    console.log('Success. Tokens were transfered\n');
}

async function deployWallet(walletKeys) {
    // Deploy `Hello wallet` contract
    // See more info about `process_message` here:
    // https://github.com/tonlabs/ever-sdk/blob/master/docs/reference/types-and-methods/mod_processing.md#process_message
    console.log('Deploying Hello wallet contract');
    await client.processing.process_message({
        send_events: false,
        message_encode_params: buildDeployOptions(walletKeys),
    });
    console.log('Success. Contract was deployed\n');
}

async function runOnChain(address, methodName) {
    // Encode the message with external call
    const params = {
        send_events: false,
        message_encode_params: {
            address,
            abi: {
                type: 'Contract',
                value: HelloWallet.abi,
            },
            call_set: {
                function_name: methodName,
                input: {},
            },
            signer: signerNone(),
        },
    };
    console.log(`Calling ${methodName} function`);
    const response = await client.processing.process_message(params);
    const { id, lt } = response.transaction;
    console.log('Success. TransactionId is: %s\n', id);
    return lt;
}


// Sometimes it is needed to execute getmethods after on-chain calls.
// This means that the downloaded account state should have the changes made by the on-chain call. 
// To ensure it, we need to remember the transaction lt (logical time) of the last call
// and then wait for the account state to have lt > the transaction lt. 
// Note that account.last_trans_lt is always bigger than transaction.lt because this field stores the end lt of transaction interval
// For more information about transaction lt interval read TON Blockchain spec https://test.ton.org/tblkch.pdf P. 4.2.1
async function waitForAccountUpdate(address, transLt) {
    console.log('Waiting for account update');
    const startTime = Date.now();
    const account = await client.net.wait_for_collection({
        collection: 'accounts',
        filter: {
            id: { eq: address },
            last_trans_lt: { gt: transLt },
        },
        result: 'boc',
    });
    const duration = Math.floor((Date.now() - startTime) / 1000);
    console.log(`Success. Account was updated, it took ${duration} sec.\n`);
    return account;
}


async function getAccount(address) {

    // `boc` or bag of cells - native blockchain data layout. Account's boc contains full account state (code and data) that
    // we will  need to execute get methods.
    const query = `
        query {
          blockchain {
            account(
              address: "${address}"
            ) {
               info {
                balance(format: DEC)
                boc
              }
            }
          }
        }`
    const {result}  = await client.net.query({query})
    const info = result.data.blockchain.account.info
    return info
}
async function runGetMethod(methodName, address, accountState) {
    // Execute the get method `getTimestamp` on the latest account's state
    // This can be managed in 3 steps:
    // 1. Download the latest Account State (BOC) 
    // 2. Encode message
    // 3. Execute the message locally on the downloaded state

    // Encode the message with `getTimestamp` call
    const { message } = await client.abi.encode_message({
        // Define contract ABI in the Application
        // See more info about ABI type here:
        // https://github.com/tonlabs/ever-sdk/blob/master/docs/reference/types-and-methods/mod_abi.md#abi
        abi: {
            type: 'Contract',
            value: HelloWallet.abi,
        },
        address,
        call_set: {
            function_name: methodName,
            input: {},
        },
        signer: { type: 'None' },
    });

    // Execute `getTimestamp` get method  (execute the message locally on TVM)
    // See more info about run_tvm method here:
    // https://github.com/tonlabs/ever-sdk/blob/master/docs/reference/types-and-methods/mod_tvm.md#run_tvm
    console.log('Run `getTimestamp` get method');
    const response = await client.tvm.run_tvm({
        message,
        account: accountState,
        abi: {
            type: 'Contract',
            value: HelloWallet.abi,
        },
    });
    return response.decoded.output
}


async function runGetMethodAfterLt(methodName, address, transLt) {
    // Wait for the account state to be more or equal the spesified logical time
    const accountState = await waitForAccountUpdate(address, transLt).then(({ result }) => result.boc);
    const result = await runGetMethod(methodName, address, accountState);
    return result;
    
}

async function sendValue(address, dest, amount, keys) {
    // Encode the message with `sendValue` function call
    const sendValueParams = {
        send_events: false,
        message_encode_params: {
            address,
            // Define contract ABI in the Application
            // See more info about ABI type here:
            // https://github.com/tonlabs/ever-sdk/blob/master/docs/reference/types-and-methods/mod_abi.md#abi
            abi: {
                type: 'Contract',
                value: HelloWallet.abi,
            },
            call_set: {
                function_name: 'sendValue',
                input: {
                    dest,
                    amount,
                    bounce: false,
                },
            },
            signer: signerKeys(keys),
        },
    };
    console.log(`Sending ${amount} tokens to ${dest}`);
    // Call `sendValue` function
    const response = await client.processing.process_message(sendValueParams);
    console.log('Success. Target account will recieve: %d tokens\n', response.fees.total_output);
    return response.transaction.lt;
}
```
{% endtab %}

{% tab title="AppKit API Implementation" %}
```javascript
async function main(client) {
    // Generate an ed25519 key pair for new account
    const keys = await TonClient.default.crypto.generate_random_sign_keys();

    const helloAcc = new Account(HelloWallet, {
        signer: signerKeys(keys),
        client,
    });

    const address = await helloAcc.getAddress();
    console.log(`Future address of the contract will be: ${address}`);

    // Request contract deployment funds form a local Evernode SE giver
    // not suitable for other networks.
    // Deploy `hello` contract.
    await helloAcc.deploy({ useGiver: true });
    console.log(`Hello contract was deployed at address: ${address}`);

    // Call `touch` function on-chain
    // On-chain execution can be done with `run` function. 
    let response = await helloAcc.run("touch", {});
    console.log(`touch execution transaction is  ${response.transaction.id}`);

    // Read local variable `timestamp` with a get method `getTimestamp`
    // This can be done with `runLocal` function. The execution of runLocal is performed off-chain and does not 
    // cost any gas.
    response = await helloAcc.runLocal("getTimestamp", {});
    console.log("getTimestamp value:", response.decoded.output)

    // Send some money to the random address
    const randomAddress = 
        "0:" + 
        Buffer.from((await client.crypto.generate_random_bytes({length: 32})).bytes, "base64").toString("hex");
    response = await helloAcc.run("sendValue", {
        dest: randomAddress,
        amount: 100_000_000, // 0.1 token
        bounce: true, // delivery will fail and money will be returned back because the random account does not exist.
    });
    console.log(`The tokens were sent, but soon they will come back because bounce = true and destination address does not exist`);
}

(async () => {
    const client = new TonClient({
        network: {
            // Local Evernode-SE instance URL here
            endpoints: ["http://localhost"]
        }
    });
    try {
        console.log("Hello localhost!");
        await main(client);
        process.exit(0);
    } catch (error) {
        if (error.code === 504) {
            console.error(`Network is inaccessible. You have to start Evernode SE using \`everdev se start\`.\n If you run SE on another port or ip, replace http://localhost endpoint with http://localhost:port or http://ip:port in index.js file.`);
        } else {
            console.error(error);
        }
    }
    client.close();
})();
```
{% endtab %}
{% endtabs %}

## Run it!

### Core API

Run:

```
$ node core
```

You will see the result of core.js file execution. Core.js file demonstrades core ever-sdk api. It is the same for all ever-sdk bindings.

```
node core
Future address of Hello wallet contract is: 0:1863addf562c5ab98f3761787458e47406675379a4dc6eb36042ba84bde5cb8d
Transfering 1000000000 tokens from giver to 0:1863addf562c5ab98f3761787458e47406675379a4dc6eb36042ba84bde5cb8d
Success. Tokens were transfered

Deploying Hello wallet contract
Success. Contract was deployed

Hello wallet balance is 986483999
Run `getTimestamp` get method
`timestamp` value is {
  value0: '0x000000000000000000000000000000000000000000000000000000006373fbb4'
}
Calling touch function
Success. TransactionId is: 1a34fbfc336ff8212793077c68bff9f49c6c3f270492afa55ca616ef40b22bec

Waiting for account update
Success. Account was updated, it took 0 sec.

Run `getTimestamp` get method
Updated `timestamp` value is {
  value0: '0x000000000000000000000000000000000000000000000000000000006373fbb6'
}
Sending 100000000 tokens to 0:9f98e8de89e19093145afe134017a783daf8bac5dee04b8810c57a348020764c
Success. Target account will recieve: 99000000 tokens

Normal exit
```

### Appkit API

Run:

```
$ node appkit
```

You will see the result of appkit.js file execution. Appkit.js demonstrates high level Appkit package.

```
Hello localhost!
Future address of the contract will be: 0:5aab70b197897e47ee65faca0ebe24244fd1373d31de2ae39aca28029e0f3469
Hello contract was deployed at address: 0:5aab70b197897e47ee65faca0ebe24244fd1373d31de2ae39aca28029e0f3469
touch execution transaction is  495d0b02905ac541b54407283e52155fbfcbcc804a82ca40d5da96e433fe2f6b
getTimestamp value: {
  value0: '0x000000000000000000000000000000000000000000000000000000006373fa68'
}
The tokens were sent, but soon they will come back because bounce = true and destination address does not exist
```

## Source code

You can find full source code of this sample here

[https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet](https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet)

# Quick Start

Create your first DApp and run it on local blockchain

- [Prerequisites](#prerequisites)
- [Prepare development environment](#prepare-development-environment)
- [Start local node (SE)](#start-local-node-se)
- [Install demo application](#install-demo-application)
- [Run it!](#run-it)
- [Detailed sample explanation](#detailed-sample-explanation)
- [Source code](#source-code)

# Prerequisites

Node.js latest version installed
[Docker](https://www.docker.com/get-started) latest version installed

# Prepare development environment

Install [TONDEV CLI](https://github.com/tonlabs/tondev) that will help you easily start local node, compile your contracts, install demo projects and create new empty projects. 

    $ npm install -g tondev

# Start local node (SE)

We will run our test on local blockchain for testing ([TON OS SE](https://github.com/tonlabs/tonos-se), start it with this command (docker should be launched). 

    $ tondev se start

# Install demo application

Create a working folder. Then create a node.js demo project with TONDEV

    $ tondev js demo hello-wallet

# Run it!

    $ cd hello-wallet
    $ npm i
    $ npm start

You will see the result of execution:

    Hello localhost TON!
    Future address of the contract will be: 0:c891d93061c4b3d7f77833b075674af527a6c3fce6fbb7dd1814b453842a5a84
    Hello contract was deployed at address: 0:c891d93061c4b3d7f77833b075674af527a6c3fce6fbb7dd1814b453842a5a84
    Contract run transaction with output null, de5fddc8814b350e6b6c8876c411935462eb1b88a96f9fa5dc176f021959ff38
    Contract reacted to your getTimestamp: {
      value0: '0x0000000000000000000000000000000000000000000000000000000060df2c77'
    }
    Contract reacted to your sendValue, target address will recieve: 99000000

# Detailed sample explanation

Here is the script code and contract definition included in this sample. 

> This script is using high level sdk functions.  Check out the same [sample implemented with low-level sdk](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/hello-wallet). 

Read below a short description of what the script does.  Look for more detailed information in other guides.

Script code:

```
const { Account } = require("@tonclient/appkit");
const { TonClient, signerKeys } = require("@tonclient/core");
const { libNode } = require("@tonclient/lib-node");

const { HelloWallet } = require("./HelloWallet.js")

// Link the platform-dependable TON-SDK binary with the target Application in Typescript
// This is a Node.js project, so we link the application with `libNode` binary
// from `@tonclient/lib-node` package
// If you want to use this code on other platforms, such as Web or React-Native,
// use  `@tonclient/lib-web` and `@tonclient/lib-react-native` packages accordingly
// (see README in  https://github.com/tonlabs/ton-client-js )
TonClient.useBinaryLibrary(libNode);

/**
 *
 * @param client {TonClient}
 * @returns {Promise<void>}
 */
async function main(client) {
    // Generate an ed25519 key pair for new account
    const keys = await TonClient.default.crypto.generate_random_sign_keys();

    const helloAcc = new Account(HelloWallet, {
        signer: signerKeys(keys),
        client,
    });

    const address = await helloAcc.getAddress();
    console.log(`Future address of the contract will be: ${address}`);

    // Request contract deployment funds form a local TON OS SE giver
    // not suitable for other networks.
    // Deploy `hello` contract.
    await helloAcc.deploy({ useGiver: true });
    console.log(`Hello contract was deployed at address: ${address}`);

    // Call `touch` function
    let response = await helloAcc.run("touch", {});
    console.log(`Contract run transaction with output ${response.decoded.output}, ${response.transaction.id}`);

    // Read local variable `timestamp` with a get method `getTimestamp`
    // This can be done with `runLocal` function. The execution of runLocal is performed off-chain and does not 
    // cost any gas.
    response = await helloAcc.runLocal("getTimestamp", {});
    console.log("Contract reacted to your getTimestamp:", response.decoded.output)

    // Send some money to the random address
    const randomAddress = 
        "0:" + 
        Buffer.from(
            (await client.crypto.generate_random_bytes({length: 32})).bytes,
            "base64"
        ).toString("hex");
    response = await helloAcc.run("sendValue", {
        dest: randomAddress,
        amount: 100_000_000, // 0.1 token
        bounce: true,
    });
    console.log("Contract reacted to your sendValue, target address will recieve:", response.fees.total_output);
}

(async () => {
    const client = new TonClient({
        network: {
            // Local TON OS SE instance URL here
            endpoints: ["http://localhost"]
        }
    });
    try {
        console.log("Hello localhost TON!");
        await main(client);
        process.exit(0);
    } catch (error) {
        if (error.code === 504) {
            console.error(`Network is inaccessible. You have to start TON OS SE using \`tondev se start\`.\n If you run SE on another port or ip, replace http://localhost endpoint with http://localhost:port or http://ip:port in index.js file.`);
        } else {
            console.error(error);
        }
    }
    client.close();
})();
```

HelloWallet.js:

```
module.exports = {
    HelloWallet: {
        abi: {
            "ABI version": 2,
            "header": ["time", "expire"],
            "functions": [
                {
                    "name": "constructor",
                    "inputs": [
                    ],
                    "outputs": [
                    ]
                },
                {
                    "name": "renderHelloWorld",
                    "inputs": [
                    ],
                    "outputs": [
                        {"name":"value0","type":"bytes"}
                    ]
                },
                {
                    "name": "touch",
                    "inputs": [
                    ],
                    "outputs": [
                    ]
                },
                {
                    "name": "getTimestamp",
                    "inputs": [
                    ],
                    "outputs": [
                        {"name":"value0","type":"uint256"}
                    ]
                },
                {
                    "name": "sendValue",
                    "inputs": [
                        {"name":"dest","type":"address"},
                        {"name":"amount","type":"uint128"},
                        {"name":"bounce","type":"bool"}
                    ],
                    "outputs": [
                    ]
                },
                {
                    "name": "timestamp",
                    "inputs": [
                    ],
                    "outputs": [
                        {"name":"timestamp","type":"uint32"}
                    ]
                }
            ],
            "data": [
            ],
            "events": [
            ]
        },
        tvc: "te6ccgECGQEAAtgAAgE0AwEBAcACAEPQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgBCSK7VMg4wMgwP/jAiDA/uMC8gsWBQQYApAh2zzTAAGOEoECANcYIPkBWPhCIPhl+RDyqN7TPwH4QyG58rQg+COBA+iogggbd0CgufK0+GPTHwH4I7zyudMfAds8+Edu8nwJBgE6ItDXCwOpOADcIccA3CHXDR/yvCHdAds8+Edu8nwGAiggghBU1r0Yu+MCIIIQaLVfP7vjAgsHAiggghBoF+U1uuMCIIIQaLVfP7rjAgoIAlgw+EJu4wD4RvJzf/hm0fhC8uBl+EUgbpIwcN74Qrry4Gb4APgj+GrbPH/4ZwkTAUrtRNDXScIBio4acO1E0PQFcPhqgED0DvK91wv/+GJw+GNw+GbiFQFSMNHbPPhKIY4cjQRwAAAAAAAAAAAAAAAAOgX5TWDIzssfyXD7AN5/+GcVBFAgghAfnWSDuuMCIIIQNzEuRbrjAiCCEDtj1H664wIgghBU1r0YuuMCEhEPDAJsMNHbPCGOJyPQ0wH6QDAxyM+HIM6NBAAAAAAAAAAAAAAAAA1Na9GIzxbMyXD7AJEw4uMAf/hnDRMBAogOABRoZWxsb1dvcmxkA1Yw+EJu4wD6QZXU0dD6QN/XDX+V1NHQ03/f1wwAldTR0NIA39HbPOMAf/hnFRATAFT4RSBukjBw3vhCuvLgZvgAVHEgyM+FgMoAc89AzgH6AoBrz0DJcPsAXwMCJDD4Qm7jANH4APgj+GrbPH/4ZxUTA3gw+EJu4wDR2zwhjigj0NMB+kAwMcjPhyDOjQQAAAAAAAAAAAAAAAAJ+dZIOM8Wy//JcPsAkTDi4wB/+GcVFBMAKPhK+Eb4Q/hCyMv/yz/KAMsfye1UAAT4SgAo7UTQ0//TP9IA0x/R+Gr4Zvhj+GICCvSkIPShGBcAFnNvbCAwLjQ2LjANAAA=",
    }
}
```

HelloWallet.js contains artifacts received from contract compilation. Read more [here](installation/3_add_contract_to_your_app.md). 

The script implements the following logic:

1. Links the project with Node.js [TON-SDK](https://github.com/tonlabs/TON-SDK) binary. If you plan to use JS SDK in Web, link it with Wasm binary. Read more [here](https://github.com/tonlabs/ton-client-js). 
2. `TONClient` instance is created and initialized with [TON OS SE](https://github.com/tonlabs/tonos-se) ("http://localhost", local blockchain) endpoint.  See the list of other available [endpoints](../docs/api/1_networks.md).
3. new `Account` type object is initialized with a generated key pair, `HelloWallet` object and client object. Read more about Account initialization [here](work_with_contracts/1_deploy.md).
4. Future address of the contract is calculated and printed to console.
5. `deploy` function is used to deploy the contract.  Flag `useGiver: true` allows to sponsor deploy with TON OS SE giver that is hard coded as the default Account giver. [You can re-assign it to your own giver](work_with_contracts/1_deploy.md#transfer-funds-to-the-future-address). 
6. `run` function is used to execute contract function `touch` on-chain.
7. `runLocal` function is used to tun get method `getTimestamp` is executed off-chain - locally to read the contract's data. 
8. `run` function is used to execute contract function `sendValue` on-chain to send some tokens to a randomly generated address.


# Source code

You can find source code of this sample here 

https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet



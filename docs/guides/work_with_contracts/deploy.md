# Deploy

Find out how to deploy a contract to Everscale Blockchain with SDK

* [About deploy](deploy.md#about-deploy)
* [Deploy steps](deploy.md#deploy-steps)
* [Observe deploy parameters](deploy.md#observe-deploy-parameters)
* [Prepare a pair of keys](deploy.md#prepare-a-pair-of-keys)
* [\[Optional\] Prepare initial contract data](deploy.md#optional-prepare-initial-contract-data)
* [Specify constructor](deploy.md#specify-constructor)
* [Calculate the future contract address](deploy.md#calculate-the-future-contract-address)
* [Transfer funds to the future address](deploy.md#transfer-funds-to-the-future-address)
* [Deploy](deploy.md#deploy-1)
  * [Pattern 1. Deploy in 1 step: `process_message` method](deploy.md#pattern-1-deploy-in-1-step-process\_message-method)
  * [Pattern 2. Deploy in 3 steps: `encode_message` -> `send_message` -> `wait_for_transaction`](deploy.md#pattern-2-deploy-in-3-steps-encode\_message----send\_message---wait\_for\_transaction)
* [Deploy a contract from another contract](deploy.md#deploy-a-contract-from-another-contract)
* [Sample Source Code](deploy.md#sample-source-code)

> [See the API reference](../../reference/types-and-methods/modules.md).

Core api is more flexible than [AppKit](https://github.com/tonlabs/ever-appkit-js), and you can perform a lot of complex logic using it. But you will need to write more code with it as well:)

You need to define the contract in your node.js application before deploy.

See the "[Add Contract to your App](add\_contract\_to\_your\_app.md)" guide to find out how to do it.

## About deploy

Deploy operation means that you upload contract code and initial data to the blockchain.

The address of the contract can be calculated before deploy and it depends on code and data.

To deploy a contract you need to sponsor its address, so that deploy fee will be paid out of these funds.

## Deploy steps

Below is the sequence of steps you need to complete:

1. Observe deploy parameters
2. Prepare a pair of keys
3. \[optional] Prepare Initial contract data
4. Calculate future address of the contract
5. Transfer funds to the future address
6. Deploy

Let's take a look at every step.

## Observe deploy parameters

Here is the structure, that is used to prepare initial contract state for deploy. It will define the future contract address.

`tvc` - As we discussed at the [previous step](add\_contract\_to\_your\_app.md), contract compilation artifact, converted into base64.

`workchain_is` - Target workchain where the contract will be deployed. By default is 0.

`initial_data` - structure that contains public contract data fields with assigned values that need to be included into the initial state.

`initial_pubkey` - owner pubkey. If not specified, pubkey is taken from `Signer`. If you manage to specify contract owner directly in `tvc`, then it will have priority above both `initial_pubkey` and `Signer`.

```javascript
type DeploySet = {
    tvc: string,
    workchain_id?: number,
    initial_data?: any,
    initial_pubkey?: string
}
```

## Prepare a pair of keys

To deploy you will need a pair of keys to sign the deployment message. The keys are used during address generation, so the future contract address depends partially on the key as well.

In fact, keys are optional for deploy, but, if you want to be the contract owner then specify Signer object for that. [Read more about Signer object in Reference](../../reference/types-and-methods/mod\_abi.md#Signer).

If you want to make another person owner of the contract then specify their pubkey via `DeploySet.initial_pubkey`.

In this guide we will use `crypto.generate_random_sign_keys` function to generate a key pair.

```javascript
const helloKeys = await client.crypto.generate_random_sign_keys();
```

> **Note**: Please store these keys in a secure environment as you will need them to calculate the future address and deploy the contract. You might also need it to call contract functions in the future.

## \[Optional] Prepare initial contract data

Placing data into storage always affects the future contract address.

This is useful for scenarios when you need to deploy several contracts with identical contract code (e.g. wallets) to different addresses.

If each contract is deployed with its own unique pair of keys, you do not need to use `DeploySet.initial_data` as the addresses will be generated using the corresponding key pair each time.

If, however, all the contracts need to have the same contract code and have to be deployed with the same pair of keys but to different addresses, you may use placing data into storage through `DeploySet.initial_data` to vary the addresses during [address calculation](deploy.md#calculate-the-future-contract-address).

To be used as `initParams` variables must be declared as `static` in the contract.

## Specify constructor

Constructor - is any contract function that will be called upon deploy. It will not influence contract address.

It, together with its parameters, is specified in the [CallSet](../../reference/types-and-methods/mod\_abi.md#callset) structure:

```javascript
type CallSet = {
    function_name: string,
    header?: FunctionHeader,
    input?: any
}
```

`function_name` - function name that will be called upon deploy. Usually, it is called `constructor`.

`header` - optional, can be specified for some complex use cases. [Read more here](message\_expiration.md).

`input` - object with function parameters.

## Calculate the future contract address

Everscale blockchain requires every contract to have a positive token balance before deployment. The contract pays for the initial deployment message reducing account balance.

We will create deploy message and get future contract address from it.

You can either specify or not specify call\_set here, it will not influence the address.

```javascript
const abi = {
    type: 'Contract',
    value: contract.abi
}
// Generate an ed25519 key pair
const helloKeys = await client.crypto.generate_random_sign_keys();

// Prepare parameters for deploy message encoding
// See more info about `encode_message` method parameters here https://github.com/tonlabs/ever-sdk/blob/master/docs/mod_abi.md#encode_message
const deployOptions = {
    abi,
    deploy_set: {
        tvc: contractPackage.tvcInBase64,
        initial_data: {}
    },
      // can be ommited here for address calculation
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
```

> **Note**: Deployment requires the same pair of keys used for address calculation.
>
> **Note**: `constructorParams` do not affect the address. Only `initParams` and the keys do. If you only plan to calculate address, you can omit this parameter.

## Transfer funds to the future address

Now that you know the address, you need to transfer the initial funds to it from your wallet or the Giver.

> **Note**: Evernode SE offers a pre-deployed giver. When in real networks, you need to use your wallet for this or deploy your own giver. We have separated guides of [deployment](https://docs.everos.dev/everdev/guides/work-with-devnet) and [usage](custom\_giver.md) of your own giver.

```javascript
// Request contract deployment funds form a local Evernode SE giver
// not suitable for other networks
await get_tokens_from_giver(client, address);
console.log(`Tokens were transfered from giver to ${address}`);
```

## Deploy

At this point, you're all set for deployment.

> **Note**: Insufficient balance will result in a failed deployment.
>
> **Note**: Everscale Blockchain does not guarantee a 100% external inbound messages success rate. There is a possibility that deployment will fail. To ensure success, please check for a completed transaction.

### Pattern 1. Deploy in 1 step: `process_message` method

This is an easy to implement pattern to deploy for server-side development and tests, but we do not recommend to use it in web and mobile applications because of network inconsistency and application crash probability. For more reliable approach - use the Pattern 2.

Method \`process\_message performs all the deploy steps sequentially in one method: create message, send it, wait for transaction and decode external outbound message generated by contract RETURN operator. The time limit can be set up in Client configuration. The default time setting is 40 seconds. You can learn more [here](message\_expiration.md).

```javascript
// Deploy `hello` contract
// See more info about `process_message` here  
// https://github.com/tonlabs/ever-sdk/blob/master/docs/mod_processing.md#process_message
await client.processing.process_message({
    send_events: false,
    message_encode_params: deployOptions
});

console.log(`Hello contract was deployed at address: ${address}`);
```

See the full example in sdk samples repository:

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index.js)

Check out how to run contract's methods in the next [section](run\_onchain.md).

### Pattern 2. Deploy in 3 steps: `encode_message` -> `send_message` -> `wait_for_transaction`

Any interaction with and within the Everscale blockchain is implemented via messages.

To deploy a contract, first you need to create a deploy message that will include all the initial data:

```javascript
// Prepare parameters for deploy message encoding
// See more info about `encode_message` method parameters here https://github.com/tonlabs/ever-sdk/blob/master/docs/mod_abi.md#encode_message
const deployOptions = {
    abi: {
        type: 'Contract',
        value: contract.abi
    },
    deploy_set: {
        tvc: contractPackage.tvcInBase64,
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
const encode_deploy_result = await client.abi.encode_message(deployOptions);
```

Now the message should be sent. `send_message` method returns the the last block created in the shard before the message was sent.

`send_events: false` flag specifies that we will not monitor message delivery events. Soon there will be a guide about message processing monitoring.

```javascript
// Send deploy message to the network
// See more info about `send_message` here  
// https://github.com/tonlabs/ever-sdk/blob/master/docs/mod_processing.md#send_message
var shard_block_id;
shard_block_id = (await client.processing.send_message({
    message: encode_deploy_result.message,
    send_events: false
    },
)).shard_block_id;
console.log(`Deploy message was sent.`);
```

After the message was sent we need to wait for the transaction:

```javascript
// Monitor message delivery. 
// See more info about `wait_for_transaction` here  
// https://github.com/tonlabs/ever-sdk/blob/master/docs/mod_processing.md#wait_for_transaction
const deploy_processing_result = await client.processing.wait_for_transaction({
  abi: {
        type: 'Contract',
        value: contract.abi
    },
    message: encode_deploy_result.message,
    shard_block_id: shard_block_id,
    send_events:false // we do not want to monitor processing events
    }
)
console.log(`Deploy transaction: ${JSON.stringify(deploy_processing_result.transaction,null,2)}`);
console.log(`Deploy fees: ${JSON.stringify(deploy_processing_result.fees,null,2)}`);
console.log(`Hello contract was deployed at address: ${address}`);
```

If `wait_for_transaction` fails with 507 error - you can perform a retry. In all other cases retry may end up with double spends.

See the full example in sdk samples repository:

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index\_pattern2.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/index\_pattern2.js)

Check out how to run contract's methods in the next [section](run\_onchain.md).

## Deploy a contract from another contract

Often developers need to deploy one contract from another contract and try to pass `tvc` file as a cell parameter for contract to use.

This is not a correct way.

**To be able to deploy a contract from another contract you need to pass the contract's code as a cell parameter.**

You can get it several ways:

* You can use SDK function [get\_code\_from\_tvc](../../reference/types-and-methods/mod\_boc.md#get\_code\_from\_tvc) of `boc` module and retrieve the code's boc from tvc file.
* You can download another account with the same `code_hash`'s `boc` from graphql, using [query\_collection](../queries\_and\_subscriptions/query\_collection.md) method and parse it with [parse\_account](../../reference/types-and-methods/mod\_boc.md#parse\_account) function to retrieve the 'code' from the result (function will return the same JSON account object as graphql).

> We encourage you not to download the account's code from API directly because we will deprecate this field in the future for resource economy.

## Sample Source Code

Full sample: [https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/hello-wallet/)

Check out [AppKit documentation](https://docs.everos.dev/appkit-js/guides/deploy) for this use case.

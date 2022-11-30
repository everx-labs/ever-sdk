# Quick start (JavaScript)

Create your first DApp and run it on local blockchain

* [Prerequisites](quick\_start.md#prerequisites)
* [Prepare development environment](quick\_start.md#prepare-development-environment)
* [Start local node (SE)](quick\_start.md#start-local-node-se)
* [Install demo application](quick\_start.md#install-demo-application)
* [Run it!](quick\_start.md#run-it)
* [Detailed sample explanation](quick\_start.md#detailed-sample-explanation)
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

## Run it!

### Explore Core API

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

### Explore Appkit API

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

## Explanations

The script implements the following logic:

1. Links the project with Node.js [Ever-SDK](https://github.com/tonlabs/ever-sdk) binary. If you plan to use JS SDK in Web, link it with Wasm binary. Read more [here](https://github.com/tonlabs/ever-sdk-js).
2. `TONClient` instance is created and initialized with [Evernode SE](https://github.com/tonlabs/evernode-se) ("[http://localhost](http://localhost)", local blockchain) endpoint. See the list of other available [endpoints](https://docs.everos.dev/ever-platform/reference/graphql-api/networks).
3. Future address is calculated from the code and data of the contract (data includes signing keys)
4. &#x20;Flag `useGiver: true` allows to sponsor deploy with Evernode SE giver that is hard coded as the default Account giver. [You can re-assign it to your own giver](guides/work\_with\_contracts/deploy.md#transfer-funds-to-the-future-address).

## Source code

You can find source code of this sample here

[https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet](https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet)

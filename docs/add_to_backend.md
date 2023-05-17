---
description: >-
  This document describes the various ways to accomplish the most important
  tasks of running a backend project that supports EVER.
---

# Add EVER to your backend

## Introduction

This document describes the various ways to accomplish the most important tasks of running a backend application that supports EVER.

There are a few different ways to accomplish the necessary tasks:

* Blockchain access may be set up either through the [Evercloud](https://docs.evercloud.dev/products/evercloud/get-started) or through your own supernode - the [DApp server](https://docs.evercloud.dev/products/dapp-server-ds).
* User account management can be accomplished either through the [everdev](https://docs.everos.dev/everdev/) command line tool or integrate into your backend with  EVER-SDK client libraries. Both of these approaches are compatible with either of the blockchain access setups.

## Setting up Blockchain Access

### Using Evercloud

Using [Evercloud](https://docs.evercloud.dev/products/evercloud/get-started) allows you to work with TVM blockchains without having to run your own node. Everdev and SDK can connect to it, as if it were a regular node. It has the same API as a node, and provides all neede capabilities.

This page lists the [cloud endpoints](https://docs.evercloud.dev/products/evercloud/networks-endpoints). To get access credentials go through this [guide](https://docs.evercloud.dev/products/evercloud/get-started).

Whenever you have to specify a network endpoint in the examples given below, use the endpoints and credentials you receive in the [Evercloud dashboard](https://dashboard.evercloud.dev/projects).

{% hint style="info" %}
Note: We recommend testing out the full setup on the developer network first.
{% endhint %}

### Using DApp Server&#x20;

If you prefer to run your own node, you may set up your own [DApp server](https://docs.evercloud.dev/products/dapp-server-ds). It is a client supernode, that may be set up on your own servers and provide full access to TVM networks. To connect to it with Everdev or SDK, it needs to have a domain name and a DNS record. You can specify its URL whenever you have to set the network in the examples given below.

Get the setup scripts in this repository: [https://github.com/tonlabs/evernode-ds](https://github.com/tonlabs/evernode-ds)

#### 1. System Requirements&#x20;

| Configuration | CPU (cores) | RAM (GiB) | Storage (GiB) | Network (Gbit/s) |
| ------------- | ----------- | --------- | ------------- | ---------------- |
| Recommended   | 24          | 128       | 2000          | 1                |

NVMe SSD disks are recommended for storage.

{% hint style="info" %}
For simplicity, all services are deployed on one host and the system requirements for it are high, so it makes sense to distribute services across different servers.\
After understanding this installation process, you can easily customize it for yourself.

[itgoldio/everscale-dapp-server](https://github.com/itgoldio/everscale-dapp-server): Consider this project if you prefer deployment via Ansible.
{% endhint %}

#### 2.1 Prerequisites

* Host OS: Linux (all scripts tested on Ubuntu 20.04).
* DApp server is accessed via HTTPS, so your server must have a fully qualified domain name.\
  A self-signed certificate will be received on start-up and will be renewed automatically.
* Installed Git, Docker Engine, Docker CLI, Docker Compose v2 or later.

#### 2.2 Configuration

**2.2.1 Set variables**

Check `configure.sh` and set at least these environment variables:

* NETWORK\_TYPE
* EVERNODE\_FQDN
* LETSENCRYPT\_EMAIL

**2.2.2 Generate credentials to access the ArangoDB web interface**

Generate credentials (usernames and password) for basic authentication and update `.htpasswd` file.\
You can generate it by running `htpasswd -nb <name> <password>`

**2.2.3 Run configuration script**

```
$ ./configure.sh
```

This script creates `./deploy` directory

#### 2.3 Deployment

Run `./up.sh`.

After the script completes normally (it takes 30 min approx.), the node starts synchronizing its state, which can take several hours.\
Use the following command to check the progress:

```
    docker exec rnode /ton-node/tools/console -C /ton-node/configs/console.json --cmd getstats
```

Script output example:

```
tonlabs console 0.1.286
COMMIT_ID: 5efe6bb8f2a974ba0e6b1ea3e58233632236e182
BUILD_DATE: 2022-10-17 02:32:44 +0300
COMMIT_DATE: 2022-08-12 00:22:07 +0300
GIT_BRANCH: master
{
	"sync_status":	"synchronization_finished",
	"masterchainblocktime":	1665988670,
	"masterchainblocknumber":	9194424,
	"node_version":	"0.51.1",
	"public_overlay_key_id":	"S4TaVdGitzTApe7GFCj8DbuRIkVEbg+ODzBxhQGIUG0=",
	"timediff":	6,
	"shards_timediff":	6,
     ----%<---------------------
}
```

If the `timediff` parameter is less than 10 seconds, synchronization with masterchain is complete.\
`"sync_status": "synchronization finished"` means synchronization with workchains is complete

## Setting up Wallet Account

Currently we can recommend the [SetcodeMultisig](https://github.com/EverSurf/multisig2) contract for use in user accounts. It is well tested and secure, supports multiple custodians, and can be set up to require several independent signatures for any transfers.&#x20;

### Using CLI tool&#x20;

[Everdev](https://docs.everos.dev/everdev/), the command line tool for development on the Everscale blockchain, allows to write scripts to deploy any smart contracts to the blockchain, call all contract methods, sign transactions, and generally manage an account.

It works with both Evercloud and DApp server.

#### 1. Install Everdev&#x20;

```sh
$ npm install -g everdev
```

It requires [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) to be installed.

If you experience any problems with installation, check out our [troubleshooting](https://docs.everos.dev/everdev/troubleshooting) section.

#### 2. Configure network connection&#x20;

Everdev has a built-in [network](https://docs.everos.dev/everdev/command-line-interface/network-tool) tool to manage your networks and access credentials.

**Using Evercloud endpoints**

Add your Evercloud endpoint to everdev and make it default:

```
everdev network add networkName <your-evercloud-endpoint>
everdev network default networkName
```

**Using DApp Server endpoint**

* If you are setting up a connection via your own DApp server, user the following command to add it to the network list (it will be named `dappserver`).

```shell
everdev network add dappserver <your_dapp_server_endpoint>
```

To set your `dappserver` network as default, use the following command:

```sh
everdev network default dappserver
```

#### 3. Set a giver contract on your network

On Everscale, you need to sponsor a contract address in advance to be able to deploy the contract.

Everdev provides a way to set an account of your choice as a giver for deployment operations, so you will not have to do a separate step of sending tokens to a new contract address every time you deploy something. This contract can some multisig wallet, for example your [Surf](https://ever.surf/) account.

**Note**: To work automatically, the giver contract should have only one custodian.

To set it up, first save the custodian keys of your giver account into a signer that will be used to sign giver transactions (Learn more about the signer tool [here](https://docs.everos.dev/everdev/command-line-interface/signer-tool)):

```sh
everdev signer add giver_sign signer_secret_key_or_seed_phrase_in_quotes
```

Then add the giver address specifying the signer to be used with it.

```sh
everdev network giver network_name giver_address --signer giver_sign --type giver_type
```

Where

`giver_type` is the type of the giver contract you selected (GiverV1 | GiverV2 | GiverV3 | SafeMultisigWallet | MsigV2| SetcodeMultisigWallet)

{% hint style="warning" %}
We recommend using [Multisig 2.0 ](https://github.com/EverSurf/multisig2)as giver, for that use `MsigV2` giver\_type.&#x20;
{% endhint %}

#### 4. Get wallet account contract files&#x20;

We recommend using[ Multisig 2.0](https://github.com/EverSurf/multisig2) contracts as a wallet. They can be found [here](https://github.com/EverSurf/multisig2). In this guide SetcodeMultisig specifically is used.

Download the contract files and place them in the working folder. Direct links to its files are as follows:

**.tvc** - Compiled contract code

SetcodeMultisig.tvc direct link:

{% embed url="https://github.com/EverSurf/multisig2/raw/main/build/SetcodeMultisig.tvc" %}

**.abi.json** - application binary interface, describing the functions of the contract

SetcodeMultisig.abi.json direct link:

{% embed url="https://raw.githubusercontent.com/EverSurf/multisig2/main/build/SetcodeMultisig.abi.json" %}

Execute the commands of the following steps from the directory with the contract files.

#### 5. Create wallet account signer&#x20;

To generate your wallet account signer enter the following command:

```shell
everdev signer generate wallet_signer
```

Or, if you already have a seed phrase, add it like this:

```
everdev signer add "your-seed-phrase-here"
```

To deploy multisig wallet account you will need to specify the public key of the signer. To view it, use the following command:

```sh
everdev signer info wallet_signer
```

The keys will be displayed in terminal (if you imported the seed phrase, it will be displayed here as well):

```sh
{
    "name": "wallet_signer",
    "description": "",
    "keys": {
        "public": "8f8779e7c1944b133a423df96d06ae770c996f19d63438dbf2f569a29529b248",
        "secret": "ce57d2666d0d2c737a03ca4e6cfa38c5ca088dbcef43eb0353896feca8aea2a5"
    }
}

```

Usually a single owner (with a single signer) per wallet account is optimal for any tasks that require automation. However, it is possible to set up accounts with multiple owners. In this case, each of the owners has to generate their own signer and provide their public keys to the deployer. Also, the signer used to deploy the account doesn't have to be among its owners.

#### 6. Deploy the wallet account contract to blockchain

Use the following command for a simple one-owner account:

```shell
everdev contract deploy SetcodeMultisig.abi.json constructor --signer wallet_signer --input owners:[<owner_public_key>],reqConfirms:1,lifetime:3600 --value 1000000000
```

Where&#x20;

`value` parameter is the amount of nanotokens to be spent on deployment (can be omitted, in which case 10 tokens from giver will be spent)

`owner_public_key` is usually [the public key](add\_to\_backend.md#6.-create-deposit-account-signer) of `wallet_signer` in the form `0x...`.

`lifetime` - time in seconds that a transaction in multi-owner accounts will persits and be available for signing by other owners. For a simple multi owner account may be set to any value, as it will be executed immediately anyway.

Example:

{% code overflow="wrap" %}
```sh
everdev contract deploy SetcodeMultisig.abi.json constructor --signer wallet_signer --input owners:[0x8f8779e7c1944b133a423df96d06ae770c996f19d63438dbf2f569a29529b248],reqConfirms:1,lifetime:3600 --value 1000000000
```
{% endcode %}

For more complex cases (multiple owners etc.) view Everdev contract tool [docs](https://docs.everos.dev/everdev/command-line-interface/contract-management).

Once the contract is deployed, its address will be displayed in terminal.

```sh
everdev contract deploy SetcodeMultisig.abi.json constructor --signer wallet_signer --input owners:[0x3da1909b7a4bd11fd9a1d79ca9713a9a8645880e0a7a12f9691c68e95d56fe75],reqConfirms:1,lifetime:3600 --value 10000000000

Configuration

  Network: dev (devnet.evercloud.dev)
  Signer:  wallet_signer (public 8f8779e7c1944b133a423df96d06ae770c996f19d63438dbf2f569a29529b248)

Address:   0:95c35b94e98c1b5c7716a9129ed5bb0798c8c336465fd8d1eb0d385e3d969494 (calculated from TVC and signer public)

Parameters of constructor:

  owners (uint256[]): ["0x3da1909b7a4bd11fd9a1d79ca9713a9a8645880e0a7a12f9691c68e95d56fe75"]
  reqConfirms (uint8): "1"
  lifetime (uint32): "3600"

Deploying...
Contract is deployed at address: 0:95c35b94e98c1b5c7716a9129ed5bb0798c8c336465fd8d1eb0d385e3d969494

```

### Using SDK&#x20;

You may integrate above described process of wallet account deployment into your backend code. The functionality is supported in SDK.

A sample is available in [this repository](https://github.com/tonlabs/sdk-samples/tree/master/demo/msig-wallet) and an overview is given below.

{% hint style="info" %}
[Bindings](https://docs.everos.dev/ever-sdk/#community-bindings) for a large number of languages have been developed for SDK.&#x20;
{% endhint %}

Note, that similar to the Everdev approach described above, you have to sponsor a user account before deploying contract code. The sample assumes you use the devnet faucet of [Evercloud Dashboard](https://dashboard.evercloud.dev/), where you can request test tokens to the contract address generated by the sample. In a production environment you may set up a giver to sponsor your contract deployment operations. An example of such a set up can be found in this [sample](https://github.com/tonlabs/sdk-samples/tree/master/demo/hello-wallet).

The recommended [SetcodeMultisig](https://github.com/tonlabs/sdk-samples/blob/master/demo/msig-wallet/contract/SetcodeMultisig.sol) contract is used.

```typescript

 async function main(client: TonClient) {
    // 
    // 1. ------------------ Deploy multisig wallet --------------------------------
    // 
    // Generate a key pair for the wallet to be deployed
    const keypair = await client.crypto.generate_random_sign_keys();

    // TODO: Save generated keypair!
    console.log('Generated wallet keys:', JSON.stringify(keypair))
    console.log('Do not forget to save the keys!')

    // To deploy a wallet we need its TVC and ABI files
    const msigTVC: string =
        readFileSync(path.resolve(__dirname, "../contract/SetcodeMultisig.tvc")).toString("base64")
    const msigABI: string =
        readFileSync(path.resolve(__dirname, "../contract/SetcodeMultisig.abi.json")).toString("utf8")

    // We need to know the future address of the wallet account,
    // because its balance must be positive for the contract to be deployed
    // Future address can be calculated by encoding the deploy message.
    // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_abi#encode_message

    const messageParams: ParamsOfEncodeMessage = {
        abi: { type: 'Json', value: msigABI },
        deploy_set: { tvc: msigTVC, initial_data: {} },
        signer: { type: 'Keys', keys: keypair },
        processing_try_index: 1
    }

    const encoded: ResultOfEncodeMessage = await client.abi.encode_message(messageParams)

    const msigAddress = encoded.address

    console.log(`You can topup your wallet from dashboard at https://dashboard.evercloud.dev`)
    console.log(`Please send >= ${MINIMAL_BALANCE} tokens to ${msigAddress}`)
    console.log(`awaiting...`)

    // Blocking here, waiting for account balance changes.
    // It is assumed that at this time you go to dashboard.evercloud.dev
    // and replenish this account.
    let balance: number
    for (; ;) {
        // The idiomatic way to send a request is to specify 
        // query and variables as separate properties.
        const getBalanceQuery = `
                query getBalance($address: String!) {
                    blockchain {
                    account(address: $address) {
                            info {
                            balance
                        }
                    }
                }
            }
            `
        const resultOfQuery: ResultOfQuery = await client.net.query({
            query: getBalanceQuery,
            variables: { address: msigAddress }
        })

        const nanotokens = parseInt(resultOfQuery.result.data.blockchain.account.info?.balance, 16)
        if (nanotokens > MINIMAL_BALANCE * 1e9) {
            balance = nanotokens / 1e9
            break
        }
        // TODO: rate limiting
        await sleep(1000)
    }
    console.log(`Account balance is: ${balance.toString(10)} tokens`)

    console.log(`Deploying wallet contract to address: ${msigAddress} and waiting for transaction...`)

    // This function returns type `ResultOfProcessMessage`, see: 
    // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_processing#process_message
    let result: ResultOfProcessMessage = await client.processing.process_message({
        message_encode_params: {
            ...messageParams,  // use the same params as for `encode_message`,
            call_set: {        // plus add `call_set`
                function_name: 'constructor',
                input: {
                    owners: [`0x${keypair.public}`],
                    reqConfirms: 1,
                    lifetime: 3600
                }
            },
        },
        send_events: false,
    })
    console.log('Contract deployed. Transaction hash', result.transaction?.id)
    assert.equal(result.transaction?.status, 3)
    assert.equal(result.transaction?.status_name, "finalized")

    //
```



## Monitoring  transactions

Lets assume we need to reliably know when customers receive or transfer funds from their wallets. Samples of transaction [pagination](https://github.com/tonlabs/sdk-samples/tree/master/demo/paginate-transactions) and [subscription](https://github.com/tonlabs/sdk-samples/tree/master/demo/subscribe-transactions) are available in the samples repository. An overview of the relevant parts is given below.

In these samples JS SDK is used. [Bindings](https://docs.everos.dev/ever-sdk/#community-bindings) for a large number of languages have been developed for SDK.&#x20;

### Pagination

The [pagination](https://github.com/tonlabs/sdk-samples/tree/master/demo/paginate-transactions) sample queries and displays transactions in workchain 0 (workchain where simple transfers happen, -1 workchain is masterchain where you can find service transactions and validator transactions) from the beginning. We can get all the transaction and filter by account addresses on the backend side.

```typescript
   async function main(client: TonClient) {
    // In this example, we want the query to return 2 items per page.
    const itemsPerPage = 25

    // Pagination connection pattern requires a cursor, which will be set latter
    let cursor: string = undefined

    // The idiomatic way to send a request is to specify 
    // query and variables as separate properties.
    const transactionsQuery = `
        query listTransactions($cursor: String, $count: Int) {
            blockchain {
                transactions(
                    workchain: 0
                    first: $count
                    after: $cursor
                 ) {
                    edges {
                        node { 
                            id
                            balance_delta
                            account_addr
                            # other transaction fields
                     }
                    }
                    pageInfo { hasNextPage endCursor }
                }
            }
        }`

    for (; ;) {
        const queryResult: ResultOfQuery = await client.net.query({
            query: transactionsQuery,
            variables: {
                count: itemsPerPage,
                cursor
            }
        });
        const transactions = queryResult.result.data.blockchain.transactions;

        for (const edge of transactions.edges) {
            console.log("Transaction id:", edge.node.id);
        }
        if (transactions.pageInfo.hasNextPage === false) {
            break;
        }
        // To read next page we initialize the cursor:
        cursor = transactions.pageInfo.endCursor;
        // TODO: rate limiting
        await sleep(1000);
    }

}
console.log("Getting all transactions in workchain 0 from the beginning/")
console.log("Most likely this process will never end, so press CTRL+C to interrupt it")
main(client)
    .then(() => {
        process.exit(0)
    })
    .catch(error => {
        console.error(error);
        process.exit(1);
    })



// This helper function is used for limiting request rate
function sleep(ms: number) { return new Promise(r => setTimeout(r, ms)) }
```



### Subscription

[Subscription](https://github.com/tonlabs/sdk-samples/tree/master/demo/subscribe-transactions) sample subscribes to new transactions of the listed accounts and lists them as they appear.

```typescript
async function main() {
    try {
        const client = new TonClient({ network: { endpoints: [endpoint] } })

        const queryText = `
            subscription my($list: [String!]!){
                transactions(
                    filter: {account_addr: { in: $list }}
                ) {
                    id
                    account_addr
                    balance_delta
                }
            }`

        // use `client.net.unsubscribe({ handle })` to close subscription
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        const { handle } = await client.net.subscribe(
            {
                subscription: queryText,
                variables: { list: addressList }
            },
            responseHandler,
        );
        console.log("Subscribed to transactions of accounts:", JSON.stringify(addressList))
        console.log("Press CTRL+C to interrupt it")

    } catch (error) {
        if (error.code === 504) {
            console.error('Network is inaccessible.');
        } else {
            console.error(error);
        }
        process.exit(1);
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function responseHandler(params: any, responseType: number) {
    // Tip: Always wrap the logic inside responseHandler in a try-catch block
    // or you will be surprised by non-informative errors due to the context
    // in which the handler is executed
    try {
        if (responseType === 100 /* GraphQL data received */) {
            if (params?.result) {
                console.log(params.result);
            }

        } else {
            // See full list of error codes here:
            // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_net#neterrorcode
            console.error(params, responseType);
        }
    } catch (err) {
        console.log(err);
    }
}
```

You may test out the demo application running this process on the developer network by cloning the [sdk-samples](https://github.com/tonlabs/sdk-samples) repository, creating a project in  [https://dashboard.evercloud.dev](https://dashboard.evercloud.dev), exporting the API endpoint as an environment variable:

```
export ENDPOINT=https://devnet.evercloud.dev/<your_project_id>/graphql
```

&#x20;and running the following command in the `/demo/subscribe-transactions` folder:

```shell
npm run subscribe-tr
```

{% hint style="warning" %}
Not all transactions that are successful are valid transfers and not all transactions that are aborted actually failed. Read [here](https://docs.everscale.network/arch/transactions#how-to-determine-a-successful-transaction) how to understand which transfers are successful transfers and which are not.
{% endhint %}

## Withdrawing from wallet accounts&#x20;

The specific function that is used to withdraw the funds depends on the contract chosen for the wallet account. Examples provided below are applicable for the [SetcodeMultisig](https://github.com/EverSurf/multisig2) contract.

### Using CLI tool

Command line `Everdev` tool may be used to automate withdrawals from wallet account in your scripts.

{% hint style="danger" %}
If the user made a mistake in the destination address, and has no control over it, these funds will be lost forever. If the account does not exist, and the user makes mistakes deploying it after the funds are transferred, they may end up being lost as well.&#x20;
{% endhint %}

So, to perform a simple transfer from a single-owner user account to any specified account, we should make sure that it is already deployed, by setting `bounce` flag to true. If the account does not exist, funds will return back.

{% code overflow="wrap" %}
```shell
everdev contract run SetcodeMultisig.abi.json sendTransaction --address <wallet_account_address> --signer wallet_signer --input dest:recipient_address,value:50000000000,bounce:true,flags:3,payload:""
```
{% endcode %}

`<wallet_account_address>` - address of the user account. Example: 0:7bf2b2ec80371601f854bff9ed0a1171714d922c8bfc86d39e67a7e3a41b2176

`wallet_signer` - name of the user account owner signer

`recipient_address` - raw address of the recipient smart contract. Example: 255a3ad9dfa8aa4f3481856aafc7d79f47d50205190bd56147138740e9b177f3

`value`: - amount of tokens to transfer in nanotokens (Example: value:10000000000 sets up a transfer of 10 tokens).

`bounce` - use true to transfer funds only to deployed accounts.

`flags` - use 3 for a simple transfer.

`payload` - use "" for simple transfer.

{% hint style="info" %}
**Note**: To transfer all funds from the account use `sendTransaction` method with flag `130` and value `0.`
{% endhint %}

{% code overflow="wrap" %}
```shell
everdev contract run SetcodeMultisig.abi.json --address <wallet_account_address> sendTransaction --signer wallet_signer --input dest:recipient_address,value:0,bounce:true,flags:130,payload:""
```
{% endcode %}

Example of regular withdrawal transaction on a single-owner multisig:

```shell
everdev contract run SetcodeMultisig.abi.json sendTransaction --signer wallet_signer --input dest:665a62042aff317ba3f32e36b712b0f4a9d35277dd76dc38c9762cc6421681cf,value:500000000000,bounce:false,flags:3,payload:""

Configuration

  Network: dev (devnet.evercloud.dev)
  Signer:  wallet_signer (public 3da1909b7a4bd11fd9a1d79ca9713a9a8645880e0a7a12f9691c68e95d56fe75)

Address:   0:95c35b94e98c1b5c7716a9129ed5bb0798c8c336465fd8d1eb0d385e3d969494

Parameters of sendTransaction:

  dest (address): "665a62042aff317ba3f32e36b712b0f4a9d35277dd76dc38c9762cc6421681cf"
  value (uint128): "500000000000"
  bounce (bool): "false"
  flags (uint8): "3"
  payload (cell): ""


Running...

Execution has finished with result:
{
    "transaction": {
        "json_version": 8,
        "id": "cbeb7f8b1aa7ac89439d9c6772b699a7c042215cef090f206ecc8b21bb230fc9",
        "boc": "te6ccgECDwEAArcAA7d5XDW5TpjBtcdxapEp7VuweYyMM2Rl/Y0esNOF49lpSUAAAOakEwMsEkhiumuyPgwNDWXdmNBHsr9g6Y6XgsntCG/AQxbMH/mAAADmo/0T8BZCW1XwAFSAICQ36AUEAQIPDE/GHimDxEADAgBvyY9CQExRYUAAAAAAAAQAAAAAAARz+2ts0g+y9Ais9VbZ65O+4BourUTTYoPq+tvoLxFJpECQJNQAnUZPYxOIAAAAAAAAAABYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAgnI1Aq0GL7DtfZycdkRDzfxJtFk47dtGidJveUmyV1aRWS0JxA5UCIBaO/paLFUAjm0/YFLGbVz5bUyQ5fEsfxqwAgHgCwYCAd0JBwEBIAgAdeAErhrcp0xg2uO4tUiU9q3YPMZGGbIy/saPWGnC8ey0pKAAABzUgmBlhshLar5JjsFmgAAAAAAAAABAAQEgCgCzSAErhrcp0xg2uO4tUiU9q3YPMZGGbIy/saPWGnC8ey0pKQAZlpiBCr/MXuj8y42txKw9KnTUnfddtw4yXYsxkIWgc9XRqUogAAYUWGAAABzUgmBlhMhLar5AAUWIASuGtynTGDa47i1SJT2rdg8xkYZsjL+xo9YacLx7LSkoDAwB4fRRR+puAACR3O0hyRMPrwVnVPX+pw1OSY/hvAY+6jc/0ST0CDjZIS4ccsC4fPFe5CZoyAH4UOealyQ5K/a8zQXPaGQm3pL0R/ZodecqXE6moZFiA4KehL5aRxo6V1W/nUAAAGHM0x8YGQltYcTHYLNgDQFjgAzLTECFX+YvdH5lxtbiVh6VOmpO+67bhxkuxZjIQtA54AAAAAAAAAAAAAAOjUpRAAQOAAA=",
        "status": 3,
        "status_name": "finalized",
        "storage": {
            "storage_fees_collected": "0x3f",
            "status_change": 0,
            "status_change_name": "unchanged"
        },
        "compute": {
            "success": true,
            "msg_state_used": false,
            "account_activated": false,
            "gas_fees": "0xc53078",
            "gas_used": 12923,
            "gas_limit": 0,
            "gas_credit": 10000,
            "mode": 0,
            "exit_code": 0,
            "vm_steps": 352,
            "vm_init_state_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "vm_final_state_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "compute_type": 1,
            "compute_type_name": "vm"
        },
        "action": {
            "success": true,
            "valid": true,
            "no_funds": false,
            "status_change": 0,
            "total_fwd_fees": "0x1e8480",
            "total_action_fees": "0x145850",
            "result_code": 0,
            "tot_actions": 2,
            "spec_actions": 0,
            "skipped_actions": 0,
            "msgs_created": 2,
            "action_list_hash": "39fdb5b66907d97a04567aab6cf5c9df700d1756a269b141f57d6df41788a4d2",
            "tot_msg_size_cells": 2,
            "tot_msg_size_bits": 1178
        },
        "credit_first": true,
        "aborted": false,
        "destroyed": false,
        "tr_type": 0,
        "tr_type_name": "ordinary",
        "lt": "0xe6a413032c1",
        "prev_trans_hash": "24862ba6bb23e0c0d0d65dd98d047b2bf60e98e9782c9ed086fc04316cc1ff98",
        "prev_trans_lt": "0xe6a3fd13f01",
        "now": 1680192863,
        "outmsg_cnt": 2,
        "orig_status": 1,
        "orig_status_name": "Active",
        "end_status": 1,
        "end_status_name": "Active",
        "in_msg": "b10b0866cb7320f9abac1ba6c5a09f7a60bb87b399142aa0a7dda28b086d9a40",
        "ext_in_msg_fee": "0x2798b8",
        "out_msgs": [
            "ff491002eaeaf22e85055d5b055383d8aaaa030bcb5ae34a65b27f15de8e2e34",
            "8f2f48998b041adbae47a3e8ee7541ee918299d316e07c7dbd221501a31b15d8"
        ],
        "account_addr": "0:95c35b94e98c1b5c7716a9129ed5bb0798c8c336465fd8d1eb0d385e3d969494",
        "workchain_id": 0,
        "total_fees": "0x10121bf",
        "balance_delta": "-0x746b5dd5ef",
        "old_hash": "3502ad062fb0ed7d9c9c764443cdfc49b45938eddb4689d26f7949b257569159",
        "new_hash": "2d09c40e5408805a3bfa5a2c55008e6d3f6052c66d5cf96d4c90e5f12c7f1ab0"
    },
    "output": {
        "transId": "0"
    },
    "out_messages": [
        null
    ]
}

```

Basic checks of the address format will be performed by the Everdev utility automatically, only addresses of a valid Everscale format will be accepted.

#### (Optional) Multi-owner accounts and Confirm transaction&#x20;

Note, that if your user account has multiple custodians, the transaction has to be confirmed by the required number of signatures to be executed. This transaction ID should be communicated to other custodians, who should use it to confirm the transaction.&#x20;

To withdraw tokens from a multi-owner account use the following command:

{% code overflow="wrap" %}
```bash
everdev contract run SetcodeMultisig.abi.json submitTransaction --address <wallet_account_address> --signer wallet_signer --input '{ "dest": "recipient_address", "value":10000000000, "bounce": false, "allBalance": false, "payload": "", "stateInit": null }'
```
{% endcode %}

`<wallet_account_address>` - address of the user account. Example: 0:7bf2b2ec80371601f854bff9ed0a1171714d922c8bfc86d39e67a7e3a41b2176

`wallet_signer` - name of the user account owner signer

`value`: - amount of tokens to transfer in nanotokens (Example: value:10000000000 sets up a transfer of 10 tokens).

`bounce` - use false to transfer funds to an already deployed account

`allBalance` - used to transfer all funds in the wallet. Use false for a simple transfer.

`payload` - use "" for simple transfer.

`stateInit` - use `null` for a simple transfer.

This will generate a transaction and display its `transId` that will have to be confirmed by other custodians.

To confirm a transaction, use the following command:

{% code overflow="wrap" %}
```shell
everdev contract run SetcodeMultisig.abi.json confirmTransaction --address <wallet_account_address> --signer wallet_signer2 --input transactionId:6954030467099431873
```
{% endcode %}

`<wallet_account_address>` - address of the user account. Example: 0:7bf2b2ec80371601f854bff9ed0a1171714d922c8bfc86d39e67a7e3a41b2176

`wallet_signer2` - signer of another multisig custodian (not the one who initiated the transaction).

`transactionId` – the ID of the transaction can be acquired from the custodian who created it.

#### Mitigating risks of token loss due to user error

The are two main cases regarding transfers to user accounts: a user may already have an active account to which they want to withdraw funds (set bounce to true), or they may want to withdraw funds to a completely new account, that doesn't exist at the time withdraw is requested (set bounce to false).

The status of the account provided by the user may be checked with the following Everdev command:

```shell
everdev contract info --address external_address
```

Example of existing account:

```shell
everdev contract info --address 0:665a62042aff317ba3f32e36b712b0f4a9d35277dd76dc38c9762cc6421681cf

Configuration

  Network: dev (devnet.evercloud.dev)
  Signer:  owner_keys (public 5ff6b5ba62b52b25ef347984912937bffaf2df88605e4e56cb64b9b617a28fea)

Address:   0:665a62042aff317ba3f32e36b712b0f4a9d35277dd76dc38c9762cc6421681cf
Account:   Active
Balance:   ≈ 51655 tokens (51655086754193 nano)
```

Example of account that doesn't exist yet:

```shell
everdev contract info --address 0:6238e23f6987883b3d1a86e1c39c63ae2baf7f93603d0ea5dc9b6e91ef54a1ab

Configuration

  Network: dev (devnet.evercloud.dev)
  Signer:  owner_keys (public 5ff6b5ba62b52b25ef347984912937bffaf2df88605e4e56cb64b9b617a28fea)

Address:   0:6238e23f6987883b3d1a86e1c39c63ae2baf7f93603d0ea5dc9b6e91ef54a1ab (calculated from TVC and signer public)
Code Hash: 80d6c47c4a25543c9b397b71716f3fae1e2c5d247174c52e2c19bd896442b105 (from TVC file)
Account:   Doesn't exist

```

The possible results of this command are the following:

`Doesn't exist` - account does not exist. It needs to be sponsored, then deployed, and only then will it be active.

`Uninit` - account already has some funds on it but contract code has not been deployed yet. User needs to deploy it.

`Active` - account already exists, and its code is deployed.

In the first to cases, the service might first transfer a small portion of the requested amount (\~1 EVER) and request that the user deploys their contract. Upon the user's confirmation that the account is deployed, its status may be rechecked, and if it became active, the remaining amount of requested funds may be safely transferred.

If the account is already active, a small portion of the requested amount may be transferred to the user, and the user may be asked what amount they received (note: a small amount of the transfer, usually less than 0.05 EVER, will be spent on fees, so it's best to ask for the whole number of tokens transferred). If the amounts match, the rest of the requested funds may be transferred as well.

### Using SDK&#x20;

You may integrate withdrawals from wallet account into your backend using SDK as well. A sample is available in [this repository](https://github.com/tonlabs/sdk-samples/tree/master/demo/msig-wallet) and an overview of the relevant part is given below.

In this sample JS SDK is used.  [Bindings](https://docs.everos.dev/ever-sdk/#community-bindings) for a large number of languages have been developed for SDK.&#x20;

This example shows how to generate a withdrawal transaction from a Multisig wallet, using its `sendTransaction` method. Note, that if Multisig has multiple custodians, the transaction will have to be confirmed with the `confirmTransaction` method.

In this example tokens are withdrawn from the user account to the account specified in `dest`. In a proper implementation, the account given by user should be used instead.

```javascript
    // We send 0.5 tokens. Value is written in nanotokens
    const amount = 0.5e9
    const dest = "-1:7777777777777777777777777777777777777777777777777777777777777777"

    console.log('Sending 0.5 token to', dest)

    result = await client.processing.process_message({
        message_encode_params: {
            address: msigAddress,
            ...messageParams, // use the same params as for `encode_message`,
            call_set: {       // plus add `call_set`
                function_name: 'sendTransaction',
                input: {
                    dest: dest,
                    value: amount,
                    bounce: true,
                    flags: 64,
                    payload: ''
                }
            },
        },
        send_events: false, // do not send intermidate events
    })
    console.log('Transfer completed. Transaction hash', result.transaction?.id)
    assert.equal(result.transaction?.status, 3)
    assert.equal(result.transaction?.status_name, "finalized")

```

#### Mitigating risks of token loss due to user error

Similarly to the everdev approach, you can add the account status check prior to sending tokens.

The are two main cases regarding transfers to user accounts: a user may already have an active account to which they want to withdraw funds, or they may want to withdraw funds to a completely new account, that doesn't exist at the time withdraw is requested.

Here is an example of checking account status in SDK:

```typescript
    let balance: number
    let accType: number
    for (; ;) {
        // The idiomatic way to send a request is to specify 
        // query and variables as separate properties.
        const getInfoQuery = `
                query getBalance($address: String!) {
                    blockchain {
                    account(address: $address) {
                            info {
                            balance
                            acc_type
                        }
                    }
                }
            }
            `
        const resultOfQuery: ResultOfQuery = await client.net.query({
            query: getInfoQuery,
            variables: { address: msigAddress }
        })
        

        const nanotokens = parseInt(resultOfQuery.result.data.blockchain.account.info?.balance, 16)
        accType = resultOfQuery.result.data.blockchain.account.info?.acc_type;
        if (nanotokens > MINIMAL_BALANCE * 1e9) {
            balance = nanotokens / 1e9
            break
        }
        // TODO: rate limiting
        await sleep(1000)
    }
    console.log(`Account balance is: ${balance.toString(10)} tokens. Account type is ${accType}`)
```

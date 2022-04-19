# Configure SDK

Find out how to create and configure `TONClient` object to start interacting with blockchain

* [Configure SDK](configure\_sdk.md#configure-sdk)
  * [Create TONClient](configure\_sdk.md#create-tonclient)
  * [Configure Client](configure\_sdk.md#configure-client)
    * [Network Config](configure\_sdk.md#network-config)
      * [endpoints](configure\_sdk.md#endpoints)
      * [server\_address](configure\_sdk.md#server\_address)
      * [network\_retries\_count](configure\_sdk.md#network\_retries\_count)
      * [message\_retries\_count](configure\_sdk.md#message\_retries\_count)
      * [message\_processing\_timeout](configure\_sdk.md#message\_processing\_timeout)
      * [wait\_for\_timeout](configure\_sdk.md#wait\_for\_timeout)
      * [out\_of\_sync\_threshold](configure\_sdk.md#out\_of\_sync\_threshold)
      * [reconnect\_timeout](configure\_sdk.md#reconnect\_timeout)
      * [access\_key](configure\_sdk.md#access\_key)
    * [Crypto Config](configure\_sdk.md#crypto-config)
      * [mnemonic\_dictionary](configure\_sdk.md#mnemonic\_dictionary)
      * [mnemonic\_word\_count](configure\_sdk.md#mnemonic\_word\_count)
      * [hdkey\_derivation\_path](configure\_sdk.md#hdkey\_derivation\_path)
    * [ABI Config](configure\_sdk.md#abi-config)
      * [workchain](configure\_sdk.md#workchain)
      * [message\_expiration\_timeout](configure\_sdk.md#message\_expiration\_timeout)
      * [message\_expiration\_timeout\_grow\_factor](configure\_sdk.md#message\_expiration\_timeout\_grow\_factor)

## Create TONClient

Make sure you completed the previous step and [installed SDK properly](add\_sdk\_to\_your\_app.md).

`TONClient` is the main class of Ever SDK Library. To start using library one needs to create and setup a TONClient instance.

The simplest initialization code can look like this: we just specify the Developer Network endpoints, other parameters are used by default. See the defaults below.

```graphql
const client = new TonClient({
network: { 
    endpoints: [
        'https://eri01.net.everos.dev',
        'https://rbx01.net.everos.dev',
        'https://gra01.net.everos.dev'
] 
    } 
});
```

If you are working with [local blockchain Evernode SE](https://github.com/tonlabs/evernode-se), specify [http://localhost](http://localhost) in the `endpoints`.

Check the full list of [supported network endpoints](../../reference/ton-os-api/networks.md).

You can find reference guide to `TonClient` here: [Ever-SDK API Documentation](../../reference/types-and-methods/modules.md).

## Configure Client

SDK provides a list of configuration parameters that can influence the behavior of the client. Use them when you create `TONClient` for more specific setup.

```graphql
export type TONConfigData = {
network?: { 
    endpoints?: string[],
    server_address?: string, // deprecated, use endpoints
    network_retries_count?: number, // default = 5
    message_retries_count?: number, // default = 5
    message_processing_timeout?: number, // default = 40000 ms
    wait_for_timeout?: number, // default = 40000 ms
    out_of_sync_threshold?: number, // default = 15000 ms
    reconnect_timeout?: number, // default = 12000 ms
      access_key?: string
    },
crypto?:{
    mnemonic_dictionary?: number, // default = 1
    mnemonic_word_count?: number, // default = 12
    hdkey_derivation_path?: string // default = "m/44'/396'/0'/0/0"
    },
abi?:{
    workchain?: number, // default = 0
    message_expiration_timeout?: number, // default = 40000 ms
    message_expiration_timeout_grow_factor?: number // default = 1.5
    }

}
```

### Network Config

#### endpoints

List of DApp Server addresses. Any correct URL format can be specified, including IP addresses. **This parameter is prevailing over `server_address`**.

For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be `net.ton.dev`. For Evernode SE the endpoint will be `http://localhost`.

At the start SDK sends requests to all the specified endpoints and chooses the one whose answer returns first. Later, if the application loses connection, SDK will try to switch to another endpoint from the list. If no endpoint is working there will be an error.

#### server\_address

**This field is deprecated, but left for backward-compatibility.** DApp Server public address. For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be `net.ton.dev`

#### network\_retries\_count

The number of automatic network retries that SDK performs in case of connection problems. The default value is 5.

#### message\_retries\_count

The number of `process_message` retries that SDK performs in case of `Message Expired (507)` error - but only for those messages, local emulation of which was successful or failed with replay protection error. The default value is 5.

Read more about reliable message delivery and `pragma expire` [here](../work\_with\_contracts/message\_expiration.md).

#### message\_processing\_timeout

Timeout that is used to process message delivery for contracts, the ABI of which does not include `expire` header. If the message is not delivered within the specified timeout, the appropriate error occurs.

#### wait\_for\_timeout

Maximum timeout that is used for query response. The default value is 40 sec.

#### out\_of\_sync\_threshold

Maximum time difference between server and client.

If client's device time is out of sync and difference is more than the threshold, an error will occur. Also an error will occur if the specified threshold is more than `message_processing_timeout/2`.

The default value is 15 sec.

#### reconnect\_timeout

Timeout between reconnect attempts.

#### access\_key

Access key to GraphQL API. At the moment is not used in production.

### Crypto Config

#### mnemonic\_dictionary

Mnemonic dictionary that will be used by default in crypto functions. If not specified, 1 dictionary will be used.

#### mnemonic\_word\_count

Mnemonic word count that will be used by default in crypto functions. If not specified the default value will be 12.

#### hdkey\_derivation\_path

Derivation path that will be used by default in crypto functions. If not specified `m/44'/396'/0'/0/0` will be used.

### ABI Config

#### workchain

Workchain id that is used by default in `DeploySet`.

#### message\_expiration\_timeout

Message lifetime for contracts, the ABI of which includes `expire` header. The default value is 40 sec.

#### message\_expiration\_timeout\_grow\_factor

Factor that increases the expiration timeout for each retry. The default value is 1.5.

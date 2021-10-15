# Configure SDK

Find out how to create and configure `TONClient` object to start interacting with blockchain

* [Create TONClient](configure_sdk.md#create-tonclient)
  * [Using default config in AppKit](configure_sdk.md#using-default-config-in-appkit)
* [Configure Client](configure_sdk.md#configure-client)
  * [Network Config](configure_sdk.md#network-config)
    * [endpoints](configure_sdk.md#endpoints)
    * [server_address](configure_sdk.md#server_address)
    * [network_retries_count](configure_sdk.md#network_retries_count)
    * [message_retries_count](configure_sdk.md#message_retries_count)
    * [message_processing_timeout](configure_sdk.md#message_processing_timeout)
    * [wait_for_timeout](configure_sdk.md#wait_for_timeout)
    * [out_of_sync_threshold](configure_sdk.md#out_of_sync_threshold)
    * [reconnect_timeout](configure_sdk.md#reconnect_timeout)
    * [access_key](configure_sdk.md#access_key)
  * [Crypto Config](configure_sdk.md#crypto-config)
    * [mnemonic_dictionary](configure_sdk.md#mnemonic_dictionary)
    * [mnemonic_word_count](configure_sdk.md#mnemonic_word_count)
    * [hdkey_derivation_path](configure_sdk.md#hdkey_derivation_path)
  * [ABI Config](configure_sdk.md#abi-config)
    * [workchain](configure_sdk.md#workchain)
    * [message_expiration_timeout](configure_sdk.md#message_expiration_timeout)
    * [message_expiration_timeout_grow_factor](configure_sdk.md#message_expiration_timeout_grow_factor)

## Create TONClient

Make sure you completed the previous step and [installed SDK properly](add_sdk_to_your_app.md).

`TONClient` is the main class of TON SDK Library. To start using library one needs to create and setup a TONClient instance.

The simplest initialization code can look like this: we just specify the network, other parameters are used by default. See the defaults below.

```
const client = new TonClient({
network: { 
    endpoints: ['net.ton.dev'] 
    } 
});
```

### Using default config in AppKit

If you are using `AppKit`, you can specify the default config. This will allow you to omit passing `client` object into every `AppKit` function later. If a function works with another client (another network), you need to create a client object for that network separately, like we did in the previous step, and pass it as a parameter.

```
TonClient.defaultConfig = {
network: {
    // Local node URL here
    endpoints: ['net.ton.dev']
  },
};
```

If you are working with [local blockchain TON OS SE](https://github.com/tonlabs/tonos-se), specify [http://localhost](http://localhost) in the `endpoints`.

Check the full list of [supported network endpoints](../../docs/ton_os_api/networks.md).

You can find reference guide to `TonClient` here: [TON-SDK API Documentation](../../docs/modules/).

## Configure Client

SDK provides a list of configuration parameters that can influence the behavior of the client. Use them when you create `TONClient` for more specific setup.

```
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

For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be `net.ton.dev`. For TON OS SE the endpoint will be `http://localhost`.

At the start SDK sends requests to all the specified endpoints and chooses the one whose answer returns first. Later, if the application loses connection, SDK will try to switch to another endpoint from the list. If no endpoint is working there will be an error.

#### server_address

**This field is deprecated, but left for backward-compatibility.** DApp Server public address. For instance, for `net.ton.dev/graphql` GraphQL endpoint the server address will be `net.ton.dev`

#### network_retries_count

The number of automatic network retries that SDK performs in case of connection problems. The default value is 5.

#### message_retries_count

The number of `process_message` retries that SDK performs in case of `Message Expired (507)` error - but only for those messages, local emulation of which was successful or failed with replay protection error. The default value is 5.

Read more about reliable message delivery and `pragma expire` [here](../work_with_contracts/message_expiration.md).

#### message_processing_timeout

Timeout that is used to process message delivery for contracts, the ABI of which does not include `expire` header. If the message is not delivered within the specified timeout, the appropriate error occurs.

#### wait_for_timeout

Maximum timeout that is used for query response. The default value is 40 sec.

#### out_of_sync_threshold

Maximum time difference between server and client.

If client's device time is out of sync and difference is more than the threshold, an error will occur. Also an error will occur if the specified threshold is more than `message_processing_timeout/2`.

The default value is 15 sec.

#### reconnect_timeout

Timeout between reconnect attempts.

#### access_key

Access key to GraphQL API. At the moment is not used in production.

### Crypto Config

#### mnemonic_dictionary

Mnemonic dictionary that will be used by default in crypto functions. If not specified, 1 dictionary will be used.

#### mnemonic_word_count

Mnemonic word count that will be used by default in crypto functions. If not specified the default value will be 12.

#### hdkey_derivation_path

Derivation path that will be used by default in crypto functions. If not specified `m/44'/396'/0'/0/0` will be used.

### ABI Config

#### workchain

Workchain id that is used by default in `DeploySet`.

#### message_expiration_timeout

Message lifetime for contracts, the ABI of which includes `expire` header. The default value is 40 sec.

#### message_expiration_timeout_grow_factor

Factor that increases the expiration timeout for each retry. The default value is 1.5.

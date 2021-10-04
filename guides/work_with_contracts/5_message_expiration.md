# Message Expiration

## Message expiration

Find out what 'message expired' means and get the understanding of 'retries' mechanism for reliable message delivery in TON Blockchain

## About message expiration

In TON blockchain, client-to-contract interaction is based on external messages. Yet, there is no guarantee for the successful reception of the message by design, and the time when the contract completes message processing is not determined.

Moreover, if, for some reason, a contract rejects a message \(e.g., if it triggers replay protection\), the message is not recorded to the blockchain, and the client has no means to find out the message processing result. In other words, there is neither exact timeout for message processing nor a tool to determine that it was rejected.

We implemented the message expiration mechanism to resolve the issue: when a client creates a message for a contract, a specific message expiration time is defined. Upon receiving this message, the contract checks whether the current time is earlier than the message expiration time. If it is, the contract execution continues. Otherwise, the contract aborts the transaction. Meanwhile, the client waits for any of the two: transaction corresponding to the original message or a new block with generation time later than the message expiration time. Thus, if a message is successfully processed, the client is notified about the transaction. In any of the following cases: the message is not delivered to the contract, delivered after expiration time, rejected by the contract, the client is informed of new block creation, meaning that the contract did not process the message. In this case, the client can create and send a new message for another attempt.

The feature must be supported by the contract itself to work properly. Let's use a simple Solidity contract as a sample.

```text
pragma solidity >=0.5.0;
pragma AbiHeader time;
pragma AbiHeader expire;

contract HelloTON {
uint32 timestamp;

// Modifier that allows public function to accept all external calls.
modifier alwaysAccept {
    tvm.accept();
    _;
    }

constructor() public {
    tvm.accept();
    timestamp = uint32(now);
    }
//Function setting set value to state variable timestamp
function touch() public alwaysAccept {
    timestamp = uint32(now);
    }
//Function returns value of state variable timestamp
function sayHello() public view returns (uint32) {
    return timestamp;
    }
}
```

The `pragma AbiHeader expire;` line initiates the message expiration option for the contract functions. TON Labs SDK automatically defines the expiration time according to the client default timeout specified in client config. We use the following formula to define the expiration time: `expire time = current time + timeout`. You can define the applied timeout at the SDK initialization via the `abi.message_expiration_timeout` parameter that takes the timeout value in ms. If no timeout is specified at initialization, the default value is 40 seconds.

```text
const client = new TonClient({
network: { 
    endpoints: ['net.ton.dev'] 
    },
abi: {
message_expiration_timeout: 120000
    }

});
```

To define an expiration time for a separate message, the `expire` value has to be defined in header parameters of a contract function call \(the `call_set.header` parameter in `process_message`, `encode_message`, functions.

> **Note**: the expire value is defined in **milliseconds**.

```text
// Encode the message with `touch` function call
const params = {
        abi= {
               type: 'Contract',
            value: Contract.abi
        },
        address,
        call_set: {
            function_name: 'touch',
            header: {
                      // timeout will be 30 seconds
                    expire: Math.floor(Date.now() / 1000) + 30 

            }
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

The expiration time is stored in the message structure and used when it is sent to determine the transaction wait for the timeout. If you need to send a delayed message, the delay length should also be considered when the expiration time is defined.

```text
// timeout will be delayTime + 30 seconds
header: {
expire: Math.floor(Date.now() / 1000) + delayTime + 30
},
```

The option can be used when an external device is used to sign transactions and the message for signing cannot be generated immediately before the signature generation \(see item 2 below\).

When an external device is used, two message creation options exist:

1. the device does not require user input for signing or user input is required before the data for signing is generated \(e.g. a user unlocks the device to create a signature and then the data for signing is sent to the device with a separate command\)
2. the device requires user input at the point when data for signing has to be generated \(e.g. a command to provide a signature with the relevant data is sent to the device before the device prompts the user to unlock it\)

In the first case default SDK timeout values can be used, as the interval between message creation and sending is supposed to be short.

In the second case the message expiration time takes into account an interval allocated to the user to sign a transaction. Note that a timer showing the available window for signing has to be displayed to the user.


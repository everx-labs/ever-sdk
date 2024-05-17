---
description: Subscribe to any updates in blockchain database, including contract events.
---

# Subscribe to Updates

## When you may need it?

Whenever you need to improve UI responsiveness or want to receive updates on your backend as they appear - you may use websocket subscriptions.&#x20;

## Usage

### Account updates

```javascript
const accountSubscription = await TonClient.default.net.subscribe_collection({
    collection: "accounts",
    filter: { id: { eq: address } },
    result: "balance",
}, (params, responseType) => {
    if (responseType === ResponseType.Custom) {
        console.log("Account has updated. Current balance is ", parseInt(params.result.balance));
    }
});
```

### Account messages

```javascript
const messageSubscription = await TonClient.default.net.subscribe_collection({
    collection: "messages",
    filter: {
        src: { eq: address },
        OR: {
            dst: { eq: address },
        }
    },
    result: "boc",
}, async (params, responseType) => {
    try {
        if (responseType === ResponseType.Custom) {
            const decoded = (await TonClient.default.abi.decode_message({
                abi: abiContract(your-contract-abi),
                message: params.result.boc,
            }));
            switch (decoded.body_type) {
            case MessageBodyType.Input:
                console.log(`External inbound message, function "${decoded.name}", parameters: `, JSON.stringify(decoded.value));
                break;
            case MessageBodyType.Output:
                console.log(`External outbound message, function "${decoded.name}", result`, JSON.stringify(decoded.value));
                break;
            case MessageBodyType.Event:
                console.log(`External outbound message, event "${decoded.name}", parameters`, JSON.stringify(decoded.value));
                break;
            }
        }
    } catch (err) {
        console.log('>>>', err);
    }
});
```

See this sample to understand how to use [subscribe\_collection](../../reference/types-and-methods/mod\_net.md#subscribe\_collection) function:

[https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/listen-and-decode](https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/listen-and-decode)

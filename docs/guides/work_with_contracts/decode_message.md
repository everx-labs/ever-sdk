---
description: How to decode messages with ABI
---

# Decode Messages(Event)

SDK can decode ABI-compatible External Inbound, External Outbound and Internal messages.

## Types of messages

ABI-compatible contracts generate abi-compatible messages, or, to be exact, abi-compatible message bodies :)

So, if you have ABI on hands, you can decode these messages.

You can use both full message boc for decoding and only message body for decoding.

## Let's decode!

Core SDK provides&#x20;

* `decode_message` method for full boc of message decoding&#x20;
* `decode_message_body` method for only message body decoding.

## Usage

```javascript
const decoded = (await client.abi.decode_message({
        abi: abiContract(HelloEventsContract.abi),
        message: boc,
    }));
switch (decoded.body_type) {
case MessageBodyType.Input:
    log_.push(`External inbound message, function "${decoded.name}", fields: ${JSON.stringify(decoded.value)}` );
    break;
case MessageBodyType.Output:
    log_.push(`External outbound message (return) of function "${decoded.name}", fields: ${JSON.stringify(decoded.value)}`);
    break;
case MessageBodyType.Event:
    log_.push(`External outbound message (event) "${decoded.name}", fields: ${JSON.stringify(decoded.value)}`);
    break;
}
```

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/subscribe-and-decode/index.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/subscribe-and-decode/index.js)

**AppKit**

[https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/listen-and-decode](https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/listen-and-decode)

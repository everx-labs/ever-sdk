# External Signing

How to perform deploy and run scenarios without disclosing the private key

* [When you may need it?](external\_signing.md#when-you-may-need-it)
  * [Patterns](external\_signing.md#patterns)
* [Use signingBox interface](external\_signing.md#use-signingbox-interface)
  * [Sample source code](external\_signing.md#sample-source-code)
* [Sign message outside sdk](external\_signing.md#sign-message-outside-sdk)
  * [Sample source code](external\_signing.md#sample-source-code-1)

## When you may need it?

Sometimes there is no access to the private key - for instance if an application signs data using HSM module or NFC card that does not disclose the private key but provides some API for signing.

### Patterns

There are 2 patterns to implement such scenario:

* use `signingBox` interface to sign message. It will allow you to invoke an API of your signing device during message creation.
* sign message separately - this approach will allow you to separate message creation into 3 steps: create unsigned message, sign it, and attach signature.

Read below for more details.

## Use signingBox interface

Developer needs to create an implementation of this interface in their project and pass it to SDK.

The implementation may incapsulate invoking of some external API, such as HSM of NFC Card.

```javascript
export interface AppSigningBox {
    get_public_key(): Promise<ResultOfAppSigningBoxGetPublicKey>,
    sign(params: ParamsOfAppSigningBoxSign): Promise<ResultOfAppSigningBoxSign>,
}
```

where

```javascript
type ResultOfAppSigningBoxGetPublicKey = {
    public_key: string
}

type ParamsOfAppSigningBoxSign = {
    unsigned: string
}

type ResultOfAppSigningBoxSign = {
    signature: string
}
```

All the methods that create messages - `encode_message`, `process_message` can take [Signer of type SigningBox object](../../reference/types-and-methods/mod\_abi.md#signer), implementing this interface, instead of a key pair.

### Sample source code

**Core sample**

[https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/signingBox](https://github.com/everx-labs/sdk-samples/tree/master/core-examples/node-js/signingBox)

**AppKit sample**

[https://github.com/everx-labs/sdk-samples/tree/master/appkit-examples/signing-box](https://github.com/everx-labs/sdk-samples/tree/master/appkit-examples/signing-box)

## Sign message outside sdk

In this case you may create an unsigned message with `encode_message` function, specifying Signer of type `External` . After that you sign it somewhere outside and attach signature with [attach_signature](../../reference/types-and-methods/mod\_abi.md#attach_signature) function.

### Sample source code

We will upload a sample with it later. If you need it in your work, please write in [SDK telegram channel](https://t.me/ever_sdk).

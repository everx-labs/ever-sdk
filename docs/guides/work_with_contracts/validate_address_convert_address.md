# Validate address, convert address

Validate and convert TON address to different formats

## When you may need it?

If you want to validate TON address and/or convert address to bounceable/non-bounceable base64/base64url formats and back.

## Convert

Use `utils.convert_address` function for that.

```javascript
// Convert address to different types
console.log("Multisig address in HEX:")
let convertedAddress = (await tonClient.utils.convert_address({
  address,
  output_format: {
    type: 'Hex'
  },
})).address;
console.log(convertedAddress);
```

If address is incorrect the function will fail with an error.

## Sample source code

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/multisig/work-with-multisig.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/multisig/work-with-multisig.js)

---
description: Validate and convert address to different formats
---

# Validate address, convert address

## When you may need it?

If you want to validate an address and/or convert address to bounceable/non-bounceable base64/base64url formats and back. These types of address are sometimes used in some clients so may be useful for integration.&#x20;

## Get address type

{% code overflow="wrap" %}
```javascript
let initialAddressType = await client.utils.get_address_type({address});
console.log(`Address type is ${JSON.stringify(initialAddressType)}`);
```
{% endcode %}

## Convert

Use `utils.convert_address` function for that.

```javascript
let convertedAddress = (await client.utils.convert_address({
    address,
    output_format: {
        type: "Hex"
    },
})).address;
console.log(`Address in raw format: ${convertedAddress}`);
```

### Validate

If address is incorrect the function `utils.convert_address` will fail with an error.

## Sample source code

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/utils.convert\_address/index.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/utils.convert\_address/index.js)

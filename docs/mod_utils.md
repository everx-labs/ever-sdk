# Module utils

 Misc utility Functions.
## Functions
[convert_address](#convert_address) –  Sends message to the network and monitors network for a result of

## Types
[AddressStringFormat](#AddressStringFormat)

[ParamsOfConvertAddress](#ParamsOfConvertAddress)

[ResultOfConvertAddress](#ResultOfConvertAddress)


# Functions
## convert_address

 Sends message to the network and monitors network for a result of
 message processing.

```ts

function convertAddress(
    params: ParamsOfConvertAddress,
    responseHandler: ResponseHandler | null,
): Promise<ResultOfConvertAddress>;

```
### Parameters
- `address`: _string_ –  Account address in any format.
- `output_format`: _[AddressStringFormat](mod_utils.md#AddressStringFormat)_ –  Specify the format to convert to.
### Result

- `address`: _string_ –  address in the specified format


# Types
## AddressStringFormat



## ParamsOfConvertAddress

- `address`: _string_ –  Account address in any format.
- `output_format`: _[AddressStringFormat](mod_utils.md#AddressStringFormat)_ –  Specify the format to convert to.


## ResultOfConvertAddress

- `address`: _string_ –  address in the specified format



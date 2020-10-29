# Module utils

 Misc utility Functions.
## Functions
[convert_address](#convert_address) – Converts address from any TON format to any TON format

## Types
[AddressStringFormat](#AddressStringFormat)

[ParamsOfConvertAddress](#ParamsOfConvertAddress)

[ResultOfConvertAddress](#ResultOfConvertAddress)


# Functions
## convert_address

Converts address from any TON format to any TON format

```ts
type ParamsOfConvertAddress = {
    address: string,
    'output_format': AddressStringFormat
};

type ResultOfConvertAddress = {
    address: string
};

function convert_address(
    params: ParamsOfConvertAddress,
): Promise<ResultOfConvertAddress>;
```
### Parameters
- `address`: _string_ – Account address in any TON format.
- `output_format`: _[AddressStringFormat](mod_utils.md#AddressStringFormat)_ – Specify the format to convert to.
### Result

- `address`: _string_ – Address in the specified format


# Types
## AddressStringFormat
```ts
type AddressStringFormat = {
    type: 'AccountId'
} | {
    type: 'Hex'
} | {
    type: 'Base64'
    url: boolean,
    test: boolean,
    bounce: boolean
};
```
Depends on value of the  `type` field.

When _type_ is _'AccountId'_


When _type_ is _'Hex'_


When _type_ is _'Base64'_


- `url`: _boolean_
- `test`: _boolean_
- `bounce`: _boolean_


## ParamsOfConvertAddress
```ts
type ParamsOfConvertAddress = {
    address: string,
    'output_format': AddressStringFormat
};
```
- `address`: _string_ – Account address in any TON format.
- `output_format`: _[AddressStringFormat](mod_utils.md#AddressStringFormat)_ – Specify the format to convert to.


## ResultOfConvertAddress
```ts
type ResultOfConvertAddress = {
    address: string
};
```
- `address`: _string_ – Address in the specified format



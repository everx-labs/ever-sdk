# Module utils

Misc utility Functions.


## Functions
[convert_address](#convert_address) – Converts address from any TON format to any TON format

[get_address_type](#get_address_type) – Validates and returns the type of any TON address.

[calc_storage_fee](#calc_storage_fee) – Calculates storage fee for an account over a specified time period

[compress_zstd](#compress_zstd) – Compresses data using Zstandard algorithm

[decompress_zstd](#decompress_zstd) – Decompresses data using Zstandard algorithm

## Types
[AddressStringFormat](#addressstringformat)

[AccountAddressType](#accountaddresstype)

[ParamsOfConvertAddress](#paramsofconvertaddress)

[ResultOfConvertAddress](#resultofconvertaddress)

[ParamsOfGetAddressType](#paramsofgetaddresstype)

[ResultOfGetAddressType](#resultofgetaddresstype)

[ParamsOfCalcStorageFee](#paramsofcalcstoragefee)

[ResultOfCalcStorageFee](#resultofcalcstoragefee)

[ParamsOfCompressZstd](#paramsofcompresszstd)

[ResultOfCompressZstd](#resultofcompresszstd)

[ParamsOfDecompressZstd](#paramsofdecompresszstd)

[ResultOfDecompressZstd](#resultofdecompresszstd)


# Functions
## convert_address

Converts address from any TON format to any TON format

```ts
type ParamsOfConvertAddress = {
    address: string,
    output_format: AddressStringFormat
}

type ResultOfConvertAddress = {
    address: string
}

function convert_address(
    params: ParamsOfConvertAddress,
): Promise<ResultOfConvertAddress>;
```
### Parameters
- `address`: _string_ – Account address in any TON format.
- `output_format`: _[AddressStringFormat](mod_utils.md#addressstringformat)_ – Specify the format to convert to.


### Result

- `address`: _string_ – Address in the specified format


## get_address_type

Validates and returns the type of any TON address.

Address types are the following

`0:919db8e740d50bf349df2eea03fa30c385d846b991ff5542e67098ee833fc7f7` - standard TON address most
commonly used in all cases. Also called as hex address
`919db8e740d50bf349df2eea03fa30c385d846b991ff5542e67098ee833fc7f7` - account ID. A part of full
address. Identifies account inside particular workchain
`EQCRnbjnQNUL80nfLuoD+jDDhdhGuZH/VULmcJjugz/H9wam` - base64 address. Also called "user-friendly".
Was used at the beginning of TON. Now it is supported for compatibility

```ts
type ParamsOfGetAddressType = {
    address: string
}

type ResultOfGetAddressType = {
    address_type: AccountAddressType
}

function get_address_type(
    params: ParamsOfGetAddressType,
): Promise<ResultOfGetAddressType>;
```
### Parameters
- `address`: _string_ – Account address in any TON format.


### Result

- `address_type`: _[AccountAddressType](mod_utils.md#accountaddresstype)_ – Account address type.


## calc_storage_fee

Calculates storage fee for an account over a specified time period

```ts
type ParamsOfCalcStorageFee = {
    account: string,
    period: number
}

type ResultOfCalcStorageFee = {
    fee: string
}

function calc_storage_fee(
    params: ParamsOfCalcStorageFee,
): Promise<ResultOfCalcStorageFee>;
```
### Parameters
- `account`: _string_
- `period`: _number_


### Result

- `fee`: _string_


## compress_zstd

Compresses data using Zstandard algorithm

```ts
type ParamsOfCompressZstd = {
    uncompressed: string,
    level?: number
}

type ResultOfCompressZstd = {
    compressed: string
}

function compress_zstd(
    params: ParamsOfCompressZstd,
): Promise<ResultOfCompressZstd>;
```
### Parameters
- `uncompressed`: _string_ – Uncompressed data.
<br>Must be encoded as base64.
- `level`?: _number_ – Compression level, from 1 to 21. Where: 1 - lowest compression level (fastest compression); 21 - highest compression level (slowest compression). If level is omitted, the default compression level is used (currently `3`).


### Result

- `compressed`: _string_ – Compressed data.
<br>Must be encoded as base64.


## decompress_zstd

Decompresses data using Zstandard algorithm

```ts
type ParamsOfDecompressZstd = {
    compressed: string
}

type ResultOfDecompressZstd = {
    decompressed: string
}

function decompress_zstd(
    params: ParamsOfDecompressZstd,
): Promise<ResultOfDecompressZstd>;
```
### Parameters
- `compressed`: _string_ – Compressed data.
<br>Must be encoded as base64.


### Result

- `decompressed`: _string_ – Decompressed data.
<br>Must be encoded as base64.


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
}
```
Depends on value of the  `type` field.

When _type_ is _'AccountId'_


When _type_ is _'Hex'_


When _type_ is _'Base64'_


- `url`: _boolean_
- `test`: _boolean_
- `bounce`: _boolean_


Variant constructors:

```ts
function addressStringFormatAccountId(): AddressStringFormat;
function addressStringFormatHex(): AddressStringFormat;
function addressStringFormatBase64(url: boolean, test: boolean, bounce: boolean): AddressStringFormat;
```

## AccountAddressType
```ts
enum AccountAddressType {
    AccountId = "AccountId",
    Hex = "Hex",
    Base64 = "Base64"
}
```
One of the following value:

- `AccountId = "AccountId"`
- `Hex = "Hex"`
- `Base64 = "Base64"`


## ParamsOfConvertAddress
```ts
type ParamsOfConvertAddress = {
    address: string,
    output_format: AddressStringFormat
}
```
- `address`: _string_ – Account address in any TON format.
- `output_format`: _[AddressStringFormat](mod_utils.md#addressstringformat)_ – Specify the format to convert to.


## ResultOfConvertAddress
```ts
type ResultOfConvertAddress = {
    address: string
}
```
- `address`: _string_ – Address in the specified format


## ParamsOfGetAddressType
```ts
type ParamsOfGetAddressType = {
    address: string
}
```
- `address`: _string_ – Account address in any TON format.


## ResultOfGetAddressType
```ts
type ResultOfGetAddressType = {
    address_type: AccountAddressType
}
```
- `address_type`: _[AccountAddressType](mod_utils.md#accountaddresstype)_ – Account address type.


## ParamsOfCalcStorageFee
```ts
type ParamsOfCalcStorageFee = {
    account: string,
    period: number
}
```
- `account`: _string_
- `period`: _number_


## ResultOfCalcStorageFee
```ts
type ResultOfCalcStorageFee = {
    fee: string
}
```
- `fee`: _string_


## ParamsOfCompressZstd
```ts
type ParamsOfCompressZstd = {
    uncompressed: string,
    level?: number
}
```
- `uncompressed`: _string_ – Uncompressed data.
<br>Must be encoded as base64.
- `level`?: _number_ – Compression level, from 1 to 21. Where: 1 - lowest compression level (fastest compression); 21 - highest compression level (slowest compression). If level is omitted, the default compression level is used (currently `3`).


## ResultOfCompressZstd
```ts
type ResultOfCompressZstd = {
    compressed: string
}
```
- `compressed`: _string_ – Compressed data.
<br>Must be encoded as base64.


## ParamsOfDecompressZstd
```ts
type ParamsOfDecompressZstd = {
    compressed: string
}
```
- `compressed`: _string_ – Compressed data.
<br>Must be encoded as base64.


## ResultOfDecompressZstd
```ts
type ResultOfDecompressZstd = {
    decompressed: string
}
```
- `decompressed`: _string_ – Decompressed data.
<br>Must be encoded as base64.



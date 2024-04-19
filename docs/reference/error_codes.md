# Error Codes

You can find error codes with descriptions on this page

* [SDK Errors](error\_codes.md#sdk-errors)
* [Solidity Runtime Errors](error\_codes.md#solidity-runtime-errors)
* [TON Virtual Machine Runtime Errors](error\_codes.md#ton-virtual-machine-runtime-errors)
  * [Action phase errors](error\_codes.md#action-phase-errors)

## SDK Errors

[Client Error codes (1-99)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_client#clienterrorcode)

[Crypto Error codes (100-199)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_crypto#cryptoerrorcode)

[Boc error codes(200-299)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_boc#bocerrorcode)

[Abi Error codes (300-399)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_abi#abierrorcode)

[TVM Error codes (400-499)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_tvm#tvmerrorcode)

[Processing Error codes (500-599)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_processing#processingerrorcode)

[Net Error Codes (600-699)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_net#neterrorcode)

[DeBot Error Codes (800-899)](https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod\_debot#deboterrorcode)

## Solidity Runtime Errors

[https://github.com/everx-labs/TON-Solidity-Compiler/blob/master/API.md#solidity-runtime-errors](https://github.com/everx-labs/TON-Solidity-Compiler/blob/master/API.md#solidity-runtime-errors)

## TON Virtual Machine Runtime Errors

`0` TVM terminated successfully

`-2` TVM terminated successfully: alternative code

`-3` Stack underflow

`-4` Stack overflow

`-5` Integer overflow

`-6` Range check error

`-7` Invalid opcode

`-8` Type check error

`-9` Cell overflow

`-10` Cell underflow

`-11` Dictionary error

`-12` Unknown error

`-13` Fatal error

`-14` Out of gas: the contract is either low on gas, or its limit is exceeded

### Action phase errors

`32` Action list invalid

`33` Too many actions

`34` Unsupported action

`35` Invalid source address

`36` Invalid destination address

`37` Too low balance to send outbound message (37) at action

`38` Too low extra to send outbound message (38) at action

`39` Message does not fit in buffer

`40` Message too large

`41` Library not found

`42` Library delete error

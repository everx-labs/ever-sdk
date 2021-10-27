# Module proofs

[UNSTABLE](UNSTABLE.md) Module for proving queried data.


## Functions
[proof_block_data](#proof_block_data) – Proves that block's data queried from DApp server can be trusted. Automatically checks block proofs and compares given data with the proven. If block's BOC is not provided, it will be queried from DApp (in this case it is required to provide `id` of block in the JSON). If `cache_proofs` in config is set to `true` (default), downloaded proofs and masterchain BOCs are saved into the persistent local storage (e.g.

## Types
[ProofsErrorCode](#ProofsErrorCode)

[ParamsOfProofBlockData](#ParamsOfProofBlockData)


# Functions
## proof_block_data

Proves that block's data queried from DApp server can be trusted. Automatically checks block proofs and compares given data with the proven. If block's BOC is not provided, it will be queried from DApp (in this case it is required to provide `id` of block in the JSON). If `cache_proofs` in config is set to `true` (default), downloaded proofs and masterchain BOCs are saved into the persistent local storage (e.g.

file system for native environments or browser's local storage for the web); otherwise all data are cached only in memory in current
client's context and will be lost after destruction of the client.

```ts
type ParamsOfProofBlockData = {
    block: any
}

function proof_block_data(
    params: ParamsOfProofBlockData,
): Promise<void>;
```
### Parameters
- `block`: _any_ – Single block's data as queried from DApp server, without modifications. The required field is `id` or top-level `boc`, others are optional.


# Types
## ProofsErrorCode
```ts
enum ProofsErrorCode {
    InvalidData = 901,
    ProofCheckFailed = 902,
    InternalError = 903,
    DataDiffersFromProven = 904
}
```
One of the following value:

- `InvalidData = 901`
- `ProofCheckFailed = 902`
- `InternalError = 903`
- `DataDiffersFromProven = 904`


## ParamsOfProofBlockData
```ts
type ParamsOfProofBlockData = {
    block: any
}
```
- `block`: _any_ – Single block's data as queried from DApp server, without modifications. The required field is `id` or top-level `boc`, others are optional.



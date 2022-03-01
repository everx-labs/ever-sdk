# Update Trusted Blocks utility

This small utility is used for downloading and storing of trusted key-blocks for Everscale MainNet and DevNet. Trusted 
key-blocks are hashes of known key-blocks of known networks. TON SDK uses these hashes for speeding up of proof-checking.  

Each downloaded block is being proven by SDK using already known trusted key-blocks hashes.

Utility saves downloaded block hashes into a [binary file](../../ton_client/src/proofs/trusted_key_blocks.bin). This 
file is being compiled into the final SDK binary.

If file already exists, it will be appended with the new key-blocks.

Usage:
```shell
$ ./update_trusted_blocks trusted_key_blocks.bin
```

Sample output:
```
*** [main.ton.dev] ***
Zerostate root_hash: 58ffca1a178daff705de54216e5433c9bd2e7d850070d334d38997847ab9e845
Proof for key_block #20922... OK. root_hash: 7c8871defeca910ba58146b9123b7a60018bf75de074d6ae61992ea7b0acd74e
Proof for key_block #22353... OK. root_hash: 9284abb429ca4fe61d4edd3e555b215fdc40cecdd48ccb389cd20a5f03900acb
Proof for key_block #26151... OK. root_hash: a1b6c2b7d61ac927c9b3e99308b7c3a4ad9d95c0ccfe6a50ddf5bdb6cc4da777
Proof for key_block #28430... OK. root_hash: 091a2e4d4d7dd7a736ac7335428ec4d41c0fc5fdf895ca016d97815ec5654ecc
Proof for key_block #44912... OK. root_hash: d38a0c92a6285e77b08da4853ce1d0d667fe8b63502fad560d89f88c6cddaf9b
Proof for key_block #47279... OK. root_hash: 51ba4f7e5b1052779428267dfdca9e291fe62bf104a53111b038b778b3884a57
Proof for key_block #64285... OK. root_hash: ac8d98b13b650798f742452efcca296668f6062488319f94af72302a57826864
Proof for key_block #66692... OK. root_hash: 2941c6a9856f0966ecb5f3a7fbf0ea0d41470c5627c2805a66ea8e2c0c25b9a9
Proof for key_block #82758... OK. root_hash: c94ac532ddbb295a5016756e7dfc5e759fe9974482a40e175085aa0b7312f627
...
```

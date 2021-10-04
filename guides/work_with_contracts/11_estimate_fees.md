# Estimate fees

Find out how to estimate fees before executing contracts on-chain
# When you may need to estimate fees?

- Before deploy to find out how much it will cost
- Before contract execution to find out how much will it cost
- To calculate how much will it cost to store a contract for a period of time

# Estimate deploy and run fees

The steps you need to complete to estimate fees of deploy and run:

1. Encode a message you will locally execute with [encode_message](../../docs/mod_abi.md#encode_message) 
2. Download account boc the same way you did in [Run ABI Get method](3_run_abi_get_method.md)
3. Run the message on the downloaded state with [run_executor](../../docs/mod_tvm.md#run_executor) function
4. Retrieve `result.fees` object

# Estimate storage fee

Use [calc_storage_fee](../../docs/mod_utils.md#calc_storage_fee) method. 

# Sample source code

**Core**

https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run_executor

**AppKit**

https://github.com/tonlabs/sdk-samples/blob/master/appkit-examples/fee-calculation/index.js
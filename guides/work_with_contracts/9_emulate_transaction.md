# Emulate Transaction

## Emulate transaction

Emulate transaction execution locally to understand why it fails

## When you may need it?

If you want to test your contract locally and find out will your operation work on the real network or not, you can emulate it locally.

## Lets debug!

Core SDK provides [run\_executor](../../docs/mod_tvm.md#run_executor) method of `tvm` module for it.

[AppKit](https://github.com/tonlabs/appkit-js) provides `localDeploy` and `localRun` functions for it.

### How to emulate contract deploy?

To emulate deploy you need to create a deploy message and execute it locally.

If you plan to emulate local execution afterwards, then retrieve account state \(BOC\) from deploy emulation result and pass it into the next local execution.

See the JavaScript samples below to understand how to emulate contract deploy.

### How to emulate contract local execution?

To emulate local execution you need to take the current contract state \(boc\), create a run message and execute it locally on the current account state.

If you emulated deploy before run, then you need to get the account state from the result of the deploy emulation.

For the subsequent call emulations, use account state retrieved from the previous local execution emulation.

See the JavaScript samples below to understand how to emulate contract deploy.

## Sample source code

**Core**

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run\_executor](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/run_executor)

**AppKit**

[https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/run\_executor](https://github.com/tonlabs/sdk-samples/tree/master/appkit-examples/run_executor)


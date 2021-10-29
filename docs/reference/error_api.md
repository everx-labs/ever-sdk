# Error API

SDK Error API

* [TONClientError](error_api.md#tonclienterror)
  * [Properties](error_api.md#properties)
* [Types](error_api.md#types)
  * [ErrorData](error_api.md#errordata)
* [Error example](error_api.md#error-example)

## TONClientError

### Properties

**code: number**

Unique error code

**message: string**

Error description

**data: ErrorData**

Additional data provided with error. All the fields in `ErrorData` are optional and their presence depends on the error code.

## Types

### ErrorData

All the fields in `ErrorData` are optional and their presence depends on the error code.

```text
{
    message_id?: string,
    shard_block_id?: string
    core_version?: string,
    waiting_expiration_time?:string,
    block_time?: string,
    phase?: string
    exit_code
    exit_arg
    account_address?: string
    local_error: ErrorData
}
```

**message\_id**

Message id

**shard\_block\_id**

The last shardchain block of the account received before the error occurred.

**core\_version**

Core library binary version used

**waiting\_expiration\_time**

Message expiration time.

**block\_time**

Creation time of the last shardchain block of the account received before the error occurred.

**phase**

Transaction execution phase when contract execution was aborted

**exit\_code**

Exit code of exception thrown by the aborted contract execution

**exit\_arg**

Exit args provided along with exit code

**account\_address**

Address of the account

**local\_error: ErrorData**

Result of local transaction emulation performed after the message was not successfully delivered.

## Error example

Here you can see an error returned by process\_message function when message was not delivered to the blockchain and got expired \(code 507\).

In such cases SDK emulated the same transaction locally and here it got a local\_error with possible reason - wrong signature - exit code = 40.

```text
{
  "code": 507,
  "message": "Message expired. Possible reason: Contract execution was terminated with error: Contract did not accept message, exit code: 40 (Invalid signature). Check sign keys. For more information about exit code check the contract source code or ask the contract developer",
  "data": {
    "message_id": "31ed01a8c91d06e526cef015b406273377c41710216cc160af9428e1bb263671",
    "shard_block_id": "c8c8020c4404b099ec3af2a38373875c1fb8128ff0e61ed3186e8d822533a99f",
    "core_version": "1.6.1",
    "waiting_expiration_time": "Thu, 04 Feb 2021 00:49:29 +0300 (1612388969)",
    "block_time": "Thu, 04 Feb 2021 00:49:30 +0300 (1612388970)",
    "local_error": {
      "code": 414,
      "message": "Contract execution was terminated with error: Contract did not accept message, exit code: 40 (Invalid signature). Check sign keys. For more information about exit code check the contract source code or ask the contract developer",
      "data": {
        "core_version": "1.6.1",
        "phase": "computeVm",
        "exit_code": 40,
        "exit_arg": "0",
        "account_address": "0:c6cfd0506f8d33891690b34fafe3f686873afc42653ef88a11d73e4866fda928",
        "description": "Invalid signature"
      }
    }
  }
}
```


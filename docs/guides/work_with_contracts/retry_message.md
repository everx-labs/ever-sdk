# Retry Message

## Resending a Message

The process_message function has the retry feature. If [message had expired](message\_expiration.md), it can be re-created and re-sent to the network. Five attempts are carried out by default. If all of them fail, `507` error is returned as well. Use the `network.message_retries_count` parameter to customize the number of attempts at SDK initialization. If set to `0`, no further attempts will be carried out.

It is a good practice to increase the timeout between retries, for instance, if they are caused by bad connection or network lags (for instance when shard is temporarily served by some slow validators). `abi.message_expiration_timeout_grow_factor` parameter defines proportion of timeout increase with each retry. The default value is 1.5.

In this example the number of retries will be decreased to 3 and expiration timeout will be increased by 1.3 times with each retry.

```graphql
const client = new TonClient({
    network: {
        endpoints: [
            'eri01.net.everos.dev',
            'rbx01.net.everos.dev',
            'gra01.net.everos.dev',
        ],
        message_retries_count: 3
    },
    abi: {
        message_expiration_timeout: 120000,
        message_expiration_timeout_grow_factor: 1.3
    }
});
```

When you use separate functions to create (`encoded_message`) and to send a message (`send_message`), re-creating and re-sending should be carried out separately.

> **Note that only `507` error code from `process_message` or `wait_for_transaction` (if you specified the correct block id that you fetched before the message was sent) guarantees that the message is expired and will not be processed by the contract.**

## When to retry with 507 error

Normally, `process_message` and `wait_for_transaction` perform local emulation of transaction in case of expired message error and attach the results to the error object of `507` error ([ErrorData.exit_code](../../reference/error\_api.md)). It will give you a possible reason why the message was not executed on-chain (check exit code meaning in contract source code).

**If you see there is no `exit_code`, then do the retry. Also retry can be executed in case of replay protection `exit_code`.** Sometimes when a network undergoes high loads, this may happen.

# Monitor Messages

## When you may need message monitor?

The **Message Monitor object** is based on the [GraphQL API extension of the same name](https://docs.evercloud.dev/reference/graphql-api/message-monitor-api). It was developed to solve the problem of batch external message processing.

## About Message Monitor

`Message Monitor` caches the transactions of the last block to provide all users with shard block walking on the backend side, giving batch results, and removing the need to send multiple queries for every message in a batch to discover their status.

Also it ensures a transaction error trace is received on the backend if the transaction was not executed onchain within the specified timeout.

`Message Monitor` allows to:

* subscribe for the statuses of a batch of messages via websocket
* query the status of a batch of messages with 1 query
* monitor object in SDK provides additional capabilities that allow to process messages using a special queue.

Message Monitor Specifications are available here:  [**Message Monitor**](../../reference/types-and-methods/mod\_processing.md#monitor\_messages)**.**

## Usage

Let’s look at the following Javascript SDK sample, that offers two use cases:

1. I want to send 100 messages at 10 messages per second and get their processing status AS SOON as possible.
2. I want to send 100 messages at 10 messages per second and only get their status when ALL messages have been processed.

The relevant part of the first use case is below:

```json
const TOTAL_NUMBER_OF_MESSAGES = 100
const SEND_INTERVAL_SECONDS = 1
const BATCH_SIZE = 10

// The message is valid (can be processed by validators) no longer that 90 seconds after creation.
const EXP_TIMEOUT_SECONDS = 90

async function main(client: TonClient) {
    log("Starting use case #1")
    let queueName = "queue_1"

    await sendMessages(queueName) // We are waiting for sending only the first batch of messages.

    let resultsCounter = 0
    while (resultsCounter < TOTAL_NUMBER_OF_MESSAGES) {
        const result = await client.processing.fetch_next_monitor_results({
            queue: queueName,
            wait_mode: MonitorFetchWaitMode.AtLeastOne,
        })

        for (const elem of result.results) {
            resultsCounter++
            const processTime = Date.now() - elem.user_data.timestamp
            log(
                `Result: message processed in ${processTime} ms.`,
                elem.error || `Status: ${elem.status}`,
            )
        }
        /* If you're interested, you can call the `get_monitor_info` function, which
         * returns how many messages in a queue do not have final status yet ("unresolved")
         * When `wait_mode = MonitorFetchWaitMode.AtLeastOne`, property "resolved" is always equals to 0
         */
        const monitorInfo = await client.processing.get_monitor_info({
            queue: queueName,
        })
        log("monitor_info", monitorInfo)
    }
    log("End of use case #1, all results received\\n")
```

Here `wait_mode` is set to `MonitorFetchWaitMode.AtLeastOne`, making sure Monitor will deliver message processing results as they come in.

Compare to the second use case, where `wait_mode` is set to `MonitorFetchWaitMode.All`, making monitor wait for all results of the queue to be received:

```json
log("Starting use case #2")

    queueName = "queue_2"
    await sendMessages(queueName) // We are waiting for sending only the first batch of messages.

    const result = await client.processing.fetch_next_monitor_results({
        queue: queueName,
        wait_mode: MonitorFetchWaitMode.All,
    })

    for (const elem of result.results) {
        const processTime = Date.now() - elem.user_data.timestamp
        log(
            `Result: message processed in ${processTime} ms.`,
            elem.error || `Status: ${elem.status}`,
        )
    }
    log(
        result.results.length === TOTAL_NUMBER_OF_MESSAGES
            ? `End of use case #2, all results received`
            : `Error occured, expected ${TOTAL_NUMBER_OF_MESSAGES}, received ${result.results.length}`,
    )
}
```

## Sample source code

* Javascript:

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/message-monitor](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/message-monitor)

* Rust:

[https://github.com/tonlabs/sdk-samples/tree/master/core-examples/rust/message-monitoring](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/rust/message-monitoring)

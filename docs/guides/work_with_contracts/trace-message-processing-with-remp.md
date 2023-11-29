# Trace message processing with REMP

## Introduction

REMP (Reliable External Messaging Protocol) is a set of protocols and data structures that are designed to keep trace of incoming external messages and predict with a high probability whether a message will be processed successfully.

REMP adds some additional guarantees/features for external messages processing:

**1. Replay protection**

If a message is processed and added into an accepted block, then the same message **(that is, the message with the same hash)** will not be collated for some time period. If the message has some expiration time (corresponding to the time period), then this effectively makes efficient replay protection.

**2. No messages are lost**

You only need to send the message once. If there will be a possibility to accept it and add it to a block, then it will be done. Message loss may occur only for blockchain overloading reasons.

**3. One can trace the message processing.**

There are several checkpoints on the message processing path (when validators received the message, when message was added to a block, when the block was finalized, etc). Upon reaching certain checkpoints one can predict that the message will be successfully processed with a high accuracy - most messages can be considered to be processed when validators acknowledge that they were received (this happens in 100-200 ms; after that it’s highly unlikely that the message is declined). Thus, depending on the message importance one may trade efficiency for reliability in the software, choosing not to trace further processing results. On the other hand, if a transaction is really important, then you can wait till the block with the transaction result is issued.

During validation, as the message passes through various stages (that is, changes some statuses), validator sends receipts about each.

This guide teaches to work with REMP and REMP receipts in EVER SDK.

## Prerequisites

* Create a project on [dashboard.evercloud.dev](https://dashboard.evercloud.dev) if you don't have one.
* Remember its Development Network HTTPS endpoint.
* Pass this endpoint as a parameter when running the example.

## Working with REMP statuses

The sample sends a message to a predeployed test contract.

```javascript
(async () => {
    try {
        console.log('Sending messsage and waiting for REMP events.');
        const { transaction } = await client.processing.process_message(
            {
                send_events: true,
                message_encode_params: {
                    address: CONTRACT_ADDRESS,
                    abi: abiContract(CONTRACT_ABI),
                    call_set: {
                        function_name: 'touch',
                    },
                    signer: signerNone(),
                },
            },
            responseHandler,
        );

        console.log(
            [
                `The message has been processed.`,
                `${rempEventCnt} REMP events received`,
                `Transaction id: ${transaction.id}, status ${transaction.status_name}`,
            ].join('\n'),
        );
        client.close();
    } catch (error) {
        if (error.code === 504) {
            console.error('Network is inaccessible.');
        } else {
            console.error(error);
        }
        process.exit(1);
    }

```

After the message is sent all REMP events related to the message are received and printed, tracing the message processing by validators.&#x20;

**Note**: By REMP events one can predict with a high probability that the message will be processed successfully, before the processing is complete. Depending on message importance, you can choose to consider it processed as soon as the corresponding receipt arrives and not spend time and resources waiting for further status receipts.

```javascript
function responseHandler(params, responseType) {
    // Tip: Always wrap the logic inside responseHandler in a try-catch block
    // or you will be surprised by non-informative errors due to the context
    // in which the handler is executed
    try {
        if (responseType === 100 /* GraphQL data received */) {
            const { type, json, error } = params;

            assert.ok(type, 'Event always has type');

            if (type.startsWith('Remp')) {
                rempEventCnt++;
                // All REMP event types starts with `Remp`
                // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_processing#processingevent
                assert.ok(json || error, 'All REMP event has `json` or `error` property');
                if (json) {
                    // We print all REMP events.
                    console.log(`\tREMP event type: ${type}, kind: ${json.kind}`);

                    // but you can pay attention to only a few kinds of events:
                    if (json.kind === 'IncludedIntoBlock') {
                        console.log('\t^^^ this message is probably to be processed successfully');
                    }
                    if (json.kind === 'IncludedIntoAcceptedBlock') {
                        console.log(
                            '\t^^^ this message is highly likely to be processed successfully',
                        );
                    }
                }
                if (error) {
                    // Errors here indicate that there was a problem processing the REMP.
                    // This does not mean that the message cannot be processed successfully,
                    // it only means that the SDK just didn't get the next status at the expected time, see
                    // TonClient config params: `first_remp_status_timeout`, `next_remp_status_timeout`
                    // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_client#networkconfig
                    //
                    // In this case, the SDK switches to the scenario of waiting for a standby transaction (sequential block reading).
                    console.log(
                        `\tREMP event type: ${type}, code: ${error.code}, message: ${error.message}`,
                    );
                }
            } else {
                // In this example we are interested only in REMP events, so we skip
                // other events like `WillFetchFirstBlock`, `WillSend`, `DidSend`.
                // console.log(`Basic event ${type}`);
            }
        } else {
            // See full list of error codes here:
            // https://docs.everos.dev/ever-sdk/reference/types-and-methods/mod_net#neterrorcode
            console.log('ERROR', params, responseType);
        }
    } catch (err) {
        console.log(err);
    }
```

See the full example in sdk samples repository:

[https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/remp/index.js](https://github.com/tonlabs/sdk-samples/blob/master/core-examples/node-js/remp/index.js)

## Expected output

```
node index.js https://devnet.evercloud.dev/your-project-id/graphql

Sending messsage and waiting for REMP events.
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempOther, kind: PutIntoQueue
	REMP event type: RempIncludedIntoBlock, kind: IncludedIntoBlock
	^^^ this message is probably to be processed successfully
	REMP event type: RempOther, kind: Duplicate
	REMP event type: RempOther, kind: Duplicate
	REMP event type: RempOther, kind: Duplicate
	REMP event type: RempOther, kind: Duplicate
	REMP event type: RempOther, kind: Duplicate
	REMP event type: RempIncludedIntoAcceptedBlock, kind: IncludedIntoAcceptedBlock
	^^^ this message is highly likely to be processed successfully
The message has been processed.
13 REMP events received
Transaction id: effed4849898e08d1fe5759532d34f23dbec061c5fd666604f817be82732cfb9, status finalized
```

## Sample Source code

Full sample: [https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/remp](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/remp)

## See also

Guides for working with REMP in GraphQL API:

{% embed url="https://docs.everplatform.dev/samples/graphql-samples/subscribe-for-remp-receipts" %}

{% embed url="https://docs.everplatform.dev/samples/graphql-samples/send-message" %}

REMP Architecture documentation:

{% embed url="https://docs.everscale.network/arch/remp" %}

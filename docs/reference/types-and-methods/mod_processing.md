# Module processing

Message processing module.

This module incorporates functions related to complex message
processing scenarios.


## Functions
[monitor_messages](mod\_processing.md#monitor_messages) – Starts monitoring for the processing results of the specified messages.

[get_monitor_info](mod\_processing.md#get_monitor_info) – Returns summary information about current state of the specified monitoring queue.

[fetch_next_monitor_results](mod\_processing.md#fetch_next_monitor_results) – Fetches next resolved results from the specified monitoring queue.

[cancel_monitor](mod\_processing.md#cancel_monitor) – Cancels all background activity and releases all allocated system resources for the specified monitoring queue.

[send_messages](mod\_processing.md#send_messages) – Sends specified messages to the blockchain.

[send_message](mod\_processing.md#send_message) – Sends message to the network

[wait_for_transaction](mod\_processing.md#wait_for_transaction) – Performs monitoring of the network for the result transaction of the external inbound message processing.

[process_message](mod\_processing.md#process_message) – Creates message, sends it to the network and monitors its processing.

## Types
[ProcessingErrorCode](mod\_processing.md#processingerrorcode)

[ProcessingEventWillFetchFirstBlockVariant](mod\_processing.md#processingeventwillfetchfirstblockvariant) – Notifies the application that the account's current shard block will be fetched from the network. This step is performed before the message sending so that sdk knows starting from which block it will search for the transaction.

[ProcessingEventFetchFirstBlockFailedVariant](mod\_processing.md#processingeventfetchfirstblockfailedvariant) – Notifies the app that the client has failed to fetch the account's current shard block.

[ProcessingEventWillSendVariant](mod\_processing.md#processingeventwillsendvariant) – Notifies the app that the message will be sent to the network. This event means that the account's current shard block was successfully fetched and the message was successfully created (`abi.encode_message` function was executed successfully).

[ProcessingEventDidSendVariant](mod\_processing.md#processingeventdidsendvariant) – Notifies the app that the message was sent to the network, i.e `processing.send_message` was successfully executed. Now, the message is in the blockchain. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event.

[ProcessingEventSendFailedVariant](mod\_processing.md#processingeventsendfailedvariant) – Notifies the app that the sending operation was failed with network error.

[ProcessingEventWillFetchNextBlockVariant](mod\_processing.md#processingeventwillfetchnextblockvariant) – Notifies the app that the next shard block will be fetched from the network.

[ProcessingEventFetchNextBlockFailedVariant](mod\_processing.md#processingeventfetchnextblockfailedvariant) – Notifies the app that the next block can't be fetched.

[ProcessingEventMessageExpiredVariant](mod\_processing.md#processingeventmessageexpiredvariant) – Notifies the app that the message was not executed within expire timeout on-chain and will never be because it is already expired. The expiration timeout can be configured with `AbiConfig` parameters.

[ProcessingEventRempSentToValidatorsVariant](mod\_processing.md#processingeventrempsenttovalidatorsvariant) – Notifies the app that the message has been delivered to the thread's validators

[ProcessingEventRempIncludedIntoBlockVariant](mod\_processing.md#processingeventrempincludedintoblockvariant) – Notifies the app that the message has been successfully included into a block candidate by the thread's collator

[ProcessingEventRempIncludedIntoAcceptedBlockVariant](mod\_processing.md#processingeventrempincludedintoacceptedblockvariant) – Notifies the app that the block candidate with the message has been accepted by the thread's validators

[ProcessingEventRempOtherVariant](mod\_processing.md#processingeventrempothervariant) – Notifies the app about some other minor REMP statuses occurring during message processing

[ProcessingEventRempErrorVariant](mod\_processing.md#processingeventremperrorvariant) – Notifies the app about any problem that has occurred in REMP processing - in this case library switches to the fallback transaction awaiting scenario (sequential block reading).

[ProcessingEvent](mod\_processing.md#processingevent)

[ResultOfProcessMessage](mod\_processing.md#resultofprocessmessage)

[DecodedOutput](mod\_processing.md#decodedoutput)

[MessageMonitoringTransactionCompute](mod\_processing.md#messagemonitoringtransactioncompute)

[MessageMonitoringTransaction](mod\_processing.md#messagemonitoringtransaction)

[MessageMonitoringParams](mod\_processing.md#messagemonitoringparams)

[MessageMonitoringResult](mod\_processing.md#messagemonitoringresult)

[MonitorFetchWaitMode](mod\_processing.md#monitorfetchwaitmode)

[MonitoredMessageBocVariant](mod\_processing.md#monitoredmessagebocvariant) – BOC of the message.

[MonitoredMessageHashAddressVariant](mod\_processing.md#monitoredmessagehashaddressvariant) – Message's hash and destination address.

[MonitoredMessage](mod\_processing.md#monitoredmessage)

[MessageMonitoringStatus](mod\_processing.md#messagemonitoringstatus)

[MessageSendingParams](mod\_processing.md#messagesendingparams)

[ParamsOfMonitorMessages](mod\_processing.md#paramsofmonitormessages)

[ParamsOfGetMonitorInfo](mod\_processing.md#paramsofgetmonitorinfo)

[MonitoringQueueInfo](mod\_processing.md#monitoringqueueinfo)

[ParamsOfFetchNextMonitorResults](mod\_processing.md#paramsoffetchnextmonitorresults)

[ResultOfFetchNextMonitorResults](mod\_processing.md#resultoffetchnextmonitorresults)

[ParamsOfCancelMonitor](mod\_processing.md#paramsofcancelmonitor)

[ParamsOfSendMessages](mod\_processing.md#paramsofsendmessages)

[ResultOfSendMessages](mod\_processing.md#resultofsendmessages)

[ParamsOfSendMessage](mod\_processing.md#paramsofsendmessage)

[ResultOfSendMessage](mod\_processing.md#resultofsendmessage)

[ParamsOfWaitForTransaction](mod\_processing.md#paramsofwaitfortransaction)

[ParamsOfProcessMessage](mod\_processing.md#paramsofprocessmessage)


# Functions
## monitor_messages

Starts monitoring for the processing results of the specified messages.

Message monitor performs background monitoring for a message processing results
for the specified set of messages.

Message monitor can serve several isolated monitoring queues.
Each monitor queue has a unique application defined identifier (or name) used
to separate several queue's.

There are two important lists inside of the monitoring queue:

- unresolved messages: contains messages requested by the application for monitoring
  and not yet resolved;

- resolved results: contains resolved processing results for monitored messages.

Each monitoring queue tracks own unresolved and resolved lists.
Application can add more messages to the monitoring queue at any time.

Message monitor accumulates resolved results.
Application should fetch this results with `fetchNextMonitorResults` function.

When both unresolved and resolved lists becomes empty, monitor stops any background activity
and frees all allocated internal memory.

If monitoring queue with specified name already exists then messages will be added
to the unresolved list.

If monitoring queue with specified name does not exist then monitoring queue will be created
with specified unresolved messages.

```ts
type ParamsOfMonitorMessages = {
    queue: string,
    messages: MessageMonitoringParams[]
}

function monitor_messages(
    params: ParamsOfMonitorMessages,
): Promise<void>;

function monitor_messages_sync(
    params: ParamsOfMonitorMessages,
): void;
```
### Parameters
- `queue`: _string_ – Name of the monitoring queue.
- `messages`: _[MessageMonitoringParams](mod\_processing.md#messagemonitoringparams)[]_ – Messages to start monitoring for.


## get_monitor_info

Returns summary information about current state of the specified monitoring queue.

```ts
type ParamsOfGetMonitorInfo = {
    queue: string
}

type MonitoringQueueInfo = {
    unresolved: number,
    resolved: number
}

function get_monitor_info(
    params: ParamsOfGetMonitorInfo,
): Promise<MonitoringQueueInfo>;

function get_monitor_info_sync(
    params: ParamsOfGetMonitorInfo,
): MonitoringQueueInfo;
```
### Parameters
- `queue`: _string_ – Name of the monitoring queue.


### Result

- `unresolved`: _number_ – Count of the unresolved messages.
- `resolved`: _number_ – Count of resolved results.


## fetch_next_monitor_results

Fetches next resolved results from the specified monitoring queue.

Results and waiting options are depends on the `wait` parameter.
All returned results will be removed from the queue's resolved list.

```ts
type ParamsOfFetchNextMonitorResults = {
    queue: string,
    wait_mode?: MonitorFetchWaitMode
}

type ResultOfFetchNextMonitorResults = {
    results: MessageMonitoringResult[]
}

function fetch_next_monitor_results(
    params: ParamsOfFetchNextMonitorResults,
): Promise<ResultOfFetchNextMonitorResults>;

function fetch_next_monitor_results_sync(
    params: ParamsOfFetchNextMonitorResults,
): ResultOfFetchNextMonitorResults;
```
### Parameters
- `queue`: _string_ – Name of the monitoring queue.
- `wait_mode`?: _[MonitorFetchWaitMode](mod\_processing.md#monitorfetchwaitmode)_ – Wait mode.
<br>Default is `NO_WAIT`.


### Result

- `results`: _[MessageMonitoringResult](mod\_processing.md#messagemonitoringresult)[]_ – List of the resolved results.


## cancel_monitor

Cancels all background activity and releases all allocated system resources for the specified monitoring queue.

```ts
type ParamsOfCancelMonitor = {
    queue: string
}

function cancel_monitor(
    params: ParamsOfCancelMonitor,
): Promise<void>;

function cancel_monitor_sync(
    params: ParamsOfCancelMonitor,
): void;
```
### Parameters
- `queue`: _string_ – Name of the monitoring queue.


## send_messages

Sends specified messages to the blockchain.

```ts
type ParamsOfSendMessages = {
    messages: MessageSendingParams[],
    monitor_queue?: string
}

type ResultOfSendMessages = {
    messages: MessageMonitoringParams[]
}

function send_messages(
    params: ParamsOfSendMessages,
): Promise<ResultOfSendMessages>;

function send_messages_sync(
    params: ParamsOfSendMessages,
): ResultOfSendMessages;
```
### Parameters
- `messages`: _[MessageSendingParams](mod\_processing.md#messagesendingparams)[]_ – Messages that must be sent to the blockchain.
- `monitor_queue`?: _string_ – Optional message monitor queue that starts monitoring for the processing results for sent messages.


### Result

- `messages`: _[MessageMonitoringParams](mod\_processing.md#messagemonitoringparams)[]_ – Messages that was sent to the blockchain for execution.


## send_message

Sends message to the network

Sends message to the network and returns the last generated shard block of the destination account
before the message was sent. It will be required later for message processing.

```ts
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
}

type ResultOfSendMessage = {
    shard_block_id: string,
    sending_endpoints: string[]
}

function send_message(
    params: ParamsOfSendMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfSendMessage>;

function send_message_sync(
    params: ParamsOfSendMessage,
): ResultOfSendMessage;
```
### Parameters
- `message`: _string_ – Message BOC.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Optional message ABI.
<br>If this parameter is specified and the message has the<br>`expire` header then expiration time will be checked against<br>the current time to prevent unnecessary sending of already expired message.<br><br>The `message already expired` error will be returned in this<br>case.<br><br>Note, that specifying `abi` for ABI compliant contracts is<br>strongly recommended, so that proper processing strategy can be<br>chosen.
- `send_events`: _boolean_ – Flag for requesting events sending
- `responseHandler`?: _[ResponseHandler](modules.md#responsehandler)_ – additional responses handler.

### Result

- `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.
<br>This block id must be used as a parameter of the<br>`wait_for_transaction`.
- `sending_endpoints`: _string[]_ – The list of endpoints to which the message was sent.
<br>This list id must be used as a parameter of the<br>`wait_for_transaction`.


## wait_for_transaction

Performs monitoring of the network for the result transaction of the external inbound message processing.

`send_events` enables intermediate events, such as `WillFetchNextBlock`,
`FetchNextBlockFailed` that may be useful for logging of new shard blocks creation
during message processing.

Note, that presence of the `abi` parameter is critical for ABI
compliant contracts. Message processing uses drastically
different strategy for processing message for contracts which
ABI includes "expire" header.

When the ABI header `expire` is present, the processing uses
`message expiration` strategy:
- The maximum block gen time is set to
  `message_expiration_timeout + transaction_wait_timeout`.
- When maximum block gen time is reached, the processing will
  be finished with `MessageExpired` error.

When the ABI header `expire` isn't present or `abi` parameter
isn't specified, the processing uses `transaction waiting`
strategy:
- The maximum block gen time is set to
  `now() + transaction_wait_timeout`.

- If maximum block gen time is reached and no result transaction is found,
the processing will exit with an error.

```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean,
    sending_endpoints?: string[]
}

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}

function wait_for_transaction(
    params: ParamsOfWaitForTransaction,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;

function wait_for_transaction_sync(
    params: ParamsOfWaitForTransaction,
): ResultOfProcessMessage;
```
### Parameters
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Optional ABI for decoding the transaction result.
<br>If it is specified, then the output messages' bodies will be<br>decoded according to this ABI.<br><br>The `abi_decoded` result field will be filled out.
- `message`: _string_ – Message BOC.
<br>Encoded with `base64`.
- `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.
<br>You must provide the same value as the `send_message` has returned.
- `send_events`: _boolean_ – Flag that enables/disables intermediate events
- `sending_endpoints`?: _string[]_ – The list of endpoints to which the message was sent.
<br>Use this field to get more informative errors.<br>Provide the same value as the `send_message` has returned.<br>If the message was not delivered (expired), SDK will log the endpoint URLs, used for its sending.
- `responseHandler`?: _[ResponseHandler](modules.md#responsehandler)_ – additional responses handler.

### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod\_tvm.md#transactionfees)_ – Transaction fees


## process_message

Creates message, sends it to the network and monitors its processing.

Creates ABI-compatible message,
sends it to the network and monitors for the result transaction.
Decodes the output messages' bodies.

If contract's ABI includes "expire" header, then
SDK implements retries in case of unsuccessful message delivery within the expiration
timeout: SDK recreates the message, sends it and processes it again.

The intermediate events, such as `WillFetchFirstBlock`, `WillSend`, `DidSend`,
`WillFetchNextBlock`, etc - are switched on/off by `send_events` flag
and logged into the supplied callback function.

The retry configuration parameters are defined in the client's `NetworkConfig` and `AbiConfig`.

If contract's ABI does not include "expire" header
then, if no transaction is found within the network timeout (see config parameter ), exits with error.

```ts
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
}

type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}

function process_message(
    params: ParamsOfProcessMessage,
    responseHandler?: ResponseHandler,
): Promise<ResultOfProcessMessage>;

function process_message_sync(
    params: ParamsOfProcessMessage,
): ResultOfProcessMessage;
```
### Parameters
- `message_encode_params`: _[ParamsOfEncodeMessage](mod\_abi.md#paramsofencodemessage)_ – Message encode parameters.
- `send_events`: _boolean_ – Flag for requesting events sending
- `responseHandler`?: _[ResponseHandler](modules.md#responsehandler)_ – additional responses handler.

### Result

- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod\_tvm.md#transactionfees)_ – Transaction fees


# Types
## ProcessingErrorCode
```ts
enum ProcessingErrorCode {
    MessageAlreadyExpired = 501,
    MessageHasNotDestinationAddress = 502,
    CanNotBuildMessageCell = 503,
    FetchBlockFailed = 504,
    SendMessageFailed = 505,
    InvalidMessageBoc = 506,
    MessageExpired = 507,
    TransactionWaitTimeout = 508,
    InvalidBlockReceived = 509,
    CanNotCheckBlockShard = 510,
    BlockNotFound = 511,
    InvalidData = 512,
    ExternalSignerMustNotBeUsed = 513,
    MessageRejected = 514,
    InvalidRempStatus = 515,
    NextRempStatusTimeout = 516
}
```
One of the following value:

- `MessageAlreadyExpired = 501`
- `MessageHasNotDestinationAddress = 502`
- `CanNotBuildMessageCell = 503`
- `FetchBlockFailed = 504`
- `SendMessageFailed = 505`
- `InvalidMessageBoc = 506`
- `MessageExpired = 507`
- `TransactionWaitTimeout = 508`
- `InvalidBlockReceived = 509`
- `CanNotCheckBlockShard = 510`
- `BlockNotFound = 511`
- `InvalidData = 512`
- `ExternalSignerMustNotBeUsed = 513`
- `MessageRejected = 514`
- `InvalidRempStatus = 515`
- `NextRempStatusTimeout = 516`


## ProcessingEventWillFetchFirstBlockVariant
Notifies the application that the account's current shard block will be fetched from the network. This step is performed before the message sending so that sdk knows starting from which block it will search for the transaction.

Fetched block will be used later in waiting phase.

```ts
type ProcessingEventWillFetchFirstBlockVariant = {
    message_id: string,
    message_dst: string
}
```
- `message_id`: _string_
- `message_dst`: _string_


## ProcessingEventFetchFirstBlockFailedVariant
Notifies the app that the client has failed to fetch the account's current shard block.

This may happen due to the network issues. Receiving this event means that message processing will not proceed -
message was not sent, and Developer can try to run `process_message` again,
in the hope that the connection is restored.

```ts
type ProcessingEventFetchFirstBlockFailedVariant = {
    error: ClientError,
    message_id: string,
    message_dst: string
}
```
- `error`: _[ClientError](mod\_client.md#clienterror)_
- `message_id`: _string_
- `message_dst`: _string_


## ProcessingEventWillSendVariant
Notifies the app that the message will be sent to the network. This event means that the account's current shard block was successfully fetched and the message was successfully created (`abi.encode_message` function was executed successfully).

```ts
type ProcessingEventWillSendVariant = {
    shard_block_id: string,
    message_id: string,
    message_dst: string,
    message: string
}
```
- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_


## ProcessingEventDidSendVariant
Notifies the app that the message was sent to the network, i.e `processing.send_message` was successfully executed. Now, the message is in the blockchain. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event.

Do not forget to specify abi of your contract as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

```ts
type ProcessingEventDidSendVariant = {
    shard_block_id: string,
    message_id: string,
    message_dst: string,
    message: string
}
```
- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_


## ProcessingEventSendFailedVariant
Notifies the app that the sending operation was failed with network error.

Nevertheless the processing will be continued at the waiting
phase because the message possibly has been delivered to the
node.
If Application exits at this phase, Developer needs to proceed with processing
after the application is restored with `wait_for_transaction` function, passing
shard_block_id and message from this event. Do not forget to specify abi of your contract
as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

```ts
type ProcessingEventSendFailedVariant = {
    shard_block_id: string,
    message_id: string,
    message_dst: string,
    message: string,
    error: ClientError
}
```
- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_


## ProcessingEventWillFetchNextBlockVariant
Notifies the app that the next shard block will be fetched from the network.

Event can occurs more than one time due to block walking
procedure.
If Application exits at this phase, Developer needs to proceed with processing
after the application is restored with `wait_for_transaction` function, passing
shard_block_id and message from this event. Do not forget to specify abi of your contract
as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

```ts
type ProcessingEventWillFetchNextBlockVariant = {
    shard_block_id: string,
    message_id: string,
    message_dst: string,
    message: string
}
```
- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_


## ProcessingEventFetchNextBlockFailedVariant
Notifies the app that the next block can't be fetched.

If no block was fetched within `NetworkConfig.wait_for_timeout` then processing stops.
This may happen when the shard stops, or there are other network issues.
In this case Developer should resume message processing with `wait_for_transaction`, passing shard_block_id,
message and contract abi to it. Note that passing ABI is crucial, because it will influence the processing strategy.

Another way to tune this is to specify long timeout in `NetworkConfig.wait_for_timeout`

```ts
type ProcessingEventFetchNextBlockFailedVariant = {
    shard_block_id: string,
    message_id: string,
    message_dst: string,
    message: string,
    error: ClientError
}
```
- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_


## ProcessingEventMessageExpiredVariant
Notifies the app that the message was not executed within expire timeout on-chain and will never be because it is already expired. The expiration timeout can be configured with `AbiConfig` parameters.

This event occurs only for the contracts which ABI includes "expire" header.

If Application specifies `NetworkConfig.message_retries_count` > 0, then `process_message`
will perform retries: will create a new message and send it again and repeat it until it reaches
the maximum retries count or receives a successful result.  All the processing
events will be repeated.

```ts
type ProcessingEventMessageExpiredVariant = {
    message_id: string,
    message_dst: string,
    message: string,
    error: ClientError
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_


## ProcessingEventRempSentToValidatorsVariant
Notifies the app that the message has been delivered to the thread's validators

```ts
type ProcessingEventRempSentToValidatorsVariant = {
    message_id: string,
    message_dst: string,
    timestamp: bigint,
    json: any
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_


## ProcessingEventRempIncludedIntoBlockVariant
Notifies the app that the message has been successfully included into a block candidate by the thread's collator

```ts
type ProcessingEventRempIncludedIntoBlockVariant = {
    message_id: string,
    message_dst: string,
    timestamp: bigint,
    json: any
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_


## ProcessingEventRempIncludedIntoAcceptedBlockVariant
Notifies the app that the block candidate with the message has been accepted by the thread's validators

```ts
type ProcessingEventRempIncludedIntoAcceptedBlockVariant = {
    message_id: string,
    message_dst: string,
    timestamp: bigint,
    json: any
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_


## ProcessingEventRempOtherVariant
Notifies the app about some other minor REMP statuses occurring during message processing

```ts
type ProcessingEventRempOtherVariant = {
    message_id: string,
    message_dst: string,
    timestamp: bigint,
    json: any
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_


## ProcessingEventRempErrorVariant
Notifies the app about any problem that has occurred in REMP processing - in this case library switches to the fallback transaction awaiting scenario (sequential block reading).

```ts
type ProcessingEventRempErrorVariant = {
    message_id: string,
    message_dst: string,
    error: ClientError
}
```
- `message_id`: _string_
- `message_dst`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_


## ProcessingEvent
```ts
type ProcessingEvent = ({
    type: 'WillFetchFirstBlock'
} & ProcessingEventWillFetchFirstBlockVariant) | ({
    type: 'FetchFirstBlockFailed'
} & ProcessingEventFetchFirstBlockFailedVariant) | ({
    type: 'WillSend'
} & ProcessingEventWillSendVariant) | ({
    type: 'DidSend'
} & ProcessingEventDidSendVariant) | ({
    type: 'SendFailed'
} & ProcessingEventSendFailedVariant) | ({
    type: 'WillFetchNextBlock'
} & ProcessingEventWillFetchNextBlockVariant) | ({
    type: 'FetchNextBlockFailed'
} & ProcessingEventFetchNextBlockFailedVariant) | ({
    type: 'MessageExpired'
} & ProcessingEventMessageExpiredVariant) | ({
    type: 'RempSentToValidators'
} & ProcessingEventRempSentToValidatorsVariant) | ({
    type: 'RempIncludedIntoBlock'
} & ProcessingEventRempIncludedIntoBlockVariant) | ({
    type: 'RempIncludedIntoAcceptedBlock'
} & ProcessingEventRempIncludedIntoAcceptedBlockVariant) | ({
    type: 'RempOther'
} & ProcessingEventRempOtherVariant) | ({
    type: 'RempError'
} & ProcessingEventRempErrorVariant)
```
Depends on value of the  `type` field.

When _type_ is _'WillFetchFirstBlock'_

Notifies the application that the account's current shard block will be fetched from the network. This step is performed before the message sending so that sdk knows starting from which block it will search for the transaction.

Fetched block will be used later in waiting phase.

- `message_id`: _string_
- `message_dst`: _string_

When _type_ is _'FetchFirstBlockFailed'_

Notifies the app that the client has failed to fetch the account's current shard block.

This may happen due to the network issues. Receiving this event means that message processing will not proceed -
message was not sent, and Developer can try to run `process_message` again,
in the hope that the connection is restored.

- `error`: _[ClientError](mod\_client.md#clienterror)_
- `message_id`: _string_
- `message_dst`: _string_

When _type_ is _'WillSend'_

Notifies the app that the message will be sent to the network. This event means that the account's current shard block was successfully fetched and the message was successfully created (`abi.encode_message` function was executed successfully).

- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_

When _type_ is _'DidSend'_

Notifies the app that the message was sent to the network, i.e `processing.send_message` was successfully executed. Now, the message is in the blockchain. If Application exits at this phase, Developer needs to proceed with processing after the application is restored with `wait_for_transaction` function, passing shard_block_id and message from this event.

Do not forget to specify abi of your contract as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_

When _type_ is _'SendFailed'_

Notifies the app that the sending operation was failed with network error.

Nevertheless the processing will be continued at the waiting
phase because the message possibly has been delivered to the
node.
If Application exits at this phase, Developer needs to proceed with processing
after the application is restored with `wait_for_transaction` function, passing
shard_block_id and message from this event. Do not forget to specify abi of your contract
as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_

When _type_ is _'WillFetchNextBlock'_

Notifies the app that the next shard block will be fetched from the network.

Event can occurs more than one time due to block walking
procedure.
If Application exits at this phase, Developer needs to proceed with processing
after the application is restored with `wait_for_transaction` function, passing
shard_block_id and message from this event. Do not forget to specify abi of your contract
as well, it is crucial for processing. See `processing.wait_for_transaction` documentation.

- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_

When _type_ is _'FetchNextBlockFailed'_

Notifies the app that the next block can't be fetched.

If no block was fetched within `NetworkConfig.wait_for_timeout` then processing stops.
This may happen when the shard stops, or there are other network issues.
In this case Developer should resume message processing with `wait_for_transaction`, passing shard_block_id,
message and contract abi to it. Note that passing ABI is crucial, because it will influence the processing strategy.

Another way to tune this is to specify long timeout in `NetworkConfig.wait_for_timeout`

- `shard_block_id`: _string_
- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_

When _type_ is _'MessageExpired'_

Notifies the app that the message was not executed within expire timeout on-chain and will never be because it is already expired. The expiration timeout can be configured with `AbiConfig` parameters.

This event occurs only for the contracts which ABI includes "expire" header.

If Application specifies `NetworkConfig.message_retries_count` > 0, then `process_message`
will perform retries: will create a new message and send it again and repeat it until it reaches
the maximum retries count or receives a successful result.  All the processing
events will be repeated.

- `message_id`: _string_
- `message_dst`: _string_
- `message`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_

When _type_ is _'RempSentToValidators'_

Notifies the app that the message has been delivered to the thread's validators

- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_

When _type_ is _'RempIncludedIntoBlock'_

Notifies the app that the message has been successfully included into a block candidate by the thread's collator

- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_

When _type_ is _'RempIncludedIntoAcceptedBlock'_

Notifies the app that the block candidate with the message has been accepted by the thread's validators

- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_

When _type_ is _'RempOther'_

Notifies the app about some other minor REMP statuses occurring during message processing

- `message_id`: _string_
- `message_dst`: _string_
- `timestamp`: _bigint_
- `json`: _any_

When _type_ is _'RempError'_

Notifies the app about any problem that has occurred in REMP processing - in this case library switches to the fallback transaction awaiting scenario (sequential block reading).

- `message_id`: _string_
- `message_dst`: _string_
- `error`: _[ClientError](mod\_client.md#clienterror)_


Variant constructors:

```ts
function processingEventWillFetchFirstBlock(message_id: string, message_dst: string): ProcessingEvent;
function processingEventFetchFirstBlockFailed(error: ClientError, message_id: string, message_dst: string): ProcessingEvent;
function processingEventWillSend(shard_block_id: string, message_id: string, message_dst: string, message: string): ProcessingEvent;
function processingEventDidSend(shard_block_id: string, message_id: string, message_dst: string, message: string): ProcessingEvent;
function processingEventSendFailed(shard_block_id: string, message_id: string, message_dst: string, message: string, error: ClientError): ProcessingEvent;
function processingEventWillFetchNextBlock(shard_block_id: string, message_id: string, message_dst: string, message: string): ProcessingEvent;
function processingEventFetchNextBlockFailed(shard_block_id: string, message_id: string, message_dst: string, message: string, error: ClientError): ProcessingEvent;
function processingEventMessageExpired(message_id: string, message_dst: string, message: string, error: ClientError): ProcessingEvent;
function processingEventRempSentToValidators(message_id: string, message_dst: string, timestamp: bigint, json: any): ProcessingEvent;
function processingEventRempIncludedIntoBlock(message_id: string, message_dst: string, timestamp: bigint, json: any): ProcessingEvent;
function processingEventRempIncludedIntoAcceptedBlock(message_id: string, message_dst: string, timestamp: bigint, json: any): ProcessingEvent;
function processingEventRempOther(message_id: string, message_dst: string, timestamp: bigint, json: any): ProcessingEvent;
function processingEventRempError(message_id: string, message_dst: string, error: ClientError): ProcessingEvent;
```

## ResultOfProcessMessage
```ts
type ResultOfProcessMessage = {
    transaction: any,
    out_messages: string[],
    decoded?: DecodedOutput,
    fees: TransactionFees
}
```
- `transaction`: _any_ – Parsed transaction.
<br>In addition to the regular transaction fields there is a<br>`boc` field encoded with `base64` which contains source<br>transaction BOC.
- `out_messages`: _string[]_ – List of output messages' BOCs.
<br>Encoded as `base64`
- `decoded`?: _[DecodedOutput](mod\_processing.md#decodedoutput)_ – Optional decoded message bodies according to the optional `abi` parameter.
- `fees`: _[TransactionFees](mod\_tvm.md#transactionfees)_ – Transaction fees


## DecodedOutput
```ts
type DecodedOutput = {
    out_messages: DecodedMessageBody | null[],
    output?: any
}
```
- `out_messages`: _[DecodedMessageBody](mod\_abi.md#decodedmessagebody)?[]_ – Decoded bodies of the out messages.
<br>If the message can't be decoded, then `None` will be stored in<br>the appropriate position.
- `output`?: _any_ – Decoded body of the function output message.


## MessageMonitoringTransactionCompute
```ts
type MessageMonitoringTransactionCompute = {
    exit_code: number
}
```
- `exit_code`: _number_ – Compute phase exit code.


## MessageMonitoringTransaction
```ts
type MessageMonitoringTransaction = {
    hash?: string,
    aborted: boolean,
    compute?: MessageMonitoringTransactionCompute
}
```
- `hash`?: _string_ – Hash of the transaction. Present if transaction was included into the blocks. When then transaction was emulated this field will be missing.
- `aborted`: _boolean_ – Aborted field of the transaction.
- `compute`?: _[MessageMonitoringTransactionCompute](mod\_processing.md#messagemonitoringtransactioncompute)_ – Optional information about the compute phase of the transaction.


## MessageMonitoringParams
```ts
type MessageMonitoringParams = {
    message: MonitoredMessage,
    wait_until: number,
    user_data?: any
}
```
- `message`: _[MonitoredMessage](mod\_processing.md#monitoredmessage)_ – Monitored message identification. Can be provided as a message's BOC or (hash, address) pair. BOC is a preferable way because it helps to determine possible error reason (using TVM execution of the message).
- `wait_until`: _number_ – Block time Must be specified as a UNIX timestamp in seconds
- `user_data`?: _any_ – User defined data associated with this message. Helps to identify this message when user received `MessageMonitoringResult`.


## MessageMonitoringResult
```ts
type MessageMonitoringResult = {
    hash: string,
    status: MessageMonitoringStatus,
    transaction?: MessageMonitoringTransaction,
    error?: string,
    user_data?: any
}
```
- `hash`: _string_ – Hash of the message.
- `status`: _[MessageMonitoringStatus](mod\_processing.md#messagemonitoringstatus)_ – Processing status.
- `transaction`?: _[MessageMonitoringTransaction](mod\_processing.md#messagemonitoringtransaction)_ – In case of `Finalized` the transaction is extracted from the block. In case of `Timeout` the transaction is emulated using the last known account state.
- `error`?: _string_ – In case of `Timeout` contains possible error reason.
- `user_data`?: _any_ – User defined data related to this message. This is the same value as passed before with `MessageMonitoringParams` or `SendMessageParams`.


## MonitorFetchWaitMode
```ts
enum MonitorFetchWaitMode {
    AtLeastOne = "AtLeastOne",
    All = "All",
    NoWait = "NoWait"
}
```
One of the following value:

- `AtLeastOne = "AtLeastOne"` – If there are no resolved results yet, then monitor awaits for the next resolved result.
- `All = "All"` – Monitor waits until all unresolved messages will be resolved. If there are no unresolved messages then monitor will wait.
- `NoWait = "NoWait"`


## MonitoredMessageBocVariant
BOC of the message.

```ts
type MonitoredMessageBocVariant = {
    boc: string
}
```
- `boc`: _string_


## MonitoredMessageHashAddressVariant
Message's hash and destination address.

```ts
type MonitoredMessageHashAddressVariant = {
    hash: string,
    address: string
}
```
- `hash`: _string_ – Hash of the message.
- `address`: _string_ – Destination address of the message.


## MonitoredMessage
```ts
type MonitoredMessage = ({
    type: 'Boc'
} & MonitoredMessageBocVariant) | ({
    type: 'HashAddress'
} & MonitoredMessageHashAddressVariant)
```
Depends on value of the  `type` field.

When _type_ is _'Boc'_

BOC of the message.

- `boc`: _string_

When _type_ is _'HashAddress'_

Message's hash and destination address.

- `hash`: _string_ – Hash of the message.
- `address`: _string_ – Destination address of the message.


Variant constructors:

```ts
function monitoredMessageBoc(boc: string): MonitoredMessage;
function monitoredMessageHashAddress(hash: string, address: string): MonitoredMessage;
```

## MessageMonitoringStatus
```ts
enum MessageMonitoringStatus {
    Finalized = "Finalized",
    Timeout = "Timeout",
    Reserved = "Reserved"
}
```
One of the following value:

- `Finalized = "Finalized"` – Returned when the messages was processed and included into finalized block before `wait_until` block time.
- `Timeout = "Timeout"` – Returned when the message was not processed until `wait_until` block time.
- `Reserved = "Reserved"` – Reserved for future statuses.
<br>Is never returned. Application should wait for one of the `Finalized` or `Timeout` statuses.<br>All other statuses are intermediate.


## MessageSendingParams
```ts
type MessageSendingParams = {
    boc: string,
    wait_until: number,
    user_data?: any
}
```
- `boc`: _string_ – BOC of the message, that must be sent to the blockchain.
- `wait_until`: _number_ – Expiration time of the message. Must be specified as a UNIX timestamp in seconds.
- `user_data`?: _any_ – User defined data associated with this message. Helps to identify this message when user received `MessageMonitoringResult`.


## ParamsOfMonitorMessages
```ts
type ParamsOfMonitorMessages = {
    queue: string,
    messages: MessageMonitoringParams[]
}
```
- `queue`: _string_ – Name of the monitoring queue.
- `messages`: _[MessageMonitoringParams](mod\_processing.md#messagemonitoringparams)[]_ – Messages to start monitoring for.


## ParamsOfGetMonitorInfo
```ts
type ParamsOfGetMonitorInfo = {
    queue: string
}
```
- `queue`: _string_ – Name of the monitoring queue.


## MonitoringQueueInfo
```ts
type MonitoringQueueInfo = {
    unresolved: number,
    resolved: number
}
```
- `unresolved`: _number_ – Count of the unresolved messages.
- `resolved`: _number_ – Count of resolved results.


## ParamsOfFetchNextMonitorResults
```ts
type ParamsOfFetchNextMonitorResults = {
    queue: string,
    wait_mode?: MonitorFetchWaitMode
}
```
- `queue`: _string_ – Name of the monitoring queue.
- `wait_mode`?: _[MonitorFetchWaitMode](mod\_processing.md#monitorfetchwaitmode)_ – Wait mode.
<br>Default is `NO_WAIT`.


## ResultOfFetchNextMonitorResults
```ts
type ResultOfFetchNextMonitorResults = {
    results: MessageMonitoringResult[]
}
```
- `results`: _[MessageMonitoringResult](mod\_processing.md#messagemonitoringresult)[]_ – List of the resolved results.


## ParamsOfCancelMonitor
```ts
type ParamsOfCancelMonitor = {
    queue: string
}
```
- `queue`: _string_ – Name of the monitoring queue.


## ParamsOfSendMessages
```ts
type ParamsOfSendMessages = {
    messages: MessageSendingParams[],
    monitor_queue?: string
}
```
- `messages`: _[MessageSendingParams](mod\_processing.md#messagesendingparams)[]_ – Messages that must be sent to the blockchain.
- `monitor_queue`?: _string_ – Optional message monitor queue that starts monitoring for the processing results for sent messages.


## ResultOfSendMessages
```ts
type ResultOfSendMessages = {
    messages: MessageMonitoringParams[]
}
```
- `messages`: _[MessageMonitoringParams](mod\_processing.md#messagemonitoringparams)[]_ – Messages that was sent to the blockchain for execution.


## ParamsOfSendMessage
```ts
type ParamsOfSendMessage = {
    message: string,
    abi?: Abi,
    send_events: boolean
}
```
- `message`: _string_ – Message BOC.
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Optional message ABI.
<br>If this parameter is specified and the message has the<br>`expire` header then expiration time will be checked against<br>the current time to prevent unnecessary sending of already expired message.<br><br>The `message already expired` error will be returned in this<br>case.<br><br>Note, that specifying `abi` for ABI compliant contracts is<br>strongly recommended, so that proper processing strategy can be<br>chosen.
- `send_events`: _boolean_ – Flag for requesting events sending


## ResultOfSendMessage
```ts
type ResultOfSendMessage = {
    shard_block_id: string,
    sending_endpoints: string[]
}
```
- `shard_block_id`: _string_ – The last generated shard block of the message destination account before the message was sent.
<br>This block id must be used as a parameter of the<br>`wait_for_transaction`.
- `sending_endpoints`: _string[]_ – The list of endpoints to which the message was sent.
<br>This list id must be used as a parameter of the<br>`wait_for_transaction`.


## ParamsOfWaitForTransaction
```ts
type ParamsOfWaitForTransaction = {
    abi?: Abi,
    message: string,
    shard_block_id: string,
    send_events: boolean,
    sending_endpoints?: string[]
}
```
- `abi`?: _[Abi](mod\_abi.md#abi)_ – Optional ABI for decoding the transaction result.
<br>If it is specified, then the output messages' bodies will be<br>decoded according to this ABI.<br><br>The `abi_decoded` result field will be filled out.
- `message`: _string_ – Message BOC.
<br>Encoded with `base64`.
- `shard_block_id`: _string_ – The last generated block id of the destination account shard before the message was sent.
<br>You must provide the same value as the `send_message` has returned.
- `send_events`: _boolean_ – Flag that enables/disables intermediate events
- `sending_endpoints`?: _string[]_ – The list of endpoints to which the message was sent.
<br>Use this field to get more informative errors.<br>Provide the same value as the `send_message` has returned.<br>If the message was not delivered (expired), SDK will log the endpoint URLs, used for its sending.


## ParamsOfProcessMessage
```ts
type ParamsOfProcessMessage = {
    message_encode_params: ParamsOfEncodeMessage,
    send_events: boolean
}
```
- `message_encode_params`: _[ParamsOfEncodeMessage](mod\_abi.md#paramsofencodemessage)_ – Message encode parameters.
- `send_events`: _boolean_ – Flag for requesting events sending



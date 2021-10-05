# Field Descriptions

Below you can find the types descriptions

* [Account type](5_field_descriptions.md#account-type)
* [Transaction Type](5_field_descriptions.md#transaction-type)
* [Message type](5_field_descriptions.md#message-type)
* [MsgEnvelope type](5_field_descriptions.md#msgenvelope-type)
* [InMsg type](5_field_descriptions.md#inmsg-type)
* [Block type](5_field_descriptions.md#block-type)
* [BlockMaster Type](5_field_descriptions.md#blockmaster-type)
* [BlockMasterShardHashesDescr type](5_field_descriptions.md#blockmastershardhashesdescr-type)
* [BlockMasterConfig Type](5_field_descriptions.md#blockmasterconfig-type)

## Account type

Recall that a smart contract and an account are the same thing in the context of the TON Blockchain, and that these terms can be used interchangeably, at least as long as only small \(or “usual”\) smart contracts are considered. A large smart-contract may employ several accounts lying in different shardchains of the same workchain for load balancing purposes.

An account is identified by its full address and is completely described by its state. In other words, there is nothing else in an account apart from its address and state.

Can be queried by following fields:

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| id | string | Account address in raw format |
| acc\_type | uint8 | The current status of the account according to original TON blockchain specification: uninitialized - 0, active - 1,frozen - 2 |
| last\_paid | uint256 | Contains either the unixtime of the most recent storage payment collected \(usually this is the unixtime of the most recent transaction\), or the unixtime when the account was created \(again, by a transaction\) |
| due\_payment | [hex string](4_query_language.md#graphql-interaction) \(uint256\) | If present, accumulates the storage payments that could not be exacted from the balance of the account, represented by a strictly positive amount of nanotokens; it can be present only for uninitialized or frozen accounts that have a balance of zero tokens \(but may have non-zero balances in other cryptocurrencies\). When due\_payment becomes larger than the value of a configurable parameter of the blockchain, the ac- count is destroyed altogether, and its balance, if any, is transferred to the zero account. |
| last\_trans\_lt | uint64 | the last account's  transaction logic time |
| balance | uint128 | Account balance in nanotokens |
| balance\_other | { currency: uint32, value: hex string \(uint256\) } | Array of other currency balances |
| split\_depth | uint8 | Number of the split depth for large contracts. Is present and non-zero only in instances of large smart contracts. |
| tick | bool | May be present only in the masterchain—and within the masterchain, only in some fundamental smart contracts required for the whole system to function |
| code | base64 | If present, contains smart-contract code encoded with in base64 |
| data | base64 | If present, contains smart-contract data encoded with in base64 |
| data library: , | base64 | If present, contains library code used in smart-contract |
| proof | base64 | Merkle proof that account is a part of shard state it cut from. Merkle proof struct encoded with base64. |
| boc | base64 | Bag of cells with the account struct encoded with base64. |

## Transaction Type

The table below shows how our GraphQL scheme matches fields of TON transaction TLB schemes. In most cases, the specification is quoted to describe the fields. Meaning of fields is also sometime self-explanatory.

For more details, check the specification at [https://test.ton.org/tblkch.pdf](https://test.ton.org/tblkch.pdf).

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| id | string | transaction hash |
| tr\_type | int | Transaction type according to the original blockchain specification, clause 4.2.4. ordinary - 0, storage - 1, tick - 2, tock - 3, splitPrepare - 4, splitInstall - 5, mergePrepare - 6, mergeInstall - 7 |
| status | unknown - 0, preliminary - 1, proposed - 2, finalized - 3, refused - 4 | Transaction processing status |
| block\_id | string | block hash |
| account\_addr | uint256 | Address of an account for the transaction \(Tip: check the notion of an Account collection in the specification\) |
| lt |  | Transaction logical time. LT and the account address define the transaction on the blockchain |
| prev\_trans\_hash |  | hash of the previous transaction for the account |
| prev\_trans\_lt |  | logical time of a previous transaction for the account |
| now |  | block creation time |
| outmsg\_cnt |  | The number of generated outbound messages \(one of the common transaction parameters defined by the specification\) |
| orig\_status |  | The initial state of account. Note that in this case the query may return 0, if the account was not active before the transaction and 1 if it was already active |
| end\_status |  | The end state of an account after a transaction, 1 is returned to indicate a finalized transaction at an active account |
| in\_msg |  | Dictionary of transaction inbound message ID's as specified in the specification |
| in\_message |  | Dictionary of transaction inbound messages as specified in the specification |
| out\_msgs |  | Dictionary of transaction outbound message ID's as specified in the specification |
| out\_message |  | Dictionary of transaction outbound messages as specified in the specification |
| total\_fees |  | Total amount of fees that entails account state change and used in Merkle update |
| total\_fees\_other |  | Same as above, but reserved for other coins that may appear in the blockchain |
| old\_hash, new\_hash | uint256 | Hashes of the account state before and after the transaction |
| credit\_first |  |  |
| **STORAGE** \(phase\) |  | The storage phase is present in ordinary, merge, split, storage and tock transactions, so a common representation for this phase includes three fields. The first defines the amount , the second can be empty, the third specifies the account status change |
| **storage\_fees\_collected**,     **storage\_fees\_due**, **status\_change** |  | Fields show amounts related to storage fees and account status change \(e.g. it may be frozen or remain active \(unchanged\)\) |
| **CREDIT** \(phase\) |  | The account is credited with the value of the inbound message received. The credit phase can result in the collection of some due payments |
| **due\_fees\_collected** |  | The sum of `due_fees_collected` and credit must equal the value of the message received, plus its `ihr_fee` if the message has not been received via Instant Hypercube Routing, IHR \(otherwise the `ihr_fee` is awarded to the validators\). |
| credit |  |  |
| credit\_other |  |  |
| **COMPUTE** \(phase\) |  | The code of the smart contract is invoked inside an instance of TVM with adequate parameters, including a copy of the inbound message and of the persistent data, and terminates with an exit code, the new persistent data, and an action list \(which includes, for instance, outbound messages to be sent\). The processing phase may lead to the creation of a new account \(uninitialized or active\), or to the activation of a previously uninitialized or frozen account. The gas payment, equal to the product of the gas price and the gas consumed, is exacted from the account balance. If there is no reason to skip the computing phase, TVM is invoked and the results of the computation are logged. Possible parameters are covered below. |
| **compute\_type** |  | 0: skipped, then only `skipped_reason` is defined. 1: not skipped, then other fields for the phase are filled |
| **skipped\_reason** |  | Reason for skipping the compute phase. According to the specification, the phase can be skipped due to the absence of funds to buy gas, absence of state of an account or a message, failure to provide a valid state in the message |
| **success** |  | This flag is set if and only if `exit_code` is either 0 or 1. |
| **msg\_state\_used** |  | This parameter reflects whether the state passed in the message has been used. If it is set, the `account_activated` flag is used \(see below\) |
| **account\_activated** |  | The flag reflects whether this has resulted in the activation of a previously frozen, uninitialized or non-existent account. |
| **gas\_fees** |  | This parameter reflects the total gas fees collected by the validators for executing this transaction. It must be equal to the product of `gas_used` and `gas_price` from the current block header. |
| **gas\_used** |  | See above |
| **gas\_limit** |  | This parameter reflects the gas limit for this instance of TVM. It equals the lesser of either the tokens credited in the credit phase from the value of the inbound message divided by the current gas price, or the global per-transaction gas limit. |
| **gas\_credit** |  | This parameter may be non-zero only for external inbound messages. It is the lesser of either the amount of gas that can be paid from the account balance or the maximum gas credit |
| mode |  |  |
| **exit\_code**, **exit\_arg** |  | These parameters represent the status values returned by TVM; for a successful transaction, `exit_code` has to be 0 or 1 |
| **vm\_steps** |  | the total number of steps performed by TVM \(usually equal to two plus the number of instructions executed, including implicit RETs\) |
| **vm\_init\_state\_hash**, **vm\_final\_state\_hash** |  | These parameters are the representation hashes of the original and resulting states of TVM |
| **ACTION** \(phase\) |  | If the smart contract has terminated successfully \(with exit code 0 or 1\), the actions from the list are performed. If it is impossible to perform all of them—for example, because of insufficient funds to transfer with an outbound message—then the transaction is aborted and the account state is rolled back. The transaction is also aborted if the smart contract did not terminate successfully, or if it was not possible to invoke the smart contract at all because it is uninitialized or frozen. |
| success |  |  |
| valid |  |  |
| **no\_funds** |  | The flag indicates absence of funds required to create an outbound message |
| **status\_change** |  | Account status change according to the list of statuses provided by the specification |
| total\_fwd\_fees |  | Amount in tokens |
| total\_action\_fees |  | Amount in tokens |
| result\_code |  |  |
| result\_arg |  |  |
| tot\_actions |  |  |
| spec\_actions |  |  |
| skipped\_actions |  |  |
| msgs\_created |  |  |
| **action\_list\_hash** |  | Hash of the action list created during the compuation phase |
| total\_msg\_size\_cells |  |  |
| total\_msg\_size\_bits |  |  |
| **BOUNCE** \(phase\) |  | If the transaction has been aborted, and the inbound message has its bounce flag set, then it is “bounced” by automatically generating an outbound message \(with the bounce flag clear\) to its original sender. Almost all value of the original inbound message \(minus gas payments and forwarding fees\) is transferred to the generated message, which otherwise has an empty body. |
| bounce\_type |  | 0 - Negfunds, 1 - Nofunds, 2 - Ok |
| msg\_size\_cells |  |  |
| msg\_size\_bits |  |  |
| req\_fwd\_fees |  |  |
| msg\_fees |  |  |
| fwd\_fees |  | Amount to be bounced back |
| **aborted** |  | The flag is set either if there is no action phase or if the action phase was unsuccessful. The bounce phase occurs only if the `aborted` flag is set and the inbound message was bounceable. |
| destroyed |  |  |
| tt |  |  |
| split\_info |  | The fields below cover split prepare and install transactions and merge prepare and install transactions, the fields correspond to the relevant schemes covered by the blockchain specification. |
| cur\_shard\_pfx\_len |  | length of the current shard prefix |
| acc\_split\_depth |  |  |
| this\_addr |  |  |
| sibling\_addr |  |  |
| prepare\_transaction |  |  |
| installed |  |  |
| proof |  |  |
| boc |  |  |

## Message type

Message layout queries. A message consists of its header followed by its body or payload. The body is essentially arbitrary, to be interpreted by the destination smart contract. It can be queried with the following fields.

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| id | hex string | message hash |
| msg\_type | internal - 0, extIn - 1, extOut - 2 | Returns the type of message |
| status | unknown - 0, queued - 1, processing - 2, preliminary - 3, proposed - 4, finalized - 5, refused - 6, transiting - 7 | Returns internal processing status according to the numbers shown. |
| block\_id | String | Block identifier of the block where the message was last seen. |
| body | base64 | Bag of cells with the message encoded with  base64 |
| split\_depth | uint8 | split depth. This is only used for special contracts in masterchain for deploy messages |
| tick | bool | This field is present only in deploy messages of special contracts \(to masterchain\) |
| tock | bool | This field is present only in deploy messages of special contracts \(to masterchain\) |
| code | base64 | Bag of cells. Represents contract code in deploy messages. |
| data | base64 | Bag of cells. Represents initial data for a contract in deploy messages |
| library | base64 | Bag of cells. Represents contract library in deploy messages |
| src | String | Returns source address string |
| dst | String | Destination address |
| created\_lt | uint64 | Logical creation time automatically set by the generating transaction. |
| created\_at | uint32 | Creation unixtime automatically set by the generating transaction. The creation unixtime equals the creation unixtime of the block containing the generating transaction. |
| ihr\_disabled | bool | IHR is disabled for the message. |
| ihr\_fee | uint128 | This fee is subtracted from the value attached to the message and awarded to the validators of the destination shardchain if they include the message by the IHR mechanism. |
| fwd\_fee | uint128 | Original total forwarding fee paid for using the HR mechanism; it is automatically computed from some configuration parameters and the size of the message at the time the message is generated. |
| import\_fee | uint128 | Importing fee of the message |
| bounce | bool | If the transaction has been aborted, and the inbound message has its `bounce` flag set to `true`, then it is “bounced” by automatically generating an outbound message \(with the `bounce` flag clear and `bounced` flag set to `true`\) to its original sender. |
| bounced | bool | If the transaction has been aborted, and the inbound message has its `bounce` flag set to `true`, then it is “bounced” by automatically generating an outbound message \(with the `bounce` flag clear and `bounced` flag set to `true`\) to its original sender. |
| value |  | Internal message value in tokens. May or may not be present |
| value\_other |  | Value of the message in other currency as name and amount of other crypto currencyMay or may not be present. |
| proof | base64 | Merkle proof that message is a part of a block it is taken from. It is a bag of cells with Merkle proof struct encoded with base64. |
| boc | base64 | A bag of cells with the message structure encoded with base64. |

## MsgEnvelope type

Message envelopes are used for attaching routing information, such as the current \(transit\) address and the next-hop address, to inbound, transit, and outbound messages.

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| msg\_id | string | Message id. Link to message id \(message hash\) |
| next\_addr | string | A line with intermediate address. Message next-hop next-hop address |
| cur\_add | string | A line with intermediate address. Message current \(or transit\) address |
| fwd\_fee\_remaining |  | Remaining forwarding fee in tokens. Explicitly represents the maximum amount of message forwarding fees that can be deducted from the message value during the remaining HR steps; it cannot exceed the value of `fwd_fee` indicated in the message itself. |

## InMsg type

A type to specify the parameter of the inbound message. You can query the source of the message, the reason for it's being imported into this block, and some information about its “fate” — its processing by a transaction or forwarding inside the block.

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| msg\_type | external - 0, ihr - 1, immediately - 2, final - 3, transit - 4, discardedFinal - 5, discardedTransit - 6 | Message type |
| msg: |  | Message ID |
| transaction |  | Transaction ID |
| ihr\_fee | uint128 | This value is subtracted from the value attached to the message and awarded to the validators of the destination shardchain if they include the message by the IHR mechanism. |
| in\_msg |  | Contains message envelope. |
| fwd\_fee |  | Forwarding message fee in tokens |
| out\_msg |  | More research required |
| transit\_fee |  | transit fee in tokens |
| transaction\_id | uint64 | transaction ID |
| proof\_delivered |  |  |

## Block type

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| status | unknown - 0, proposed - 1, finalized - 2, refused - 3 | Returns block processing status |
| global\_id | uint32 | global block ID |
| want\_split | bool |  |
| seq\_no |  |  |
| after\_merge | bool |  |
| gen\_utime | uint 32 | Block creation time |
| gen\_catchain\_seqno |  |  |
| flags |  |  |
| master\_ref |  |  |
| prev\_ref |  | External block reference for previous block. |
| prev\_alt\_ref |  | External block reference for previous block in case of shard merge. |
| prev\_vert\_ref |  | External block reference for previous block in case of vertical blocks. |
| prev\_vert\_alt\_ref |  |  |
| version | uin32 | block version identifier |
| gen\_validator\_list\_hash\_short |  |  |
| before\_split | bool |  |
| after\_split | bool |  |
| want\_merge | bool |  |
| vert\_seq\_no |  |  |
| start\_lt: | uint64 | Logical creation time automatically set by the block formation start. Logical time is a component of the TON Blockchain that also plays an important role in message delivery is the logical time, usually denoted by Lt. It is a non-negative 64-bit integer, assigned to certain events. For more details, see the TON blockchain specification |
| end\_lt | uint64 | Logical creation time automatically set by the block formation end. |
| workchain\_id | uint32 | workchain identifier |
| shard |  |  |
| min\_ref\_mc\_seqno |  | Returns last known master block at the time of shard generation. |
| prev\_key\_block\_seqno |  | Last key block seq\_no |
| value\_flow |  |  |
| to\_next\_blk |  | Amount of tokens amount to the next block. |
| to\_next\_blk\_other |  | Amount of other cryptocurrencies to the next block. |
| exported |  | Amount of tokens exported. |
| exported\_other |  | Amount of other cryptocurrencies exported. |
| imported |  | Amount of tokens imported. |
| imported\_other |  | Amount of other cryptocurrencies imported. |
| from\_prev\_blk |  | Amount of tokens transferred from previous block. |
| from\_prev\_blk\_other |  | Amount of other cryptocurrencies transferred from previous block. |
| master | [BlockMaster](5_field_descriptions.md#blockmaster-type) | Contains information about shards, key block config params, etc. Present only in masterchain blocks |
| minted |  | Amount of tokens minted in this block. |
| minted\_other |  | Amount of other cryptocurrencies minted in this block. |
| fees\_imported |  | Amount of import fees in tokens |
| fees\_imported\_other |  | Amount of import fees in other currrencies. |
| in\_msg\_descr |  | Array of InMsg decribed messages. |
| rand\_seed |  | Need more research. |
| out\_msg\_descr |  |  |

## BlockMaster Type

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| min\_shard\_gen\_utime |  |  |
| max\_shard\_gen\_utime |  |  |
| shard\_hashes | BlockMasterShardHashes { workchain\_id: Int shard: String descr: [BlockMasterShardHashesDescr](5_field_descriptions.md#blockmastershardhashesdescr-type)} | List of shards present in the masterchain block |
| shard\_fees | BlockMasterShardFees |  |
| recover\_create\_msg | InMsg |  |
| prev\_blk\_signatures | BlockMasterPrevBlkSignatures |  |
| config\_addr | String |  |
| config | [BlockMasterConfig](5_field_descriptions.md#blockmasterconfig-type) | Blockchain config information. Present only in key blocks |

## BlockMasterShardHashesDescr type

ShardHashes is represented by a dictionary with 32-bit workchain\_ids as keys, and “shard binary trees”, represented by TL-B type BinTree ShardDescr, as values. Each leaf of this shard binary tree contains a value of type ShardDescr, which describes a single shard by indicating the sequence number seq\_no, the logical time lt, and the hash hash of the latest \(signed\) block of the corresponding shardchain.

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| seq\_no | uint32 | sequence number |
| reg\_mc\_seqno | string | Representation hash of shard block's root cell. Returns last known master block at the time of shard generation. The shard block configuration is derived from that block. |
| start\_lt |  | Logical time of the shardchain start. |
| end\_lt |  | Logical time of the shardchain end. |
| root\_hash | string | Representation hash of shard block's root cell. Returns last known master block at the time of shard generation. The shard block configuration is derived from that block. |
| file\_hash |  | Shard block file hash. |
| before\_split | bool | TON Blockchain supports dynamic sharding, so the shard configuration may change from block to block because of shard merge and split events. Therefore, we cannot simply say that each shardchain corresponds to a fixed set of accountchains. A shardchain block and its state may each be classified into two distinct parts. The parts with the ISP-dictated form of will be called the split parts of the block and its state, while the remainder will be called the non-split parts. The masterchain cannot be split or merged. |
| before\_merge | bool |  |
| want\_split | bool |  |
| want\_merge | bool |  |
| nx\_cc\_updated | bool |  |
| flags |  |  |
| next\_catchain\_seqno |  |  |
| next\_validator\_shard |  |  |
| min\_ref\_mc\_seqno |  |  |
| gen\_utime | uint32 | Time of the block creation |
| split\_type |  | none - 0, split - 2, merge - 3 |
| split |  |  |
| fees\_collected |  | Amount of fees collected int his shard in tokens. |
| fees\_collected\_other |  | Amount of fees collected int his shard in other currencies. |
| funds\_created |  | Amount of funds created in this shard in tokens. |
| funds\_created\_other |  | Amount of funds created in this shard in other currencies. |

## BlockMasterConfig Type

BlockMasterConfig type field is present only in key blocks. The previous key block seq\_no is always present in all masterchain blocks in prev\_key\_block\_seqno field.

This structure contains all TON blockchain configurations parameters.

| FIELD | TYPE | DESCRIPTION |
| :--- | :--- | :--- |
| p15 | `validators_elected_for`: Float, `elections_start_before`: Float, `elections_end_before`: Float, `stake_held_for`: Float | **validators\_elected\_for** - period of time in ms validators are elected for. **elections\_start\_before** - how much time in ms before utime\_until \(p34\) elections start. **elections\_end\_before** - how much time in ms before utime\_until  \(p34\) elections end. **stake\_held\_for** - for how much time in ms after utime\_until \(p34\) stake is frozen in elector contract |
| p16 | `max_validators`: int, `min_validators`: int | **max\_validators** - maximum possible number of active validators. **min\_validators** - minimum number of validators required for consensus |
| p17 | `min_stake`: hex string, `max_stake`: hex string, `min_total_stake`: hex string, `max_stake_factor`: float | **min\_stake** - minimum validator stake for elections. **max\_stake** - maximum validator stake for elections. **min\_total\_stake** - total stake required for elections to happen. **max\_stake\_factor** - maximum max\_factor value that can be passed to the elector contract by a validator \(_max-factor is the maximum ratio allowed between particular validator stake and the minimal validator stake in the elected validator group._\) |
| p32 | `utime_since`: Unix timestamp, `utime_since_string`: date string, `utime_until`: Unix timestamp, `utime_until_string`: date string, `total`: Int, `total_weight`\(...\): String, `list`: \[ValidatorSetList\] | **Previous validator set**, which validated from **`utime_since`** till **`utime_until`**. **`total`** - number of active validators. **`total_weight`** - sum of all active validators' weights. In a TON blockchain it is a constant and equals 2^60. **`list`**- list of active validators. **`publik_key`** - validator's public key \(temporary key, used to sign blocks, valid only for 1 cycle of validation\). **`weight`** - validator's weight. **`adnt_addr`** - validator's address \(temporary address, valid only for 1 cycle of validation\) |
| p34 | `utime_since`: Unix timestamp, `utime_since_string`: date string, `utime_until`: Unix timestamp, `utime_until_string`: date string, `total`: Int, `total_weight`\(...\): String, `list`: \[ValidatorSetList\] | **Current validator set**, obliged to validate from **`utime_since`** till **`utime_until`**. **`total`** - number of active validators. **`total_weight`** - sum of all active validators' weights. In a TON blockchain it is a constant and equals 2^60. **`list`** - list of active validators. **`publik_key`** - validator's public key \(temporary key, used to sign blocks, valid only for 1 cycle of validation\). **`weight`** - validator's weight. **`adnt_addr`** - validator's address \(temporary address, valid only for 1 cycle of validation\) |
| p36 | `utime_since`: Unix timestamp, `utime_since_string`: date string, `utime_until`: Unix timestamp, `utime_until_string`: date string, `total`: Int,  `total_weight`\(...\): String, `list`: \[ValidatorSetList\] | **Next validator set**, obliged to validate from **`utime_since`** till **`utime_until`**. **`total`** - number of active validators. **`total_weight`** - sum of all active validators' weights. In a TON blockchain it is a constant and equals 2^60. **`list`** - list of active validators. **`publik_key`** - validator's public key \(temporary key, used to sign blocks, valid only for 1 cycle of validation\). **`weight`** - validator's weight. **`adnt_addr`** - validator's address \(temporary address, valid only for 1 cycle of validation\) |


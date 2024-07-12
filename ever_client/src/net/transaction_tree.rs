/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

use serde_json::Value;

use crate::client::ClientContext;
use crate::error::ClientResult;
use crate::net::{ParamsOfQueryCollection, ServerLink, MESSAGES_COLLECTION};

use crate::abi::{decode_message_body, Abi, DecodedMessageBody, ParamsOfDecodeMessageBody};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::Arc;

const DEFAULT_WAITING_TIMEOUT: u32 = 60000;
const DEFAULT_TRANSACTION_MAX_COUNT: u32 = 50;

fn get_string(v: &Value, name: &str) -> Option<String> {
    v[name].as_str().map(|x| x.to_string())
}

fn required_string(v: &Value, name: &str) -> ClientResult<String> {
    v[name].as_str().map(|x| x.to_string()).ok_or_else(|| {
        crate::net::Error::invalid_server_response(format!("Missing required field {}", name))
    })
}

//-------------------------------------------------------------------------- query_transaction_tree

#[derive(Serialize, Deserialize, ApiType, Default, Clone)]
pub struct ParamsOfQueryTransactionTree {
    /// Input message id.
    pub in_msg: String,

    /// List of contract ABIs that will be used to decode message bodies.
    /// Library will try to decode each returned message body using any ABI from the registry.
    pub abi_registry: Option<Vec<Abi>>,

    /// Timeout used to limit waiting time for the missing messages and transaction.
    ///
    /// If some of the following messages and transactions are missing yet
    //  the function will wait for their appearance.
    /// The maximum waiting time is regulated by this option.
    ///
    /// Default value is 60000 (1 min). If `timeout` is set to 0 then function will wait infinitely
    /// until the whole transaction tree is executed
    pub timeout: Option<u32>,

    /// Maximum transaction count to wait. If transaction tree contains more transaction then this
    /// parameter then only first `transaction_max_count` transaction are awaited and returned.
    ///
    /// Default value is 50. If `transaction_max_count` is set to 0 then no limitation on
    /// transaction count is used and all transaction are returned.
    pub transaction_max_count: Option<u32>,
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone, Debug)]
pub struct MessageNode {
    /// Message id.
    pub id: String,

    /// Source transaction id.
    ///
    /// This field is missing for an external inbound messages.
    pub src_transaction_id: Option<String>,

    /// Destination transaction id.
    ///
    /// This field is missing for an external outbound messages.
    pub dst_transaction_id: Option<String>,

    /// Source address.
    pub src: Option<String>,

    /// Destination address.
    pub dst: Option<String>,

    /// Transferred tokens value.
    pub value: Option<String>,

    /// Bounce flag.
    pub bounce: bool,

    /// Decoded body.
    ///
    /// Library tries to decode message body using provided `params.abi_registry`.
    /// This field will be missing if none of the provided abi can be used to decode.
    pub decoded_body: Option<DecodedMessageBody>,
}

impl MessageNode {
    fn from(
        value: &Value,
        client: &Arc<ClientContext>,
        abi_registry: &Option<Vec<Abi>>,
        src_transactions: &HashMap<String, Option<String>>,
    ) -> ClientResult<Self> {
        let id = required_string(value, "id")?;
        Ok(Self {
            id: id.clone(),
            src_transaction_id: get_string(&value["src_transaction"], "id")
                .or_else(|| src_transactions.get(&id).unwrap_or(&None).clone()),
            dst_transaction_id: get_string(&value["dst_transaction"], "id"),
            src: get_string(value, "src"),
            dst: get_string(value, "dst"),
            value: get_string(value, "value"),
            bounce: value["bounce"].as_bool().unwrap_or(false),
            decoded_body: Self::try_decode_body(value, client, abi_registry),
        })
    }

    fn try_decode_body(
        message: &Value,
        client: &Arc<ClientContext>,
        abi_registry: &Option<Vec<Abi>>,
    ) -> Option<DecodedMessageBody> {
        if let Some(abi_registry) = abi_registry {
            if !abi_registry.is_empty() {
                if let Some(body) = message["body"].as_str() {
                    let is_internal = message["msg_type"].as_u64().unwrap_or(0) == 0;
                    for abi in abi_registry {
                        if let Ok(result) = decode_message_body(
                            client.clone(),
                            ParamsOfDecodeMessageBody {
                                body: body.to_string(),
                                abi: abi.clone(),
                                is_internal,
                                ..Default::default()
                            },
                        ) {
                            return Some(result);
                        }
                    }
                }
            }
        }
        None
    }
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone, Debug)]
pub struct TransactionNode {
    /// Transaction id.
    pub id: String,

    /// In message id.
    pub in_msg: String,

    /// Out message ids.
    pub out_msgs: Vec<String>,

    /// Account address.
    pub account_addr: String,

    /// Transactions total fees.
    pub total_fees: String,

    /// Aborted flag.
    pub aborted: bool,

    /// Compute phase exit code.
    pub exit_code: Option<u32>,
}

impl TransactionNode {
    fn from(value: &Value, message: &MessageNode) -> ClientResult<Self> {
        Ok(Self {
            id: message
                .dst_transaction_id
                .clone()
                .unwrap_or_else(|| String::default()),
            in_msg: message.id.clone(),
            aborted: value["aborted"].as_bool().unwrap_or(false),
            account_addr: message.dst.clone().unwrap_or_else(|| String::default()),
            exit_code: value["compute"]["exit_code"].as_u64().map(|x| x as u32),
            total_fees: value["total_fees"].as_str().unwrap_or("0x0").to_string(),
            out_msgs: if let Some(msgs) = value["out_msgs"].as_array() {
                msgs.iter()
                    .map(|x| x.as_str().unwrap_or("").to_string())
                    .collect()
            } else {
                Vec::default()
            },
        })
    }
}

#[derive(Serialize, Deserialize, ApiType, Default, Clone, Debug)]
pub struct ResultOfQueryTransactionTree {
    /// Messages.
    pub messages: Vec<MessageNode>,

    /// Transactions.
    pub transactions: Vec<TransactionNode>,
}

async fn query_next_portion(
    server_link: &ServerLink,
    timeout: u32,
    queue: &mut Vec<(Option<String>, String)>,
) -> ClientResult<(Vec<Value>, HashMap<String, Option<String>>)> {
    let mut src_transactions = HashMap::new();
    let mut has_none_src_transaction = false;
    while !queue.is_empty() && src_transactions.len() < 20 {
        let (tr, msg) = queue.remove(0);
        if tr.is_none() {
            has_none_src_transaction = true;
        }
        src_transactions.insert(msg, tr);
    }
    let mut result_fields = r#"
        id src dst msg_type value bounce body
        dst_transaction {
            id aborted compute { exit_code } total_fees out_msgs
        }"#
    .to_string();
    if has_none_src_transaction {
        result_fields.push_str(" src_transaction { id }");
    }
    let mut result_messages = Vec::new();
    let mut message_ids = src_transactions
        .keys()
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    // Wait for all required messages but not more than one minute
    let time_limit = server_link.client_env.now_ms() + timeout as u64;
    loop {
        let mut messages = server_link
            .query_collection(
                ParamsOfQueryCollection {
                    collection: MESSAGES_COLLECTION.to_string(),
                    result: result_fields.clone(),
                    filter: Some(json!({
                        "id": { "in":  Vec::from_iter(&message_ids) }
                    })),
                    limit: None,
                    order: None,
                },
                None,
            )
            .await?
            .as_array()
            .ok_or_else(|| crate::net::Error::invalid_server_response("Message array expected"))?
            .to_owned();
        while let Some(message) = messages.pop() {
            let id = message["id"].as_str().ok_or_else(|| {
                crate::net::Error::invalid_server_response("Message id is missing")
            })?;
            message_ids.remove(id);
            result_messages.push(message);
        }
        if message_ids.is_empty() {
            break;
        }
        if timeout != 0 && server_link.client_env.now_ms() > time_limit {
            return Err(crate::net::Error::query_transaction_tree_timeout(timeout));
        }
        server_link.client_env.set_timer(1000).await?;
    }
    Ok((result_messages, src_transactions))
}

/// Returns a tree of transactions triggered by a specific message.
///
/// Performs recursive retrieval of a transactions tree produced by a specific message:
/// in_msg -> dst_transaction -> out_messages -> dst_transaction -> ...
/// If the chain of transactions execution is in progress while the function is running,
/// it will wait for the next transactions to appear until the full tree or more than 50 transactions
/// are received.
///
/// All the retrieved messages and transactions are included
/// into `result.messages` and `result.transactions` respectively.
///
/// Function reads transactions layer by layer, by pages of 20 transactions.
///
/// The retrieval process goes like this:
/// Let's assume we have an infinite chain of transactions and each transaction generates 5 messages.
/// 1. Retrieve 1st message (input parameter) and corresponding transaction - put it into result.
/// It is the first level of the tree of transactions - its root.
/// Retrieve 5 out message ids from the transaction for next steps.
/// 2. Retrieve 5 messages and corresponding transactions on the 2nd layer. Put them into result.
/// Retrieve 5*5 out message ids from these transactions for next steps
/// 3. Retrieve 20 (size of the page) messages and transactions (3rd layer) and 20*5=100 message ids (4th layer).
/// 4. Retrieve the last 5 messages and 5 transactions on the 3rd layer + 15 messages and transactions (of 100) from the 4th layer
/// + 25 message ids of the 4th layer + 75 message ids of the 5th layer.
/// 5. Retrieve 20 more messages and 20 more transactions of the 4th layer + 100 more message ids of the 5th layer.
/// 6. Now we have 1+5+20+20+20 = 66 transactions, which is more than 50. Function exits with the tree of
/// 1m->1t->5m->5t->25m->25t->35m->35t. If we see any message ids in the last transactions out_msgs, which don't have
/// corresponding messages in the function result, it means that the full tree was not received and we need to continue iteration.
///
/// To summarize, it is guaranteed that each message in `result.messages` has the corresponding transaction
/// in the `result.transactions`.
/// But there is no guarantee that all messages from transactions `out_msgs` are
/// presented in `result.messages`.
/// So the application has to continue retrieval for missing messages if it requires.
#[api_function]
pub async fn query_transaction_tree(
    context: Arc<ClientContext>,
    params: ParamsOfQueryTransactionTree,
) -> ClientResult<ResultOfQueryTransactionTree> {
    let server_link = context.get_server_link()?;
    let mut transaction_nodes = Vec::new();
    let mut message_nodes = Vec::new();
    let mut query_queue: Vec<(Option<String>, String)> = vec![(None, params.in_msg.clone())];
    let timeout = params.timeout.unwrap_or(DEFAULT_WAITING_TIMEOUT);
    let transaction_max_count = params
        .transaction_max_count
        .unwrap_or(DEFAULT_TRANSACTION_MAX_COUNT) as usize;
    while !query_queue.is_empty()
        && (transaction_max_count == 0 || transaction_nodes.len() < transaction_max_count)
    {
        let (messages, src_transactions) =
            query_next_portion(server_link, timeout, &mut query_queue).await?;
        for message in messages {
            let message_node =
                MessageNode::from(&message, &context, &params.abi_registry, &src_transactions)?;
            let transaction = &message["dst_transaction"];
            if transaction.is_object() {
                let transaction_node = TransactionNode::from(&transaction, &message_node)?;
                for out_msg in &transaction_node.out_msgs {
                    query_queue.push((Some(transaction_node.id.clone()), out_msg.clone()));
                }
                transaction_nodes.push(transaction_node)
            };
            message_nodes.push(message_node);
        }
    }
    Ok(ResultOfQueryTransactionTree {
        transactions: transaction_nodes,
        messages: message_nodes,
    })
}

use crate::*;
use futures::stream::Stream;
use tvm::block::{
    Transaction as TvmTransaction, TransactionProcessingStatus, MessageId, 
    TransactionId, GenericId
};

#[derive(Debug)]
pub struct Transaction {
    tr: TvmTransaction,
}

// The struct represents performed transaction and allows to access their properties.
#[allow(dead_code)]
impl Transaction {

    // Asynchronously loads a Transaction instance or None if transaction with given id is not exists
    pub fn load(id: TransactionId) -> SdkResult<Box<dyn Stream<Item = Option<Transaction>, Error = SdkError>>> {
        let map = queries_helper::load_record_fields(
            TRANSACTIONS_TABLE_NAME,
            &id.to_hex_string(),
            TRANSACTION_FIELDS_ORDINARY)?
                .and_then(|val| {
                    if val == serde_json::Value::Null {
                        Ok(None)
                    } else {
                        let tr: TvmTransaction = serde_json::from_value(val)
                            .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing transaction: {}", err)))?;

                        Ok(Some(Transaction { tr }))
                    }
            });

        Ok(Box::new(map))
    }

    // Returns transaction's processing status
    pub fn status(&self) -> TransactionProcessingStatus {
        self.tr.processing_status()
    }

    // Returns blockchain's transaction struct
    // Some node-specifed methods won't work. All TonStructVariant fields has Client variant.
    pub fn tr(&self) -> &TvmTransaction {
         &self.tr
    }

    // Returns id of transaction's input message if it exists
    pub fn in_message_id(&self) -> Option<MessageId> {
        self.tr.in_message().map(|m| m.client_ref_unwrap().clone())
    }

    // Asynchronously loads an instance of transaction's input message
    pub fn load_in_message(&self) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        match self.in_message_id() {
            Some(m) => Message::load(m),
            None => bail!(SdkErrorKind::InvalidOperation("transaction doesn't have inbound message".into()))
        }
    }

    // Returns id of transaction's out messages if it exists
    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.tr.out_msgs_sdk()
    }

    // Returns message's identifier
    pub fn id(&self) -> TransactionId {
        // On client side id is ready allways. It is never be calculated, just returned.
        self.tr.calc_id().unwrap()
    }

    // Asynchronously loads an instances of transaction's out messages
    pub fn load_out_messages(&self) -> SdkResult<Box<dyn Stream<Item = Option<Message>, Error = SdkError>>> {
        let mut msg_id_iter = self.out_messages_id().iter();
        if let Some(id) = msg_id_iter.next().clone() {
            let mut stream = Message::load(id.clone())?;
            for id in msg_id_iter {
                stream = Box::new(stream.chain(Message::load(id.clone())?));
            }
            Ok(stream)
        } else {
            // TODO how to return empty Stream?
            bail!(SdkErrorKind::NoData);
        }
    }
}


#[allow(dead_code)]
const TRANSACTION_FIELDS_MESSAGES: &str = r#"
    id
    account_addr
    in_msg
    out_msgs
    outmsg_cnt
    status
"#;

const TRANSACTION_FIELDS_ORDINARY: &str = r#"
    id
    account_addr
    block_id
    description {
      ...on TransactionDescriptionOrdinaryVariant {
        Ordinary {
          aborted
          storage_ph {
            status_change
          }
          compute_ph {
            ...on TrComputePhaseSkippedVariant {
              Skipped {reason}
            }
            ...on TrComputePhaseVmVariant {
              Vm {
                success
                exit_code
              }
            }
          }
          action {
            success
            valid
            result_code
            no_funds
          }
        }
      }
  	}    
    end_status
    in_msg
    now
    orig_status
    out_msgs
    outmsg_cnt
    status
"#;

#[allow(dead_code)]
const TRANSACTION_FIELDS_FULL: &str = r#"
    id
    account_addr
    block_id
    description {
        ...on TransactionDescriptionOrdinaryVariant {
            Ordinary {
                aborted
                compute_ph {
                    ...on TrComputePhaseVmVariant {
                        Vm {
                            exit_code
                            gas_credit
                            gas_fees
                            gas_limit
                            gas_used
                            success
                        }
                    }
                    ...on TrComputePhaseSkippedVariant {
                        Skipped {
                            reason
                        }
                    }
                }
            }
        }
        ...on TransactionDescriptionStorageVariant {
            Storage {
                storage_fees_collected
                storage_fees_due
                status_change
            }
        }
        ...on TransactionDescriptionTickTockVariant {
            TickTock {
                tt
                aborted
                compute_ph {
                    ...on TrComputePhaseVmVariant {
                        Vm {
                            exit_code
                            gas_credit
                            gas_fees
                            gas_limit
                            gas_used
                            success
                        }
                    }
                    ...on TrComputePhaseSkippedVariant {
                        Skipped {
                            reason
                        }
                    }
                }
            }
        }
        ...on TransactionDescriptionSplitPrepareVariant {
            SplitPrepare {
                aborted
                split_info {
                    cur_shard_pfx_len
                    acc_split_depth
                    this_addr
                    sibling_addr
                }
                compute_ph {
                    ...on TrComputePhaseVmVariant {
                        Vm {
                            exit_code
                            gas_credit
                            gas_fees
                            gas_limit
                            gas_used
                            success
                        }
                    }
                    ...on TrComputePhaseSkippedVariant {
                        Skipped {
                            reason
                        }
                    }
                }
            }
        }
        ...on TransactionDescriptionSplitInstallVariant {
            SplitInstall {
                split_info {
                    cur_shard_pfx_len
                    acc_split_depth
                    this_addr
                    sibling_addr
                }
                prepare_transaction
                installed
            }
        }
        ...on TransactionDescriptionMergePrepareVariant {
            MergePrepare {
                split_info {
                    cur_shard_pfx_len
                    acc_split_depth
                    this_addr
                    sibling_addr
                }
                storage_ph {
                    storage_fees_collected
                    storage_fees_due
                    status_change
                }
                aborted
            }
        }
        ...on TransactionDescriptionMergeInstallVariant {
            MergeInstall {
                split_info {
                    cur_shard_pfx_len
                    acc_split_depth
                    this_addr
                    sibling_addr
                }
                compute_ph {
                    ...on TrComputePhaseVmVariant {
                        Vm {
                            exit_code
                            gas_credit
                            gas_fees
                            gas_limit
                            gas_used
                            success
                        }
                    }
                    ...on TrComputePhaseSkippedVariant {
                        Skipped {
                            reason
                        }
                    }
                }
                prepare_transaction
                aborted
            }
        }

    }
    end_status
    in_msg
    now
    orig_status
    out_msgs
    outmsg_cnt
    status
    total_fees
"#;

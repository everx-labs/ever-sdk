/// [UNSTABLE](UNSTABLE.md) Describes how much funds will be debited from the target
///  contract balance as a result of the transaction.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType, PartialEq)]
pub struct Spending {
    /// Amount of nanotokens that will be sent to `dst` address.
    pub amount: u64,
    /// Destination address of recipient of funds.
    pub dst: String,
}

/// [UNSTABLE](UNSTABLE.md) Describes the operation that the DeBot wants to perform.
#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag = "type")]
pub enum DebotActivity {
    /// DeBot wants to create new transaction in blockchain.
    Transaction {
        /// External inbound message BOC.
        msg: String,
        /// Target smart contract address.
        dst: String,
        ///List of spendings as a result of transaction.
        out: Vec<Spending>,
        /// Transaction total fee.
        fee: u64,
        /// Indicates if target smart contract updates its code.
        setcode: bool,
        /// Public key from keypair that was used to sign external message.
        signkey: String,
        /// Signing box handle used to sign external message.
        signing_box_handle: u32,
    },
}

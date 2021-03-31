
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Spending {
    pub amount: u64,
    pub dst: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ApiType)]
#[serde(tag="type")]
pub enum DebotActivity {
    Transaction {
        msg: String,
        dst: String,
        out: Vec<Spending>,
        fee: u64,
        setcode: bool,
        signkey: String,
    }
}
use super::ProcessingEvent;


#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RempStatusData {
    pub message_id: String,
    pub timestamp: u64,
    #[serde(deserialize_with = "deserialize_json_from_string")]
    pub json: serde_json::Value,
}

pub fn deserialize_json_from_string<'de, D>(d: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = d.deserialize_option(ever_sdk::json_helper::StringVisitor)?;

    if "null" == string {
        Ok(serde_json::Value::Null)
    } else {
        Ok(serde_json::from_str(&string).map_err(|err| serde::de::Error::custom(err))?)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RempStatus {
    RejectedByFullnode(RempStatusData),
    SentToValidators(RempStatusData),
    IncludedIntoBlock(RempStatusData),
    IncludedIntoAcceptedBlock(RempStatusData),
    Finalized(RempStatusData),
    Other(RempStatusData),
}

impl RempStatus {
    pub fn into_event(self, message_dst: String) -> ProcessingEvent {
        match self {
            RempStatus::SentToValidators(data) => {
                ProcessingEvent::RempSentToValidators { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
            RempStatus::IncludedIntoBlock(data) => {
                ProcessingEvent::RempIncludedIntoBlock { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
            RempStatus::IncludedIntoAcceptedBlock(data) => {
                ProcessingEvent::RempIncludedIntoAcceptedBlock { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
            RempStatus::Other(data) => {
                ProcessingEvent::RempOther { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
            RempStatus::RejectedByFullnode(data) => {
                ProcessingEvent::RempOther { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
            RempStatus::Finalized(data) => {
                ProcessingEvent::RempOther { message_id: data.message_id, message_dst, timestamp: data.timestamp, json: data.json }
            },
        }
    }
}
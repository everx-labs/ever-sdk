use tvm::block::MessageProcessingStatus;


pub fn parse_message_status(status: &str) -> MessageProcessingStatus {
    let status_str = status.to_string();
    match status_str.as_ref() {
        "Unknown" => MessageProcessingStatus::Unknown,
        "Queued" => MessageProcessingStatus::Queued,
        "Processing" => MessageProcessingStatus::Processing,
        "Preliminary" => MessageProcessingStatus::Preliminary,
        "Proposed" => MessageProcessingStatus::Proposed,
        "Finalized" => MessageProcessingStatus::Finalized,
        "Refused" => MessageProcessingStatus::Refused,
        "Transiting" => MessageProcessingStatus::Transiting,
        _ => MessageProcessingStatus::Unknown,
    }
}
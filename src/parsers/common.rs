use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMessage {
    pub sender_id: String,
    pub sender_name: String,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text {
        body: String,
        formatted: Option<String>,
    },
    Image {
        url: String,
        caption: Option<String>,
    },
    Video {
        url: String,
        caption: Option<String>,
    },
    Audio {
        url: String,
        caption: Option<String>,
    },
    File {
        url: String,
        filename: String,
    },
    Sticker {
        url: String,
        emoji: Option<String>,
    },
}

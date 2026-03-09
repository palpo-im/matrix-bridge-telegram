use serde::{Deserialize, Serialize};

/// Intermediate message format used for bidirectional conversion between Matrix and Telegram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMessage {
    pub sender_id: String,
    pub sender_name: String,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub timestamp: u64,
    pub edit_of: Option<String>,
    pub forward_from: Option<ForwardInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardInfo {
    pub sender_name: String,
    pub sender_id: Option<String>,
    pub date: Option<u64>,
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
        mime_type: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        size: Option<u64>,
    },
    Video {
        url: String,
        caption: Option<String>,
        mime_type: Option<String>,
        duration: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        size: Option<u64>,
    },
    Audio {
        url: String,
        caption: Option<String>,
        mime_type: Option<String>,
        duration: Option<u32>,
        size: Option<u64>,
    },
    File {
        url: String,
        filename: String,
        mime_type: Option<String>,
        size: Option<u64>,
    },
    Sticker {
        url: String,
        emoji: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Location {
        latitude: f64,
        longitude: f64,
        description: Option<String>,
    },
    Contact {
        phone: String,
        first_name: String,
        last_name: Option<String>,
        vcard: Option<String>,
    },
    Notice {
        body: String,
    },
}

impl MessageContent {
    pub fn text(body: impl Into<String>) -> Self {
        MessageContent::Text {
            body: body.into(),
            formatted: None,
        }
    }

    pub fn formatted_text(body: impl Into<String>, html: impl Into<String>) -> Self {
        MessageContent::Text {
            body: body.into(),
            formatted: Some(html.into()),
        }
    }

    pub fn notice(body: impl Into<String>) -> Self {
        MessageContent::Notice { body: body.into() }
    }

    pub fn plain_body(&self) -> String {
        match self {
            MessageContent::Text { body, .. } => body.clone(),
            MessageContent::Image { caption, .. } => caption.clone().unwrap_or_default(),
            MessageContent::Video { caption, .. } => caption.clone().unwrap_or_default(),
            MessageContent::Audio { caption, .. } => caption.clone().unwrap_or_default(),
            MessageContent::File { filename, .. } => filename.clone(),
            MessageContent::Sticker { emoji, .. } => emoji.clone().unwrap_or_default(),
            MessageContent::Location { description, .. } => {
                description.clone().unwrap_or_else(|| "Location".to_string())
            }
            MessageContent::Contact {
                first_name,
                last_name,
                ..
            } => {
                if let Some(last) = last_name {
                    format!("{} {}", first_name, last)
                } else {
                    first_name.clone()
                }
            }
            MessageContent::Notice { body } => body.clone(),
        }
    }
}

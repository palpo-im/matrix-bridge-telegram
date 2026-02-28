use crate::parsers::common::{CommonMessage, MessageContent};

pub struct MatrixParser;

impl MatrixParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_matrix_event(_event: &serde_json::Value) -> Option<CommonMessage> {
        None
    }

    pub fn matrix_to_telegram(content: &MessageContent) -> String {
        match content {
            MessageContent::Text { body, .. } => body.clone(),
            MessageContent::Image { url, caption } => {
                format!("[Image: {}] {}", url, caption.as_deref().unwrap_or(""))
            }
            MessageContent::Video { url, caption } => {
                format!("[Video: {}] {}", url, caption.as_deref().unwrap_or(""))
            }
            MessageContent::Audio { url, caption } => {
                format!("[Audio: {}] {}", url, caption.as_deref().unwrap_or(""))
            }
            MessageContent::File { url, filename } => {
                format!("[File: {}] {}", filename, url)
            }
            MessageContent::Sticker { url, .. } => {
                format!("[Sticker: {}]", url)
            }
        }
    }
}

impl Default for MatrixParser {
    fn default() -> Self {
        Self::new()
    }
}

use crate::parsers::common::{CommonMessage, MessageContent};

pub struct TelegramParser;

impl TelegramParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_telegram_message(_message: &serde_json::Value) -> Option<CommonMessage> {
        None
    }

    pub fn telegram_to_matrix(content: &MessageContent) -> String {
        match content {
            MessageContent::Text { body, formatted } => {
                if let Some(html) = formatted {
                    format!("<p>{}</p>", html)
                } else {
                    format!("<p>{}</p>", body)
                }
            }
            MessageContent::Image { url, caption } => {
                format!(
                    "<img src=\"{}\" alt=\"{}\" />",
                    url,
                    caption.as_deref().unwrap_or("")
                )
            }
            MessageContent::Video { url, caption } => {
                format!(
                    "<video src=\"{}\">{}</video>",
                    url,
                    caption.as_deref().unwrap_or("")
                )
            }
            MessageContent::Audio { url, caption } => {
                format!(
                    "<audio src=\"{}\">{}</audio>",
                    url,
                    caption.as_deref().unwrap_or("")
                )
            }
            MessageContent::File { url, filename } => {
                format!("<a href=\"{}\">{}</a>", url, filename)
            }
            MessageContent::Sticker { url, emoji } => {
                format!(
                    "<img src=\"{}\" alt=\"{}\" />",
                    url,
                    emoji.as_deref().unwrap_or("")
                )
            }
        }
    }
}

impl Default for TelegramParser {
    fn default() -> Self {
        Self::new()
    }
}

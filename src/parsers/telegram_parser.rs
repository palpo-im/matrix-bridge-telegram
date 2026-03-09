use crate::parsers::common::{CommonMessage, ForwardInfo, MessageContent};
use crate::utils::formatting::escape_html;

/// Converts Telegram message formats to Matrix-compatible formats.
pub struct TelegramParser;

impl TelegramParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a Telegram message JSON (teloxide format) into a CommonMessage.
    pub fn parse_telegram_message(message: &serde_json::Value) -> Option<CommonMessage> {
        let sender_id = message
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .map(|id| id.to_string())?;

        let sender_name = Self::extract_sender_name(message.get("from")?);

        let timestamp = message.get("date").and_then(|d| d.as_u64()).unwrap_or(0);

        let reply_to = message
            .get("reply_to_message")
            .and_then(|r| r.get("message_id"))
            .and_then(|id| id.as_i64())
            .map(|id| id.to_string());

        let forward_from = Self::extract_forward_info(message);

        let content = Self::extract_content(message)?;

        Some(CommonMessage {
            sender_id,
            sender_name,
            content,
            reply_to,
            timestamp,
            edit_of: None,
            forward_from,
        })
    }

    fn extract_sender_name(from: &serde_json::Value) -> String {
        let first = from
            .get("first_name")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        let last = from.get("last_name").and_then(|n| n.as_str()).unwrap_or("");
        if last.is_empty() {
            first.to_string()
        } else {
            format!("{} {}", first, last)
        }
    }

    fn extract_forward_info(message: &serde_json::Value) -> Option<ForwardInfo> {
        if let Some(fwd_from) = message.get("forward_from") {
            let name = Self::extract_sender_name(fwd_from);
            let id = fwd_from
                .get("id")
                .and_then(|i| i.as_i64())
                .map(|i| i.to_string());
            Some(ForwardInfo {
                sender_name: name,
                sender_id: id,
                date: message.get("forward_date").and_then(|d| d.as_u64()),
            })
        } else if let Some(name) = message
            .get("forward_sender_name")
            .and_then(|n| n.as_str())
        {
            Some(ForwardInfo {
                sender_name: name.to_string(),
                sender_id: None,
                date: message.get("forward_date").and_then(|d| d.as_u64()),
            })
        } else {
            None
        }
    }

    fn extract_content(message: &serde_json::Value) -> Option<MessageContent> {
        // Photo
        if let Some(photos) = message.get("photo").and_then(|p| p.as_array()) {
            if let Some(largest) = photos.last() {
                let file_id = largest
                    .get("file_id")
                    .and_then(|f| f.as_str())
                    .unwrap_or("")
                    .to_string();
                let caption = message
                    .get("caption")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());
                let width = largest.get("width").and_then(|w| w.as_u64()).map(|w| w as u32);
                let height = largest
                    .get("height")
                    .and_then(|h| h.as_u64())
                    .map(|h| h as u32);
                return Some(MessageContent::Image {
                    url: file_id,
                    caption,
                    mime_type: Some("image/jpeg".to_string()),
                    width,
                    height,
                    size: largest.get("file_size").and_then(|s| s.as_u64()),
                });
            }
        }

        // Document
        if let Some(doc) = message.get("document") {
            let file_id = doc
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let filename = doc
                .get("file_name")
                .and_then(|f| f.as_str())
                .unwrap_or("file")
                .to_string();
            let mime = doc
                .get("mime_type")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string());
            let size = doc.get("file_size").and_then(|s| s.as_u64());
            let caption = message
                .get("caption")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());

            // Check if it's an animation (GIF)
            if message.get("animation").is_some() {
                return Some(MessageContent::Video {
                    url: file_id,
                    caption,
                    mime_type: mime,
                    duration: None,
                    width: doc.get("width").and_then(|w| w.as_u64()).map(|w| w as u32),
                    height: doc
                        .get("height")
                        .and_then(|h| h.as_u64())
                        .map(|h| h as u32),
                    size,
                });
            }

            return Some(MessageContent::File {
                url: file_id,
                filename,
                mime_type: mime,
                size,
            });
        }

        // Video
        if let Some(video) = message.get("video") {
            let file_id = video
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let caption = message
                .get("caption")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
            return Some(MessageContent::Video {
                url: file_id,
                caption,
                mime_type: video
                    .get("mime_type")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string()),
                duration: video
                    .get("duration")
                    .and_then(|d| d.as_u64())
                    .map(|d| d as u32),
                width: video
                    .get("width")
                    .and_then(|w| w.as_u64())
                    .map(|w| w as u32),
                height: video
                    .get("height")
                    .and_then(|h| h.as_u64())
                    .map(|h| h as u32),
                size: video.get("file_size").and_then(|s| s.as_u64()),
            });
        }

        // Audio / Voice
        if let Some(audio) = message
            .get("audio")
            .or_else(|| message.get("voice"))
        {
            let file_id = audio
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let caption = message
                .get("caption")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
            return Some(MessageContent::Audio {
                url: file_id,
                caption,
                mime_type: audio
                    .get("mime_type")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string()),
                duration: audio
                    .get("duration")
                    .and_then(|d| d.as_u64())
                    .map(|d| d as u32),
                size: audio.get("file_size").and_then(|s| s.as_u64()),
            });
        }

        // Sticker
        if let Some(sticker) = message.get("sticker") {
            let file_id = sticker
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let emoji = sticker
                .get("emoji")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string());
            return Some(MessageContent::Sticker {
                url: file_id,
                emoji,
                width: sticker
                    .get("width")
                    .and_then(|w| w.as_u64())
                    .map(|w| w as u32),
                height: sticker
                    .get("height")
                    .and_then(|h| h.as_u64())
                    .map(|h| h as u32),
            });
        }

        // Location
        if let Some(location) = message.get("location") {
            let lat = location.get("latitude").and_then(|l| l.as_f64())?;
            let lon = location.get("longitude").and_then(|l| l.as_f64())?;
            return Some(MessageContent::Location {
                latitude: lat,
                longitude: lon,
                description: message
                    .get("venue")
                    .and_then(|v| v.get("title"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string()),
            });
        }

        // Contact
        if let Some(contact) = message.get("contact") {
            let phone = contact
                .get("phone_number")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string();
            let first = contact
                .get("first_name")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let last = contact
                .get("last_name")
                .and_then(|l| l.as_str())
                .map(|s| s.to_string());
            let vcard = contact
                .get("vcard")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return Some(MessageContent::Contact {
                phone,
                first_name: first,
                last_name: last,
                vcard,
            });
        }

        // Text message (check last as other types may also have text)
        if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
            let entities = message
                .get("entities")
                .and_then(|e| e.as_array())
                .cloned()
                .unwrap_or_default();
            let formatted = Self::format_entities(text, &entities);
            return Some(MessageContent::Text {
                body: text.to_string(),
                formatted: if formatted != escape_html(text) {
                    Some(formatted)
                } else {
                    None
                },
            });
        }

        None
    }

    /// Convert Telegram entities to HTML for Matrix.
    fn format_entities(text: &str, entities: &[serde_json::Value]) -> String {
        if entities.is_empty() {
            return escape_html(text);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut result = String::new();
        let mut pos = 0usize;

        // Sort entities by offset
        let mut sorted_entities = entities.to_vec();
        sorted_entities.sort_by_key(|e| e.get("offset").and_then(|o| o.as_u64()).unwrap_or(0));

        for entity in &sorted_entities {
            let offset = entity
                .get("offset")
                .and_then(|o| o.as_u64())
                .unwrap_or(0) as usize;
            let length = entity
                .get("length")
                .and_then(|l| l.as_u64())
                .unwrap_or(0) as usize;
            let entity_type = entity
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            // Add text before this entity
            if offset > pos {
                let before: String = chars[pos..offset.min(chars.len())].iter().collect();
                result.push_str(&escape_html(&before));
            }

            let entity_text: String = chars
                [offset.min(chars.len())..(offset + length).min(chars.len())]
                .iter()
                .collect();
            let escaped = escape_html(&entity_text);

            match entity_type {
                "bold" => result.push_str(&format!("<strong>{}</strong>", escaped)),
                "italic" => result.push_str(&format!("<em>{}</em>", escaped)),
                "underline" => result.push_str(&format!("<u>{}</u>", escaped)),
                "strikethrough" => result.push_str(&format!("<del>{}</del>", escaped)),
                "code" => result.push_str(&format!("<code>{}</code>", escaped)),
                "pre" => {
                    let lang = entity
                        .get("language")
                        .and_then(|l| l.as_str())
                        .unwrap_or("");
                    if lang.is_empty() {
                        result.push_str(&format!("<pre><code>{}</code></pre>", escaped));
                    } else {
                        result.push_str(&format!(
                            "<pre><code class=\"language-{}\">{}</code></pre>",
                            escape_html(lang),
                            escaped
                        ));
                    }
                }
                "text_link" => {
                    let url = entity
                        .get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("");
                    result.push_str(&format!("<a href=\"{}\">{}</a>", escape_html(url), escaped));
                }
                "text_mention" => {
                    let user_id = entity
                        .get("user")
                        .and_then(|u| u.get("id"))
                        .and_then(|id| id.as_i64())
                        .unwrap_or(0);
                    result.push_str(&format!(
                        "<a href=\"https://matrix.to/#/@telegram_{}:localhost\">{}</a>",
                        user_id, escaped
                    ));
                }
                "mention" => {
                    // @username mention
                    result.push_str(&format!(
                        "<a href=\"https://t.me/{}\">{}</a>",
                        escape_html(entity_text.trim_start_matches('@')),
                        escaped
                    ));
                }
                "url" => {
                    result.push_str(&format!("<a href=\"{}\">{}</a>", escaped, escaped));
                }
                "email" => {
                    result.push_str(&format!(
                        "<a href=\"mailto:{}\">{}</a>",
                        escaped, escaped
                    ));
                }
                "phone_number" => {
                    result.push_str(&format!("<a href=\"tel:{}\">{}</a>", escaped, escaped));
                }
                "spoiler" => {
                    result.push_str(&format!(
                        "<span data-mx-spoiler>{}</span>",
                        escaped
                    ));
                }
                "blockquote" => {
                    result.push_str(&format!("<blockquote>{}</blockquote>", escaped));
                }
                "bot_command" | "hashtag" | "cashtag" => {
                    result.push_str(&escaped);
                }
                _ => {
                    result.push_str(&escaped);
                }
            }

            pos = offset + length;
        }

        // Add remaining text
        if pos < chars.len() {
            let remaining: String = chars[pos..].iter().collect();
            result.push_str(&escape_html(&remaining));
        }

        result
    }

    /// Convert a CommonMessage content to Matrix HTML.
    pub fn telegram_to_matrix(content: &MessageContent) -> String {
        match content {
            MessageContent::Text { body, formatted } => {
                if let Some(html) = formatted {
                    html.clone()
                } else {
                    escape_html(body)
                }
            }
            MessageContent::Image { url, caption, .. } => {
                let mut html = format!("<img src=\"{}\" />", escape_html(url));
                if let Some(cap) = caption {
                    if !cap.is_empty() {
                        html.push_str(&format!("<br/>{}", escape_html(cap)));
                    }
                }
                html
            }
            MessageContent::Video { url, caption, .. } => {
                let mut html = format!("<video src=\"{}\" />", escape_html(url));
                if let Some(cap) = caption {
                    if !cap.is_empty() {
                        html.push_str(&format!("<br/>{}", escape_html(cap)));
                    }
                }
                html
            }
            MessageContent::Audio { url, caption, .. } => {
                let mut html = format!("<audio src=\"{}\" />", escape_html(url));
                if let Some(cap) = caption {
                    if !cap.is_empty() {
                        html.push_str(&format!("<br/>{}", escape_html(cap)));
                    }
                }
                html
            }
            MessageContent::File { url, filename, .. } => {
                format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(url),
                    escape_html(filename)
                )
            }
            MessageContent::Sticker { url, emoji, .. } => {
                format!(
                    "<img src=\"{}\" alt=\"{}\" height=\"256\" />",
                    escape_html(url),
                    escape_html(emoji.as_deref().unwrap_or(""))
                )
            }
            MessageContent::Location {
                latitude,
                longitude,
                description,
            } => {
                let desc = description.as_deref().unwrap_or("Location");
                format!(
                    "<a href=\"https://www.openstreetmap.org/?mlat={}&mlon={}\">{}</a>",
                    latitude, longitude, escape_html(desc)
                )
            }
            MessageContent::Contact {
                phone,
                first_name,
                last_name,
                ..
            } => {
                let name = if let Some(last) = last_name {
                    format!("{} {}", first_name, last)
                } else {
                    first_name.clone()
                };
                format!(
                    "<b>{}</b>: <a href=\"tel:{}\">{}</a>",
                    escape_html(&name),
                    escape_html(phone),
                    escape_html(phone)
                )
            }
            MessageContent::Notice { body } => escape_html(body),
        }
    }

    /// Build a forward header HTML snippet.
    pub fn format_forward_header(forward: &ForwardInfo) -> String {
        format!(
            "<blockquote>Forwarded from <b>{}</b></blockquote>",
            escape_html(&forward.sender_name)
        )
    }
}

impl Default for TelegramParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_entities_bold() {
        let entities = vec![serde_json::json!({
            "type": "bold",
            "offset": 0,
            "length": 5
        })];
        let result = TelegramParser::format_entities("hello world", &entities);
        assert_eq!(result, "<strong>hello</strong> world");
    }

    #[test]
    fn test_format_entities_mixed() {
        let entities = vec![
            serde_json::json!({"type": "bold", "offset": 0, "length": 5}),
            serde_json::json!({"type": "italic", "offset": 6, "length": 5}),
        ];
        let result = TelegramParser::format_entities("hello world", &entities);
        assert_eq!(result, "<strong>hello</strong> <em>world</em>");
    }

    #[test]
    fn test_format_entities_link() {
        let entities = vec![serde_json::json!({
            "type": "text_link",
            "offset": 0,
            "length": 4,
            "url": "https://example.com"
        })];
        let result = TelegramParser::format_entities("link here", &entities);
        assert_eq!(
            result,
            "<a href=\"https://example.com\">link</a> here"
        );
    }

    #[test]
    fn test_text_to_matrix() {
        let content = MessageContent::text("hello");
        assert_eq!(TelegramParser::telegram_to_matrix(&content), "hello");
    }
}

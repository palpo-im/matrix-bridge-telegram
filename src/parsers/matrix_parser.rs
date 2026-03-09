use crate::parsers::common::{CommonMessage, MessageContent};
use crate::utils::formatting::strip_html_tags;

/// Converts Matrix message formats to Telegram-compatible formats.
pub struct MatrixParser;

impl MatrixParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a Matrix room event JSON into a CommonMessage.
    pub fn parse_matrix_event(event: &serde_json::Value) -> Option<CommonMessage> {
        let sender = event.get("sender").and_then(|s| s.as_str())?;
        let content = event.get("content")?;
        let msgtype = content.get("msgtype").and_then(|m| m.as_str())?;

        let timestamp = event
            .get("origin_server_ts")
            .and_then(|ts| ts.as_u64())
            .unwrap_or(0)
            / 1000;

        let reply_to = content
            .get("m.relates_to")
            .and_then(|r| r.get("m.in_reply_to"))
            .and_then(|r| r.get("event_id"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        let edit_of = content
            .get("m.relates_to")
            .and_then(|r| {
                let rel_type = r.get("rel_type").and_then(|t| t.as_str())?;
                if rel_type == "m.replace" {
                    r.get("event_id").and_then(|e| e.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });

        let msg_content = match msgtype {
            "m.text" => {
                let body = content.get("body").and_then(|b| b.as_str()).unwrap_or("");
                let formatted = content
                    .get("formatted_body")
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_string());
                MessageContent::Text {
                    body: body.to_string(),
                    formatted,
                }
            }
            "m.emote" => {
                let body = content.get("body").and_then(|b| b.as_str()).unwrap_or("");
                MessageContent::Text {
                    body: format!("* {}", body),
                    formatted: content
                        .get("formatted_body")
                        .and_then(|f| f.as_str())
                        .map(|s| format!("* {}", s)),
                }
            }
            "m.notice" => {
                let body = content.get("body").and_then(|b| b.as_str()).unwrap_or("");
                MessageContent::Notice {
                    body: body.to_string(),
                }
            }
            "m.image" => {
                let url = content
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let body = content
                    .get("body")
                    .and_then(|b| b.as_str())
                    .map(|s| s.to_string());
                let info = content.get("info");
                MessageContent::Image {
                    url,
                    caption: body,
                    mime_type: info
                        .and_then(|i| i.get("mimetype"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string()),
                    width: info
                        .and_then(|i| i.get("w"))
                        .and_then(|w| w.as_u64())
                        .map(|w| w as u32),
                    height: info
                        .and_then(|i| i.get("h"))
                        .and_then(|h| h.as_u64())
                        .map(|h| h as u32),
                    size: info.and_then(|i| i.get("size")).and_then(|s| s.as_u64()),
                }
            }
            "m.video" => {
                let url = content
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let body = content
                    .get("body")
                    .and_then(|b| b.as_str())
                    .map(|s| s.to_string());
                let info = content.get("info");
                MessageContent::Video {
                    url,
                    caption: body,
                    mime_type: info
                        .and_then(|i| i.get("mimetype"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string()),
                    duration: info
                        .and_then(|i| i.get("duration"))
                        .and_then(|d| d.as_u64())
                        .map(|d| (d / 1000) as u32),
                    width: info
                        .and_then(|i| i.get("w"))
                        .and_then(|w| w.as_u64())
                        .map(|w| w as u32),
                    height: info
                        .and_then(|i| i.get("h"))
                        .and_then(|h| h.as_u64())
                        .map(|h| h as u32),
                    size: info.and_then(|i| i.get("size")).and_then(|s| s.as_u64()),
                }
            }
            "m.audio" => {
                let url = content
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let body = content
                    .get("body")
                    .and_then(|b| b.as_str())
                    .map(|s| s.to_string());
                let info = content.get("info");
                MessageContent::Audio {
                    url,
                    caption: body,
                    mime_type: info
                        .and_then(|i| i.get("mimetype"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string()),
                    duration: info
                        .and_then(|i| i.get("duration"))
                        .and_then(|d| d.as_u64())
                        .map(|d| (d / 1000) as u32),
                    size: info.and_then(|i| i.get("size")).and_then(|s| s.as_u64()),
                }
            }
            "m.file" => {
                let url = content
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let filename = content
                    .get("body")
                    .and_then(|b| b.as_str())
                    .unwrap_or("file")
                    .to_string();
                let info = content.get("info");
                MessageContent::File {
                    url,
                    filename,
                    mime_type: info
                        .and_then(|i| i.get("mimetype"))
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string()),
                    size: info.and_then(|i| i.get("size")).and_then(|s| s.as_u64()),
                }
            }
            "m.location" => {
                let geo_uri = content.get("geo_uri").and_then(|g| g.as_str()).unwrap_or("");
                let (lat, lon) = Self::parse_geo_uri(geo_uri);
                let body = content
                    .get("body")
                    .and_then(|b| b.as_str())
                    .map(|s| s.to_string());
                MessageContent::Location {
                    latitude: lat,
                    longitude: lon,
                    description: body,
                }
            }
            _ => return None,
        };

        Some(CommonMessage {
            sender_id: sender.to_string(),
            sender_name: sender.to_string(),
            content: msg_content,
            reply_to,
            timestamp,
            edit_of,
            forward_from: None,
        })
    }

    fn parse_geo_uri(uri: &str) -> (f64, f64) {
        let coords = uri.strip_prefix("geo:").unwrap_or(uri);
        let parts: Vec<&str> = coords.split(',').collect();
        let lat = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let lon = parts.get(1).and_then(|s| {
            // May contain ;uncertainty suffix
            let clean = s.split(';').next().unwrap_or(s);
            clean.parse().ok()
        }).unwrap_or(0.0);
        (lat, lon)
    }

    /// Convert HTML formatted body to Telegram plain text with basic formatting.
    pub fn html_to_telegram(html: &str) -> String {
        let mut result = html.to_string();

        // Convert HTML tags to Telegram MarkdownV2 equivalents
        result = result.replace("<strong>", "*").replace("</strong>", "*");
        result = result.replace("<b>", "*").replace("</b>", "*");
        result = result.replace("<em>", "_").replace("</em>", "_");
        result = result.replace("<i>", "_").replace("</i>", "_");
        result = result.replace("<u>", "__").replace("</u>", "__");
        result = result.replace("<del>", "~").replace("</del>", "~");
        result = result.replace("<s>", "~").replace("</s>", "~");
        result = result.replace("<code>", "`").replace("</code>", "`");
        result = result.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
        result = result.replace("<p>", "").replace("</p>", "\n");
        result = result.replace("&lt;", "<").replace("&gt;", ">");
        result = result.replace("&amp;", "&").replace("&quot;", "\"");
        result = result.replace("&#39;", "'");

        // Handle pre blocks
        let pre_re = regex::Regex::new(r#"<pre><code(?:\s+class="language-(\w+)")?>([\s\S]*?)</code></pre>"#).ok();
        if let Some(re) = pre_re {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if lang.is_empty() {
                    format!("```\n{}\n```", code)
                } else {
                    format!("```{}\n{}\n```", lang, code)
                }
            }).to_string();
        }

        // Handle links
        let link_re = regex::Regex::new(r#"<a\s+href="([^"]*)"[^>]*>([^<]*)</a>"#).ok();
        if let Some(re) = link_re {
            result = re.replace_all(&result, "$2 ($1)").to_string();
        }

        // Handle blockquotes
        let bq_re = regex::Regex::new(r"<blockquote>([\s\S]*?)</blockquote>").ok();
        if let Some(re) = bq_re {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                text.lines()
                    .map(|line| format!("> {}", line))
                    .collect::<Vec<_>>()
                    .join("\n")
            }).to_string();
        }

        // Handle spoilers
        let spoiler_re = regex::Regex::new(r"<span\s+data-mx-spoiler[^>]*>([^<]*)</span>").ok();
        if let Some(re) = spoiler_re {
            result = re.replace_all(&result, "||$1||").to_string();
        }

        // Strip remaining HTML tags
        result = strip_html_tags(&result);

        // Clean up excessive newlines
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }

        result.trim().to_string()
    }

    /// Convert a CommonMessage content to Telegram plain text.
    pub fn matrix_to_telegram(content: &MessageContent) -> String {
        match content {
            MessageContent::Text { body, formatted } => {
                if let Some(html) = formatted {
                    Self::html_to_telegram(html)
                } else {
                    body.clone()
                }
            }
            MessageContent::Image { caption, .. } => {
                caption.clone().unwrap_or_default()
            }
            MessageContent::Video { caption, .. } => {
                caption.clone().unwrap_or_default()
            }
            MessageContent::Audio { caption, .. } => {
                caption.clone().unwrap_or_default()
            }
            MessageContent::File { filename, .. } => filename.clone(),
            MessageContent::Sticker { emoji, .. } => emoji.clone().unwrap_or_default(),
            MessageContent::Location {
                latitude,
                longitude,
                description,
            } => {
                if let Some(desc) = description {
                    format!("{} ({}, {})", desc, latitude, longitude)
                } else {
                    format!("Location: {}, {}", latitude, longitude)
                }
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
                format!("{}: {}", name, phone)
            }
            MessageContent::Notice { body } => body.clone(),
        }
    }

    /// Build a Matrix JSON event content from a CommonMessage.
    pub fn to_matrix_content(msg: &CommonMessage) -> serde_json::Value {
        let mut content = match &msg.content {
            MessageContent::Text { body, formatted } => {
                let mut c = serde_json::json!({
                    "msgtype": "m.text",
                    "body": body,
                });
                if let Some(html) = formatted {
                    c["format"] = serde_json::json!("org.matrix.custom.html");
                    c["formatted_body"] = serde_json::json!(html);
                }
                c
            }
            MessageContent::Image {
                url,
                caption,
                mime_type,
                width,
                height,
                size,
                ..
            } => {
                let body = caption.as_deref().unwrap_or("image");
                let mut info = serde_json::json!({});
                if let Some(m) = mime_type {
                    info["mimetype"] = serde_json::json!(m);
                }
                if let Some(w) = width {
                    info["w"] = serde_json::json!(w);
                }
                if let Some(h) = height {
                    info["h"] = serde_json::json!(h);
                }
                if let Some(s) = size {
                    info["size"] = serde_json::json!(s);
                }
                serde_json::json!({
                    "msgtype": "m.image",
                    "body": body,
                    "url": url,
                    "info": info,
                })
            }
            MessageContent::Video {
                url,
                caption,
                mime_type,
                duration,
                width,
                height,
                size,
            } => {
                let body = caption.as_deref().unwrap_or("video");
                let mut info = serde_json::json!({});
                if let Some(m) = mime_type {
                    info["mimetype"] = serde_json::json!(m);
                }
                if let Some(d) = duration {
                    info["duration"] = serde_json::json!(d * 1000);
                }
                if let Some(w) = width {
                    info["w"] = serde_json::json!(w);
                }
                if let Some(h) = height {
                    info["h"] = serde_json::json!(h);
                }
                if let Some(s) = size {
                    info["size"] = serde_json::json!(s);
                }
                serde_json::json!({
                    "msgtype": "m.video",
                    "body": body,
                    "url": url,
                    "info": info,
                })
            }
            MessageContent::Audio {
                url,
                caption,
                mime_type,
                duration,
                size,
            } => {
                let body = caption.as_deref().unwrap_or("audio");
                let mut info = serde_json::json!({});
                if let Some(m) = mime_type {
                    info["mimetype"] = serde_json::json!(m);
                }
                if let Some(d) = duration {
                    info["duration"] = serde_json::json!(d * 1000);
                }
                if let Some(s) = size {
                    info["size"] = serde_json::json!(s);
                }
                serde_json::json!({
                    "msgtype": "m.audio",
                    "body": body,
                    "url": url,
                    "info": info,
                })
            }
            MessageContent::File {
                url,
                filename,
                mime_type,
                size,
            } => {
                let mut info = serde_json::json!({});
                if let Some(m) = mime_type {
                    info["mimetype"] = serde_json::json!(m);
                }
                if let Some(s) = size {
                    info["size"] = serde_json::json!(s);
                }
                serde_json::json!({
                    "msgtype": "m.file",
                    "body": filename,
                    "url": url,
                    "info": info,
                })
            }
            MessageContent::Sticker {
                url,
                emoji,
                width,
                height,
            } => {
                let mut info = serde_json::json!({"mimetype": "image/png"});
                if let Some(w) = width {
                    info["w"] = serde_json::json!(w);
                }
                if let Some(h) = height {
                    info["h"] = serde_json::json!(h);
                }
                serde_json::json!({
                    "msgtype": "m.image",
                    "body": emoji.as_deref().unwrap_or("sticker"),
                    "url": url,
                    "info": info,
                })
            }
            MessageContent::Location {
                latitude,
                longitude,
                description,
            } => {
                let body = description
                    .as_deref()
                    .unwrap_or("Location");
                serde_json::json!({
                    "msgtype": "m.location",
                    "body": body,
                    "geo_uri": format!("geo:{},{}", latitude, longitude),
                })
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
                serde_json::json!({
                    "msgtype": "m.text",
                    "body": format!("{}: {}", name, phone),
                })
            }
            MessageContent::Notice { body } => {
                serde_json::json!({
                    "msgtype": "m.notice",
                    "body": body,
                })
            }
        };

        // Add reply relation if present
        if let Some(ref reply_to) = msg.reply_to {
            content["m.relates_to"] = serde_json::json!({
                "m.in_reply_to": {
                    "event_id": reply_to
                }
            });
        }

        content
    }
}

impl Default for MatrixParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_telegram_bold() {
        assert_eq!(MatrixParser::html_to_telegram("<strong>hello</strong>"), "*hello*");
    }

    #[test]
    fn test_html_to_telegram_complex() {
        let html = "<strong>bold</strong> and <em>italic</em>";
        let result = MatrixParser::html_to_telegram(html);
        assert_eq!(result, "*bold* and _italic_");
    }

    #[test]
    fn test_parse_geo_uri() {
        let (lat, lon) = MatrixParser::parse_geo_uri("geo:48.8584,2.2945");
        assert!((lat - 48.8584).abs() < 0.0001);
        assert!((lon - 2.2945).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_to_telegram_text() {
        let content = MessageContent::text("hello world");
        assert_eq!(MatrixParser::matrix_to_telegram(&content), "hello world");
    }
}

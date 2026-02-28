pub mod command_handler;
pub mod event_handler;

use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;

pub struct MatrixAppservice {
    config: Arc<Config>,
}

impl MatrixAppservice {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn set_processor(&self, _processor: Arc<MatrixEventProcessor>) {}

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Matrix Appservice started");
        Ok(())
    }

    pub async fn get_room_members(&self, _room_id: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    pub async fn send_message(&self, _room_id: &str, _content: serde_json::Value) -> Result<String> {
        Ok(String::new())
    }

    pub async fn send_text(&self, room_id: &str, text: &str) -> Result<String> {
        let content = serde_json::json!({
            "msgtype": "m.text",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_emote(&self, room_id: &str, text: &str) -> Result<String> {
        let content = serde_json::json!({
            "msgtype": "m.emote",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_notice(&self, room_id: &str, text: &str) -> Result<String> {
        let content = serde_json::json!({
            "msgtype": "m.notice",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_image(&self, room_id: &str, url: &str, body: &str) -> Result<String> {
        let content = serde_json::json!({
            "msgtype": "m.image",
            "url": url,
            "body": body
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_file(&self, room_id: &str, url: &str, filename: &str) -> Result<String> {
        let content = serde_json::json!({
            "msgtype": "m.file",
            "url": url,
            "body": filename
        });
        self.send_message(room_id, content).await
    }

    pub async fn redact_event(&self, _room_id: &str, _event_id: &str, _reason: Option<&str>) -> Result<()> {
        Ok(())
    }

    pub async fn get_user_mxid(&self, telegram_user_id: i64) -> String {
        let template = &self.config.portal.username_template;
        template.replace("{userid}", &telegram_user_id.to_string())
    }
}

pub struct MatrixEventProcessor {
    handler: Arc<dyn MatrixEventHandler + Send + Sync>,
    age_limit_ms: u64,
}

impl MatrixEventProcessor {
    pub fn with_age_limit(handler: Arc<dyn MatrixEventHandler + Send + Sync>, age_limit_ms: u64) -> Self {
        Self { handler, age_limit_ms }
    }

    pub fn age_limit_ms(&self) -> u64 {
        self.age_limit_ms
    }
}

#[async_trait::async_trait]
pub trait MatrixEventHandler: Send + Sync {
    async fn handle_room_message(&self, room_id: &str, event: &serde_json::Value);
    async fn handle_room_redaction(&self, room_id: &str, event: &serde_json::Value);
    async fn handle_room_member(&self, room_id: &str, event: &serde_json::Value);
}

pub struct MatrixEventHandlerImpl {
    matrix_client: Arc<MatrixAppservice>,
    bridge: Option<Arc<crate::bridge::BridgeCore>>,
}

impl MatrixEventHandlerImpl {
    pub fn new(matrix_client: Arc<MatrixAppservice>) -> Self {
        Self {
            matrix_client,
            bridge: None,
        }
    }

    pub fn set_bridge(&mut self, bridge: Arc<crate::bridge::BridgeCore>) {
        self.bridge = Some(bridge);
    }
}

#[async_trait::async_trait]
impl MatrixEventHandler for MatrixEventHandlerImpl {
    async fn handle_room_message(&self, room_id: &str, event: &serde_json::Value) {
        tracing::debug!("Handling Matrix message in room {}: {:?}", room_id, event);
        
        if let Some(content) = event.get("content") {
            if let Some(msgtype) = content.get("msgtype").and_then(|m| m.as_str()) {
                match msgtype {
                    "m.text" | "m.emote" | "m.notice" => {
                        if let Some(body) = content.get("body").and_then(|b| b.as_str()) {
                            tracing::info!("Matrix text message: {}", body);
                        }
                    }
                    "m.image" => {
                        if let Some(url) = content.get("url").and_then(|u| u.as_str()) {
                            tracing::info!("Matrix image: {}", url);
                        }
                    }
                    "m.file" => {
                        if let Some(url) = content.get("url").and_then(|u| u.as_str()) {
                            tracing::info!("Matrix file: {}", url);
                        }
                    }
                    "m.video" => {
                        if let Some(url) = content.get("url").and_then(|u| u.as_str()) {
                            tracing::info!("Matrix video: {}", url);
                        }
                    }
                    "m.audio" => {
                        if let Some(url) = content.get("url").and_then(|u| u.as_str()) {
                            tracing::info!("Matrix audio: {}", url);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    async fn handle_room_redaction(&self, room_id: &str, event: &serde_json::Value) {
        tracing::debug!("Handling Matrix redaction in room {}: {:?}", room_id, event);
    }

    async fn handle_room_member(&self, room_id: &str, event: &serde_json::Value) {
        tracing::debug!("Handling Matrix member event in room {}: {:?}", room_id, event);
    }
}

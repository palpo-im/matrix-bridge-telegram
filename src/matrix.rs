pub mod command_handler;
pub mod event_handler;

use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::Config;

/// Matrix Appservice client that communicates with the homeserver via HTTP.
pub struct MatrixAppservice {
    config: Arc<Config>,
    http_client: Client,
    processor: RwLock<Option<Arc<MatrixEventProcessor>>>,
}

impl MatrixAppservice {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            http_client,
            processor: RwLock::new(None),
        })
    }

    pub async fn set_processor(&self, processor: Arc<MatrixEventProcessor>) {
        *self.processor.write().await = Some(processor);
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Base URL for Matrix client-server API calls.
    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/_matrix/client/v3{}",
            self.config.bridge.homeserver_url, path
        )
    }

    /// Authorization header value using the appservice token.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.registration.appservice_token)
    }

    pub async fn start(&self) -> Result<()> {
        info!("Matrix Appservice started");
        Ok(())
    }

    /// Send a message event to a room.
    pub async fn send_message(&self, room_id: &str, content: serde_json::Value) -> Result<String> {
        let txn_id = uuid::Uuid::new_v4().to_string();
        let url = self.api_url(&format!(
            "/rooms/{}/send/m.room.message/{}",
            urlencoding_encode(room_id),
            txn_id
        ));

        debug!("Sending message to room {}", room_id);

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&content)
            .send()
            .await?;

        let status = response.status();
        let body: serde_json::Value = response.json().await?;

        if !status.is_success() {
            let errcode = body
                .get("errcode")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown");
            let error = body
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error");
            return Err(anyhow::anyhow!(
                "Matrix API error {}: {} - {}",
                status,
                errcode,
                error
            ));
        }

        let event_id = body
            .get("event_id")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string();

        debug!("Sent message to {}, event_id: {}", room_id, event_id);
        Ok(event_id)
    }

    /// Send a message as a puppet user (appservice impersonation).
    pub async fn send_message_as(
        &self,
        room_id: &str,
        user_id: &str,
        content: serde_json::Value,
    ) -> Result<String> {
        let txn_id = uuid::Uuid::new_v4().to_string();
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!(
                "/rooms/{}/send/m.room.message/{}",
                urlencoding_encode(room_id),
                txn_id
            )),
            urlencoding_encode(user_id)
        );

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&content)
            .send()
            .await?;

        let status = response.status();
        let body: serde_json::Value = response.json().await?;

        if !status.is_success() {
            let errcode = body.get("errcode").and_then(|e| e.as_str()).unwrap_or("");
            let error = body.get("error").and_then(|e| e.as_str()).unwrap_or("");
            return Err(anyhow::anyhow!("Matrix API error: {} - {}", errcode, error));
        }

        Ok(body
            .get("event_id")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string())
    }

    pub async fn send_text(&self, room_id: &str, text: &str) -> Result<String> {
        let content = json!({
            "msgtype": "m.text",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_html(
        &self,
        room_id: &str,
        text: &str,
        html: &str,
    ) -> Result<String> {
        let content = json!({
            "msgtype": "m.text",
            "body": text,
            "format": "org.matrix.custom.html",
            "formatted_body": html
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_emote(&self, room_id: &str, text: &str) -> Result<String> {
        let content = json!({
            "msgtype": "m.emote",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_notice(&self, room_id: &str, text: &str) -> Result<String> {
        let content = json!({
            "msgtype": "m.notice",
            "body": text
        });
        self.send_message(room_id, content).await
    }

    pub async fn send_image(
        &self,
        room_id: &str,
        url: &str,
        body: &str,
        info: Option<serde_json::Value>,
    ) -> Result<String> {
        let mut content = json!({
            "msgtype": "m.image",
            "url": url,
            "body": body
        });
        if let Some(i) = info {
            content["info"] = i;
        }
        self.send_message(room_id, content).await
    }

    pub async fn send_file(
        &self,
        room_id: &str,
        url: &str,
        filename: &str,
        info: Option<serde_json::Value>,
    ) -> Result<String> {
        let mut content = json!({
            "msgtype": "m.file",
            "url": url,
            "body": filename
        });
        if let Some(i) = info {
            content["info"] = i;
        }
        self.send_message(room_id, content).await
    }

    pub async fn send_video(
        &self,
        room_id: &str,
        url: &str,
        body: &str,
        info: Option<serde_json::Value>,
    ) -> Result<String> {
        let mut content = json!({
            "msgtype": "m.video",
            "url": url,
            "body": body
        });
        if let Some(i) = info {
            content["info"] = i;
        }
        self.send_message(room_id, content).await
    }

    pub async fn send_audio(
        &self,
        room_id: &str,
        url: &str,
        body: &str,
        info: Option<serde_json::Value>,
    ) -> Result<String> {
        let mut content = json!({
            "msgtype": "m.audio",
            "url": url,
            "body": body
        });
        if let Some(i) = info {
            content["info"] = i;
        }
        self.send_message(room_id, content).await
    }

    pub async fn send_location(
        &self,
        room_id: &str,
        body: &str,
        geo_uri: &str,
    ) -> Result<String> {
        let content = json!({
            "msgtype": "m.location",
            "body": body,
            "geo_uri": geo_uri
        });
        self.send_message(room_id, content).await
    }

    /// Redact (delete) a message event.
    pub async fn redact_event(
        &self,
        room_id: &str,
        event_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let txn_id = uuid::Uuid::new_v4().to_string();
        let url = self.api_url(&format!(
            "/rooms/{}/redact/{}/{}",
            urlencoding_encode(room_id),
            urlencoding_encode(event_id),
            txn_id
        ));

        let mut body = json!({});
        if let Some(r) = reason {
            body["reason"] = json!(r);
        }

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err: serde_json::Value = response.json().await?;
            warn!("Failed to redact event: {:?}", err);
        }

        Ok(())
    }

    /// Send a read receipt for an event.
    pub async fn send_read_receipt(&self, room_id: &str, event_id: &str) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/receipt/m.read/{}",
            urlencoding_encode(room_id),
            urlencoding_encode(event_id)
        ));

        self.http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({}))
            .send()
            .await?;

        Ok(())
    }

    /// Send a typing notification.
    pub async fn send_typing(
        &self,
        room_id: &str,
        user_id: &str,
        typing: bool,
        timeout_ms: u64,
    ) -> Result<()> {
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!(
                "/rooms/{}/typing/{}",
                urlencoding_encode(room_id),
                urlencoding_encode(user_id)
            )),
            urlencoding_encode(user_id)
        );

        let body = json!({
            "typing": typing,
            "timeout": timeout_ms
        });

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    /// Get members of a room.
    pub async fn get_room_members(&self, room_id: &str) -> Result<Vec<String>> {
        let url = self.api_url(&format!(
            "/rooms/{}/members",
            urlencoding_encode(room_id)
        ));

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        let members = body
            .get("chunk")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        let membership = e
                            .get("content")
                            .and_then(|c| c.get("membership"))
                            .and_then(|m| m.as_str());
                        if membership == Some("join") {
                            e.get("state_key")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(members)
    }

    /// Create a room for a portal.
    pub async fn create_room(
        &self,
        name: Option<&str>,
        alias: Option<&str>,
        topic: Option<&str>,
        invite: &[String],
        is_direct: bool,
    ) -> Result<String> {
        let url = self.api_url("/createRoom");

        let mut body = json!({
            "preset": if is_direct { "trusted_private_chat" } else { "private_chat" },
            "creation_content": {
                "m.federate": self.config.portal.federate_rooms
            }
        });

        if let Some(n) = name {
            body["name"] = json!(n);
        }
        if let Some(a) = alias {
            body["room_alias_name"] = json!(a);
        }
        if let Some(t) = topic {
            body["topic"] = json!(t);
        }
        if !invite.is_empty() {
            body["invite"] = json!(invite);
        }
        if is_direct {
            body["is_direct"] = json!(true);
        }

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        let room_id = result
            .get("room_id")
            .and_then(|r| r.as_str())
            .ok_or_else(|| anyhow::anyhow!("no room_id in create room response"))?
            .to_string();

        info!("Created Matrix room: {}", room_id);
        Ok(room_id)
    }

    /// Ensure a puppet user is registered on the homeserver.
    pub async fn ensure_registered(&self, localpart: &str) -> Result<()> {
        let url = self.api_url("/register");

        let body = json!({
            "type": "m.login.application_service",
            "username": localpart
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 400 {
            // 400 means already registered, which is fine
            Ok(())
        } else {
            let err: serde_json::Value = response.json().await?;
            Err(anyhow::anyhow!("Failed to register user: {:?}", err))
        }
    }

    /// Set the display name for a puppet user.
    pub async fn set_displayname(&self, user_id: &str, displayname: &str) -> Result<()> {
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!(
                "/profile/{}/displayname",
                urlencoding_encode(user_id)
            )),
            urlencoding_encode(user_id)
        );

        let body = json!({ "displayname": displayname });

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    /// Set the avatar for a puppet user.
    pub async fn set_avatar_url(&self, user_id: &str, avatar_url: &str) -> Result<()> {
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!(
                "/profile/{}/avatar_url",
                urlencoding_encode(user_id)
            )),
            urlencoding_encode(user_id)
        );

        let body = json!({ "avatar_url": avatar_url });

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    /// Join a room as a puppet user.
    pub async fn join_room_as(&self, room_id: &str, user_id: &str) -> Result<()> {
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!("/join/{}", urlencoding_encode(room_id))),
            urlencoding_encode(user_id)
        );

        self.http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({}))
            .send()
            .await?;

        Ok(())
    }

    /// Leave a room as a puppet user.
    pub async fn leave_room_as(&self, room_id: &str, user_id: &str) -> Result<()> {
        let url = format!(
            "{}?user_id={}",
            self.api_url(&format!(
                "/rooms/{}/leave",
                urlencoding_encode(room_id)
            )),
            urlencoding_encode(user_id)
        );

        self.http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({}))
            .send()
            .await?;

        Ok(())
    }

    /// Invite a user to a room.
    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/invite",
            urlencoding_encode(room_id)
        ));

        let body = json!({ "user_id": user_id });

        self.http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    /// Kick a user from a room.
    pub async fn kick_user(
        &self,
        room_id: &str,
        user_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/kick",
            urlencoding_encode(room_id)
        ));

        let mut body = json!({ "user_id": user_id });
        if let Some(r) = reason {
            body["reason"] = json!(r);
        }

        self.http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    /// Set the room name.
    pub async fn set_room_name(&self, room_id: &str, name: &str) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/state/m.room.name",
            urlencoding_encode(room_id)
        ));

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({ "name": name }))
            .send()
            .await?;

        Ok(())
    }

    /// Set the room topic.
    pub async fn set_room_topic(&self, room_id: &str, topic: &str) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/state/m.room.topic",
            urlencoding_encode(room_id)
        ));

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({ "topic": topic }))
            .send()
            .await?;

        Ok(())
    }

    /// Set the room avatar.
    pub async fn set_room_avatar(&self, room_id: &str, avatar_url: &str) -> Result<()> {
        let url = self.api_url(&format!(
            "/rooms/{}/state/m.room.avatar",
            urlencoding_encode(room_id)
        ));

        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&json!({ "url": avatar_url }))
            .send()
            .await?;

        Ok(())
    }

    /// Set power levels for a user in a room.
    pub async fn set_power_level(
        &self,
        room_id: &str,
        user_id: &str,
        power_level: i64,
    ) -> Result<()> {
        // First get current power levels
        let url = self.api_url(&format!(
            "/rooms/{}/state/m.room.power_levels",
            urlencoding_encode(room_id)
        ));

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let mut power_levels: serde_json::Value = if response.status().is_success() {
            response.json().await?
        } else {
            json!({ "users": {} })
        };

        // Update the user's power level
        if let Some(users) = power_levels.get_mut("users") {
            users[user_id] = json!(power_level);
        } else {
            power_levels["users"] = json!({ user_id: power_level });
        }

        // Set the updated power levels
        self.http_client
            .put(&url)
            .header("Authorization", self.auth_header())
            .json(&power_levels)
            .send()
            .await?;

        Ok(())
    }

    /// Generate the Matrix user ID for a Telegram user.
    pub async fn get_user_mxid(&self, telegram_user_id: i64) -> String {
        let localpart = self
            .config
            .portal
            .username_template
            .replace("{userid}", &telegram_user_id.to_string());
        format!("@{}:{}", localpart, self.config.bridge.domain)
    }

    /// Generate the Matrix room alias for a Telegram chat.
    pub fn get_room_alias(&self, chat_name: &str) -> String {
        let alias_localpart = self
            .config
            .portal
            .alias_template
            .replace("{groupname}", chat_name);
        format!("#{}:{}", alias_localpart, self.config.bridge.domain)
    }

    /// Process an incoming appservice transaction.
    pub async fn process_transaction(&self, transaction: &serde_json::Value) -> Result<()> {
        let events = transaction
            .get("events")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();

        let processor = self.processor.read().await;
        let processor = match processor.as_ref() {
            Some(p) => p,
            None => {
                warn!("No event processor set, dropping {} events", events.len());
                return Ok(());
            }
        };

        for event in &events {
            let event_type = event
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("");
            let room_id = event
                .get("room_id")
                .and_then(|r| r.as_str())
                .unwrap_or("");

            // Check event age
            let age = event
                .get("unsigned")
                .and_then(|u| u.get("age"))
                .and_then(|a| a.as_u64())
                .unwrap_or(0);

            if age > processor.age_limit_ms() {
                debug!(
                    "Skipping old event {} (age {}ms > {}ms)",
                    event_type,
                    age,
                    processor.age_limit_ms()
                );
                continue;
            }

            match event_type {
                "m.room.message" => {
                    processor.handler.handle_room_message(room_id, event).await;
                }
                "m.room.redaction" => {
                    processor.handler.handle_room_redaction(room_id, event).await;
                }
                "m.room.member" => {
                    processor.handler.handle_room_member(room_id, event).await;
                }
                "m.reaction" => {
                    processor.handler.handle_room_reaction(room_id, event).await;
                }
                _ => {
                    debug!("Unhandled event type: {}", event_type);
                }
            }
        }

        Ok(())
    }
}

pub struct MatrixEventProcessor {
    pub(crate) handler: Arc<dyn MatrixEventHandler + Send + Sync>,
    age_limit_ms: u64,
}

impl MatrixEventProcessor {
    pub fn with_age_limit(
        handler: Arc<dyn MatrixEventHandler + Send + Sync>,
        age_limit_ms: u64,
    ) -> Self {
        Self {
            handler,
            age_limit_ms,
        }
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
    async fn handle_room_reaction(&self, room_id: &str, event: &serde_json::Value);
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

    /// Check if the sender is a bridge ghost user.
    fn is_bridge_ghost(&self, sender: &str) -> bool {
        let prefix = format!(
            "@{}",
            self.matrix_client
                .config
                .portal
                .username_template
                .split("{userid}")
                .next()
                .unwrap_or("telegram_")
        );
        sender.starts_with(&prefix)
    }
}

#[async_trait::async_trait]
impl MatrixEventHandler for MatrixEventHandlerImpl {
    async fn handle_room_message(&self, room_id: &str, event: &serde_json::Value) {
        let sender = match event.get("sender").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => return,
        };

        // Don't process messages from our own ghost users
        if self.is_bridge_ghost(sender) {
            return;
        }

        let event_id = event
            .get("event_id")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        let content = match event.get("content") {
            Some(c) => c,
            None => return,
        };

        // Check for edit
        if let Some(relates_to) = content.get("m.relates_to") {
            if relates_to.get("rel_type").and_then(|t| t.as_str()) == Some("m.replace") {
                if let Some(original_event_id) = relates_to.get("event_id").and_then(|e| e.as_str()) {
                    let new_content = content.get("m.new_content").unwrap_or(content);
                    let new_body = new_content.get("body").and_then(|b| b.as_str()).unwrap_or("");
                    if let Some(ref bridge) = self.bridge {
                        if let Err(e) = bridge.handle_matrix_edit(room_id, event_id, sender, original_event_id, new_body).await {
                            error!("Failed to handle Matrix edit: {}", e);
                        }
                    }
                    return;
                }
            }
        }

        let msgtype = content
            .get("msgtype")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        // Handle commands
        if let Some(body) = content.get("body").and_then(|b| b.as_str()) {
            let prefix = &self.matrix_client.config.bridge.command_prefix;
            let check_prefix = if prefix.is_empty() { "!tg" } else { prefix };
            if body.starts_with(check_prefix) {
                if let Some(ref bridge) = self.bridge {
                    let cmd_handler = command_handler::CommandHandler::new();
                    if let Some(response) = cmd_handler
                        .process_with_bridge(room_id, sender, body, bridge)
                        .await
                    {
                        if let Err(e) = self.matrix_client.send_notice(room_id, &response).await {
                            error!("Failed to send command response: {}", e);
                        }
                    }
                }
                return;
            }
        }

        // Forward message to Telegram via bridge
        if let Some(ref bridge) = self.bridge {
            match msgtype {
                "m.text" | "m.emote" | "m.notice" => {
                    let body = content
                        .get("body")
                        .and_then(|b| b.as_str())
                        .unwrap_or("");
                    if let Err(e) = bridge
                        .handle_matrix_message(room_id, event_id, sender, body)
                        .await
                    {
                        error!("Failed to handle Matrix message: {}", e);
                    }
                }
                "m.image" | "m.file" | "m.video" | "m.audio" => {
                    if let Err(e) = bridge
                        .handle_matrix_media(room_id, event_id, sender, content)
                        .await
                    {
                        error!("Failed to handle Matrix media: {}", e);
                    }
                }
                "m.location" => {
                    if let Err(e) = bridge
                        .handle_matrix_location(room_id, event_id, sender, content)
                        .await
                    {
                        error!("Failed to handle Matrix location: {}", e);
                    }
                }
                _ => {
                    debug!("Unhandled message type: {}", msgtype);
                }
            }
        }
    }

    async fn handle_room_redaction(&self, room_id: &str, event: &serde_json::Value) {
        let sender = match event.get("sender").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => return,
        };

        if self.is_bridge_ghost(sender) {
            return;
        }

        let redacts = event
            .get("redacts")
            .and_then(|r| r.as_str())
            .unwrap_or("");

        if redacts.is_empty() {
            return;
        }

        if let Some(ref bridge) = self.bridge {
            if let Err(e) = bridge.handle_matrix_redaction(room_id, redacts).await {
                error!("Failed to handle Matrix redaction: {}", e);
            }
        }
    }

    async fn handle_room_member(&self, room_id: &str, event: &serde_json::Value) {
        let membership = event
            .get("content")
            .and_then(|c| c.get("membership"))
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let state_key = event
            .get("state_key")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        let sender = event
            .get("sender")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        debug!(
            "Member event in {}: {} {} by {}",
            room_id, state_key, membership, sender
        );

        if let Some(ref bridge) = self.bridge {
            match membership {
                "join" => {
                    if let Err(e) = bridge.handle_matrix_join(room_id, state_key).await {
                        error!("Failed to handle Matrix join: {}", e);
                    }
                }
                "leave" => {
                    if sender == state_key {
                        if let Err(e) = bridge.handle_matrix_leave(room_id, state_key).await {
                            error!("Failed to handle Matrix leave: {}", e);
                        }
                    } else {
                        // This is a kick
                        if let Err(e) = bridge
                            .handle_matrix_kick(room_id, state_key, sender)
                            .await
                        {
                            error!("Failed to handle Matrix kick: {}", e);
                        }
                    }
                }
                "ban" => {
                    if let Err(e) = bridge
                        .handle_matrix_ban(room_id, state_key, sender)
                        .await
                    {
                        error!("Failed to handle Matrix ban: {}", e);
                    }
                }
                "invite" => {
                    if let Err(e) = bridge
                        .handle_matrix_invite(room_id, state_key, sender)
                        .await
                    {
                        error!("Failed to handle Matrix invite: {}", e);
                    }
                }
                _ => {}
            }
        }
    }

    async fn handle_room_reaction(&self, room_id: &str, event: &serde_json::Value) {
        let sender = match event.get("sender").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => return,
        };

        // Don't process reactions from our own ghost users
        if self.is_bridge_ghost(sender) {
            return;
        }

        let content = match event.get("content") {
            Some(c) => c,
            None => return,
        };

        let relates_to = match content.get("m.relates_to") {
            Some(r) => r,
            None => return,
        };

        let target_event_id = match relates_to.get("event_id").and_then(|e| e.as_str()) {
            Some(id) => id,
            None => return,
        };

        let reaction_key = match relates_to.get("key").and_then(|k| k.as_str()) {
            Some(k) => k,
            None => return,
        };

        if let Some(ref bridge) = self.bridge {
            if let Err(e) = bridge
                .handle_matrix_reaction(room_id, target_event_id, sender, reaction_key)
                .await
            {
                error!("Failed to handle Matrix reaction: {}", e);
            }
        }
    }
}

/// Simple URL encoding for path segments.
fn urlencoding_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b'!'
            | b':'
            | b'@' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

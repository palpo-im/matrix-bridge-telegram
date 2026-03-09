pub mod message_flow;
pub mod portal;
pub mod puppet;
pub mod user_sync;

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::db::models::{MessageMapping, PortalInfo as DbPortalInfo};
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;
use crate::media::MediaHandler;
use crate::telegram::TelegramClient;

pub struct BridgeCore {
    matrix_client: Arc<MatrixAppservice>,
    telegram_client: Arc<TelegramClient>,
    db_manager: Arc<DatabaseManager>,
    media_handler: MediaHandler,
    running: RwLock<bool>,
    portal_manager: PortalManager,
    puppet_manager: PuppetManager,
}

impl BridgeCore {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        telegram_client: Arc<TelegramClient>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        let config = matrix_client.config().clone();
        let media = MediaHandler::new(Arc::new(config))
            .with_file_store(db_manager.telegram_file_store());

        Self {
            matrix_client,
            telegram_client,
            db_manager,
            media_handler: media,
            running: RwLock::new(false),
            portal_manager: PortalManager::new(),
            puppet_manager: PuppetManager::new(),
        }
    }

    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!("Bridge core started");

        // Start the Telegram bot
        self.telegram_client.start().await?;

        // Load existing portals from database
        self.load_portals().await?;

        // Keep running until stopped
        while *self.running.read().await {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Bridge core stopped");
    }

    /// Load existing portals from the database into memory.
    async fn load_portals(&self) -> Result<()> {
        let portal_store = self.db_manager.portal_store();
        let portals = portal_store.list_all(1000).await.unwrap_or_default();

        for p in portals {
            self.portal_manager
                .add_portal(PortalInfo {
                    matrix_room_id: p.matrix_room_id,
                    telegram_chat_id: p.telegram_chat_id,
                    telegram_chat_type: p.telegram_chat_type,
                    title: p.telegram_chat_title,
                })
                .await;
        }

        let count = self.portal_manager.count().await;
        info!("Loaded {} portals from database", count);
        Ok(())
    }

    pub fn matrix_client(&self) -> &MatrixAppservice {
        &self.matrix_client
    }

    pub fn telegram_client(&self) -> &TelegramClient {
        &self.telegram_client
    }

    pub fn db_manager(&self) -> &DatabaseManager {
        &self.db_manager
    }

    pub fn portal_manager(&self) -> &PortalManager {
        &self.portal_manager
    }

    pub fn puppet_manager(&self) -> &PuppetManager {
        &self.puppet_manager
    }

    // ========================================================================
    // Telegram -> Matrix message handlers
    // ========================================================================

    /// Handle a text message from Telegram.
    pub async fn handle_telegram_message(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        content: &str,
    ) -> Result<()> {
        debug!(
            "Handling Telegram message: chat={}, msg={}, sender={}",
            chat_id, message_id, sender_id
        );

        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => {
                debug!("No portal for Telegram chat {}, skipping", chat_id);
                return Ok(());
            }
        };

        // Ensure puppet exists for the sender
        self.ensure_puppet(sender_id).await?;

        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        // Send message to Matrix as the puppet
        let event_id = self
            .matrix_client
            .send_message_as(
                &portal.matrix_room_id,
                &puppet_mxid,
                serde_json::json!({
                    "msgtype": "m.text",
                    "body": content
                }),
            )
            .await?;

        // Store the message mapping
        self.store_message_mapping(
            message_id as i64,
            chat_id,
            &portal.matrix_room_id,
            &event_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_telegram_photo(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        file_id: &str,
        file_unique_id: &str,
        caption: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        // Transfer media from Telegram to Matrix
        if let Some(token) = self.telegram_client.bot_token() {
            match self
                .media_handler
                .transfer_telegram_to_matrix(token, file_id, file_unique_id, Some("photo.jpg"))
                .await
            {
                Ok(mxc_url) => {
                    let body = if caption.is_empty() {
                        "photo.jpg"
                    } else {
                        caption
                    };
                    let content = serde_json::json!({
                        "msgtype": "m.image",
                        "body": body,
                        "url": mxc_url,
                        "info": {"mimetype": "image/jpeg"}
                    });
                    let event_id = self
                        .matrix_client
                        .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
                        .await?;
                    self.store_message_mapping(
                        message_id as i64,
                        chat_id,
                        &portal.matrix_room_id,
                        &event_id,
                    )
                    .await?;
                }
                Err(e) => {
                    warn!("Failed to transfer photo: {}", e);
                    // Fallback: send caption as text
                    if !caption.is_empty() {
                        self.handle_telegram_message(chat_id, message_id, sender_id, caption)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_document(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        file_id: &str,
        file_unique_id: &str,
        filename: &str,
        mime_type: &str,
        caption: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        if let Some(token) = self.telegram_client.bot_token() {
            match self
                .media_handler
                .transfer_telegram_to_matrix(token, file_id, file_unique_id, Some(filename))
                .await
            {
                Ok(mxc_url) => {
                    let content = serde_json::json!({
                        "msgtype": "m.file",
                        "body": filename,
                        "url": mxc_url,
                        "info": {"mimetype": mime_type}
                    });
                    let event_id = self
                        .matrix_client
                        .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
                        .await?;
                    self.store_message_mapping(
                        message_id as i64,
                        chat_id,
                        &portal.matrix_room_id,
                        &event_id,
                    )
                    .await?;

                    // Send caption separately if present
                    if !caption.is_empty() {
                        self.matrix_client
                            .send_message_as(
                                &portal.matrix_room_id,
                                &puppet_mxid,
                                serde_json::json!({
                                    "msgtype": "m.text",
                                    "body": caption
                                }),
                            )
                            .await?;
                    }
                }
                Err(e) => warn!("Failed to transfer document: {}", e),
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_video(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        file_id: &str,
        file_unique_id: &str,
        caption: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        if let Some(token) = self.telegram_client.bot_token() {
            match self
                .media_handler
                .transfer_telegram_to_matrix(token, file_id, file_unique_id, Some("video.mp4"))
                .await
            {
                Ok(mxc_url) => {
                    let body = if caption.is_empty() { "video.mp4" } else { caption };
                    let content = serde_json::json!({
                        "msgtype": "m.video",
                        "body": body,
                        "url": mxc_url,
                        "info": {"mimetype": "video/mp4"}
                    });
                    let event_id = self
                        .matrix_client
                        .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
                        .await?;
                    self.store_message_mapping(
                        message_id as i64,
                        chat_id,
                        &portal.matrix_room_id,
                        &event_id,
                    )
                    .await?;
                }
                Err(e) => warn!("Failed to transfer video: {}", e),
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_audio(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        file_id: &str,
        file_unique_id: &str,
        caption: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        if let Some(token) = self.telegram_client.bot_token() {
            match self
                .media_handler
                .transfer_telegram_to_matrix(token, file_id, file_unique_id, Some("audio.ogg"))
                .await
            {
                Ok(mxc_url) => {
                    let body = if caption.is_empty() { "audio.ogg" } else { caption };
                    let content = serde_json::json!({
                        "msgtype": "m.audio",
                        "body": body,
                        "url": mxc_url,
                        "info": {"mimetype": "audio/ogg"}
                    });
                    let event_id = self
                        .matrix_client
                        .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
                        .await?;
                    self.store_message_mapping(
                        message_id as i64,
                        chat_id,
                        &portal.matrix_room_id,
                        &event_id,
                    )
                    .await?;
                }
                Err(e) => warn!("Failed to transfer audio: {}", e),
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_sticker(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        file_id: &str,
        file_unique_id: &str,
        emoji: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        if let Some(token) = self.telegram_client.bot_token() {
            match self
                .media_handler
                .transfer_telegram_to_matrix(token, file_id, file_unique_id, Some("sticker.webp"))
                .await
            {
                Ok(mxc_url) => {
                    let body = if emoji.is_empty() { "sticker" } else { emoji };
                    let content = serde_json::json!({
                        "msgtype": "m.image",
                        "body": body,
                        "url": mxc_url,
                        "info": {"mimetype": "image/webp", "w": 256, "h": 256}
                    });
                    let event_id = self
                        .matrix_client
                        .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
                        .await?;
                    self.store_message_mapping(
                        message_id as i64,
                        chat_id,
                        &portal.matrix_room_id,
                        &event_id,
                    )
                    .await?;
                }
                Err(e) => warn!("Failed to transfer sticker: {}", e),
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_location(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        latitude: f64,
        longitude: f64,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        let content = serde_json::json!({
            "msgtype": "m.location",
            "body": format!("Location: {}, {}", latitude, longitude),
            "geo_uri": format!("geo:{},{}", latitude, longitude)
        });

        let event_id = self
            .matrix_client
            .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
            .await?;

        self.store_message_mapping(
            message_id as i64,
            chat_id,
            &portal.matrix_room_id,
            &event_id,
        )
        .await?;

        Ok(())
    }

    pub async fn handle_telegram_contact(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        phone: &str,
        first_name: &str,
        last_name: Option<&str>,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(sender_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(sender_id).await;

        let name = if let Some(last) = last_name {
            format!("{} {}", first_name, last)
        } else {
            first_name.to_string()
        };

        let content = serde_json::json!({
            "msgtype": "m.text",
            "body": format!("Contact: {} ({})", name, phone),
            "format": "org.matrix.custom.html",
            "formatted_body": format!("<b>{}</b>: <a href=\"tel:{}\">{}</a>", name, phone, phone)
        });

        let event_id = self
            .matrix_client
            .send_message_as(&portal.matrix_room_id, &puppet_mxid, content)
            .await?;

        self.store_message_mapping(
            message_id as i64,
            chat_id,
            &portal.matrix_room_id,
            &event_id,
        )
        .await?;

        Ok(())
    }

    /// Handle a Telegram message deletion.
    pub async fn handle_telegram_deletion(
        &self,
        chat_id: i64,
        message_ids: &[i32],
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        let message_store = self.db_manager.message_store();
        for &msg_id in message_ids {
            if let Ok(Some(mapping)) = message_store
                .get_by_telegram_message(chat_id, msg_id as i64)
                .await
            {
                if let Err(e) = self
                    .matrix_client
                    .redact_event(&portal.matrix_room_id, &mapping.matrix_event_id, None)
                    .await
                {
                    warn!("Failed to redact Matrix event: {}", e);
                }
                let _ = message_store
                    .delete_by_telegram_message(chat_id, msg_id as i64)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn handle_telegram_join(&self, chat_id: i64, user_id: i64) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.ensure_puppet(user_id).await?;
        let puppet_mxid = self.matrix_client.get_user_mxid(user_id).await;
        self.matrix_client
            .join_room_as(&portal.matrix_room_id, &puppet_mxid)
            .await?;

        Ok(())
    }

    pub async fn handle_telegram_leave(&self, chat_id: i64, user_id: i64) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        let puppet_mxid = self.matrix_client.get_user_mxid(user_id).await;
        self.matrix_client
            .leave_room_as(&portal.matrix_room_id, &puppet_mxid)
            .await?;

        Ok(())
    }

    pub async fn handle_telegram_title_change(
        &self,
        chat_id: i64,
        new_title: &str,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_telegram_chat(chat_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        self.matrix_client
            .set_room_name(&portal.matrix_room_id, new_title)
            .await?;

        // Update in-memory portal
        self.portal_manager
            .update_title(chat_id, new_title)
            .await;

        Ok(())
    }

    pub async fn handle_telegram_pin(
        &self,
        _chat_id: i64,
        _message_id: i32,
    ) -> Result<()> {
        // Pin handling would require Matrix room state management
        debug!("Pin handling not yet fully implemented");
        Ok(())
    }

    // ========================================================================
    // Matrix -> Telegram message handlers
    // ========================================================================

    pub async fn handle_matrix_message(
        &self,
        room_id: &str,
        event_id: &str,
        sender: &str,
        content: &str,
    ) -> Result<()> {
        debug!(
            "Handling Matrix message: room={}, event={}, sender={}",
            room_id, event_id, sender
        );

        let portal = match self.portal_manager.get_by_matrix_room(room_id).await {
            Some(p) => p,
            None => {
                debug!("No portal for Matrix room {}, skipping", room_id);
                return Ok(());
            }
        };

        // Format sender name for relay mode
        let display_name = Self::extract_localpart(sender);
        let text = format!("<b>{}</b>: {}", display_name, content);

        let msg_id = self
            .telegram_client
            .send_html_message(portal.telegram_chat_id, &text)
            .await?;

        if let Some(tg_msg_id) = msg_id {
            self.store_message_mapping(
                tg_msg_id as i64,
                portal.telegram_chat_id,
                room_id,
                event_id,
            )
            .await?;
        }

        Ok(())
    }

    /// Handle a media message from Matrix.
    pub async fn handle_matrix_media(
        &self,
        room_id: &str,
        event_id: &str,
        sender: &str,
        content: &serde_json::Value,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_matrix_room(room_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        let mxc_url = content
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("");
        let msgtype = content
            .get("msgtype")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let body = content
            .get("body")
            .and_then(|b| b.as_str())
            .unwrap_or("");

        if mxc_url.is_empty() {
            return Ok(());
        }

        // Download from Matrix
        let display_name = Self::extract_localpart(sender);
        let caption = Some(format!("{}: {}", display_name, body));

        match self.media_handler.transfer_matrix_to_telegram(mxc_url).await {
            Ok((data, _content_type)) => {
                let msg_id = match msgtype {
                    "m.image" => {
                        self.telegram_client
                            .send_photo(
                                portal.telegram_chat_id,
                                data,
                                body,
                                caption.as_deref(),
                            )
                            .await?
                    }
                    "m.video" => {
                        self.telegram_client
                            .send_video(
                                portal.telegram_chat_id,
                                data,
                                body,
                                caption.as_deref(),
                            )
                            .await?
                    }
                    "m.audio" => {
                        self.telegram_client
                            .send_audio(
                                portal.telegram_chat_id,
                                data,
                                body,
                                caption.as_deref(),
                            )
                            .await?
                    }
                    _ => {
                        self.telegram_client
                            .send_document(
                                portal.telegram_chat_id,
                                data,
                                body,
                                caption.as_deref(),
                            )
                            .await?
                    }
                };

                if let Some(tg_msg_id) = msg_id {
                    self.store_message_mapping(
                        tg_msg_id as i64,
                        portal.telegram_chat_id,
                        room_id,
                        event_id,
                    )
                    .await?;
                }
            }
            Err(e) => {
                warn!("Failed to transfer media to Telegram: {}", e);
                // Fallback: send as text with URL
                let text = format!("{}: {} [{}]", display_name, body, mxc_url);
                self.telegram_client
                    .send_message(portal.telegram_chat_id, &text)
                    .await?;
            }
        }

        Ok(())
    }

    /// Handle a location message from Matrix.
    pub async fn handle_matrix_location(
        &self,
        room_id: &str,
        event_id: &str,
        _sender: &str,
        content: &serde_json::Value,
    ) -> Result<()> {
        let portal = match self.portal_manager.get_by_matrix_room(room_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        let geo_uri = content
            .get("geo_uri")
            .and_then(|g| g.as_str())
            .unwrap_or("");
        let coords = geo_uri.strip_prefix("geo:").unwrap_or(geo_uri);
        let parts: Vec<&str> = coords.split(',').collect();
        let lat: f64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let lon: f64 = parts
            .get(1)
            .and_then(|s| s.split(';').next()?.parse().ok())
            .unwrap_or(0.0);

        let msg_id = self
            .telegram_client
            .send_location(portal.telegram_chat_id, lat, lon)
            .await?;

        if let Some(tg_msg_id) = msg_id {
            self.store_message_mapping(
                tg_msg_id as i64,
                portal.telegram_chat_id,
                room_id,
                event_id,
            )
            .await?;
        }

        Ok(())
    }

    /// Handle a Matrix message redaction (deletion).
    pub async fn handle_matrix_redaction(
        &self,
        room_id: &str,
        redacted_event_id: &str,
    ) -> Result<()> {
        if self
            .matrix_client
            .config()
            .bridge
            .disable_deletion_forwarding
        {
            return Ok(());
        }

        let portal = match self.portal_manager.get_by_matrix_room(room_id).await {
            Some(p) => p,
            None => return Ok(()),
        };

        let message_store = self.db_manager.message_store();
        if let Ok(Some(mapping)) = message_store
            .get_by_matrix_event(room_id, redacted_event_id)
            .await
        {
            self.telegram_client
                .delete_message(portal.telegram_chat_id, mapping.telegram_message_id as i32)
                .await?;
            let _ = message_store.delete(mapping.id).await;
        }

        Ok(())
    }

    /// Handle a Matrix join event.
    pub async fn handle_matrix_join(&self, _room_id: &str, _user_id: &str) -> Result<()> {
        debug!("Matrix join event - not forwarded to Telegram");
        Ok(())
    }

    /// Handle a Matrix leave event.
    pub async fn handle_matrix_leave(&self, _room_id: &str, _user_id: &str) -> Result<()> {
        debug!("Matrix leave event - not forwarded to Telegram");
        Ok(())
    }

    /// Handle a Matrix kick event.
    pub async fn handle_matrix_kick(
        &self,
        _room_id: &str,
        _target: &str,
        _kicker: &str,
    ) -> Result<()> {
        debug!("Matrix kick event - not forwarded to Telegram");
        Ok(())
    }

    /// Handle a Matrix ban event.
    pub async fn handle_matrix_ban(
        &self,
        _room_id: &str,
        _target: &str,
        _banner: &str,
    ) -> Result<()> {
        debug!("Matrix ban event - not forwarded to Telegram");
        Ok(())
    }

    /// Handle a Matrix invite event.
    pub async fn handle_matrix_invite(
        &self,
        _room_id: &str,
        _invitee: &str,
        _inviter: &str,
    ) -> Result<()> {
        debug!("Matrix invite event - not forwarded to Telegram");
        Ok(())
    }

    // ========================================================================
    // Portal management
    // ========================================================================

    /// Bridge a Matrix room to a Telegram chat.
    pub async fn bridge_room(
        &self,
        room_id: &str,
        chat_id: i64,
        chat_type: &str,
        title: Option<&str>,
    ) -> Result<()> {
        // Check if already bridged
        if self.portal_manager.get_by_matrix_room(room_id).await.is_some() {
            return Err(anyhow::anyhow!("Room {} is already bridged", room_id));
        }
        if self
            .portal_manager
            .get_by_telegram_chat(chat_id)
            .await
            .is_some()
        {
            return Err(anyhow::anyhow!(
                "Telegram chat {} is already bridged",
                chat_id
            ));
        }

        let portal_info = PortalInfo {
            matrix_room_id: room_id.to_string(),
            telegram_chat_id: chat_id,
            telegram_chat_type: chat_type.to_string(),
            title: title.map(|s| s.to_string()),
        };

        // Store in database
        let db_portal = DbPortalInfo {
            id: 0,
            matrix_room_id: room_id.to_string(),
            telegram_chat_id: chat_id,
            telegram_chat_type: chat_type.to_string(),
            telegram_chat_title: title.map(|s| s.to_string()),
            telegram_chat_username: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.db_manager.portal_store().insert(&db_portal).await?;

        // Add to in-memory cache
        self.portal_manager.add_portal(portal_info).await;

        info!(
            "Bridged room {} to Telegram chat {} ({})",
            room_id, chat_id, chat_type
        );
        Ok(())
    }

    /// Remove the bridge for a Matrix room.
    pub async fn unbridge_room(&self, room_id: &str) -> Result<()> {
        if let Some(_portal) = self.portal_manager.get_by_matrix_room(room_id).await {
            // Remove from database
            let db_store = self.db_manager.portal_store();
            if let Ok(Some(db_portal)) = db_store.get_by_matrix_room(room_id).await {
                let _ = db_store.delete(db_portal.id).await;
            }

            // Remove from in-memory cache
            self.portal_manager.remove_portal(room_id).await;

            info!("Unbridged room {}", room_id);
        }
        Ok(())
    }

    // ========================================================================
    // Puppet management
    // ========================================================================

    /// Ensure a puppet user exists in Matrix for a Telegram user.
    async fn ensure_puppet(&self, telegram_user_id: i64) -> Result<()> {
        if self
            .puppet_manager
            .get_by_telegram_id(telegram_user_id)
            .await
            .is_some()
        {
            return Ok(());
        }

        let mxid = self.matrix_client.get_user_mxid(telegram_user_id).await;

        // Extract localpart for registration
        let localpart = mxid
            .strip_prefix('@')
            .and_then(|s| s.split(':').next())
            .unwrap_or(&mxid);

        // Register the puppet on the homeserver
        if let Err(e) = self.matrix_client.ensure_registered(localpart).await {
            warn!("Failed to register puppet {}: {}", mxid, e);
        }

        let puppet = PuppetInfo {
            telegram_user_id,
            matrix_mxid: mxid,
            displayname: None,
            avatar_mxc: None,
        };
        self.puppet_manager.add_puppet(puppet).await;

        Ok(())
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Store a message mapping in the database.
    async fn store_message_mapping(
        &self,
        telegram_message_id: i64,
        telegram_chat_id: i64,
        matrix_room_id: &str,
        matrix_event_id: &str,
    ) -> Result<()> {
        let mapping = MessageMapping {
            id: 0,
            telegram_message_id,
            telegram_chat_id,
            matrix_room_id: matrix_room_id.to_string(),
            matrix_event_id: matrix_event_id.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        if let Err(e) = self.db_manager.message_store().insert(&mapping).await {
            warn!("Failed to store message mapping: {}", e);
        }

        Ok(())
    }

    /// Extract localpart from a Matrix user ID (@user:domain -> user).
    fn extract_localpart(mxid: &str) -> &str {
        mxid.strip_prefix('@')
            .and_then(|s| s.split(':').next())
            .unwrap_or(mxid)
    }
}

// ============================================================================
// Portal and Puppet managers (in-memory with DB backing)
// ============================================================================

pub struct PortalManager {
    portals: RwLock<Vec<PortalInfo>>,
}

#[derive(Debug, Clone)]
pub struct PortalInfo {
    pub matrix_room_id: String,
    pub telegram_chat_id: i64,
    pub telegram_chat_type: String,
    pub title: Option<String>,
}

impl PortalManager {
    pub fn new() -> Self {
        Self {
            portals: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_by_matrix_room(&self, room_id: &str) -> Option<PortalInfo> {
        let portals = self.portals.read().await;
        portals
            .iter()
            .find(|p| p.matrix_room_id == room_id)
            .cloned()
    }

    pub async fn get_by_telegram_chat(&self, chat_id: i64) -> Option<PortalInfo> {
        let portals = self.portals.read().await;
        portals
            .iter()
            .find(|p| p.telegram_chat_id == chat_id)
            .cloned()
    }

    pub async fn add_portal(&self, portal: PortalInfo) {
        let mut portals = self.portals.write().await;
        // Avoid duplicates
        portals.retain(|p| {
            p.matrix_room_id != portal.matrix_room_id
                && p.telegram_chat_id != portal.telegram_chat_id
        });
        portals.push(portal);
    }

    pub async fn remove_portal(&self, matrix_room_id: &str) {
        let mut portals = self.portals.write().await;
        portals.retain(|p| p.matrix_room_id != matrix_room_id);
    }

    pub async fn update_title(&self, chat_id: i64, title: &str) {
        let mut portals = self.portals.write().await;
        if let Some(portal) = portals
            .iter_mut()
            .find(|p| p.telegram_chat_id == chat_id)
        {
            portal.title = Some(title.to_string());
        }
    }

    pub async fn list_all(&self) -> Vec<PortalInfo> {
        let portals = self.portals.read().await;
        portals.clone()
    }

    pub async fn count(&self) -> usize {
        let portals = self.portals.read().await;
        portals.len()
    }
}

impl Default for PortalManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PuppetManager {
    puppets: RwLock<Vec<PuppetInfo>>,
}

#[derive(Debug, Clone)]
pub struct PuppetInfo {
    pub telegram_user_id: i64,
    pub matrix_mxid: String,
    pub displayname: Option<String>,
    pub avatar_mxc: Option<String>,
}

impl PuppetManager {
    pub fn new() -> Self {
        Self {
            puppets: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_by_telegram_id(&self, telegram_id: i64) -> Option<PuppetInfo> {
        let puppets = self.puppets.read().await;
        puppets
            .iter()
            .find(|p| p.telegram_user_id == telegram_id)
            .cloned()
    }

    pub async fn get_by_matrix_id(&self, mxid: &str) -> Option<PuppetInfo> {
        let puppets = self.puppets.read().await;
        puppets
            .iter()
            .find(|p| p.matrix_mxid == mxid)
            .cloned()
    }

    pub async fn add_puppet(&self, puppet: PuppetInfo) {
        let mut puppets = self.puppets.write().await;
        puppets.retain(|p| p.telegram_user_id != puppet.telegram_user_id);
        puppets.push(puppet);
    }

    pub async fn update_displayname(&self, telegram_id: i64, displayname: &str) {
        let mut puppets = self.puppets.write().await;
        if let Some(puppet) = puppets
            .iter_mut()
            .find(|p| p.telegram_user_id == telegram_id)
        {
            puppet.displayname = Some(displayname.to_string());
        }
    }

    pub async fn update_avatar(&self, telegram_id: i64, avatar_mxc: &str) {
        let mut puppets = self.puppets.write().await;
        if let Some(puppet) = puppets
            .iter_mut()
            .find(|p| p.telegram_user_id == telegram_id)
        {
            puppet.avatar_mxc = Some(avatar_mxc.to_string());
        }
    }

    pub async fn list_all(&self) -> Vec<PuppetInfo> {
        let puppets = self.puppets.read().await;
        puppets.clone()
    }

    pub async fn count(&self) -> usize {
        let puppets = self.puppets.read().await;
        puppets.len()
    }
}

impl Default for PuppetManager {
    fn default() -> Self {
        Self::new()
    }
}

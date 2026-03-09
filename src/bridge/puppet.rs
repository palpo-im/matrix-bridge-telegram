use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, info};

use crate::bridge::{BridgeCore, PuppetInfo};
use crate::telegram::client::TelegramApiHelper;

/// Helper for puppet lifecycle: register, update display info, sync avatars.
pub struct PuppetManagerHelper;

impl PuppetManagerHelper {
    pub fn new() -> Self {
        Self
    }

    /// Create and register a puppet user on the Matrix homeserver.
    pub async fn create_puppet(
        bridge: &BridgeCore,
        telegram_user_id: i64,
        displayname: Option<&str>,
    ) -> Result<PuppetInfo> {
        let mxid = bridge
            .matrix_client()
            .get_user_mxid(telegram_user_id)
            .await;

        info!(
            "Creating puppet: Telegram {} -> Matrix {}",
            telegram_user_id, mxid
        );

        // Extract localpart for registration
        let localpart = mxid
            .strip_prefix('@')
            .and_then(|s| s.split(':').next())
            .unwrap_or(&mxid);

        // Register the user on the homeserver
        bridge
            .matrix_client()
            .ensure_registered(localpart)
            .await?;

        // Set display name if provided
        if let Some(name) = displayname {
            let formatted_name = bridge
                .matrix_client()
                .config()
                .portal
                .displayname_template
                .replace("{displayname}", name);
            bridge
                .matrix_client()
                .set_displayname(&mxid, &formatted_name)
                .await?;
        }

        let puppet = PuppetInfo {
            telegram_user_id,
            matrix_mxid: mxid,
            displayname: displayname.map(|s| s.to_string()),
            avatar_mxc: None,
        };

        bridge.puppet_manager().add_puppet(puppet.clone()).await;

        Ok(puppet)
    }

    /// Update the displayname of an existing puppet.
    pub async fn update_displayname(
        bridge: &BridgeCore,
        telegram_user_id: i64,
        displayname: &str,
    ) -> Result<()> {
        let mxid = bridge
            .matrix_client()
            .get_user_mxid(telegram_user_id)
            .await;

        let formatted = bridge
            .matrix_client()
            .config()
            .portal
            .displayname_template
            .replace("{displayname}", displayname);

        bridge
            .matrix_client()
            .set_displayname(&mxid, &formatted)
            .await?;

        bridge
            .puppet_manager()
            .update_displayname(telegram_user_id, displayname)
            .await;

        info!(
            "Updated puppet displayname for Telegram {} to '{}'",
            telegram_user_id, displayname
        );
        Ok(())
    }

    /// Sync a puppet's avatar from Telegram to Matrix.
    pub async fn sync_avatar(
        bridge: &BridgeCore,
        telegram_user_id: i64,
    ) -> Result<()> {
        let bot_token = match bridge.telegram_client().bot_token() {
            Some(t) => t.to_string(),
            None => return Ok(()),
        };

        let api = TelegramApiHelper::new(&bot_token);

        // Get profile photos
        let photos = api
            .get_user_profile_photos(telegram_user_id)
            .await?;

        let photo_file_id = photos
            .get("result")
            .and_then(|r| r.get("photos"))
            .and_then(|p| p.as_array())
            .and_then(|arr| arr.first())
            .and_then(|p| p.as_array())
            .and_then(|sizes| sizes.last())
            .and_then(|s| s.get("file_id"))
            .and_then(|f| f.as_str());

        if let Some(file_id) = photo_file_id {
            let _file_unique_id = photos
                .get("result")
                .and_then(|r| r.get("photos"))
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|p| p.as_array())
                .and_then(|sizes| sizes.last())
                .and_then(|s| s.get("file_unique_id"))
                .and_then(|f| f.as_str())
                .unwrap_or(file_id);

            // Download and upload the avatar
            let data = api.download_file_by_id(file_id).await?;
            let config = Arc::new(bridge.matrix_client().config().clone());
            let media = crate::media::MediaHandler::new(config);
            let mxc_url = media
                .upload_to_matrix(&data, "image/jpeg", Some("avatar.jpg"))
                .await?;

            let mxid = bridge
                .matrix_client()
                .get_user_mxid(telegram_user_id)
                .await;

            bridge
                .matrix_client()
                .set_avatar_url(&mxid, &mxc_url)
                .await?;

            bridge
                .puppet_manager()
                .update_avatar(telegram_user_id, &mxc_url)
                .await;

            info!(
                "Updated avatar for Telegram user {} to {}",
                telegram_user_id, mxc_url
            );
        } else {
            debug!(
                "No profile photo for Telegram user {}",
                telegram_user_id
            );
        }

        Ok(())
    }

    /// Ensure a puppet exists and join it to a room.
    pub async fn ensure_in_room(
        bridge: &BridgeCore,
        telegram_user_id: i64,
        room_id: &str,
    ) -> Result<()> {
        // Create puppet if doesn't exist
        if bridge
            .puppet_manager()
            .get_by_telegram_id(telegram_user_id)
            .await
            .is_none()
        {
            Self::create_puppet(bridge, telegram_user_id, None).await?;
        }

        let mxid = bridge
            .matrix_client()
            .get_user_mxid(telegram_user_id)
            .await;

        // Join the room
        if let Err(e) = bridge
            .matrix_client()
            .join_room_as(room_id, &mxid)
            .await
        {
            // May already be in the room
            debug!("Join room attempt for {}: {}", mxid, e);
        }

        Ok(())
    }
}

impl Default for PuppetManagerHelper {
    fn default() -> Self {
        Self::new()
    }
}

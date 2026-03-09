use anyhow::Result;
use chrono::Utc;
use tracing::{debug, info, warn};

use crate::bridge::{BridgeCore, PortalInfo};
use crate::db::models::PortalInfo as DbPortalInfo;

/// Helper for portal lifecycle management: create, update, delete portals.
pub struct PortalManagerHelper;

impl PortalManagerHelper {
    pub fn new() -> Self {
        Self
    }

    /// Create a new portal bridging a Matrix room to a Telegram chat.
    pub async fn create_portal(
        bridge: &BridgeCore,
        matrix_room_id: &str,
        telegram_chat_id: i64,
        telegram_chat_type: &str,
        title: Option<&str>,
    ) -> Result<PortalInfo> {
        info!(
            "Creating portal: Matrix {} <-> Telegram {} ({})",
            matrix_room_id, telegram_chat_id, telegram_chat_type
        );

        let portal = PortalInfo {
            matrix_room_id: matrix_room_id.to_string(),
            telegram_chat_id,
            telegram_chat_type: telegram_chat_type.to_string(),
            title: title.map(|s| s.to_string()),
        };

        // Persist to database
        let db_portal = DbPortalInfo {
            id: 0,
            matrix_room_id: matrix_room_id.to_string(),
            telegram_chat_id,
            telegram_chat_type: telegram_chat_type.to_string(),
            telegram_chat_title: title.map(|s| s.to_string()),
            telegram_chat_username: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        bridge.db_manager().portal_store().insert(&db_portal).await?;

        // Add to in-memory cache
        bridge.portal_manager().add_portal(portal.clone()).await;

        Ok(portal)
    }

    /// Create a portal by auto-creating a Matrix room for a Telegram chat.
    pub async fn create_portal_with_room(
        bridge: &BridgeCore,
        telegram_chat_id: i64,
        telegram_chat_type: &str,
        title: Option<&str>,
        is_direct: bool,
    ) -> Result<PortalInfo> {
        info!(
            "Creating Matrix room for Telegram chat {} ({})",
            telegram_chat_id, telegram_chat_type
        );

        // Generate room alias
        let alias_name = if let Some(t) = title {
            let clean: String = t
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            Some(format!(
                "telegram_{}",
                if clean.is_empty() {
                    telegram_chat_id.to_string()
                } else {
                    clean
                }
            ))
        } else {
            None
        };

        // Create the Matrix room
        let room_id = bridge
            .matrix_client()
            .create_room(
                title,
                alias_name.as_deref(),
                None,
                &[],
                is_direct,
            )
            .await?;

        Self::create_portal(
            bridge,
            &room_id,
            telegram_chat_id,
            telegram_chat_type,
            title,
        )
        .await
    }

    /// Update portal metadata (title, etc.)
    pub async fn update_portal_title(
        bridge: &BridgeCore,
        telegram_chat_id: i64,
        new_title: &str,
    ) -> Result<()> {
        if let Some(portal) = bridge
            .portal_manager()
            .get_by_telegram_chat(telegram_chat_id)
            .await
        {
            // Update Matrix room name
            bridge
                .matrix_client()
                .set_room_name(&portal.matrix_room_id, new_title)
                .await?;

            // Update in-memory
            bridge
                .portal_manager()
                .update_title(telegram_chat_id, new_title)
                .await;

            // Update in database
            let store = bridge.db_manager().portal_store();
            if let Ok(Some(mut db_portal)) = store
                .get_by_telegram_chat(telegram_chat_id)
                .await
            {
                db_portal.telegram_chat_title = Some(new_title.to_string());
                db_portal.updated_at = Utc::now();
                let _ = store.update(&db_portal).await;
            }

            info!(
                "Updated portal title for Telegram chat {} to '{}'",
                telegram_chat_id, new_title
            );
        }

        Ok(())
    }

    /// Delete a portal and optionally leave the Matrix room.
    pub async fn delete_portal(
        bridge: &BridgeCore,
        matrix_room_id: &str,
        leave_room: bool,
    ) -> Result<()> {
        info!("Deleting portal for room {}", matrix_room_id);

        // Remove from database
        let store = bridge.db_manager().portal_store();
        if let Ok(Some(db_portal)) = store.get_by_matrix_room(matrix_room_id).await {
            let _ = store.delete(db_portal.id).await;
        }

        // Remove from in-memory cache
        bridge.portal_manager().remove_portal(matrix_room_id).await;

        // Optionally leave the Matrix room
        if leave_room {
            // The bridge bot leaves the room
            let bot_mxid = format!(
                "@{}:{}",
                bridge.matrix_client().config().registration.sender_localpart,
                bridge.matrix_client().config().bridge.domain
            );
            if let Err(e) = bridge
                .matrix_client()
                .leave_room_as(matrix_room_id, &bot_mxid)
                .await
            {
                warn!("Failed to leave room {}: {}", matrix_room_id, e);
            }
        }

        Ok(())
    }

    /// Sync Telegram chat members into the Matrix room as puppets.
    pub async fn sync_members(
        bridge: &BridgeCore,
        portal: &PortalInfo,
    ) -> Result<()> {
        let max_sync = bridge
            .matrix_client()
            .config()
            .portal
            .max_initial_member_sync;

        if max_sync <= 0 {
            return Ok(());
        }

        let member_count = bridge
            .telegram_client()
            .get_chat_member_count(portal.telegram_chat_id)
            .await?;

        debug!(
            "Telegram chat {} has {} members (syncing up to {})",
            portal.telegram_chat_id, member_count, max_sync
        );

        // For bot API, we can't list all members easily.
        // Members will be lazily created as they send messages.
        Ok(())
    }
}

impl Default for PortalManagerHelper {
    fn default() -> Self {
        Self::new()
    }
}

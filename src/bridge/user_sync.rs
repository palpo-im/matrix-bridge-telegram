use anyhow::Result;
use chrono::Utc;
use tracing::{debug, info};

use crate::bridge::BridgeCore;
use crate::db::models::UserMapping;

/// Synchronizes user mappings between Matrix and Telegram.
pub struct UserSync;

impl UserSync {
    pub fn new() -> Self {
        Self
    }

    /// Sync a single user mapping: store the association between a Telegram user and a Matrix user.
    pub async fn sync_user(
        &self,
        bridge: &BridgeCore,
        telegram_user_id: i64,
        matrix_mxid: &str,
        username: Option<&str>,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<()> {
        info!(
            "Syncing user: Telegram {} -> Matrix {}",
            telegram_user_id, matrix_mxid
        );

        let user_store = bridge.db_manager().user_store();

        // Check if mapping already exists
        if let Ok(Some(mut existing)) = user_store.get_by_telegram_id(telegram_user_id).await {
            // Update if needed
            let mut updated = false;
            if existing.matrix_user_id != matrix_mxid {
                existing.matrix_user_id = matrix_mxid.to_string();
                updated = true;
            }
            if username.is_some() && existing.telegram_username != username.map(|s| s.to_string()) {
                existing.telegram_username = username.map(|s| s.to_string());
                updated = true;
            }
            if first_name.is_some()
                && existing.telegram_first_name != first_name.map(|s| s.to_string())
            {
                existing.telegram_first_name = first_name.map(|s| s.to_string());
                updated = true;
            }
            if last_name.is_some()
                && existing.telegram_last_name != last_name.map(|s| s.to_string())
            {
                existing.telegram_last_name = last_name.map(|s| s.to_string());
                updated = true;
            }

            if updated {
                existing.updated_at = Utc::now();
                user_store.update(&existing).await?;
                debug!("Updated user mapping for Telegram {}", telegram_user_id);
            }
        } else {
            // Create new mapping
            let mapping = UserMapping {
                id: 0,
                matrix_user_id: matrix_mxid.to_string(),
                telegram_user_id,
                telegram_username: username.map(|s| s.to_string()),
                telegram_first_name: first_name.map(|s| s.to_string()),
                telegram_last_name: last_name.map(|s| s.to_string()),
                telegram_phone: None,
                telegram_avatar: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            user_store.insert(&mapping).await?;
            info!("Created user mapping for Telegram {}", telegram_user_id);
        }

        Ok(())
    }

    /// Look up a Matrix user ID by Telegram user ID.
    pub async fn get_matrix_id(
        bridge: &BridgeCore,
        telegram_user_id: i64,
    ) -> Result<Option<String>> {
        let user_store = bridge.db_manager().user_store();
        match user_store.get_by_telegram_id(telegram_user_id).await? {
            Some(mapping) => Ok(Some(mapping.matrix_user_id)),
            None => Ok(None),
        }
    }

    /// Look up a Telegram user ID by Matrix user ID.
    pub async fn get_telegram_id(
        bridge: &BridgeCore,
        matrix_mxid: &str,
    ) -> Result<Option<i64>> {
        let user_store = bridge.db_manager().user_store();
        match user_store.get_by_matrix_id(matrix_mxid).await? {
            Some(mapping) => Ok(Some(mapping.telegram_user_id)),
            None => Ok(None),
        }
    }

    /// Sync all known users from the database, updating puppets as needed.
    pub async fn sync_all_users(&self, _bridge: &BridgeCore) -> Result<u32> {
        info!("Starting full user sync");
        // In a real implementation, this would iterate all user mappings
        // and ensure puppet profiles are up to date.
        // With the bot API, we can't proactively fetch all users,
        // so this is more of a placeholder for reconciliation.
        Ok(0)
    }
}

impl Default for UserSync {
    fn default() -> Self {
        Self::new()
    }
}

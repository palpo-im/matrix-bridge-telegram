use anyhow::Result;
use tracing::{debug, info, warn};

use crate::bridge::BridgeCore;

/// Handles backfilling historical messages from Telegram to Matrix.
pub struct BackfillManager {
    max_messages: u32,
    enabled: bool,
}

impl BackfillManager {
    pub fn new(max_messages: u32) -> Self {
        Self {
            max_messages,
            enabled: max_messages > 0,
        }
    }

    /// Check if backfill is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Backfill messages for a portal.
    /// Note: The Telegram Bot API has limited support for fetching message history.
    /// Full backfill requires the user/client API (MTProto), which is not available
    /// through the bot API. This implementation handles what's possible with the bot API.
    pub async fn backfill_portal(
        &self,
        _bridge: &BridgeCore,
        telegram_chat_id: i64,
        matrix_room_id: &str,
    ) -> Result<u32> {
        if !self.enabled {
            debug!("Backfill is disabled");
            return Ok(0);
        }

        info!(
            "Starting backfill for Telegram chat {} -> Matrix room {} (max: {} messages)",
            telegram_chat_id, matrix_room_id, self.max_messages
        );

        // The Telegram Bot API does not support fetching message history.
        // Backfill with the bot API is limited to:
        // 1. Forwarded messages that the bot receives
        // 2. Messages in channels where the bot is an admin
        //
        // For full backfill support, a user/client API (MTProto) connection
        // would be needed, similar to mautrix-telegram's approach.
        //
        // Currently, we only log that backfill was requested and return 0.
        warn!(
            "Backfill requested for chat {}, but Bot API has limited history access. \
             Messages will be bridged as they arrive in real-time.",
            telegram_chat_id
        );

        Ok(0)
    }

    /// Mark a portal as having been backfilled.
    pub async fn mark_backfilled(
        &self,
        _bridge: &BridgeCore,
        _telegram_chat_id: i64,
    ) -> Result<()> {
        // In a full implementation, this would store backfill state in the database
        // to avoid re-backfilling on restart.
        Ok(())
    }
}

impl Default for BackfillManager {
    fn default() -> Self {
        Self::new(0)
    }
}

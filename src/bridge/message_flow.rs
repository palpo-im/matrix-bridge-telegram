use crate::bridge::{BridgeCore, PortalInfo};

pub struct MessageFlow;

impl MessageFlow {
    pub fn new() -> Self {
        Self
    }

    pub async fn telegram_to_matrix(
        &self,
        _bridge: &BridgeCore,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        text: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Forwarding Telegram message {}:{} from {} to Matrix: {}",
            chat_id,
            message_id,
            sender_id,
            text
        );
        Ok(())
    }

    pub async fn matrix_to_telegram(
        &self,
        _bridge: &BridgeCore,
        room_id: &str,
        event_id: &str,
        sender: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(
            "Forwarding Matrix message {}:{} from {} to Telegram: {}",
            room_id,
            event_id,
            sender,
            text
        );
        Ok(())
    }
}

impl Default for MessageFlow {
    fn default() -> Self {
        Self::new()
    }
}

use anyhow::Result;
use tracing::debug;

use crate::bridge::BridgeCore;
use crate::parsers::matrix_parser::MatrixParser;
use crate::parsers::telegram_parser::TelegramParser;

/// Orchestrates bidirectional message forwarding between Telegram and Matrix.
pub struct MessageFlow;

impl MessageFlow {
    pub fn new() -> Self {
        Self
    }

    /// Forward a Telegram message to Matrix, handling formatting conversion.
    pub async fn telegram_to_matrix(
        &self,
        bridge: &BridgeCore,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        text: &str,
    ) -> Result<()> {
        debug!(
            "Forwarding Telegram message {}:{} from {} to Matrix",
            chat_id, message_id, sender_id
        );

        bridge
            .handle_telegram_message(chat_id, message_id, sender_id, text)
            .await
    }

    /// Forward a Matrix message to Telegram, handling formatting conversion.
    pub async fn matrix_to_telegram(
        &self,
        bridge: &BridgeCore,
        room_id: &str,
        event_id: &str,
        sender: &str,
        text: &str,
    ) -> Result<()> {
        debug!(
            "Forwarding Matrix message {}:{} from {} to Telegram",
            room_id, event_id, sender
        );

        bridge
            .handle_matrix_message(room_id, event_id, sender, text)
            .await
    }

    /// Process a Telegram message with full entity/formatting conversion.
    pub fn convert_telegram_to_matrix_content(
        raw_message: &serde_json::Value,
    ) -> Option<serde_json::Value> {
        let common = TelegramParser::parse_telegram_message(raw_message)?;
        Some(MatrixParser::to_matrix_content(&common))
    }

    /// Convert a Matrix event to Telegram text format.
    pub fn convert_matrix_to_telegram_text(event: &serde_json::Value) -> Option<String> {
        let common = MatrixParser::parse_matrix_event(event)?;
        Some(MatrixParser::matrix_to_telegram(&common.content))
    }
}

impl Default for MessageFlow {
    fn default() -> Self {
        Self::new()
    }
}

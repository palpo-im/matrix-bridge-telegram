use crate::bridge::PortalInfo;

pub struct PortalManagerHelper;

impl PortalManagerHelper {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_portal(
        &self,
        matrix_room_id: &str,
        telegram_chat_id: i64,
        telegram_chat_type: &str,
    ) -> anyhow::Result<PortalInfo> {
        tracing::info!(
            "Creating portal: Matrix {} <-> Telegram {} ({})",
            matrix_room_id,
            telegram_chat_id,
            telegram_chat_type
        );
        Ok(PortalInfo {
            matrix_room_id: matrix_room_id.to_string(),
            telegram_chat_id,
            telegram_chat_type: telegram_chat_type.to_string(),
            title: None,
        })
    }

    pub async fn delete_portal(&self, matrix_room_id: &str) -> anyhow::Result<()> {
        tracing::info!("Deleting portal: {}", matrix_room_id);
        Ok(())
    }
}

impl Default for PortalManagerHelper {
    fn default() -> Self {
        Self::new()
    }
}

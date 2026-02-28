pub struct UserSync;

impl UserSync {
    pub fn new() -> Self {
        Self
    }

    pub async fn sync_user(&self, telegram_user_id: i64, matrix_mxid: &str) -> anyhow::Result<()> {
        tracing::info!(
            "Syncing user: Telegram {} -> Matrix {}",
            telegram_user_id,
            matrix_mxid
        );
        Ok(())
    }

    pub async fn sync_all_users(&self) -> anyhow::Result<()> {
        tracing::info!("Syncing all users");
        Ok(())
    }
}

impl Default for UserSync {
    fn default() -> Self {
        Self::new()
    }
}

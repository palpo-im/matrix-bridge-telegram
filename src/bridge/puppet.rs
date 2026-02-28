use crate::bridge::PuppetInfo;

pub struct PuppetManagerHelper;

impl PuppetManagerHelper {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_puppet(
        &self,
        telegram_user_id: i64,
        matrix_mxid: &str,
        displayname: Option<&str>,
    ) -> anyhow::Result<PuppetInfo> {
        tracing::info!(
            "Creating puppet: Telegram {} -> Matrix {}",
            telegram_user_id,
            matrix_mxid
        );
        Ok(PuppetInfo {
            telegram_user_id,
            matrix_mxid: matrix_mxid.to_string(),
            displayname: displayname.map(|s| s.to_string()),
            avatar_mxc: None,
        })
    }

    pub async fn sync_puppet_info(&self, telegram_user_id: i64) -> anyhow::Result<()> {
        tracing::info!("Syncing puppet info for Telegram user {}", telegram_user_id);
        Ok(())
    }
}

impl Default for PuppetManagerHelper {
    fn default() -> Self {
        Self::new()
    }
}

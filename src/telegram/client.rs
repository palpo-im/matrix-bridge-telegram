use reqwest::Client;
use tracing::debug;

/// Additional Telegram API helper methods beyond what teloxide provides.
pub struct TelegramApiHelper {
    bot_token: String,
    http_client: Client,
}

impl TelegramApiHelper {
    pub fn new(bot_token: &str) -> Self {
        Self {
            bot_token: bot_token.to_string(),
            http_client: Client::new(),
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }

    /// Get file download path for a file_id.
    pub async fn get_file_path(&self, file_id: &str) -> anyhow::Result<String> {
        let url = self.api_url("getFile");
        let body = serde_json::json!({ "file_id": file_id });

        let response: serde_json::Value = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let file_path = response
            .get("result")
            .and_then(|r| r.get("file_path"))
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("no file_path in response"))?;

        Ok(file_path.to_string())
    }

    /// Download a file by its file_path.
    pub async fn download_file(&self, file_path: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            self.bot_token, file_path
        );
        debug!("Downloading file from {}", url);
        let bytes = self.http_client.get(&url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Download a file by its file_id (combines get_file_path + download_file).
    pub async fn download_file_by_id(&self, file_id: &str) -> anyhow::Result<Vec<u8>> {
        let path = self.get_file_path(file_id).await?;
        self.download_file(&path).await
    }

    /// Get basic user profile info.
    pub async fn get_user_profile_photos(
        &self,
        user_id: i64,
    ) -> anyhow::Result<serde_json::Value> {
        let url = self.api_url("getUserProfilePhotos");
        let body = serde_json::json!({
            "user_id": user_id,
            "limit": 1
        });

        let response: serde_json::Value = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    /// Get chat administrators.
    pub async fn get_chat_administrators(
        &self,
        chat_id: i64,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let url = self.api_url("getChatAdministrators");
        let body = serde_json::json!({ "chat_id": chat_id });

        let response: serde_json::Value = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let admins = response
            .get("result")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(admins)
    }

    /// Ban a user from a chat.
    pub async fn ban_chat_member(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> anyhow::Result<()> {
        let url = self.api_url("banChatMember");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "user_id": user_id
        });

        self.http_client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Unban a user from a chat.
    pub async fn unban_chat_member(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> anyhow::Result<()> {
        let url = self.api_url("unbanChatMember");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "user_id": user_id,
            "only_if_banned": true
        });

        self.http_client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Pin a message in a chat.
    pub async fn pin_chat_message(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> anyhow::Result<()> {
        let url = self.api_url("pinChatMessage");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "disable_notification": true
        });

        self.http_client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Set chat title.
    pub async fn set_chat_title(
        &self,
        chat_id: i64,
        title: &str,
    ) -> anyhow::Result<()> {
        let url = self.api_url("setChatTitle");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "title": title
        });

        self.http_client.post(&url).json(&body).send().await?;
        Ok(())
    }

    /// Set chat description.
    pub async fn set_chat_description(
        &self,
        chat_id: i64,
        description: &str,
    ) -> anyhow::Result<()> {
        let url = self.api_url("setChatDescription");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "description": description
        });

        self.http_client.post(&url).json(&body).send().await?;
        Ok(())
    }
}

use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::db::{TelegramFileInfo, TelegramFileStore};

/// Handles media transfer between Matrix and Telegram.
pub struct MediaHandler {
    config: Arc<Config>,
    http_client: Client,
    file_store: Option<Arc<dyn TelegramFileStore>>,
}

impl MediaHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            http_client: Client::new(),
            file_store: None,
        }
    }

    pub fn with_file_store(mut self, store: Arc<dyn TelegramFileStore>) -> Self {
        self.file_store = Some(store);
        self
    }

    /// Download media from Matrix homeserver given an mxc:// URL.
    pub async fn download_matrix_media(&self, mxc_url: &str) -> Result<(Vec<u8>, String)> {
        let homeserver_url = &self.config.bridge.homeserver_url;

        // Parse mxc://server/media_id
        let path = mxc_url
            .strip_prefix("mxc://")
            .ok_or_else(|| anyhow::anyhow!("invalid mxc URL: {}", mxc_url))?;
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("invalid mxc URL format: {}", mxc_url));
        }
        let server_name = parts[0];
        let media_id = parts[1];

        // Use the /_matrix/media/v3/download endpoint
        let download_url = format!(
            "{}/_matrix/media/v3/download/{}/{}",
            homeserver_url, server_name, media_id
        );

        debug!("Downloading Matrix media from {}", download_url);

        let response = self
            .http_client
            .get(&download_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.registration.appservice_token),
            )
            .send()
            .await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = response.bytes().await?.to_vec();
        info!(
            "Downloaded {} bytes of {} from Matrix",
            bytes.len(),
            content_type
        );

        Ok((bytes, content_type))
    }

    /// Upload media to the Matrix homeserver, returns an mxc:// URL.
    pub async fn upload_to_matrix(
        &self,
        data: &[u8],
        content_type: &str,
        filename: Option<&str>,
    ) -> Result<String> {
        let homeserver_url = &self.config.bridge.homeserver_url;

        let mut url = format!("{}/_matrix/media/v3/upload", homeserver_url);
        if let Some(name) = filename {
            url.push_str(&format!("?filename={}", urlencoding::encode(name)));
        }

        debug!("Uploading {} bytes to Matrix as {}", data.len(), content_type);

        let response = self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.registration.appservice_token),
            )
            .header("Content-Type", content_type)
            .body(data.to_vec())
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        let mxc_url = body
            .get("content_uri")
            .and_then(|u| u.as_str())
            .ok_or_else(|| anyhow::anyhow!("no content_uri in upload response"))?
            .to_string();

        info!("Uploaded media to Matrix: {}", mxc_url);
        Ok(mxc_url)
    }

    /// Download a file from Telegram using the Bot API.
    pub async fn download_telegram_file(
        &self,
        bot_token: &str,
        file_id: &str,
    ) -> Result<(Vec<u8>, String)> {
        // First, get the file path from Telegram
        let get_file_url = format!(
            "https://api.telegram.org/bot{}/getFile?file_id={}",
            bot_token, file_id
        );

        let response: serde_json::Value = self
            .http_client
            .get(&get_file_url)
            .send()
            .await?
            .json()
            .await?;

        let file_path = response
            .get("result")
            .and_then(|r| r.get("file_path"))
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("no file_path in Telegram response"))?;

        let file_size = response
            .get("result")
            .and_then(|r| r.get("file_size"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0);

        // Download the actual file
        let download_url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            bot_token, file_path
        );

        debug!("Downloading Telegram file from {}", download_url);

        let response = self.http_client.get(&download_url).send().await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = response.bytes().await?.to_vec();
        info!(
            "Downloaded {} bytes from Telegram (expected {})",
            bytes.len(),
            file_size
        );

        Ok((bytes, content_type))
    }

    /// Transfer a Telegram file to Matrix: download from TG, upload to Matrix, return mxc URL.
    /// Uses cache if available.
    pub async fn transfer_telegram_to_matrix(
        &self,
        bot_token: &str,
        file_id: &str,
        file_unique_id: &str,
        filename: Option<&str>,
    ) -> Result<String> {
        // Check cache first
        if let Some(ref store) = self.file_store {
            if let Ok(Some(cached)) = store.get_by_telegram_id(file_unique_id).await {
                debug!("Using cached mxc URL for file {}", file_unique_id);
                return Ok(cached.mxc_url);
            }
        }

        // Download from Telegram
        let (data, content_type) = self.download_telegram_file(bot_token, file_id).await?;

        // Upload to Matrix
        let mxc_url = self.upload_to_matrix(&data, &content_type, filename).await?;

        // Cache the result
        if let Some(ref store) = self.file_store {
            let file_info = TelegramFileInfo {
                id: 0,
                telegram_file_id: file_id.to_string(),
                telegram_file_unique_id: file_unique_id.to_string(),
                mxc_url: mxc_url.clone(),
                mime_type: Some(content_type),
                file_name: filename.map(|s| s.to_string()),
                file_size: Some(data.len() as i64),
                created_at: chrono::Utc::now(),
            };
            if let Err(e) = store.insert(&file_info).await {
                warn!("Failed to cache file info: {}", e);
            }
        }

        Ok(mxc_url)
    }

    /// Transfer a Matrix file to Telegram: download from Matrix, return bytes + content type.
    pub async fn transfer_matrix_to_telegram(
        &self,
        mxc_url: &str,
    ) -> Result<(Vec<u8>, String)> {
        self.download_matrix_media(mxc_url).await
    }

    /// Guess a filename from content type.
    pub fn filename_from_content_type(content_type: &str) -> &str {
        match content_type {
            "image/jpeg" => "image.jpg",
            "image/png" => "image.png",
            "image/gif" => "image.gif",
            "image/webp" => "image.webp",
            "video/mp4" => "video.mp4",
            "video/webm" => "video.webm",
            "audio/ogg" => "audio.ogg",
            "audio/mpeg" => "audio.mp3",
            "audio/mp4" => "audio.m4a",
            _ => "file",
        }
    }
}

/// URL-encode helper (minimal implementation to avoid extra dependency).
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}

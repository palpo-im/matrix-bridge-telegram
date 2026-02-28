use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;

pub struct MediaHandler {
    config: Arc<Config>,
}

impl MediaHandler {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn download_matrix_media(&self, _mxc_url: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    pub async fn upload_to_matrix(&self, _data: &[u8], _content_type: &str) -> Result<String> {
        Ok(String::new())
    }

    pub async fn download_telegram_media(&self, _file_id: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    pub async fn upload_to_telegram(&self, _data: &[u8], _filename: &str) -> Result<String> {
        Ok(String::new())
    }
}

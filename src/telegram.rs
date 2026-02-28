pub mod client;
pub mod handler;

use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;

pub struct TelegramClient {
    config: Arc<Config>,
    bridge: Option<Arc<crate::bridge::BridgeCore>>,
}

impl TelegramClient {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { config, bridge: None })
    }

    pub async fn set_bridge(&self, bridge: Arc<crate::bridge::BridgeCore>) {
        let _ = bridge;
    }

    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

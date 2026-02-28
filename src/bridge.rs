pub mod message_flow;
pub mod portal;
pub mod puppet;
pub mod user_sync;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;
use crate::telegram::TelegramClient;

pub struct BridgeCore {
    matrix_client: Arc<MatrixAppservice>,
    telegram_client: Arc<TelegramClient>,
    db_manager: Arc<DatabaseManager>,
    running: RwLock<bool>,
}

impl BridgeCore {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        telegram_client: Arc<TelegramClient>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            matrix_client,
            telegram_client,
            db_manager,
            running: RwLock::new(false),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        while *running {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

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
    portal_manager: PortalManager,
    puppet_manager: PuppetManager,
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
            portal_manager: PortalManager::new(),
            puppet_manager: PuppetManager::new(),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        tracing::info!("Bridge core started");
        
        while *self.running.read().await {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Bridge core stopped");
    }

    pub fn matrix_client(&self) -> &MatrixAppservice {
        &self.matrix_client
    }

    pub fn telegram_client(&self) -> &TelegramClient {
        &self.telegram_client
    }

    pub fn db_manager(&self) -> &DatabaseManager {
        &self.db_manager
    }

    pub fn portal_manager(&self) -> &PortalManager {
        &self.portal_manager
    }

    pub fn puppet_manager(&self) -> &PuppetManager {
        &self.puppet_manager
    }

    pub async fn handle_telegram_message(
        &self,
        chat_id: i64,
        message_id: i32,
        sender_id: i64,
        content: &str,
    ) -> Result<()> {
        tracing::debug!(
            "Handling Telegram message: chat={}, msg={}, sender={}",
            chat_id,
            message_id,
            sender_id
        );

        let portal_store = self.db_manager.portal_store();
        if let Some(portal) = portal_store.get_by_telegram_chat(chat_id).await? {
            let mxid = self.matrix_client.get_user_mxid(sender_id).await;
            let _ = self.matrix_client.send_text(&portal.matrix_room_id, content).await?;
        }

        Ok(())
    }

    pub async fn handle_matrix_message(
        &self,
        room_id: &str,
        event_id: &str,
        sender: &str,
        content: &str,
    ) -> Result<()> {
        tracing::debug!(
            "Handling Matrix message: room={}, event={}, sender={}",
            room_id,
            event_id,
            sender
        );

        let portal_store = self.db_manager.portal_store();
        if let Some(portal) = portal_store.get_by_matrix_room(room_id).await? {
            let _ = self.telegram_client.send_message(portal.telegram_chat_id, content).await?;
        }

        Ok(())
    }
}

pub struct PortalManager {
    portals: RwLock<Vec<PortalInfo>>,
}

#[derive(Debug, Clone)]
pub struct PortalInfo {
    pub matrix_room_id: String,
    pub telegram_chat_id: i64,
    pub telegram_chat_type: String,
    pub title: Option<String>,
}

impl PortalManager {
    pub fn new() -> Self {
        Self {
            portals: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_by_matrix_room(&self, room_id: &str) -> Option<PortalInfo> {
        let portals = self.portals.read().await;
        portals.iter().find(|p| p.matrix_room_id == room_id).cloned()
    }

    pub async fn get_by_telegram_chat(&self, chat_id: i64) -> Option<PortalInfo> {
        let portals = self.portals.read().await;
        portals.iter().find(|p| p.telegram_chat_id == chat_id).cloned()
    }

    pub async fn add_portal(&self, portal: PortalInfo) {
        let mut portals = self.portals.write().await;
        portals.push(portal);
    }

    pub async fn remove_portal(&self, matrix_room_id: &str) {
        let mut portals = self.portals.write().await;
        portals.retain(|p| p.matrix_room_id != matrix_room_id);
    }
}

impl Default for PortalManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PuppetManager {
    puppets: RwLock<Vec<PuppetInfo>>,
}

#[derive(Debug, Clone)]
pub struct PuppetInfo {
    pub telegram_user_id: i64,
    pub matrix_mxid: String,
    pub displayname: Option<String>,
    pub avatar_mxc: Option<String>,
}

impl PuppetManager {
    pub fn new() -> Self {
        Self {
            puppets: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_by_telegram_id(&self, telegram_id: i64) -> Option<PuppetInfo> {
        let puppets = self.puppets.read().await;
        puppets.iter().find(|p| p.telegram_user_id == telegram_id).cloned()
    }

    pub async fn get_by_matrix_id(&self, mxid: &str) -> Option<PuppetInfo> {
        let puppets = self.puppets.read().await;
        puppets.iter().find(|p| p.matrix_mxid == mxid).cloned()
    }

    pub async fn add_puppet(&self, puppet: PuppetInfo) {
        let mut puppets = self.puppets.write().await;
        puppets.push(puppet);
    }

    pub async fn update_displayname(&self, telegram_id: i64, displayname: &str) {
        let mut puppets = self.puppets.write().await;
        if let Some(puppet) = puppets.iter_mut().find(|p| p.telegram_user_id == telegram_id) {
            puppet.displayname = Some(displayname.to_string());
        }
    }
}

impl Default for PuppetManager {
    fn default() -> Self {
        Self::new()
    }
}

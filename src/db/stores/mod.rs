use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::db::{
    DatabaseError, MessageMapping, PortalInfo, ProcessedEvent, ReactionMapping,
    TelegramFileInfo, TelegramUserInfo, UserMapping,
};

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn get_by_matrix_id(&self, matrix_id: &str) -> Result<Option<UserMapping>, DatabaseError>;
    async fn get_by_telegram_id(&self, telegram_id: i64) -> Result<Option<UserMapping>, DatabaseError>;
    async fn insert(&self, mapping: &UserMapping) -> Result<UserMapping, DatabaseError>;
    async fn update(&self, mapping: &UserMapping) -> Result<(), DatabaseError>;
    async fn delete(&self, id: i64) -> Result<(), DatabaseError>;
}

#[async_trait]
pub trait PortalStore: Send + Sync {
    async fn get_by_matrix_room(&self, room_id: &str) -> Result<Option<PortalInfo>, DatabaseError>;
    async fn get_by_telegram_chat(&self, chat_id: i64) -> Result<Option<PortalInfo>, DatabaseError>;
    async fn insert(&self, portal: &PortalInfo) -> Result<PortalInfo, DatabaseError>;
    async fn update(&self, portal: &PortalInfo) -> Result<(), DatabaseError>;
    async fn delete(&self, id: i64) -> Result<(), DatabaseError>;
    async fn list_all(&self, limit: i64) -> Result<Vec<PortalInfo>, DatabaseError>;
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn get_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn get_by_matrix_event(
        &self,
        room_id: &str,
        event_id: &str,
    ) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn insert(&self, mapping: &MessageMapping) -> Result<MessageMapping, DatabaseError>;
    async fn update(&self, mapping: &MessageMapping) -> Result<(), DatabaseError>;
    async fn delete(&self, id: i64) -> Result<(), DatabaseError>;
    async fn delete_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<(), DatabaseError>;
}

#[async_trait]
pub trait ReactionStore: Send + Sync {
    async fn get_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Vec<ReactionMapping>, DatabaseError>;
    async fn get_by_matrix_event(
        &self,
        event_id: &str,
    ) -> Result<Option<ReactionMapping>, DatabaseError>;
    async fn insert(&self, reaction: &ReactionMapping) -> Result<ReactionMapping, DatabaseError>;
    async fn delete(&self, id: i64) -> Result<(), DatabaseError>;
    async fn delete_by_telegram_reaction(
        &self,
        chat_id: i64,
        message_id: i64,
        user_id: i64,
        emoji: &str,
    ) -> Result<(), DatabaseError>;
}

#[async_trait]
pub trait TelegramFileStore: Send + Sync {
    async fn get_by_telegram_id(
        &self,
        file_unique_id: &str,
    ) -> Result<Option<TelegramFileInfo>, DatabaseError>;
    async fn insert(&self, file: &TelegramFileInfo) -> Result<TelegramFileInfo, DatabaseError>;
    async fn delete(&self, id: i64) -> Result<(), DatabaseError>;
}

pub struct InMemoryUserStore {
    users: parking_lot::RwLock<Vec<UserMapping>>,
}

impl InMemoryUserStore {
    pub fn new() -> Self {
        Self {
            users: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryUserStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserStore for InMemoryUserStore {
    async fn get_by_matrix_id(&self, matrix_id: &str) -> Result<Option<UserMapping>, DatabaseError> {
        let users = self.users.read();
        Ok(users.iter().find(|u| u.matrix_user_id == matrix_id).cloned())
    }

    async fn get_by_telegram_id(&self, telegram_id: i64) -> Result<Option<UserMapping>, DatabaseError> {
        let users = self.users.read();
        Ok(users.iter().find(|u| u.telegram_user_id == telegram_id).cloned())
    }

    async fn insert(&self, mapping: &UserMapping) -> Result<UserMapping, DatabaseError> {
        let mut users = self.users.write();
        let new_id = users.len() as i64 + 1;
        let mut new_mapping = mapping.clone();
        new_mapping.id = new_id;
        users.push(new_mapping.clone());
        Ok(new_mapping)
    }

    async fn update(&self, mapping: &UserMapping) -> Result<(), DatabaseError> {
        let mut users = self.users.write();
        if let Some(existing) = users.iter_mut().find(|u| u.id == mapping.id) {
            *existing = mapping.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), DatabaseError> {
        let mut users = self.users.write();
        users.retain(|u| u.id != id);
        Ok(())
    }
}

pub struct InMemoryPortalStore {
    portals: parking_lot::RwLock<Vec<PortalInfo>>,
}

impl InMemoryPortalStore {
    pub fn new() -> Self {
        Self {
            portals: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryPortalStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PortalStore for InMemoryPortalStore {
    async fn get_by_matrix_room(&self, room_id: &str) -> Result<Option<PortalInfo>, DatabaseError> {
        let portals = self.portals.read();
        Ok(portals.iter().find(|p| p.matrix_room_id == room_id).cloned())
    }

    async fn get_by_telegram_chat(&self, chat_id: i64) -> Result<Option<PortalInfo>, DatabaseError> {
        let portals = self.portals.read();
        Ok(portals.iter().find(|p| p.telegram_chat_id == chat_id).cloned())
    }

    async fn insert(&self, portal: &PortalInfo) -> Result<PortalInfo, DatabaseError> {
        let mut portals = self.portals.write();
        let new_id = portals.len() as i64 + 1;
        let mut new_portal = portal.clone();
        new_portal.id = new_id;
        portals.push(new_portal.clone());
        Ok(new_portal)
    }

    async fn update(&self, portal: &PortalInfo) -> Result<(), DatabaseError> {
        let mut portals = self.portals.write();
        if let Some(existing) = portals.iter_mut().find(|p| p.id == portal.id) {
            *existing = portal.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), DatabaseError> {
        let mut portals = self.portals.write();
        portals.retain(|p| p.id != id);
        Ok(())
    }

    async fn list_all(&self, limit: i64) -> Result<Vec<PortalInfo>, DatabaseError> {
        let portals = self.portals.read();
        Ok(portals.iter().take(limit as usize).cloned().collect())
    }
}

pub struct InMemoryMessageStore {
    messages: parking_lot::RwLock<Vec<MessageMapping>>,
}

impl InMemoryMessageStore {
    pub fn new() -> Self {
        Self {
            messages: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryMessageStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageStore for InMemoryMessageStore {
    async fn get_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Option<MessageMapping>, DatabaseError> {
        let messages = self.messages.read();
        Ok(messages
            .iter()
            .find(|m| m.telegram_chat_id == chat_id && m.telegram_message_id == message_id)
            .cloned())
    }

    async fn get_by_matrix_event(
        &self,
        room_id: &str,
        event_id: &str,
    ) -> Result<Option<MessageMapping>, DatabaseError> {
        let messages = self.messages.read();
        Ok(messages
            .iter()
            .find(|m| m.matrix_room_id == room_id && m.matrix_event_id == event_id)
            .cloned())
    }

    async fn insert(&self, mapping: &MessageMapping) -> Result<MessageMapping, DatabaseError> {
        let mut messages = self.messages.write();
        let new_id = messages.len() as i64 + 1;
        let mut new_mapping = mapping.clone();
        new_mapping.id = new_id;
        messages.push(new_mapping.clone());
        Ok(new_mapping)
    }

    async fn update(&self, mapping: &MessageMapping) -> Result<(), DatabaseError> {
        let mut messages = self.messages.write();
        if let Some(existing) = messages.iter_mut().find(|m| m.id == mapping.id) {
            *existing = mapping.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), DatabaseError> {
        let mut messages = self.messages.write();
        messages.retain(|m| m.id != id);
        Ok(())
    }

    async fn delete_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<(), DatabaseError> {
        let mut messages = self.messages.write();
        messages.retain(|m| !(m.telegram_chat_id == chat_id && m.telegram_message_id == message_id));
        Ok(())
    }
}

pub struct InMemoryReactionStore {
    reactions: parking_lot::RwLock<Vec<ReactionMapping>>,
}

impl InMemoryReactionStore {
    pub fn new() -> Self {
        Self {
            reactions: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryReactionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReactionStore for InMemoryReactionStore {
    async fn get_by_telegram_message(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Vec<ReactionMapping>, DatabaseError> {
        let reactions = self.reactions.read();
        Ok(reactions
            .iter()
            .filter(|r| r.telegram_chat_id == chat_id && r.telegram_message_id == message_id)
            .cloned()
            .collect())
    }

    async fn get_by_matrix_event(
        &self,
        event_id: &str,
    ) -> Result<Option<ReactionMapping>, DatabaseError> {
        let reactions = self.reactions.read();
        Ok(reactions.iter().find(|r| r.matrix_event_id == event_id).cloned())
    }

    async fn insert(&self, reaction: &ReactionMapping) -> Result<ReactionMapping, DatabaseError> {
        let mut reactions = self.reactions.write();
        let new_id = reactions.len() as i64 + 1;
        let mut new_reaction = reaction.clone();
        new_reaction.id = new_id;
        reactions.push(new_reaction.clone());
        Ok(new_reaction)
    }

    async fn delete(&self, id: i64) -> Result<(), DatabaseError> {
        let mut reactions = self.reactions.write();
        reactions.retain(|r| r.id != id);
        Ok(())
    }

    async fn delete_by_telegram_reaction(
        &self,
        chat_id: i64,
        message_id: i64,
        user_id: i64,
        emoji: &str,
    ) -> Result<(), DatabaseError> {
        let mut reactions = self.reactions.write();
        reactions.retain(|r| {
            !(r.telegram_chat_id == chat_id
                && r.telegram_message_id == message_id
                && r.telegram_user_id == user_id
                && r.reaction_emoji == emoji)
        });
        Ok(())
    }
}

pub struct InMemoryTelegramFileStore {
    files: parking_lot::RwLock<Vec<TelegramFileInfo>>,
}

impl InMemoryTelegramFileStore {
    pub fn new() -> Self {
        Self {
            files: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryTelegramFileStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TelegramFileStore for InMemoryTelegramFileStore {
    async fn get_by_telegram_id(
        &self,
        file_unique_id: &str,
    ) -> Result<Option<TelegramFileInfo>, DatabaseError> {
        let files = self.files.read();
        Ok(files
            .iter()
            .find(|f| f.telegram_file_unique_id == file_unique_id)
            .cloned())
    }

    async fn insert(&self, file: &TelegramFileInfo) -> Result<TelegramFileInfo, DatabaseError> {
        let mut files = self.files.write();
        let new_id = files.len() as i64 + 1;
        let mut new_file = file.clone();
        new_file.id = new_id;
        files.push(new_file.clone());
        Ok(new_file)
    }

    async fn delete(&self, id: i64) -> Result<(), DatabaseError> {
        let mut files = self.files.write();
        files.retain(|f| f.id != id);
        Ok(())
    }
}

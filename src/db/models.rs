use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMapping {
    pub id: i64,
    pub matrix_user_id: String,
    pub telegram_user_id: i64,
    pub telegram_username: Option<String>,
    pub telegram_first_name: Option<String>,
    pub telegram_last_name: Option<String>,
    pub telegram_phone: Option<String>,
    pub telegram_avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalInfo {
    pub id: i64,
    pub matrix_room_id: String,
    pub telegram_chat_id: i64,
    pub telegram_chat_type: String,
    pub telegram_chat_title: Option<String>,
    pub telegram_chat_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMapping {
    pub id: i64,
    pub telegram_message_id: i64,
    pub telegram_chat_id: i64,
    pub matrix_room_id: String,
    pub matrix_event_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionMapping {
    pub id: i64,
    pub telegram_message_id: i64,
    pub telegram_chat_id: i64,
    pub telegram_user_id: i64,
    pub reaction_emoji: String,
    pub matrix_event_id: String,
    pub matrix_room_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramFileInfo {
    pub id: i64,
    pub telegram_file_id: String,
    pub telegram_file_unique_id: String,
    pub mxc_url: String,
    pub mime_type: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUserInfo {
    pub id: i64,
    pub telegram_user_id: i64,
    pub matrix_mxid: String,
    pub displayname: Option<String>,
    pub avatar_mxc: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvent {
    pub id: i64,
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub processed_at: DateTime<Utc>,
}

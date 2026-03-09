use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use teloxide::prelude::*;
use teloxide::types::{
    ChatId, InputFile, MessageId, ParseMode, ReplyParameters,
};
use tracing::{info, warn};

use crate::bridge::BridgeCore;
use crate::config::Config;

pub mod client;
pub mod handler;

pub struct TelegramClient {
    config: Arc<Config>,
    bot: Option<Bot>,
    bridge: RwLock<Option<Arc<BridgeCore>>>,
}

impl TelegramClient {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let bot = config
            .auth
            .bot_token
            .as_ref()
            .filter(|t| t.as_str() != "disabled" && !t.is_empty())
            .map(|token| Bot::new(token));
        Ok(Self {
            config,
            bot,
            bridge: RwLock::new(None),
        })
    }

    pub async fn set_bridge(&self, bridge: Arc<BridgeCore>) {
        *self.bridge.write() = Some(bridge);
    }

    pub fn bot(&self) -> Option<&Bot> {
        self.bot.as_ref()
    }

    pub fn bot_token(&self) -> Option<&str> {
        self.config
            .auth
            .bot_token
            .as_deref()
            .filter(|t| *t != "disabled" && !t.is_empty())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Start the Telegram bot polling loop.
    pub async fn start(&self) -> Result<()> {
        if let Some(ref bot) = self.bot {
            info!("Starting Telegram bot polling...");
            let bot = bot.clone();
            let bridge = self.bridge.read().clone();

            tokio::spawn(async move {
                let handler = handler::TelegramUpdateHandler::new(bridge);
                handler.run(bot).await;
            });
        } else {
            warn!("No bot token configured, Telegram bot not started");
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Telegram client stopped");
        Ok(())
    }

    /// Send a text message to a Telegram chat.
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let result = bot
                .send_message(ChatId(chat_id), text)
                .await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send an HTML-formatted message to a Telegram chat.
    pub async fn send_html_message(
        &self,
        chat_id: i64,
        html: &str,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let result = bot
                .send_message(ChatId(chat_id), html)
                .parse_mode(ParseMode::Html)
                .await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send a message as a reply to another message.
    pub async fn send_reply(
        &self,
        chat_id: i64,
        reply_to_message_id: i32,
        text: &str,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let result = bot
                .send_message(ChatId(chat_id), text)
                .reply_parameters(ReplyParameters::new(MessageId(reply_to_message_id)))
                .await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Edit a text message.
    pub async fn edit_message(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
    ) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.edit_message_text(ChatId(chat_id), MessageId(message_id), text)
                .await?;
        }
        Ok(())
    }

    /// Delete a message.
    pub async fn delete_message(&self, chat_id: i64, message_id: i32) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.delete_message(ChatId(chat_id), MessageId(message_id))
                .await?;
        }
        Ok(())
    }

    /// Send a photo to a Telegram chat.
    pub async fn send_photo(
        &self,
        chat_id: i64,
        data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let file = InputFile::memory(data).file_name(filename.to_string());
            let mut req = bot.send_photo(ChatId(chat_id), file);
            if let Some(cap) = caption {
                req = req.caption(cap);
            }
            let result = req.await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send a document to a Telegram chat.
    pub async fn send_document(
        &self,
        chat_id: i64,
        data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let file = InputFile::memory(data).file_name(filename.to_string());
            let mut req = bot.send_document(ChatId(chat_id), file);
            if let Some(cap) = caption {
                req = req.caption(cap);
            }
            let result = req.await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send a video to a Telegram chat.
    pub async fn send_video(
        &self,
        chat_id: i64,
        data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let file = InputFile::memory(data).file_name(filename.to_string());
            let mut req = bot.send_video(ChatId(chat_id), file);
            if let Some(cap) = caption {
                req = req.caption(cap);
            }
            let result = req.await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send an audio file to a Telegram chat.
    pub async fn send_audio(
        &self,
        chat_id: i64,
        data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let file = InputFile::memory(data).file_name(filename.to_string());
            let mut req = bot.send_audio(ChatId(chat_id), file);
            if let Some(cap) = caption {
                req = req.caption(cap);
            }
            let result = req.await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send a location to a Telegram chat.
    pub async fn send_location(
        &self,
        chat_id: i64,
        latitude: f64,
        longitude: f64,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let result = bot
                .send_location(ChatId(chat_id), latitude, longitude)
                .await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Send a contact to a Telegram chat.
    pub async fn send_contact(
        &self,
        chat_id: i64,
        phone: &str,
        first_name: &str,
        last_name: Option<&str>,
    ) -> Result<Option<i32>> {
        if let Some(ref bot) = self.bot {
            let mut req = bot.send_contact(ChatId(chat_id), phone, first_name);
            if let Some(last) = last_name {
                req = req.last_name(last);
            }
            let result = req.await?;
            Ok(Some(result.id.0))
        } else {
            Ok(None)
        }
    }

    /// Get chat info from Telegram.
    pub async fn get_chat(&self, chat_id: i64) -> Result<Option<serde_json::Value>> {
        if let Some(ref bot) = self.bot {
            let chat = bot.get_chat(ChatId(chat_id)).await?;
            let json = serde_json::to_value(&chat)?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    /// Get chat member count.
    pub async fn get_chat_member_count(&self, chat_id: i64) -> Result<u32> {
        if let Some(ref bot) = self.bot {
            let count = bot.get_chat_member_count(ChatId(chat_id)).await?;
            Ok(count as u32)
        } else {
            Ok(0)
        }
    }

    /// Send a chat action (typing, upload_photo, etc.).
    pub async fn send_chat_action(
        &self,
        chat_id: i64,
        action: teloxide::types::ChatAction,
    ) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.send_chat_action(ChatId(chat_id), action).await?;
        }
        Ok(())
    }
}

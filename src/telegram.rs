use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use teloxide::{prelude::*, types::MessageId, Bot};

use crate::bridge::BridgeCore;
use crate::config::Config;

pub struct TelegramClient {
    config: Arc<Config>,
    bot: Option<Bot>,
    bridge: RwLock<Option<Arc<BridgeCore>>>,
}

impl TelegramClient {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let bot = config.auth.bot_token.as_ref()
            .filter(|t| t.as_str() != "disabled" && !t.is_empty())
            .map(Bot::new);
        Ok(Self { config, bot, bridge: RwLock::new(None) })
    }

    pub async fn set_bridge(&self, bridge: Arc<BridgeCore>) {
        *self.bridge.write() = Some(bridge);
    }

    pub async fn start(&self) -> Result<()> {
        if let Some(ref bot) = self.bot {
            tracing::info!("Starting Telegram bot...");
            let bot = bot.clone();
            tokio::spawn(async move {
                teloxide::repl(bot, |msg: Message| async move {
                    tracing::info!("Telegram message from chat {}", msg.chat.id);
                    respond(())
                }).await
            });
        }
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> { Ok(()) }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.send_message(teloxide::types::ChatId(chat_id), text).await?;
        }
        Ok(())
    }

    pub async fn edit_message(&self, chat_id: i64, message_id: i32, text: &str) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.edit_message_text(teloxide::types::ChatId(chat_id), MessageId(message_id), text).await?;
        }
        Ok(())
    }

    pub async fn delete_message(&self, chat_id: i64, message_id: i32) -> Result<()> {
        if let Some(ref bot) = self.bot {
            bot.delete_message(teloxide::types::ChatId(chat_id), MessageId(message_id)).await?;
        }
        Ok(())
    }
}

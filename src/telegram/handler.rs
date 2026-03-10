use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{ChatId, MediaKind, MessageKind};
use tracing::{debug, error, info};

use crate::bridge::BridgeCore;

/// Handles incoming Telegram updates and routes them to the bridge.
pub struct TelegramUpdateHandler {
    bridge: Option<Arc<BridgeCore>>,
}

impl TelegramUpdateHandler {
    pub fn new(bridge: Option<Arc<BridgeCore>>) -> Self {
        Self { bridge }
    }

    /// Run the update handler loop using teloxide dispatcher.
    pub async fn run(&self, bot: Bot) {
        let bridge = self.bridge.clone();

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let bridge = bridge.clone();
            async move {
                if let Some(ref bridge) = bridge {
                    if let Err(e) = Self::handle_message(bridge, &bot, &msg).await {
                        error!("Error handling Telegram message: {}", e);
                    }
                }
                respond(())
            }
        })
        .await;
    }

    fn owned_file_ids(file: &teloxide::types::FileMeta) -> (String, String) {
        (file.id.to_string(), file.unique_id.to_string())
    }

    /// Process a single Telegram message.
    async fn handle_message(bridge: &BridgeCore, bot: &Bot, msg: &Message) -> anyhow::Result<()> {
        let chat_id = msg.chat.id.0;
        let message_id = msg.id.0;

        let sender_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

        let sender_name = msg
            .from
            .as_ref()
            .map(|u| {
                let first = &u.first_name;
                if let Some(ref last) = u.last_name {
                    format!("{} {}", first, last)
                } else {
                    first.clone()
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        debug!(
            "Telegram message in chat {}: msg_id={}, sender={} ({})",
            chat_id, message_id, sender_name, sender_id
        );

        // Handle different message content types
        match &msg.kind {
            MessageKind::Common(common) => {
                match &common.media_kind {
                    MediaKind::Text(text) => {
                        // Check for bot commands before forwarding to bridge
                        if let Some(handled) =
                            Self::handle_command(bridge, bot, msg, &text.text).await?
                        {
                            if handled {
                                return Ok(());
                            }
                        }

                        bridge
                            .handle_telegram_message(chat_id, message_id, sender_id, &text.text)
                            .await?;
                    }
                    MediaKind::Photo(photo) => {
                        let caption = photo.caption.as_deref().unwrap_or("");
                        if let Some(largest) = photo.photo.last() {
                            let (file_id, file_unique_id) = Self::owned_file_ids(&largest.file);
                            bridge
                                .handle_telegram_photo(
                                    chat_id,
                                    message_id,
                                    sender_id,
                                    &file_id,
                                    &file_unique_id,
                                    caption,
                                )
                                .await?;
                        }
                    }
                    MediaKind::Document(doc) => {
                        let caption = doc.caption.as_deref().unwrap_or("");
                        let filename = doc.document.file_name.as_deref().unwrap_or("file");
                        let mime = doc
                            .document
                            .mime_type
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "application/octet-stream".to_string());
                        let (file_id, file_unique_id) = Self::owned_file_ids(&doc.document.file);
                        bridge
                            .handle_telegram_document(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                filename,
                                &mime,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Video(video) => {
                        let caption = video.caption.as_deref().unwrap_or("");
                        let (file_id, file_unique_id) = Self::owned_file_ids(&video.video.file);
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Audio(audio) => {
                        let caption = audio.caption.as_deref().unwrap_or("");
                        let (file_id, file_unique_id) = Self::owned_file_ids(&audio.audio.file);
                        bridge
                            .handle_telegram_audio(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Voice(voice) => {
                        let (file_id, file_unique_id) = Self::owned_file_ids(&voice.voice.file);
                        bridge
                            .handle_telegram_audio(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                "",
                            )
                            .await?;
                    }
                    MediaKind::Sticker(sticker) => {
                        let emoji = sticker.sticker.emoji.as_deref().unwrap_or("");
                        let (file_id, file_unique_id) = Self::owned_file_ids(&sticker.sticker.file);
                        bridge
                            .handle_telegram_sticker(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                emoji,
                            )
                            .await?;
                    }
                    MediaKind::Location(loc) => {
                        bridge
                            .handle_telegram_location(
                                chat_id,
                                message_id,
                                sender_id,
                                loc.location.latitude,
                                loc.location.longitude,
                            )
                            .await?;
                    }
                    MediaKind::Venue(venue) => {
                        bridge
                            .handle_telegram_location(
                                chat_id,
                                message_id,
                                sender_id,
                                venue.venue.location.latitude,
                                venue.venue.location.longitude,
                            )
                            .await?;
                    }
                    MediaKind::Contact(contact) => {
                        bridge
                            .handle_telegram_contact(
                                chat_id,
                                message_id,
                                sender_id,
                                &contact.contact.phone_number,
                                &contact.contact.first_name,
                                contact.contact.last_name.as_deref(),
                            )
                            .await?;
                    }
                    MediaKind::Animation(anim) => {
                        let caption = anim.caption.as_deref().unwrap_or("");
                        let (file_id, file_unique_id) = Self::owned_file_ids(&anim.animation.file);
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::VideoNote(vn) => {
                        let (file_id, file_unique_id) = Self::owned_file_ids(&vn.video_note.file);
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &file_id,
                                &file_unique_id,
                                "",
                            )
                            .await?;
                    }
                    _ => {
                        debug!("Unhandled media type in chat {}", chat_id);
                    }
                }
            }
            MessageKind::NewChatMembers(members) => {
                for user in &members.new_chat_members {
                    info!("User {} joined chat {}", user.first_name, chat_id);
                    bridge
                        .handle_telegram_join(chat_id, user.id.0 as i64)
                        .await?;
                }
            }
            MessageKind::LeftChatMember(member) => {
                info!(
                    "User {} left chat {}",
                    member.left_chat_member.first_name, chat_id
                );
                bridge
                    .handle_telegram_leave(chat_id, member.left_chat_member.id.0 as i64)
                    .await?;
            }
            MessageKind::NewChatTitle(title) => {
                bridge
                    .handle_telegram_title_change(chat_id, &title.new_chat_title)
                    .await?;
            }
            MessageKind::NewChatPhoto(_) => {
                debug!("New chat photo in {}", chat_id);
            }
            MessageKind::Pinned(pinned) => {
                // The pinned message is a MaybeInaccessibleMessage
                if let teloxide::types::MaybeInaccessibleMessage::Regular(pinned_msg) =
                    pinned.pinned.as_ref()
                {
                    bridge.handle_telegram_pin(chat_id, pinned_msg.id.0).await?;
                }
            }
            _ => {
                debug!("Unhandled message kind in chat {}", chat_id);
            }
        }

        Ok(())
    }

    /// Handle bot commands (messages starting with /).
    /// Returns Ok(Some(true)) if the command was handled,
    /// Ok(Some(false)) if the message starts with / but is not a recognized command,
    /// Ok(None) if the message is not a command.
    async fn handle_command(
        bridge: &BridgeCore,
        bot: &Bot,
        msg: &Message,
        text: &str,
    ) -> anyhow::Result<Option<bool>> {
        if !text.starts_with('/') {
            return Ok(None);
        }

        // Extract the command (strip the leading / and any @botname suffix, take first word)
        let first_word = text.split_whitespace().next().unwrap_or("");
        let command = first_word
            .strip_prefix('/')
            .unwrap_or(first_word)
            .split('@')
            .next()
            .unwrap_or("");

        let chat_id = ChatId(msg.chat.id.0);

        match command {
            "start" => {
                let welcome = "Welcome to the Matrix-Telegram Bridge Bot!\n\n\
                    This bot bridges messages between Telegram chats and Matrix rooms.\n\n\
                    Use /help to see available commands.";
                bot.send_message(chat_id, welcome).await?;
                info!("Handled /start command in chat {}", msg.chat.id.0);
                Ok(Some(true))
            }
            "help" => {
                let help_text = "Available commands:\n\n\
                    /start - Show welcome message\n\
                    /bridge - Show bridge status for this chat\n\
                    /help - Show this help message";
                bot.send_message(chat_id, help_text).await?;
                info!("Handled /help command in chat {}", msg.chat.id.0);
                Ok(Some(true))
            }
            "bridge" => {
                let portal = bridge
                    .portal_manager()
                    .get_by_telegram_chat(msg.chat.id.0)
                    .await;

                let reply = match portal {
                    Some(p) => {
                        let title_part = p
                            .title
                            .as_deref()
                            .map(|t| format!(" ({})", t))
                            .unwrap_or_default();
                        format!(
                            "This chat is bridged to Matrix room: {}{}\nChat type: {}",
                            p.matrix_room_id, title_part, p.telegram_chat_type
                        )
                    }
                    None => "This chat is not currently bridged to any Matrix room.".to_string(),
                };
                bot.send_message(chat_id, reply).await?;
                info!("Handled /bridge command in chat {}", msg.chat.id.0);
                Ok(Some(true))
            }
            _ => {
                // Unknown command -- let it pass through to the bridge as a regular message
                debug!("Unknown command /{} in chat {}", command, msg.chat.id.0);
                Ok(Some(false))
            }
        }
    }
}

impl Default for TelegramUpdateHandler {
    fn default() -> Self {
        Self::new(None)
    }
}

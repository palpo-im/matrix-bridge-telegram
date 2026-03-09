use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{
    MediaKind, MessageKind,
};
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

        teloxide::repl(bot, move |msg: Message| {
            let bridge = bridge.clone();
            async move {
                if let Some(ref bridge) = bridge {
                    if let Err(e) = Self::handle_message(bridge, &msg).await {
                        error!("Error handling Telegram message: {}", e);
                    }
                }
                respond(())
            }
        })
        .await;
    }

    /// Process a single Telegram message.
    async fn handle_message(
        bridge: &BridgeCore,
        msg: &Message,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id.0;
        let message_id = msg.id.0;

        let sender_id = msg
            .from
            .as_ref()
            .map(|u| u.id.0 as i64)
            .unwrap_or(0);

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
                        bridge
                            .handle_telegram_message(
                                chat_id,
                                message_id,
                                sender_id,
                                &text.text,
                            )
                            .await?;
                    }
                    MediaKind::Photo(photo) => {
                        let caption = photo
                            .caption
                            .as_deref()
                            .unwrap_or("");
                        if let Some(largest) = photo.photo.last() {
                            bridge
                                .handle_telegram_photo(
                                    chat_id,
                                    message_id,
                                    sender_id,
                                    &largest.file.id,
                                    &largest.file.unique_id,
                                    caption,
                                )
                                .await?;
                        }
                    }
                    MediaKind::Document(doc) => {
                        let caption = doc
                            .caption
                            .as_deref()
                            .unwrap_or("");
                        let filename = doc
                            .document
                            .file_name
                            .as_deref()
                            .unwrap_or("file");
                        let mime = doc
                            .document
                            .mime_type
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "application/octet-stream".to_string());
                        bridge
                            .handle_telegram_document(
                                chat_id,
                                message_id,
                                sender_id,
                                &doc.document.file.id,
                                &doc.document.file.unique_id,
                                filename,
                                &mime,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Video(video) => {
                        let caption = video
                            .caption
                            .as_deref()
                            .unwrap_or("");
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &video.video.file.id,
                                &video.video.file.unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Audio(audio) => {
                        let caption = audio
                            .caption
                            .as_deref()
                            .unwrap_or("");
                        bridge
                            .handle_telegram_audio(
                                chat_id,
                                message_id,
                                sender_id,
                                &audio.audio.file.id,
                                &audio.audio.file.unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::Voice(voice) => {
                        bridge
                            .handle_telegram_audio(
                                chat_id,
                                message_id,
                                sender_id,
                                &voice.voice.file.id,
                                &voice.voice.file.unique_id,
                                "",
                            )
                            .await?;
                    }
                    MediaKind::Sticker(sticker) => {
                        let emoji = sticker
                            .sticker
                            .emoji
                            .as_deref()
                            .unwrap_or("");
                        bridge
                            .handle_telegram_sticker(
                                chat_id,
                                message_id,
                                sender_id,
                                &sticker.sticker.file.id,
                                &sticker.sticker.file.unique_id,
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
                        let caption = anim
                            .caption
                            .as_deref()
                            .unwrap_or("");
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &anim.animation.file.id,
                                &anim.animation.file.unique_id,
                                caption,
                            )
                            .await?;
                    }
                    MediaKind::VideoNote(vn) => {
                        bridge
                            .handle_telegram_video(
                                chat_id,
                                message_id,
                                sender_id,
                                &vn.video_note.file.id,
                                &vn.video_note.file.unique_id,
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
                    info!(
                        "User {} joined chat {}",
                        user.first_name, chat_id
                    );
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
                if let teloxide::types::MaybeInaccessibleMessage::Regular(pinned_msg) = pinned.pinned.as_ref() {
                    bridge
                        .handle_telegram_pin(chat_id, pinned_msg.id.0)
                        .await?;
                }
            }
            _ => {
                debug!("Unhandled message kind in chat {}", chat_id);
            }
        }

        Ok(())
    }
}

impl Default for TelegramUpdateHandler {
    fn default() -> Self {
        Self::new(None)
    }
}

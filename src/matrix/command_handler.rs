use salvo::prelude::*;

use crate::bridge::BridgeCore;

/// Processes bridge commands sent via Matrix messages (e.g. !tg help).
pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> Self {
        Self
    }

    /// Process a command message with bridge context.
    pub async fn process_with_bridge(
        &self,
        room_id: &str,
        sender: &str,
        body: &str,
        bridge: &BridgeCore,
    ) -> Option<String> {
        let prefix = &bridge.matrix_client().config().bridge.command_prefix;
        let check_prefix = if prefix.is_empty() { "!tg" } else { prefix };

        let command = body.strip_prefix(check_prefix)?.trim();
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return Some(
                "No command specified. Use !tg help for available commands.".to_string(),
            );
        }

        let is_admin = bridge
            .matrix_client()
            .config()
            .bridge
            .admin_mxid
            .as_deref()
            == Some(sender);

        match parts[0] {
            "ping" => Some(Self::handle_ping()),
            "help" => Some(Self::handle_help(is_admin)),
            "bridge" => Some(Self::handle_bridge(room_id, &parts[1..], bridge).await),
            "unbridge" => Some(Self::handle_unbridge(room_id, bridge).await),
            "login" => Some(Self::handle_login()),
            "logout" => Some(Self::handle_logout()),
            "whoami" => Some(Self::handle_whoami(sender, bridge).await),
            "status" => Some(Self::handle_status(bridge).await),
            "list" => {
                if is_admin {
                    Some(Self::handle_list(bridge).await)
                } else {
                    Some("You don't have permission to use this command.".to_string())
                }
            }
            "sync" => Some(Self::handle_sync(room_id, bridge).await),
            _ => Some(format!(
                "Unknown command: {}. Use !tg help for available commands.",
                parts[0]
            )),
        }
    }

    /// Simple process without bridge context (for standalone use).
    pub async fn process(&self, _room_id: &str, _sender: &str, body: &str) -> Option<String> {
        if body.starts_with("!tg ") {
            let command = body.strip_prefix("!tg ").unwrap_or("");
            let parts: Vec<&str> = command.split_whitespace().collect();

            if parts.is_empty() {
                return Some(
                    "No command specified. Use !tg help for available commands.".to_string(),
                );
            }

            match parts[0] {
                "ping" => Some(Self::handle_ping()),
                "help" => Some(Self::handle_help(false)),
                _ => Some(format!(
                    "Unknown command: {}. Use !tg help for available commands.",
                    parts[0]
                )),
            }
        } else {
            None
        }
    }

    fn handle_ping() -> String {
        "Pong! Matrix-Telegram bridge is running.".to_string()
    }

    fn handle_help(is_admin: bool) -> String {
        let mut help = String::from("Available commands:\n");
        help.push_str("- !tg ping - Check if bridge is running\n");
        help.push_str("- !tg help - Show this help message\n");
        help.push_str("- !tg bridge <chat_id> [type] - Bridge this room to a Telegram chat\n");
        help.push_str("- !tg unbridge - Remove bridge from this room\n");
        help.push_str("- !tg login - Log in to Telegram\n");
        help.push_str("- !tg logout - Log out from Telegram\n");
        help.push_str("- !tg whoami - Show your account info\n");
        help.push_str("- !tg status - Show bridge status\n");
        help.push_str("- !tg sync - Sync members for this room\n");

        if is_admin {
            help.push_str("\nAdmin commands:\n");
            help.push_str("- !tg list - List all bridged rooms\n");
        }

        help
    }

    async fn handle_bridge(room_id: &str, args: &[&str], bridge: &BridgeCore) -> String {
        if args.is_empty() {
            return "Usage: !tg bridge <chat_id> [type]\ntype can be: user, chat, channel, supergroup"
                .to_string();
        }

        let chat_id: i64 = match args[0].parse() {
            Ok(id) => id,
            Err(_) => return format!("Invalid chat ID: {}", args[0]),
        };

        let chat_type = args.get(1).unwrap_or(&"chat");

        // Check if already bridged
        if bridge
            .portal_manager()
            .get_by_matrix_room(room_id)
            .await
            .is_some()
        {
            return "This room is already bridged to a Telegram chat.".to_string();
        }

        if bridge
            .portal_manager()
            .get_by_telegram_chat(chat_id)
            .await
            .is_some()
        {
            return format!(
                "Telegram chat {} is already bridged to another room.",
                chat_id
            );
        }

        // Try to get chat info
        let title = match bridge.telegram_client().get_chat(chat_id).await {
            Ok(Some(chat_info)) => chat_info
                .get("title")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string()),
            _ => None,
        };

        match bridge
            .bridge_room(room_id, chat_id, chat_type, title.as_deref())
            .await
        {
            Ok(()) => format!(
                "Successfully bridged this room to Telegram chat {} ({}).",
                chat_id,
                title.as_deref().unwrap_or("unknown")
            ),
            Err(e) => format!("Failed to bridge: {}", e),
        }
    }

    async fn handle_unbridge(room_id: &str, bridge: &BridgeCore) -> String {
        if bridge
            .portal_manager()
            .get_by_matrix_room(room_id)
            .await
            .is_none()
        {
            return "This room is not bridged to any Telegram chat.".to_string();
        }

        match bridge.unbridge_room(room_id).await {
            Ok(()) => "Successfully unbridged this room.".to_string(),
            Err(e) => format!("Failed to unbridge: {}", e),
        }
    }

    fn handle_login() -> String {
        "To log in to Telegram, please use the bridge web interface or contact the bridge administrator.\n\
         Note: This bridge uses a bot token for relaying messages. Individual user login is not supported in this version."
            .to_string()
    }

    fn handle_logout() -> String {
        "Individual user sessions are not supported in this version. The bridge uses a shared bot account."
            .to_string()
    }

    async fn handle_whoami(sender: &str, bridge: &BridgeCore) -> String {
        let user_store = bridge.db_manager().user_store();
        match user_store.get_by_matrix_id(sender).await {
            Ok(Some(mapping)) => {
                let mut info = format!("Matrix ID: {}\n", sender);
                info.push_str(&format!(
                    "Telegram ID: {}\n",
                    mapping.telegram_user_id
                ));
                if let Some(ref username) = mapping.telegram_username {
                    info.push_str(&format!("Telegram username: @{}\n", username));
                }
                if let Some(ref first) = mapping.telegram_first_name {
                    info.push_str(&format!("Name: {}", first));
                    if let Some(ref last) = mapping.telegram_last_name {
                        info.push_str(&format!(" {}", last));
                    }
                    info.push('\n');
                }
                info
            }
            _ => format!(
                "Matrix ID: {}\nNo Telegram account linked.",
                sender
            ),
        }
    }

    async fn handle_status(bridge: &BridgeCore) -> String {
        let portal_count = bridge.portal_manager().count().await;
        let puppet_count = bridge.puppet_manager().count().await;
        let bot_status = if bridge.telegram_client().bot().is_some() {
            "connected"
        } else {
            "not configured"
        };

        format!(
            "Bridge Status:\n\
             - Telegram bot: {}\n\
             - Bridged rooms: {}\n\
             - Active puppets: {}",
            bot_status, portal_count, puppet_count
        )
    }

    async fn handle_list(bridge: &BridgeCore) -> String {
        let portals = bridge.portal_manager().list_all().await;
        if portals.is_empty() {
            return "No bridged rooms.".to_string();
        }

        let mut result = format!("Bridged rooms ({}):\n", portals.len());
        for portal in &portals {
            result.push_str(&format!(
                "- {} <-> Telegram {} ({}) {}\n",
                portal.matrix_room_id,
                portal.telegram_chat_id,
                portal.telegram_chat_type,
                portal.title.as_deref().unwrap_or("")
            ));
        }
        result
    }

    async fn handle_sync(_room_id: &str, _bridge: &BridgeCore) -> String {
        "Member sync triggered. Puppets will be created as members send messages.".to_string()
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Salvo handler for command webhook (if used as HTTP endpoint).
#[handler]
pub async fn handle_command(req: &mut Request, res: &mut Response) {
    let body: Option<serde_json::Value> = req.parse_json().await.ok();

    if let Some(event) = body {
        if let Some(content) = event.get("content") {
            if let Some(body) = content.get("body").and_then(|b| b.as_str()) {
                if body.starts_with("!tg ") {
                    let handler = CommandHandler::new();
                    let room_id = event
                        .get("room_id")
                        .and_then(|r| r.as_str())
                        .unwrap_or("");
                    let sender = event
                        .get("sender")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");

                    if let Some(response) = handler.process(room_id, sender, body).await {
                        res.render(Json(serde_json::json!({
                            "msgtype": "m.notice",
                            "body": response
                        })));
                        return;
                    }
                }
            }
        }
    }

    res.render(Json(serde_json::json!({})));
}

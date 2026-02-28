use salvo::prelude::*;

#[handler]
pub async fn handle_command(req: &mut Request, res: &mut Response) {
    let body: Option<serde_json::Value> = req.parse_json().await.ok();
    
    if let Some(event) = body {
        if let Some(content) = event.get("content") {
            if let Some(body) = content.get("body").and_then(|b| b.as_str()) {
                if body.starts_with("!tg ") {
                    let command = body.strip_prefix("!tg ").unwrap_or("");
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    
                    if parts.is_empty() {
                        res.render(Json(serde_json::json!({
                            "msgtype": "m.notice",
                            "body": "No command specified. Use !tg help for available commands."
                        })));
                        return;
                    }
                    
                    let response = match parts[0] {
                        "ping" => handle_ping(),
                        "help" => handle_help(),
                        "bridge" => handle_bridge(&parts[1..]),
                        "unbridge" => handle_unbridge(&parts[1..]),
                        "login" => handle_login(),
                        "logout" => handle_logout(),
                        "whoami" => handle_whoami(),
                        _ => handle_unknown(parts[0]),
                    };
                    
                    res.render(Json(response));
                }
            }
        }
    }
}

fn handle_ping() -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": "Pong! Matrix-Telegram bridge is running."
    })
}

fn handle_help() -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "formatted_body": "<p><strong>Available commands:</strong></p>\n<ul>\n<li><code>!tg ping</code> - Check if bridge is running</li>\n<li><code>!tg help</code> - Show this help message</li>\n<li><code>!tg bridge &lt;chat_id&gt;</code> - Bridge this room to a Telegram chat</li>\n<li><code>!tg unbridge</code> - Remove bridge from this room</li>\n<li><code>!tg login</code> - Log in to Telegram</li>\n<li><code>!tg logout</code> - Log out from Telegram</li>\n<li><code>!tg whoami</code> - Show your Telegram account info</li>\n</ul>",
        "format": "org.matrix.custom.html",
        "body": "Available commands:\n- !tg ping - Check if bridge is running\n- !tg help - Show this help message\n- !tg bridge <chat_id> - Bridge this room to a Telegram chat\n- !tg unbridge - Remove bridge from this room\n- !tg login - Log in to Telegram\n- !tg logout - Log out from Telegram\n- !tg whoami - Show your Telegram account info"
    })
}

fn handle_bridge(args: &[&str]) -> serde_json::Value {
    if args.is_empty() {
        return serde_json::json!({
            "msgtype": "m.notice",
            "body": "Usage: !tg bridge <chat_id>"
        });
    }
    
    let chat_id = args[0];
    
    serde_json::json!({
        "msgtype": "m.notice",
        "body": format!("Bridging to Telegram chat {}...", chat_id)
    })
}

fn handle_unbridge(_args: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": "Unbridging this room..."
    })
}

fn handle_login() -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": "To log in to Telegram, please visit the bridge web interface or use the login-qr command."
    })
}

fn handle_logout() -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": "Logging out from Telegram..."
    })
}

fn handle_whoami() -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": "You are not logged in to Telegram."
    })
}

fn handle_unknown(cmd: &str) -> serde_json::Value {
    serde_json::json!({
        "msgtype": "m.notice",
        "body": format!("Unknown command: {}. Use !tg help for available commands.", cmd)
    })
}

pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, _room_id: &str, _sender: &str, body: &str) -> Option<String> {
        if body.starts_with("!tg ") {
            let command = body.strip_prefix("!tg ").unwrap_or("");
            let parts: Vec<&str> = command.split_whitespace().collect();
            
            if !parts.is_empty() {
                return Some(format!("Processing command: {}", parts[0]));
            }
        }
        None
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

use salvo::prelude::*;

/// Third-party protocol discovery endpoint (Matrix appservice spec).
#[handler]
pub async fn get_protocols(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "user_fields": ["username"],
        "location_fields": ["chat_id"],
        "icon": "mxc://",
        "field_types": {
            "username": {
                "regexp": "[a-zA-Z0-9_]+",
                "placeholder": "Telegram username"
            },
            "chat_id": {
                "regexp": "-?[0-9]+",
                "placeholder": "Telegram chat ID"
            }
        },
        "instances": [{
            "network_id": "telegram",
            "desc": "Telegram Messenger",
            "icon": "mxc://",
            "fields": {
                "network": "telegram"
            }
        }]
    })));
}

/// Get network info for a Telegram chat.
#[handler]
pub async fn get_network(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "network_id": "telegram",
        "desc": "Telegram Messenger"
    })));
}

/// Look up a user across the bridge.
#[handler]
pub async fn get_user(req: &mut Request, res: &mut Response) {
    let _userid = req.query::<String>("userid").unwrap_or_default();

    // Return empty list if no matching users
    res.render(Json(serde_json::json!([])));
}

/// Look up a location (chat/channel) across the bridge.
#[handler]
pub async fn get_location(req: &mut Request, res: &mut Response) {
    let _alias = req.query::<String>("alias").unwrap_or_default();

    // Return empty list if no matching locations
    res.render(Json(serde_json::json!([])));
}

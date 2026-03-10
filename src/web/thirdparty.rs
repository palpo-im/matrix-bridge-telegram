use salvo::prelude::*;
use tracing::debug;

/// Third-party protocol discovery endpoint.
/// GET /_matrix/app/v1/thirdparty/protocol/telegram
#[handler]
pub async fn get_protocols(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "user_fields": ["username"],
        "location_fields": ["chat_id"],
        "icon": "mxc://",
        "field_types": {
            "username": {
                "regexp": "[a-zA-Z0-9_]{5,32}",
                "placeholder": "Telegram username (without @)"
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

/// Get network info.
/// GET /_matrix/app/v1/thirdparty/network
#[handler]
pub async fn get_network(res: &mut Response) {
    res.render(Json(serde_json::json!([{
        "network_id": "telegram",
        "desc": "Telegram Messenger",
        "icon": "mxc://",
        "fields": {
            "network": "telegram"
        }
    }])));
}

/// Look up a third-party user.
/// GET /_matrix/app/v1/thirdparty/user
#[handler]
pub async fn get_user(req: &mut Request, res: &mut Response) {
    let userid = req.query::<String>("userid").unwrap_or_default();
    debug!("API: Third-party user lookup: {}", userid);

    // Return empty list - users are resolved dynamically
    res.render(Json(serde_json::json!([])));
}

/// Look up a third-party location (chat/channel).
/// GET /_matrix/app/v1/thirdparty/location
#[handler]
pub async fn get_location(req: &mut Request, res: &mut Response) {
    let alias = req.query::<String>("alias").unwrap_or_default();
    debug!("API: Third-party location lookup: {}", alias);

    // Return empty list - locations are resolved dynamically
    res.render(Json(serde_json::json!([])));
}

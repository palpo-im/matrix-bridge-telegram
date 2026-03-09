use salvo::prelude::*;
use tracing::debug;

/// List all bridged portals.
#[handler]
pub async fn list_bridges(res: &mut Response) {
    // In a full implementation, this would read from the database.
    // For now, return an empty list.
    res.render(Json(serde_json::json!({
        "bridges": []
    })));
}

/// Create a new bridge between a Matrix room and a Telegram chat.
#[handler]
pub async fn create_bridge(req: &mut Request, res: &mut Response) {
    let body: Option<serde_json::Value> = req.parse_json().await.ok();

    let (room_id, chat_id) = match body {
        Some(ref b) => {
            let room = b.get("room_id").and_then(|r| r.as_str());
            let chat = b.get("chat_id").and_then(|c| c.as_i64());
            (room, chat)
        }
        None => (None, None),
    };

    if room_id.is_none() || chat_id.is_none() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json(serde_json::json!({
            "error": "Missing room_id or chat_id"
        })));
        return;
    }

    let room_id = room_id.unwrap();
    let chat_id = chat_id.unwrap();
    let chat_type = body
        .as_ref()
        .and_then(|b| b.get("chat_type"))
        .and_then(|t| t.as_str())
        .unwrap_or("chat");

    debug!(
        "API: Creating bridge {} <-> {} ({})",
        room_id, chat_id, chat_type
    );

    res.render(Json(serde_json::json!({
        "status": "created",
        "room_id": room_id,
        "chat_id": chat_id,
        "chat_type": chat_type
    })));
}

/// Get bridge info for a specific room.
#[handler]
pub async fn get_bridge(req: &mut Request, res: &mut Response) {
    let room_id = req.param::<String>("room_id").unwrap_or_default();

    debug!("API: Getting bridge for room {}", room_id);

    res.render(Json(serde_json::json!({
        "room_id": room_id,
        "status": "not_found"
    })));
}

/// Delete/unbridge a room.
#[handler]
pub async fn delete_bridge(req: &mut Request, res: &mut Response) {
    let room_id = req.param::<String>("room_id").unwrap_or_default();

    debug!("API: Deleting bridge for room {}", room_id);

    res.render(Json(serde_json::json!({
        "status": "deleted",
        "room_id": room_id
    })));
}

/// List all portal mappings.
#[handler]
pub async fn list_portals(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "portals": []
    })));
}

/// Get user info.
#[handler]
pub async fn get_user_info(req: &mut Request, res: &mut Response) {
    let user_id = req.param::<String>("user_id").unwrap_or_default();

    debug!("API: Getting user info for {}", user_id);

    res.render(Json(serde_json::json!({
        "user_id": user_id,
        "telegram_id": null,
        "status": "not_found"
    })));
}

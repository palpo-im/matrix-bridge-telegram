use salvo::prelude::*;
use tracing::debug;

/// List all bridged portals.
/// GET /_matrix/app/v1/rooms
#[handler]
pub async fn list_bridges(res: &mut Response) {
    debug!("API: Listing bridges");
    res.render(Json(serde_json::json!({
        "bridges": [],
        "total": 0
    })));
}

/// Create a new bridge between a Matrix room and a Telegram chat.
/// POST /_matrix/app/v1/bridges
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
            "errcode": "M_BAD_JSON",
            "error": "Missing required fields: room_id, chat_id"
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

    // Note: Actual bridge creation should be done through the bridge core.
    // This endpoint acknowledges the request for external provisioning tools.
    res.status_code(StatusCode::CREATED);
    res.render(Json(serde_json::json!({
        "status": "created",
        "room_id": room_id,
        "chat_id": chat_id,
        "chat_type": chat_type
    })));
}

/// Get bridge info for a specific bridge ID or room.
/// GET /_matrix/app/v1/bridges/{id}
#[handler]
pub async fn get_bridge(req: &mut Request, res: &mut Response) {
    let bridge_id = req.param::<String>("room_id").unwrap_or_default();

    debug!("API: Getting bridge info for {}", bridge_id);

    res.status_code(StatusCode::NOT_FOUND);
    res.render(Json(serde_json::json!({
        "errcode": "M_NOT_FOUND",
        "error": format!("Bridge not found: {}", bridge_id)
    })));
}

/// Delete/unbridge a room.
/// DELETE /_matrix/app/v1/bridges/{id}
#[handler]
pub async fn delete_bridge(req: &mut Request, res: &mut Response) {
    let bridge_id = req.param::<String>("room_id").unwrap_or_default();

    debug!("API: Deleting bridge {}", bridge_id);

    res.render(Json(serde_json::json!({
        "status": "deleted",
        "bridge_id": bridge_id
    })));
}

/// List all portal mappings.
#[handler]
pub async fn list_portals(res: &mut Response) {
    debug!("API: Listing portals");
    res.render(Json(serde_json::json!({
        "portals": [],
        "total": 0
    })));
}

/// Get user info.
#[handler]
pub async fn get_user_info(req: &mut Request, res: &mut Response) {
    let user_id = req.param::<String>("user_id").unwrap_or_default();

    debug!("API: Getting user info for {}", user_id);

    res.status_code(StatusCode::NOT_FOUND);
    res.render(Json(serde_json::json!({
        "errcode": "M_NOT_FOUND",
        "error": format!("User not found: {}", user_id)
    })));
}

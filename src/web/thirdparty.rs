use salvo::prelude::*;

#[handler]
pub async fn get_protocols(res: &mut Response) {
    res.render(Json(serde_json::json!([{
        "id": "telegram",
        "name": "Telegram",
        "description": "Telegram Messenger Bridge"
    }])));
}

#[handler]
pub async fn get_network(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_implemented"
    })));
}

#[handler]
pub async fn get_user(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_implemented"
    })));
}

#[handler]
pub async fn get_location(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_implemented"
    })));
}

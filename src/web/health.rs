use salvo::prelude::*;

#[handler]
pub async fn health_check(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ok",
        "service": "matrix-bridge-telegram"
    })));
}

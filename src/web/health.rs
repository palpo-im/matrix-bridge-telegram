use salvo::prelude::*;

/// Health check endpoint returning bridge status.
#[handler]
pub async fn health_check(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "ok",
        "service": "matrix-bridge-telegram",
        "version": env!("CARGO_PKG_VERSION"),
    })));
}

use salvo::prelude::*;

#[handler]
pub async fn list_bridges(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "bridges": []
    })));
}

#[handler]
pub async fn create_bridge(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_implemented"
    })));
}

#[handler]
pub async fn get_bridge(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_found"
    })));
}

#[handler]
pub async fn delete_bridge(res: &mut Response) {
    res.render(Json(serde_json::json!({
        "status": "not_implemented"
    })));
}

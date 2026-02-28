use std::sync::Arc;

pub struct AdminNotifier {
    admin_mxid: Option<String>,
}

impl AdminNotifier {
    pub fn new(admin_mxid: Option<String>) -> Self {
        Self { admin_mxid }
    }

    pub async fn notify(&self, message: &str) {
        if let Some(ref admin) = self.admin_mxid {
            tracing::warn!("Admin notification for {}: {}", admin, message);
        }
    }
}

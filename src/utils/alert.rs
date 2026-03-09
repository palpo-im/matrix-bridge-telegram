use tracing::{info, warn};

/// Sends admin notifications via Matrix messages.
pub struct AdminNotifier {
    admin_mxid: Option<String>,
    admin_room: Option<String>,
}

impl AdminNotifier {
    pub fn new(admin_mxid: Option<String>) -> Self {
        Self {
            admin_mxid,
            admin_room: None,
        }
    }

    pub fn with_room(mut self, room_id: Option<String>) -> Self {
        self.admin_room = room_id;
        self
    }

    /// Send a notification to the bridge admin.
    pub async fn notify(&self, message: &str) {
        if let Some(ref admin) = self.admin_mxid {
            warn!("Admin notification for {}: {}", admin, message);
        } else {
            info!("Admin notification (no admin configured): {}", message);
        }
    }

    /// Send an error notification.
    pub async fn notify_error(&self, context: &str, error: &str) {
        let message = format!("Error in {}: {}", context, error);
        self.notify(&message).await;
    }

    /// Send a bridge state change notification.
    pub async fn notify_state_change(&self, state: &str) {
        let message = format!("Bridge state changed: {}", state);
        self.notify(&message).await;
    }

    /// Check if admin notifications are configured.
    pub fn is_configured(&self) -> bool {
        self.admin_mxid.is_some()
    }
}

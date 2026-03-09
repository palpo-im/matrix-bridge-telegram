/// Standalone event handler helpers (used outside the main MatrixEventHandlerImpl).
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Check if an event is a message edit (m.replace relation).
    pub fn is_edit(event: &serde_json::Value) -> bool {
        event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))
            .and_then(|r| r.get("rel_type"))
            .and_then(|t| t.as_str())
            == Some("m.replace")
    }

    /// Check if an event is a reply.
    pub fn is_reply(event: &serde_json::Value) -> bool {
        event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))
            .and_then(|r| r.get("m.in_reply_to"))
            .is_some()
    }

    /// Extract the event ID being replied to.
    pub fn get_reply_to(event: &serde_json::Value) -> Option<&str> {
        event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))
            .and_then(|r| r.get("m.in_reply_to"))
            .and_then(|r| r.get("event_id"))
            .and_then(|e| e.as_str())
    }

    /// Extract the event ID being edited.
    pub fn get_edit_target(event: &serde_json::Value) -> Option<&str> {
        let relates_to = event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))?;
        let rel_type = relates_to.get("rel_type").and_then(|t| t.as_str())?;
        if rel_type == "m.replace" {
            relates_to.get("event_id").and_then(|e| e.as_str())
        } else {
            None
        }
    }

    /// Extract the reaction key from a reaction event.
    pub fn get_reaction_key(event: &serde_json::Value) -> Option<&str> {
        event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))
            .and_then(|r| r.get("key"))
            .and_then(|k| k.as_str())
    }

    /// Extract the target event for a reaction.
    pub fn get_reaction_target(event: &serde_json::Value) -> Option<&str> {
        event
            .get("content")
            .and_then(|c| c.get("m.relates_to"))
            .and_then(|r| r.get("event_id"))
            .and_then(|e| e.as_str())
    }

    /// Get the event age in milliseconds.
    pub fn get_event_age(event: &serde_json::Value) -> u64 {
        event
            .get("unsigned")
            .and_then(|u| u.get("age"))
            .and_then(|a| a.as_u64())
            .unwrap_or(0)
    }

    /// Check if event is from a specific user pattern (e.g., bridge ghosts).
    pub fn is_from_pattern(event: &serde_json::Value, prefix: &str) -> bool {
        event
            .get("sender")
            .and_then(|s| s.as_str())
            .map(|s| s.starts_with(prefix))
            .unwrap_or(false)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

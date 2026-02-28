pub mod command_handler;
pub mod event_handler;

use std::sync::Arc;

use anyhow::Result;

use crate::config::Config;

pub struct MatrixAppservice {
    config: Arc<Config>,
}

impl MatrixAppservice {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn set_processor(&self, _processor: Arc<MatrixEventProcessor>) {}
}

pub struct MatrixEventProcessor {
    handler: Arc<dyn MatrixEventHandler + Send + Sync>,
    age_limit_ms: u64,
}

impl MatrixEventProcessor {
    pub fn with_age_limit(handler: Arc<dyn MatrixEventHandler + Send + Sync>, age_limit_ms: u64) -> Self {
        Self { handler, age_limit_ms }
    }
}

#[async_trait::async_trait]
pub trait MatrixEventHandler {
    async fn handle_room_message(&self, room_id: &str, event: &serde_json::Value);
}

pub struct MatrixEventHandlerImpl {
    matrix_client: Arc<MatrixAppservice>,
    bridge: Option<Arc<crate::bridge::BridgeCore>>,
}

impl MatrixEventHandlerImpl {
    pub fn new(matrix_client: Arc<MatrixAppservice>) -> Self {
        Self {
            matrix_client,
            bridge: None,
        }
    }

    pub fn set_bridge(&mut self, bridge: Arc<crate::bridge::BridgeCore>) {
        self.bridge = Some(bridge);
    }
}

#[async_trait::async_trait]
impl MatrixEventHandler for MatrixEventHandlerImpl {
    async fn handle_room_message(&self, _room_id: &str, _event: &serde_json::Value) {}
}

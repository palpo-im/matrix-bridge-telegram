pub mod health;
pub mod metrics;
pub mod provisioning;
pub mod thirdparty;

use std::sync::Arc;

use salvo::Listener;

use anyhow::Result;

use crate::bridge::BridgeCore;
use crate::config::Config;
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;

pub struct WebServer {
    config: Arc<Config>,
}

impl WebServer {
    pub async fn new(
        config: Arc<Config>,
        _matrix_client: Arc<MatrixAppservice>,
        _db_manager: Arc<DatabaseManager>,
        _bridge: Arc<BridgeCore>,
    ) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn start(&self) -> Result<()> {
        let bind_addr: String = format!("{}:{}", self.config.bridge.bind_address, self.config.bridge.port);
        tracing::info!("Web server listening on {}", bind_addr);
        
        let router = salvo::Router::new()
            .push(salvo::Router::with_path("/health").get(health::health_check));
        
        let service = salvo::Service::new(router);
        let acceptor = salvo::conn::TcpListener::new(bind_addr).bind().await;
        salvo::Server::new(acceptor).serve(service).await;
        
        Ok(())
    }
}

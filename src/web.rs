pub mod health;
pub mod metrics;
pub mod provisioning;
pub mod thirdparty;

use std::sync::Arc;

use salvo::prelude::*;
use salvo::Listener;
use tracing::{debug, info, warn};

use anyhow::Result;

use crate::bridge::BridgeCore;
use crate::config::Config;
use crate::db::DatabaseManager;
use crate::matrix::MatrixAppservice;

pub struct WebServer {
    config: Arc<Config>,
    matrix_client: Arc<MatrixAppservice>,
    db_manager: Arc<DatabaseManager>,
    bridge: Arc<BridgeCore>,
}

impl WebServer {
    pub async fn new(
        config: Arc<Config>,
        matrix_client: Arc<MatrixAppservice>,
        db_manager: Arc<DatabaseManager>,
        bridge: Arc<BridgeCore>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            matrix_client,
            db_manager,
            bridge,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let bind_addr: String = format!(
            "{}:{}",
            self.config.bridge.bind_address, self.config.bridge.port
        );
        info!("Web server listening on {}", bind_addr);

        // Store shared state for handlers
        let _bridge = self.bridge.clone();
        let matrix_client = self.matrix_client.clone();
        let config = self.config.clone();
        let _db_manager = self.db_manager.clone();

        let router = salvo::Router::new()
            // Health check
            .push(salvo::Router::with_path("/health").get(health::health_check))
            // Metrics endpoint
            .push(salvo::Router::with_path("/metrics").get(metrics::metrics_endpoint))
            // Appservice transaction endpoint
            .push(
                salvo::Router::with_path("/transactions/<txn_id>")
                    .put(TransactionHandler {
                        matrix_client: matrix_client.clone(),
                        config: config.clone(),
                    }),
            )
            // Third-party protocol discovery (Matrix spec)
            .push(
                salvo::Router::with_path("/thirdparty/protocol/telegram")
                    .get(thirdparty::get_protocols),
            )
            .push(
                salvo::Router::with_path("/thirdparty/user")
                    .get(thirdparty::get_user),
            )
            .push(
                salvo::Router::with_path("/thirdparty/location")
                    .get(thirdparty::get_location),
            )
            // Provisioning API
            .push(
                salvo::Router::with_path("/v1")
                    .push(
                        salvo::Router::with_path("/bridges")
                            .get(provisioning::list_bridges)
                            .post(provisioning::create_bridge),
                    )
                    .push(
                        salvo::Router::with_path("/bridges/<room_id>")
                            .get(provisioning::get_bridge)
                            .delete(provisioning::delete_bridge),
                    )
                    .push(
                        salvo::Router::with_path("/portals")
                            .get(provisioning::list_portals),
                    )
                    .push(
                        salvo::Router::with_path("/users/<user_id>")
                            .get(provisioning::get_user_info),
                    ),
            );

        let service = salvo::Service::new(router);
        let acceptor = salvo::conn::TcpListener::new(bind_addr).bind().await;
        salvo::Server::new(acceptor).serve(service).await;

        Ok(())
    }
}

/// Handler for Matrix Appservice transaction PUT requests.
#[derive(Clone)]
struct TransactionHandler {
    matrix_client: Arc<MatrixAppservice>,
    config: Arc<Config>,
}

#[handler]
impl TransactionHandler {
    async fn handle(&self, req: &mut Request, res: &mut Response) {
        // Verify the homeserver token
        let hs_token = req
            .query::<String>("access_token")
            .or_else(|| {
                req.headers()
                    .get("Authorization")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.strip_prefix("Bearer "))
                    .map(|s| s.to_string())
            });

        if let Some(ref token) = hs_token {
            if token != &self.config.registration.homeserver_token {
                res.status_code(StatusCode::FORBIDDEN);
                res.render(Json(serde_json::json!({
                    "errcode": "M_FORBIDDEN",
                    "error": "Invalid homeserver token"
                })));
                return;
            }
        } else {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "errcode": "M_UNAUTHORIZED",
                "error": "Missing access token"
            })));
            return;
        }

        // Parse the transaction body
        let body: serde_json::Value = match req.parse_json().await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to parse transaction body: {}", e);
                res.render(Json(serde_json::json!({})));
                return;
            }
        };

        let txn_id = req.param::<String>("txn_id").unwrap_or_default();
        debug!("Received transaction {}", txn_id);

        // Process the events
        if let Err(e) = self.matrix_client.process_transaction(&body).await {
            warn!("Error processing transaction {}: {}", txn_id, e);
        }

        // Always return 200 OK
        res.render(Json(serde_json::json!({})));
    }
}

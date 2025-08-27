//! Server module for OCPP CSMS

use crate::{
    auth::AuthService, database::DatabasePool, metrics::MetricsRegistry, CsmsError, CsmsResult,
};
use axum::{
    extract::{ws::WebSocket, ConnectInfo, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dashmap::DashMap;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: String,
    /// WebSocket port
    pub websocket_port: u16,
    /// HTTP API port
    pub http_port: u16,
    /// Metrics port
    pub metrics_port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<std::path::PathBuf>,
    /// TLS private key path
    pub tls_key_path: Option<std::path::PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            websocket_port: 8080,
            http_port: 3000,
            metrics_port: 9090,
            max_connections: 1000,
            connection_timeout: 30,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// CSMS server state
#[derive(Clone)]
pub struct ServerState {
    /// Database pool
    pub database: Arc<DatabasePool>,
    /// Metrics registry
    pub metrics: Arc<MetricsRegistry>,
    /// Authentication service
    pub auth: Arc<AuthService>,
    /// Active WebSocket connections
    pub connections: Arc<DashMap<Uuid, ConnectionInfo>>,
    /// Server configuration
    pub config: ServerConfig,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: Uuid,
    /// Charge point ID
    pub charge_point_id: Option<String>,
    /// Remote address
    pub remote_addr: SocketAddr,
    /// Connected timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ConnectionInfo {
    pub fn new(remote_addr: SocketAddr) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            charge_point_id: None,
            remote_addr,
            connected_at: now,
            last_activity: now,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now();
    }
}

/// CSMS server implementation
pub struct CsmsServer {
    /// Server state
    state: ServerState,
}

impl CsmsServer {
    /// Create new CSMS server
    pub fn new(
        config: ServerConfig,
        _manager: Arc<RwLock<crate::manager::ChargePointManager>>,
        database: Arc<DatabasePool>,
        metrics: Arc<MetricsRegistry>,
    ) -> Self {
        let auth_config = crate::auth::AuthConfig::default();
        let auth = Arc::new(AuthService::new(auth_config));

        let state = ServerState {
            database,
            metrics,
            auth,
            connections: Arc::new(DashMap::new()),
            config,
        };

        Self { state }
    }

    /// Start the server
    pub async fn run(self) -> CsmsResult<()> {
        info!("Starting OCPP CSMS server");

        // Start WebSocket server
        let ws_server = self.start_websocket_server();

        // Start HTTP API server
        let http_server = self.start_http_server();

        // Start metrics server
        let metrics_server = self.start_metrics_server();

        // Wait for all servers
        tokio::select! {
            result = ws_server => {
                error!("WebSocket server exited: {:?}", result);
                result
            }
            result = http_server => {
                error!("HTTP server exited: {:?}", result);
                result
            }
            result = metrics_server => {
                error!("Metrics server exited: {:?}", result);
                result
            }
        }
    }

    /// Start WebSocket server for OCPP connections
    async fn start_websocket_server(&self) -> CsmsResult<()> {
        let addr = format!(
            "{}:{}",
            self.state.config.bind_address, self.state.config.websocket_port
        );
        info!("Starting WebSocket server on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| CsmsError::Internal {
                message: format!("Failed to bind WebSocket server: {}", e),
            })?;

        let app = Router::new()
            .route("/ocpp/:charge_point_id", get(ws_handler))
            .with_state(self.state.clone());

        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        );

        server.await.map_err(|e| CsmsError::Internal {
            message: format!("WebSocket server error: {}", e),
        })
    }

    /// Start HTTP API server
    async fn start_http_server(&self) -> CsmsResult<()> {
        let addr = format!(
            "{}:{}",
            self.state.config.bind_address, self.state.config.http_port
        );
        info!("Starting HTTP API server on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| CsmsError::Internal {
                message: format!("Failed to bind HTTP server: {}", e),
            })?;

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/api/v1/charge-points", get(list_charge_points))
            .route("/api/v1/charge-points/:id", get(get_charge_point))
            .with_state(self.state.clone());

        let server = axum::serve(listener, app.into_make_service());

        server.await.map_err(|e| CsmsError::Internal {
            message: format!("HTTP server error: {}", e),
        })
    }

    /// Start metrics server
    async fn start_metrics_server(&self) -> CsmsResult<()> {
        let addr = format!(
            "{}:{}",
            self.state.config.bind_address, self.state.config.metrics_port
        );
        info!("Starting metrics server on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| CsmsError::Internal {
                message: format!("Failed to bind metrics server: {}", e),
            })?;

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(self.state.clone());

        let server = axum::serve(listener, app.into_make_service());

        server.await.map_err(|e| CsmsError::Internal {
            message: format!("Metrics server error: {}", e),
        })
    }
}

/// WebSocket handler for OCPP connections
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
    Path(charge_point_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    info!(
        "WebSocket upgrade request from {} for charge point {}",
        addr, charge_point_id
    );

    // Validate charge point ID
    if charge_point_id.is_empty() || charge_point_id.len() > 20 {
        return (StatusCode::BAD_REQUEST, "Invalid charge point ID").into_response();
    }

    // Check connection limit
    if state.connections.len() >= state.config.max_connections {
        warn!(
            "Connection limit reached, rejecting connection from {}",
            addr
        );
        return (StatusCode::SERVICE_UNAVAILABLE, "Connection limit reached").into_response();
    }

    ws.protocols(["ocpp1.6"])
        .on_upgrade(move |socket| handle_websocket(socket, state, charge_point_id, addr))
}

/// Handle WebSocket connection
async fn handle_websocket(
    socket: WebSocket,
    state: ServerState,
    charge_point_id: String,
    addr: SocketAddr,
) {
    let mut connection_info = ConnectionInfo::new(addr);
    connection_info.charge_point_id = Some(charge_point_id.clone());

    let connection_id = connection_info.id;
    state.connections.insert(connection_id, connection_info);

    info!(
        "WebSocket connection established: {} ({})",
        connection_id, charge_point_id
    );
    state
        .metrics
        .set_active_connections(state.connections.len() as i64);

    let (_sender, mut receiver) = socket.split();

    // Connection cleanup on drop
    let cleanup_state = state.clone();
    let cleanup = scopeguard::guard(connection_id, move |id| {
        cleanup_state.connections.remove(&id);
        cleanup_state
            .metrics
            .set_active_connections(cleanup_state.connections.len() as i64);
        info!("WebSocket connection cleaned up: {}", id);
    });

    // Message handling loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                if let Err(e) = handle_websocket_message(&state, connection_id, msg).await {
                    error!("Error handling WebSocket message: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Explicit cleanup
    drop(cleanup);
}

/// Handle individual WebSocket message
async fn handle_websocket_message(
    state: &ServerState,
    connection_id: Uuid,
    msg: axum::extract::ws::Message,
) -> CsmsResult<()> {
    use axum::extract::ws::Message;

    match msg {
        Message::Text(text) => {
            debug!(
                "Received text message from {}: {} bytes",
                connection_id,
                text.len()
            );

            // Update connection activity
            if let Some(mut conn) = state.connections.get_mut(&connection_id) {
                conn.update_activity();
            }

            // Parse and handle OCPP message
            // TODO: Implement OCPP message parsing and handling
            state.metrics.record_message_received("Unknown");
        }
        Message::Binary(_) => {
            warn!("Received unexpected binary message from {}", connection_id);
        }
        Message::Ping(_data) => {
            debug!("Received ping from {}", connection_id);
            // Pong is handled automatically by axum
        }
        Message::Pong(_) => {
            debug!("Received pong from {}", connection_id);
        }
        Message::Close(frame) => {
            info!("Received close frame from {}: {:?}", connection_id, frame);
        }
    }

    Ok(())
}

/// Health check endpoint
async fn health_handler(State(state): State<ServerState>) -> impl IntoResponse {
    // Perform basic health checks
    let db_healthy = state.database.health_check().await.is_ok();
    let metrics_healthy = state.metrics.is_healthy();

    let health = serde_json::json!({
        "status": if db_healthy && metrics_healthy { "healthy" } else { "unhealthy" },
        "checks": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "metrics": if metrics_healthy { "healthy" } else { "unhealthy" },
        },
        "connections": state.connections.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let status_code = if db_healthy && metrics_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health))
}

/// List charge points endpoint
async fn list_charge_points(State(_state): State<ServerState>) -> impl IntoResponse {
    // TODO: Implement charge point listing from database
    Json(serde_json::json!({
        "charge_points": [],
        "total": 0
    }))
}

/// Get charge point endpoint
async fn get_charge_point(
    State(_state): State<ServerState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement charge point lookup from database
    (StatusCode::NOT_FOUND, "Charge point not found")
}

/// Metrics endpoint
async fn metrics_handler(State(state): State<ServerState>) -> impl IntoResponse {
    match state.metrics.gather() {
        Ok(metric_families) => {
            let encoder = prometheus::TextEncoder::new();
            match encoder.encode_to_string(&metric_families) {
                Ok(metrics) => (StatusCode::OK, metrics),
                Err(e) => {
                    error!("Failed to encode metrics: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to encode metrics".to_string(),
                    )
                }
            }
        }
        Err(e) => {
            error!("Failed to gather metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to gather metrics".to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.websocket_port, 8080);
        assert_eq!(config.http_port, 3000);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_connection_info_creation() {
        let addr = "127.0.0.1:12345".parse().unwrap();
        let info = ConnectionInfo::new(addr);
        assert_eq!(info.remote_addr, addr);
        assert!(info.charge_point_id.is_none());
    }

    #[test]
    fn test_connection_info_activity_update() {
        let addr = "127.0.0.1:12345".parse().unwrap();
        let mut info = ConnectionInfo::new(addr);
        let old_activity = info.last_activity;

        std::thread::sleep(std::time::Duration::from_millis(1));
        info.update_activity();

        assert!(info.last_activity > old_activity);
    }
}

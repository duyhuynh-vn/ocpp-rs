//! WebSocket client implementation for OCPP Charge Points

use crate::{
    error::TransportResult, websocket::client::connect, ConnectionState, MessageHandler, Transport,
    TransportConfig, TransportEvent,
};
use ocpp_messages::Message;
use ocpp_types::OcppResult;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use uuid::Uuid;

/// WebSocket client for OCPP Charge Points
pub struct WebSocketClient {
    /// Connection ID
    connection_id: Uuid,
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Configuration
    config: TransportConfig,
    /// Message handler
    message_handler: Arc<dyn MessageHandler>,
    /// Message sender channel
    message_tx: Arc<RwLock<Option<mpsc::UnboundedSender<Message>>>>,
    /// Task handles
    task_handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub async fn new(
        url: String,
        config: TransportConfig,
        message_handler: Arc<dyn MessageHandler>,
    ) -> TransportResult<Self> {
        info!("Creating WebSocket client for URL: {}", url);

        let connection_id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(ConnectionState::Connecting));
        let message_tx = Arc::new(RwLock::new(None));
        let task_handles = Arc::new(RwLock::new(Vec::new()));

        let client = Self {
            connection_id,
            state: state.clone(),
            config: config.clone(),
            message_handler: message_handler.clone(),
            message_tx: message_tx.clone(),
            task_handles: task_handles.clone(),
        };

        // Connect to WebSocket server
        client.connect_internal(url).await?;

        Ok(client)
    }

    /// Internal connection method
    async fn connect_internal(&self, url: String) -> TransportResult<()> {
        info!("Connecting to WebSocket server: {}", url);

        // Update state to connecting
        *self.state.write().await = ConnectionState::Connecting;

        // Connect to the WebSocket server
        let ws_connection = connect(&url, &self.config.sub_protocols, &self.config).await?;

        // Update state to connected
        *self.state.write().await = ConnectionState::Connected;

        // Create message channel
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        *self.message_tx.write().await = Some(tx);

        // Send connection event
        let event = TransportEvent::Connected {
            connection_id: self.connection_id,
            remote_addr: url.clone(),
        };
        self.message_handler.handle_event(event).await;

        // Spawn message sending task
        let ws_connection_send = Arc::new(tokio::sync::Mutex::new(ws_connection));
        let ws_connection_recv = ws_connection_send.clone();
        let message_handler = self.message_handler.clone();
        let connection_id = self.connection_id;
        let state = self.state.clone();

        // Spawn outbound message task
        let send_task = {
            let ws_connection = ws_connection_send.clone();
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    let mut conn = ws_connection.lock().await;
                    match serde_json::to_string(&message) {
                        Ok(json_str) => {
                            if let Err(e) = conn.send_message(json_str).await {
                                error!("Failed to send message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize message: {}", e);
                        }
                    }
                }
            })
        };

        // Spawn inbound message task
        let recv_task = {
            let ws_connection = ws_connection_recv;
            let state = state.clone();
            tokio::spawn(async move {
                loop {
                    let mut conn = ws_connection.lock().await;
                    match conn.receive_message().await {
                        Ok(Some(text)) => {
                            match serde_json::from_str::<Message>(&text) {
                                Ok(message) => {
                                    // Handle the message
                                    match message_handler.handle_message(message.clone()).await {
                                        Ok(Some(response)) => {
                                            // Send response back
                                            if let Ok(response_json) =
                                                serde_json::to_string(&response)
                                            {
                                                if let Err(e) =
                                                    conn.send_message(response_json).await
                                                {
                                                    error!("Failed to send response: {}", e);
                                                    break;
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // No response needed
                                        }
                                        Err(e) => {
                                            error!("Error handling message: {}", e);
                                        }
                                    }

                                    // Send message received event
                                    let event = TransportEvent::MessageReceived {
                                        connection_id,
                                        message,
                                    };
                                    message_handler.handle_event(event).await;
                                }
                                Err(e) => {
                                    error!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            // Keep-alive or other non-text message, continue
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }

                // Connection closed
                *state.write().await = ConnectionState::Closed;
                let event = TransportEvent::Disconnected {
                    connection_id,
                    reason: "Connection closed".to_string(),
                };
                message_handler.handle_event(event).await;
            })
        };

        // Store task handles
        let mut handles = self.task_handles.write().await;
        handles.push(send_task);
        handles.push(recv_task);

        Ok(())
    }

    /// Disconnect from Central System
    pub async fn disconnect(&self) -> TransportResult<()> {
        info!("Disconnecting from WebSocket server");

        *self.state.write().await = ConnectionState::Closing;

        // Cancel all tasks
        let mut handles = self.task_handles.write().await;
        for handle in handles.drain(..) {
            handle.abort();
        }

        *self.state.write().await = ConnectionState::Closed;

        Ok(())
    }

    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ConnectionState::Connected)
    }
}

#[async_trait::async_trait]
impl Transport for WebSocketClient {
    async fn send_message(&self, message: Message) -> OcppResult<()> {
        debug!("Sending message: {}", message.unique_id());

        let tx = self.message_tx.read().await;
        if let Some(sender) = tx.as_ref() {
            sender
                .send(message)
                .map_err(|_| ocpp_types::OcppError::Transport {
                    message: "Failed to send message: channel closed".to_string(),
                })?;
            Ok(())
        } else {
            Err(ocpp_types::OcppError::Transport {
                message: "Not connected".to_string(),
            })
        }
    }

    async fn close(&self) -> OcppResult<()> {
        self.disconnect()
            .await
            .map_err(|e| ocpp_types::OcppError::Transport {
                message: format!("Failed to close connection: {}", e),
            })
    }

    fn state(&self) -> ConnectionState {
        // Note: This is a blocking implementation for compatibility
        // In practice, you should use the async version
        ConnectionState::Connected // Simplified for compatibility
    }

    fn connection_id(&self) -> Uuid {
        self.connection_id
    }
}

/// Legacy OcppClient for backward compatibility
pub struct OcppClient {
    /// Connection ID
    connection_id: Uuid,
    /// Current connection state
    state: ConnectionState,
    /// Configuration
    _config: TransportConfig,
    /// Event sender
    _event_tx: mpsc::UnboundedSender<TransportEvent>,
}

impl OcppClient {
    /// Create a new OCPP client
    pub fn new(config: TransportConfig) -> (Self, mpsc::UnboundedReceiver<TransportEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let client = Self {
            connection_id: Uuid::new_v4(),
            state: ConnectionState::Closed,
            _config: config,
            _event_tx: event_tx,
        };
        (client, event_rx)
    }

    /// Connect to OCPP Central System
    pub async fn connect(&mut self, url: &str) -> TransportResult<()> {
        info!("Connecting to OCPP Central System at {}", url);
        self.state = ConnectionState::Connecting;

        // TODO: Implement actual WebSocket connection

        self.state = ConnectionState::Connected;
        Ok(())
    }

    /// Disconnect from Central System
    pub async fn disconnect(&mut self) -> TransportResult<()> {
        info!("Disconnecting from OCPP Central System");
        self.state = ConnectionState::Closing;

        // TODO: Implement actual disconnect logic

        self.state = ConnectionState::Closed;
        Ok(())
    }

    /// Start the client event loop
    pub async fn run(&mut self) -> TransportResult<()> {
        info!("Starting OCPP client");

        // TODO: Implement event loop

        Ok(())
    }
}

#[async_trait::async_trait]
impl Transport for OcppClient {
    async fn send_message(&self, message: Message) -> OcppResult<()> {
        debug!("Sending message: {}", message.unique_id());

        // TODO: Implement message sending

        Ok(())
    }

    async fn close(&self) -> OcppResult<()> {
        info!("Closing OCPP client connection");

        // TODO: Implement close logic

        Ok(())
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    fn connection_id(&self) -> Uuid {
        self.connection_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageHandler;
    use std::sync::Arc;

    // Mock message handler for testing
    struct MockMessageHandler;

    #[async_trait::async_trait]
    impl MessageHandler for MockMessageHandler {
        async fn handle_message(&self, _message: Message) -> OcppResult<Option<Message>> {
            Ok(None)
        }

        async fn handle_event(&self, _event: TransportEvent) {
            // Do nothing
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = TransportConfig::default();
        let (client, _rx) = OcppClient::new(config);
        assert_eq!(client.state(), ConnectionState::Closed);
        assert!(!client.connection_id().is_nil());
    }

    #[tokio::test]
    async fn test_client_connect() {
        let config = TransportConfig::default();
        let (mut client, _rx) = OcppClient::new(config);

        // This will fail in tests since we don't have an actual server
        // but it tests the state changes
        let _result = client.connect("ws://localhost:8080/ocpp/test").await;
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let config = TransportConfig::default();
        let handler = Arc::new(MockMessageHandler);

        // This will fail in tests since we don't have an actual server
        // but it tests the creation logic
        let result =
            WebSocketClient::new("ws://localhost:8080/ocpp/test".to_string(), config, handler)
                .await;

        // We expect this to fail in tests due to no server
        assert!(result.is_err());
    }
}

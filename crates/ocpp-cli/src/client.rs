//! # OCPP WebSocket Client
//!
//! This module provides WebSocket client functionality for connecting to
//! OCPP Central Systems and handling message exchange.

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

/// WebSocket client for OCPP communication
pub struct OcppWebSocketClient {
    /// Connection URL
    url: String,
    /// Client configuration
    config: ClientConfig,
    /// Connection state
    state: ConnectionState,
    /// Message sender
    message_tx: Option<mpsc::UnboundedSender<String>>,
    /// Event receiver
    event_rx: Option<mpsc::UnboundedReceiver<ClientEvent>>,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Ping interval
    pub ping_interval: Duration,
    /// Maximum reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection delay
    pub reconnect_delay: Duration,
    /// Message buffer size
    pub message_buffer_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            ping_interval: Duration::from_secs(30),
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(5),
            message_buffer_size: 1000,
        }
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

/// Client events
#[derive(Debug, Clone)]
pub enum ClientEvent {
    Connected,
    Disconnected { reason: String },
    MessageReceived { message: String },
    MessageSent { message: String },
    Error { error: String },
    Pong,
}

/// OCPP message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcppMessage {
    /// Message type (2=Call, 3=CallResult, 4=CallError)
    pub message_type: u8,
    /// Unique message identifier
    pub message_id: String,
    /// Action name or error code
    pub action: String,
    /// Message payload
    pub payload: serde_json::Value,
}

impl OcppWebSocketClient {
    /// Create new WebSocket client
    pub fn new(url: String, config: ClientConfig) -> Self {
        Self {
            url,
            config,
            state: ConnectionState::Disconnected,
            message_tx: None,
            event_rx: None,
        }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        self.state = ConnectionState::Connecting;

        info!("Connecting to OCPP WebSocket: {}", self.url);

        let url = Url::parse(&self.url)?;

        // Attempt connection with timeout
        let connect_result =
            tokio::time::timeout(self.config.connect_timeout, connect_async(&url)).await;

        match connect_result {
            Ok(Ok((ws_stream, response))) => {
                info!(
                    "WebSocket connected successfully. Status: {}",
                    response.status()
                );
                self.state = ConnectionState::Connected;

                // Split the stream
                let (write, read) = ws_stream.split();

                // Setup channels
                let (message_tx, message_rx) = mpsc::unbounded_channel();
                let (event_tx, event_rx) = mpsc::unbounded_channel();

                self.message_tx = Some(message_tx);
                self.event_rx = Some(event_rx);

                // Spawn message handling tasks
                self.spawn_message_handler(write, message_rx, event_tx.clone())
                    .await;
                self.spawn_read_handler(read, event_tx.clone()).await;
                self.spawn_ping_handler(event_tx).await;

                Ok(())
            }
            Ok(Err(e)) => {
                error!("WebSocket connection failed: {}", e);
                self.state = ConnectionState::Failed;
                Err(anyhow::anyhow!("Connection failed: {}", e))
            }
            Err(_) => {
                error!("WebSocket connection timed out");
                self.state = ConnectionState::Failed;
                Err(anyhow::anyhow!("Connection timeout"))
            }
        }
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from WebSocket");
        self.state = ConnectionState::Disconnected;
        self.message_tx = None;
        self.event_rx = None;
        Ok(())
    }

    /// Send OCPP message
    pub async fn send_message(&self, message: OcppMessage) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            let json_str = serde_json::to_string(&message)?;
            tx.send(json_str)?;
            debug!("Queued OCPP message: {}", message.action);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Client not connected"))
        }
    }

    /// Send raw JSON message
    pub async fn send_raw_message(&self, message: &str) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(message.to_string())?;
            debug!("Queued raw message");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Client not connected"))
        }
    }

    /// Get next client event
    pub async fn next_event(&mut self) -> Option<ClientEvent> {
        if let Some(rx) = &mut self.event_rx {
            rx.recv().await
        } else {
            None
        }
    }

    /// Get connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Reconnect with retry logic
    pub async fn reconnect(&mut self) -> Result<()> {
        self.state = ConnectionState::Reconnecting;

        for attempt in 1..=self.config.max_reconnect_attempts {
            info!(
                "Reconnection attempt {} of {}",
                attempt, self.config.max_reconnect_attempts
            );

            match self.connect().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    warn!("Reconnection attempt {} failed: {}", attempt, e);
                    if attempt < self.config.max_reconnect_attempts {
                        tokio::time::sleep(self.config.reconnect_delay).await;
                    }
                }
            }
        }

        self.state = ConnectionState::Failed;
        Err(anyhow::anyhow!("Max reconnection attempts exceeded"))
    }

    // Private helper methods

    async fn spawn_message_handler(
        &self,
        mut write: futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            tokio_tungstenite::tungstenite::Message,
        >,
        mut message_rx: mpsc::UnboundedReceiver<String>,
        event_tx: mpsc::UnboundedSender<ClientEvent>,
    ) {
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                let ws_message = Message::Text(message.clone());

                match write.send(ws_message).await {
                    Ok(()) => {
                        debug!("Message sent successfully");
                        let _ = event_tx.send(ClientEvent::MessageSent { message });
                    }
                    Err(e) => {
                        error!("Failed to send message: {}", e);
                        let _ = event_tx.send(ClientEvent::Error {
                            error: format!("Send error: {}", e),
                        });
                        break;
                    }
                }
            }
        });
    }

    async fn spawn_read_handler(
        &self,
        mut read: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        event_tx: mpsc::UnboundedSender<ClientEvent>,
    ) {
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        debug!("Received text message: {}", text);
                        let _ = event_tx.send(ClientEvent::MessageReceived { message: text });
                    }
                    Ok(Message::Binary(data)) => {
                        debug!("Received binary message: {} bytes", data.len());
                        if let Ok(text) = String::from_utf8(data) {
                            let _ = event_tx.send(ClientEvent::MessageReceived { message: text });
                        }
                    }
                    Ok(Message::Ping(_)) => {
                        debug!("Received ping");
                        // Pong will be sent automatically by tungstenite
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received pong");
                        let _ = event_tx.send(ClientEvent::Pong);
                    }
                    Ok(Message::Close(frame)) => {
                        info!("WebSocket closed: {:?}", frame);
                        let _ = event_tx.send(ClientEvent::Disconnected {
                            reason: frame.map_or("Unknown".to_string(), |f| f.reason.to_string()),
                        });
                        break;
                    }
                    Ok(Message::Frame(_)) => {
                        debug!("Received frame message");
                        // Frame messages are handled internally by tungstenite
                    }
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        let _ = event_tx.send(ClientEvent::Error {
                            error: format!("Read error: {}", e),
                        });
                        break;
                    }
                }
            }
        });
    }

    async fn spawn_ping_handler(&self, _event_tx: mpsc::UnboundedSender<ClientEvent>) {
        let interval = self.config.ping_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                // In a real implementation, this would send ping frames
                // For now, we just simulate the ping functionality
                debug!("Ping interval tick");

                // This would normally check if we've received recent messages
                // and send a ping if needed
            }
        });
    }
}

/// Utility functions for OCPP message handling
pub mod utils {
    use super::*;

    /// Parse OCPP message from JSON string
    pub fn parse_ocpp_message(json: &str) -> Result<OcppMessage> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        if let Some(array) = value.as_array() {
            if array.len() >= 3 {
                let message_type = array[0].as_u64().unwrap_or(0) as u8;
                let message_id = array[1].as_str().unwrap_or("").to_string();

                match message_type {
                    2 => {
                        // Call message [2, "messageId", "action", payload]
                        let action = array[2].as_str().unwrap_or("").to_string();
                        let payload = array.get(3).cloned().unwrap_or(serde_json::Value::Null);

                        Ok(OcppMessage {
                            message_type,
                            message_id,
                            action,
                            payload,
                        })
                    }
                    3 => {
                        // CallResult message [3, "messageId", payload]
                        let payload = array.get(2).cloned().unwrap_or(serde_json::Value::Null);

                        Ok(OcppMessage {
                            message_type,
                            message_id,
                            action: "CallResult".to_string(),
                            payload,
                        })
                    }
                    4 => {
                        // CallError message [4, "messageId", "errorCode", "errorDescription", errorDetails]
                        let error_code = array[2].as_str().unwrap_or("").to_string();
                        let error_description = array[3].as_str().unwrap_or("").to_string();
                        let error_details =
                            array.get(4).cloned().unwrap_or(serde_json::Value::Null);

                        Ok(OcppMessage {
                            message_type,
                            message_id,
                            action: error_code,
                            payload: serde_json::json!({
                                "errorDescription": error_description,
                                "errorDetails": error_details
                            }),
                        })
                    }
                    _ => Err(anyhow::anyhow!("Invalid message type: {}", message_type)),
                }
            } else {
                Err(anyhow::anyhow!(
                    "Invalid OCPP message format: insufficient array elements"
                ))
            }
        } else {
            Err(anyhow::anyhow!("Invalid OCPP message format: not an array"))
        }
    }

    /// Convert OCPP message to JSON string
    pub fn format_ocpp_message(message: &OcppMessage) -> Result<String> {
        let json_array = match message.message_type {
            2 => {
                // Call message [2, "messageId", "action", payload]
                serde_json::json!([
                    message.message_type,
                    message.message_id,
                    message.action,
                    message.payload
                ])
            }
            3 => {
                // CallResult message [3, "messageId", payload]
                serde_json::json!([message.message_type, message.message_id, message.payload])
            }
            4 => {
                // CallError message [4, "messageId", "errorCode", "errorDescription", errorDetails]
                let error_description = message
                    .payload
                    .get("errorDescription")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let error_details = message
                    .payload
                    .get("errorDetails")
                    .unwrap_or(&serde_json::Value::Null);

                serde_json::json!([
                    message.message_type,
                    message.message_id,
                    message.action,
                    error_description,
                    error_details
                ])
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid message type: {}",
                    message.message_type
                ));
            }
        };

        Ok(serde_json::to_string(&json_array)?)
    }

    /// Generate unique message ID
    pub fn generate_message_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Validate OCPP message format
    pub fn validate_message_format(json: &str) -> Result<()> {
        parse_ocpp_message(json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 5);
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_parse_call_message() {
        let json = r#"[2, "12345", "Heartbeat", {}]"#;
        let message = utils::parse_ocpp_message(json).unwrap();

        assert_eq!(message.message_type, 2);
        assert_eq!(message.message_id, "12345");
        assert_eq!(message.action, "Heartbeat");
    }

    #[test]
    fn test_parse_call_result_message() {
        let json = r#"[3, "12345", {"currentTime": "2024-01-01T10:00:00Z"}]"#;
        let message = utils::parse_ocpp_message(json).unwrap();

        assert_eq!(message.message_type, 3);
        assert_eq!(message.message_id, "12345");
        assert_eq!(message.action, "CallResult");
    }

    #[test]
    fn test_parse_call_error_message() {
        let json = r#"[4, "12345", "InternalError", "Something went wrong", {}]"#;
        let message = utils::parse_ocpp_message(json).unwrap();

        assert_eq!(message.message_type, 4);
        assert_eq!(message.message_id, "12345");
        assert_eq!(message.action, "InternalError");
    }

    #[test]
    fn test_format_call_message() {
        let message = OcppMessage {
            message_type: 2,
            message_id: "test-123".to_string(),
            action: "BootNotification".to_string(),
            payload: serde_json::json!({"chargePointVendor": "Test"}),
        };

        let json = utils::format_ocpp_message(&message).unwrap();
        assert!(json.contains("BootNotification"));
        assert!(json.contains("test-123"));
    }

    #[test]
    fn test_generate_message_id() {
        let id1 = utils::generate_message_id();
        let id2 = utils::generate_message_id();

        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_validate_message_format() {
        let valid_json = r#"[2, "12345", "Heartbeat", {}]"#;
        assert!(utils::validate_message_format(valid_json).is_ok());

        let invalid_json = r#"{"not": "an array"}"#;
        assert!(utils::validate_message_format(invalid_json).is_err());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = ClientConfig::default();
        let client = OcppWebSocketClient::new("wss://example.com/ocpp/test".to_string(), config);

        assert_eq!(client.state(), ConnectionState::Disconnected);
        assert!(!client.is_connected());
    }
}

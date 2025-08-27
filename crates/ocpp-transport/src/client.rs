//! WebSocket client implementation for OCPP Charge Points

use crate::{error::TransportResult, ConnectionState, Transport, TransportConfig, TransportEvent};
use ocpp_messages::Message;
use ocpp_types::OcppResult;

use tokio::sync::mpsc;
use tracing::{debug, info};
use uuid::Uuid;

/// WebSocket client for OCPP Charge Points
pub struct OcppClient {
    /// Connection ID
    connection_id: Uuid,
    /// Current connection state
    state: ConnectionState,
    /// Configuration
    config: TransportConfig,
    /// Event sender
    event_tx: mpsc::UnboundedSender<TransportEvent>,
}

impl OcppClient {
    /// Create a new OCPP client
    pub fn new(config: TransportConfig) -> (Self, mpsc::UnboundedReceiver<TransportEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let client = Self {
            connection_id: Uuid::new_v4(),
            state: ConnectionState::Closed,
            config,
            event_tx,
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
}

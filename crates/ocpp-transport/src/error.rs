//! Transport layer error types

use thiserror::Error;

/// Transport layer errors
#[derive(Error, Debug, Clone)]
pub enum TransportError {
    /// WebSocket connection error
    #[error("WebSocket connection error: {message}")]
    ConnectionError { message: String },

    /// WebSocket protocol error
    #[error("WebSocket protocol error: {message}")]
    ProtocolError { message: String },

    /// Message serialization error
    #[error("Message serialization error: {message}")]
    SerializationError { message: String },

    /// Message too large
    #[error("Message too large: {size} bytes exceeds limit of {limit} bytes")]
    MessageTooLarge { size: usize, limit: usize },

    /// Connection timeout
    #[error("Connection timeout after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    /// Connection closed unexpectedly
    #[error("Connection closed: {reason}")]
    ConnectionClosed { reason: String },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// Invalid subprotocol
    #[error("Invalid subprotocol: {protocol}")]
    InvalidSubprotocol { protocol: String },

    /// Buffer overflow
    #[error("Buffer overflow: {buffer_type}")]
    BufferOverflow { buffer_type: String },

    /// IO error
    #[error("IO error: {message}")]
    IoError { message: String },

    /// TLS error
    #[error("TLS error: {message}")]
    TlsError { message: String },

    /// Handshake error
    #[error("WebSocket handshake error: {message}")]
    HandshakeError { message: String },

    /// Connection not ready
    #[error("Connection not ready, current state: {state:?}")]
    NotReady { state: crate::ConnectionState },

    /// Internal error
    #[error("Internal transport error: {message}")]
    Internal { message: String },
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        TransportError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tungstenite::Error> for TransportError {
    fn from(err: tungstenite::Error) -> Self {
        match err {
            tungstenite::Error::ConnectionClosed => TransportError::ConnectionClosed {
                reason: "WebSocket connection closed".to_string(),
            },
            tungstenite::Error::AlreadyClosed => TransportError::ConnectionClosed {
                reason: "WebSocket already closed".to_string(),
            },
            tungstenite::Error::Protocol(msg) => TransportError::ProtocolError {
                message: msg.to_string(),
            },
            tungstenite::Error::Io(io_err) => TransportError::IoError {
                message: io_err.to_string(),
            },
            tungstenite::Error::Tls(tls_err) => TransportError::TlsError {
                message: tls_err.to_string(),
            },
            _ => TransportError::Internal {
                message: err.to_string(),
            },
        }
    }
}

impl From<ocpp_types::OcppError> for TransportError {
    fn from(err: ocpp_types::OcppError) -> Self {
        match err {
            ocpp_types::OcppError::Json { message } => {
                TransportError::SerializationError { message }
            }
            ocpp_types::OcppError::Timeout { operation: _ } => TransportError::Timeout {
                timeout_secs: 30, // Default timeout
            },
            ocpp_types::OcppError::Transport { message } => TransportError::Internal { message },
            _ => TransportError::Internal {
                message: err.to_string(),
            },
        }
    }
}

/// Result type for transport operations
pub type TransportResult<T> = Result<T, TransportError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_display() {
        let error = TransportError::MessageTooLarge {
            size: 100000,
            limit: 65536,
        };
        let message = error.to_string();
        assert!(message.contains("100000"));
        assert!(message.contains("65536"));
    }

    #[test]
    fn test_error_conversion_from_io() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let transport_error = TransportError::from(io_error);

        match transport_error {
            TransportError::IoError { message } => {
                assert!(message.contains("Connection refused"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_conversion_from_serde() {
        let serde_error = serde_json::from_str::<i32>("invalid json").unwrap_err();
        let transport_error = TransportError::from(serde_error);

        match transport_error {
            TransportError::SerializationError { message } => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_conversion_from_tungstenite() {
        let ws_error = tungstenite::Error::ConnectionClosed;
        let transport_error = TransportError::from(ws_error);

        match transport_error {
            TransportError::ConnectionClosed { reason } => {
                assert!(reason.contains("closed"));
            }
            _ => panic!("Expected ConnectionClosed"),
        }
    }
}

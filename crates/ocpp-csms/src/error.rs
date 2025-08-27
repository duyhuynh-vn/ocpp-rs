//! Error types for OCPP CSMS operations

use thiserror::Error;

/// Main error type for CSMS operations
#[derive(Error, Debug)]
pub enum CsmsError {
    /// Database error
    #[error("Database error: {message}")]
    Database { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Transport error
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Authentication error
    #[error("Authentication error: {message}")]
    Authentication { message: String },

    /// Authorization error
    #[error("Authorization error: {message}")]
    Authorization { message: String },

    /// Validation error
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Charge point not found
    #[error("Charge point not found: {charge_point_id}")]
    ChargePointNotFound { charge_point_id: String },

    /// Transaction not found
    #[error("Transaction not found: {transaction_id}")]
    TransactionNotFound { transaction_id: i32 },

    /// Connector not available
    #[error("Connector not available: {charge_point_id}/{connector_id}")]
    ConnectorNotAvailable {
        charge_point_id: String,
        connector_id: u32,
    },

    /// Internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl From<sqlx::Error> for CsmsError {
    fn from(err: sqlx::Error) -> Self {
        CsmsError::Database {
            message: err.to_string(),
        }
    }
}

impl From<ocpp_types::OcppError> for CsmsError {
    fn from(err: ocpp_types::OcppError) -> Self {
        match err {
            ocpp_types::OcppError::Database { message } => CsmsError::Database { message },
            ocpp_types::OcppError::ValidationError { message } => CsmsError::Validation { message },
            ocpp_types::OcppError::Transport { message } => CsmsError::Transport { message },
            _ => CsmsError::Internal {
                message: err.to_string(),
            },
        }
    }
}

impl From<ocpp_transport::TransportError> for CsmsError {
    fn from(err: ocpp_transport::TransportError) -> Self {
        CsmsError::Transport {
            message: err.to_string(),
        }
    }
}

/// Result type for CSMS operations
pub type CsmsResult<T> = Result<T, CsmsError>;

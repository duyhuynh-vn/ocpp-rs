//! # Charge Point Error Handling
//!
//! This module provides comprehensive error handling for the OCPP charge point
//! implementation, including specific error types for different failure modes.

use thiserror::Error;

/// Charge point error types
#[derive(Error, Debug, Clone)]
pub enum ChargePointError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Protocol violation
    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    /// Message serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Authentication/authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Invalid operation for current state
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Connector error
    #[error("Connector error on connector {connector_id}: {message}")]
    ConnectorError { connector_id: u32, message: String },

    /// Transaction error
    #[error("Transaction error (ID: {transaction_id}): {message}")]
    TransactionError {
        transaction_id: i32,
        message: String,
    },

    /// Hardware fault
    #[error("Hardware fault: {fault_type} - {description}")]
    HardwareFault {
        fault_type: String,
        description: String,
    },

    /// Timeout error
    #[error("Timeout: {operation} after {duration_ms}ms")]
    Timeout { operation: String, duration_ms: u64 },

    /// Resource unavailable
    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },

    /// Boot notification failed
    #[error("Boot notification failed: {reason}")]
    BootNotificationFailed { reason: String },

    /// Central system error
    #[error("Central system error: {0}")]
    CentralSystemError(String),

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not supported
    #[error("Feature not supported: {feature}")]
    NotSupported { feature: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for {operation}: {limit} per {window}")]
    RateLimitExceeded {
        operation: String,
        limit: u32,
        window: String,
    },
}

impl ChargePointError {
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::ConnectionError(message.into())
    }

    /// Create a transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::TransportError(message.into())
    }

    /// Create a protocol violation error
    pub fn protocol_violation(message: impl Into<String>) -> Self {
        Self::ProtocolViolation(message.into())
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Create an authorization failed error
    pub fn authorization_failed(message: impl Into<String>) -> Self {
        Self::AuthorizationFailed(message.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::InvalidOperation(message.into())
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a connector error
    pub fn connector(connector_id: u32, message: impl Into<String>) -> Self {
        Self::ConnectorError {
            connector_id,
            message: message.into(),
        }
    }

    /// Create a transaction error
    pub fn transaction(transaction_id: i32, message: impl Into<String>) -> Self {
        Self::TransactionError {
            transaction_id,
            message: message.into(),
        }
    }

    /// Create a hardware fault error
    pub fn hardware_fault(fault_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self::HardwareFault {
            fault_type: fault_type.into(),
            description: description.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_ms,
        }
    }

    /// Create a resource unavailable error
    pub fn resource_unavailable(resource: impl Into<String>) -> Self {
        Self::ResourceUnavailable {
            resource: resource.into(),
        }
    }

    /// Create a boot notification failed error
    pub fn boot_notification_failed(reason: impl Into<String>) -> Self {
        Self::BootNotificationFailed {
            reason: reason.into(),
        }
    }

    /// Create a central system error
    pub fn central_system(message: impl Into<String>) -> Self {
        Self::CentralSystemError(message.into())
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Create a not supported error
    pub fn not_supported(feature: impl Into<String>) -> Self {
        Self::NotSupported {
            feature: feature.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(
        operation: impl Into<String>,
        limit: u32,
        window: impl Into<String>,
    ) -> Self {
        Self::RateLimitExceeded {
            operation: operation.into(),
            limit,
            window: window.into(),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors
            Self::ConnectionError(_)
            | Self::TransportError(_)
            | Self::Timeout { .. }
            | Self::ResourceUnavailable { .. }
            | Self::CentralSystemError(_)
            | Self::RateLimitExceeded { .. } => true,

            // Non-recoverable errors
            Self::ProtocolViolation(_)
            | Self::SerializationError(_)
            | Self::AuthorizationFailed(_)
            | Self::ConfigurationError(_)
            | Self::HardwareFault { .. }
            | Self::ValidationError { .. }
            | Self::InternalError(_)
            | Self::NotSupported { .. }
            | Self::InvalidOperation(_)
            | Self::ConnectorError { .. }
            | Self::TransactionError { .. }
            | Self::BootNotificationFailed { .. } => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ValidationError { .. } | Self::InvalidOperation(_) => ErrorSeverity::Warning,

            Self::ConnectionError(_)
            | Self::TransportError(_)
            | Self::AuthorizationFailed(_)
            | Self::Timeout { .. }
            | Self::ResourceUnavailable { .. }
            | Self::CentralSystemError(_)
            | Self::RateLimitExceeded { .. } => ErrorSeverity::Error,

            Self::ProtocolViolation(_)
            | Self::SerializationError(_)
            | Self::ConfigurationError(_)
            | Self::HardwareFault { .. }
            | Self::ConnectorError { .. }
            | Self::TransactionError { .. }
            | Self::BootNotificationFailed { .. }
            | Self::InternalError(_)
            | Self::NotSupported { .. } => ErrorSeverity::Critical,
        }
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ConnectionError(_) | Self::TransportError(_) => ErrorCategory::Network,
            Self::ProtocolViolation(_) | Self::SerializationError(_) => ErrorCategory::Protocol,
            Self::AuthorizationFailed(_) => ErrorCategory::Authentication,
            Self::ConfigurationError(_) | Self::ValidationError { .. } => {
                ErrorCategory::Configuration
            }
            Self::ConnectorError { .. } | Self::HardwareFault { .. } => ErrorCategory::Hardware,
            Self::TransactionError { .. } => ErrorCategory::Transaction,
            Self::Timeout { .. } => ErrorCategory::Performance,
            Self::ResourceUnavailable { .. } | Self::RateLimitExceeded { .. } => {
                ErrorCategory::Resource
            }
            Self::BootNotificationFailed { .. } | Self::CentralSystemError(_) => {
                ErrorCategory::System
            }
            Self::InvalidOperation(_) | Self::InternalError(_) | Self::NotSupported { .. } => {
                ErrorCategory::Logic
            }
        }
    }

    /// Get suggested retry delay in milliseconds
    pub fn suggested_retry_delay(&self) -> Option<u64> {
        match self {
            Self::ConnectionError(_) => Some(5000),         // 5 seconds
            Self::TransportError(_) => Some(3000),          // 3 seconds
            Self::Timeout { .. } => Some(1000),             // 1 second
            Self::ResourceUnavailable { .. } => Some(2000), // 2 seconds
            Self::CentralSystemError(_) => Some(10000),     // 10 seconds
            Self::RateLimitExceeded { .. } => Some(60000),  // 1 minute
            _ => None,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Warning - operation can continue
    Warning,
    /// Error - operation failed but system is stable
    Error,
    /// Critical - system stability may be compromised
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Network/connection related errors
    Network,
    /// Protocol/message format errors
    Protocol,
    /// Authentication/authorization errors
    Authentication,
    /// Configuration errors
    Configuration,
    /// Hardware/connector errors
    Hardware,
    /// Transaction processing errors
    Transaction,
    /// Performance/timeout errors
    Performance,
    /// Resource availability errors
    Resource,
    /// System/central system errors
    System,
    /// Logic/programming errors
    Logic,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Protocol => write!(f, "Protocol"),
            ErrorCategory::Authentication => write!(f, "Authentication"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Hardware => write!(f, "Hardware"),
            ErrorCategory::Transaction => write!(f, "Transaction"),
            ErrorCategory::Performance => write!(f, "Performance"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::System => write!(f, "System"),
            ErrorCategory::Logic => write!(f, "Logic"),
        }
    }
}

/// Error context for additional debugging information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error source location
    pub source: Option<String>,
    /// Additional context data
    pub context: std::collections::HashMap<String, String>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new() -> Self {
        Self {
            source: None,
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create with source location
    pub fn with_source(source: impl Into<String>) -> Self {
        Self {
            source: Some(source.into()),
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add context data
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended error with context
#[derive(Debug, Clone)]
pub struct ExtendedError {
    /// Base error
    pub error: ChargePointError,
    /// Error context
    pub context: ErrorContext,
    /// Error ID for tracking
    pub id: uuid::Uuid,
}

impl ExtendedError {
    /// Create new extended error
    pub fn new(error: ChargePointError) -> Self {
        Self {
            error,
            context: ErrorContext::new(),
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Create with context
    pub fn with_context(error: ChargePointError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Add context data
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context = self.context.add_context(key, value);
        self
    }
}

impl std::fmt::Display for ExtendedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.id, self.error)?;
        if let Some(source) = &self.context.source {
            write!(f, " (source: {})", source)?;
        }
        if !self.context.context.is_empty() {
            write!(f, " context: {:?}", self.context.context)?;
        }
        Ok(())
    }
}

impl std::error::Error for ExtendedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Result type for charge point operations
pub type ChargePointResult<T> = Result<T, ChargePointError>;

/// Result type for extended errors
pub type ExtendedResult<T> = Result<T, ExtendedError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ChargePointError::connection("Connection timeout");
        assert!(matches!(error, ChargePointError::ConnectionError(_)));
        assert_eq!(error.to_string(), "Connection error: Connection timeout");
    }

    #[test]
    fn test_error_properties() {
        let error = ChargePointError::connection("Test");
        assert!(error.is_recoverable());
        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert_eq!(error.category(), ErrorCategory::Network);
        assert_eq!(error.suggested_retry_delay(), Some(5000));

        let critical_error =
            ChargePointError::hardware_fault("Overheating", "Temperature too high");
        assert!(!critical_error.is_recoverable());
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);
        assert_eq!(critical_error.category(), ErrorCategory::Hardware);
        assert_eq!(critical_error.suggested_retry_delay(), None);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::with_source("connector.rs:123")
            .add_context("connector_id", "1")
            .add_context("operation", "start_transaction");

        assert_eq!(context.source, Some("connector.rs:123".to_string()));
        assert_eq!(context.get_context("connector_id"), Some(&"1".to_string()));
        assert_eq!(
            context.get_context("operation"),
            Some(&"start_transaction".to_string())
        );
    }

    #[test]
    fn test_extended_error() {
        let error = ChargePointError::connector(1, "Cable lock failed");
        let context = ErrorContext::with_source("connector.rs:456").add_context("retry_count", "3");

        let extended = ExtendedError::with_context(error, context)
            .add_context("timestamp", "2024-01-01T12:00:00Z");

        assert!(!extended.id.is_nil());
        assert!(extended.to_string().contains("Cable lock failed"));
        assert!(extended
            .to_string()
            .contains(extended.id.to_string().as_str()));
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            ChargePointError::connection("test"),
            ChargePointError::transport("test"),
            ChargePointError::protocol_violation("test"),
            ChargePointError::serialization("test"),
            ChargePointError::authorization_failed("test"),
            ChargePointError::invalid_operation("test"),
            ChargePointError::configuration("test"),
            ChargePointError::connector(1, "test"),
            ChargePointError::transaction(123, "test"),
            ChargePointError::hardware_fault("type", "desc"),
            ChargePointError::timeout("op", 1000),
            ChargePointError::resource_unavailable("resource"),
            ChargePointError::boot_notification_failed("reason"),
            ChargePointError::central_system("test"),
            ChargePointError::validation("field", "msg"),
            ChargePointError::internal("test"),
            ChargePointError::not_supported("feature"),
            ChargePointError::rate_limit_exceeded("op", 10, "minute"),
        ];

        // Ensure all variants can be created and have string representation
        for error in errors {
            assert!(!error.to_string().is_empty());
            let _severity = error.severity();
            let _category = error.category();
            let _recoverable = error.is_recoverable();
        }
    }
}

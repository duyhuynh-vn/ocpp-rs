//! # Simulator Error Handling
//!
//! This module provides error types and handling for the OCPP simulator,
//! including specific error types for different failure modes in simulation scenarios.

use thiserror::Error;

/// Simulator error types
#[derive(Error, Debug, Clone)]
pub enum SimulatorError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Charge point error
    #[error("Charge point error: {0}")]
    ChargePointError(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// API server error
    #[error("API server error: {0}")]
    ApiServerError(String),

    /// Scenario execution error
    #[error("Scenario execution error: {scenario} - {message}")]
    ScenarioExecutionError { scenario: String, message: String },

    /// Scenario validation error
    #[error("Scenario validation error: {scenario} - {message}")]
    ScenarioValidationError { scenario: String, message: String },

    /// Fault injection error
    #[error("Fault injection error on connector {connector_id}: {message}")]
    FaultInjectionError { connector_id: u32, message: String },

    /// Meter simulation error
    #[error("Meter simulation error: {0}")]
    MeterSimulationError(String),

    /// Event handling error
    #[error("Event handling error: {0}")]
    EventHandlingError(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Timeout error
    #[error("Timeout: {operation} after {duration_ms}ms")]
    TimeoutError { operation: String, duration_ms: u64 },

    /// Invalid state error
    #[error("Invalid state for operation: {operation} in state {current_state}")]
    InvalidStateError {
        operation: String,
        current_state: String,
    },

    /// Resource not found error
    #[error("Resource not found: {resource_type} '{resource_id}'")]
    ResourceNotFoundError {
        resource_type: String,
        resource_id: String,
    },

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// External service error
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {operation} - {limit} requests per {window}")]
    RateLimitError {
        operation: String,
        limit: u32,
        window: String,
    },

    /// Concurrent operation error
    #[error("Concurrent operation not allowed: {0}")]
    ConcurrentOperationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not supported error
    #[error("Feature not supported: {feature}")]
    NotSupportedError { feature: String },
}

impl SimulatorError {
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a charge point error
    pub fn charge_point(message: impl Into<String>) -> Self {
        Self::ChargePointError(message.into())
    }

    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::ConnectionError(message.into())
    }

    /// Create an API server error
    pub fn api_server(message: impl Into<String>) -> Self {
        Self::ApiServerError(message.into())
    }

    /// Create a scenario execution error
    pub fn scenario_execution(scenario: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ScenarioExecutionError {
            scenario: scenario.into(),
            message: message.into(),
        }
    }

    /// Create a scenario validation error
    pub fn scenario_validation(scenario: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ScenarioValidationError {
            scenario: scenario.into(),
            message: message.into(),
        }
    }

    /// Create a fault injection error
    pub fn fault_injection(connector_id: u32, message: impl Into<String>) -> Self {
        Self::FaultInjectionError {
            connector_id,
            message: message.into(),
        }
    }

    /// Create a meter simulation error
    pub fn meter_simulation(message: impl Into<String>) -> Self {
        Self::MeterSimulationError(message.into())
    }

    /// Create an event handling error
    pub fn event_handling(message: impl Into<String>) -> Self {
        Self::EventHandlingError(message.into())
    }

    /// Create a WebSocket error
    pub fn websocket(message: impl Into<String>) -> Self {
        Self::WebSocketError(message.into())
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Create an I/O error
    pub fn io(message: impl Into<String>) -> Self {
        Self::IoError(message.into())
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration_ms: u64) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            duration_ms,
        }
    }

    /// Create an invalid state error
    pub fn invalid_state(operation: impl Into<String>, current_state: impl Into<String>) -> Self {
        Self::InvalidStateError {
            operation: operation.into(),
            current_state: current_state.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self::ResourceNotFoundError {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalServiceError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::AuthenticationError(message.into())
    }

    /// Create an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::AuthorizationError(message.into())
    }

    /// Create a rate limit error
    pub fn rate_limit(operation: impl Into<String>, limit: u32, window: impl Into<String>) -> Self {
        Self::RateLimitError {
            operation: operation.into(),
            limit,
            window: window.into(),
        }
    }

    /// Create a concurrent operation error
    pub fn concurrent_operation(message: impl Into<String>) -> Self {
        Self::ConcurrentOperationError(message.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Create a not supported error
    pub fn not_supported(feature: impl Into<String>) -> Self {
        Self::NotSupportedError {
            feature: feature.into(),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors - can be retried or worked around
            Self::ConnectionError(_)
            | Self::TimeoutError { .. }
            | Self::ExternalServiceError { .. }
            | Self::RateLimitError { .. }
            | Self::ConcurrentOperationError(_)
            | Self::WebSocketError(_) => true,

            // Non-recoverable errors - require intervention
            Self::ConfigurationError(_)
            | Self::ValidationError { .. }
            | Self::AuthenticationError(_)
            | Self::AuthorizationError(_)
            | Self::NotSupportedError { .. }
            | Self::SerializationError(_)
            | Self::ScenarioValidationError { .. }
            | Self::InvalidStateError { .. }
            | Self::InternalError(_) => false,

            // Context-dependent errors
            Self::ChargePointError(_)
            | Self::ApiServerError(_)
            | Self::ScenarioExecutionError { .. }
            | Self::FaultInjectionError { .. }
            | Self::MeterSimulationError(_)
            | Self::EventHandlingError(_)
            | Self::IoError(_)
            | Self::ResourceNotFoundError { .. } => false,
        }
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ConfigurationError(_) | Self::ValidationError { .. } => {
                ErrorCategory::Configuration
            }
            Self::ChargePointError(_) => ErrorCategory::ChargePoint,
            Self::ConnectionError(_) | Self::WebSocketError(_) => ErrorCategory::Network,
            Self::ApiServerError(_) => ErrorCategory::Api,
            Self::ScenarioExecutionError { .. } | Self::ScenarioValidationError { .. } => {
                ErrorCategory::Scenario
            }
            Self::FaultInjectionError { .. } => ErrorCategory::FaultInjection,
            Self::MeterSimulationError(_) => ErrorCategory::MeterSimulation,
            Self::EventHandlingError(_) => ErrorCategory::EventHandling,
            Self::SerializationError(_) => ErrorCategory::Serialization,
            Self::IoError(_) => ErrorCategory::Io,
            Self::TimeoutError { .. } => ErrorCategory::Timeout,
            Self::InvalidStateError { .. } => ErrorCategory::State,
            Self::ResourceNotFoundError { .. } => ErrorCategory::Resource,
            Self::ExternalServiceError { .. } => ErrorCategory::ExternalService,
            Self::AuthenticationError(_) | Self::AuthorizationError(_) => ErrorCategory::Auth,
            Self::RateLimitError { .. } => ErrorCategory::RateLimit,
            Self::ConcurrentOperationError(_) => ErrorCategory::Concurrency,
            Self::InternalError(_) => ErrorCategory::Internal,
            Self::NotSupportedError { .. } => ErrorCategory::NotSupported,
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ValidationError { .. }
            | Self::InvalidStateError { .. }
            | Self::ResourceNotFoundError { .. }
            | Self::NotSupportedError { .. } => ErrorSeverity::Warning,

            Self::ConnectionError(_)
            | Self::TimeoutError { .. }
            | Self::ExternalServiceError { .. }
            | Self::RateLimitError { .. }
            | Self::ConcurrentOperationError(_)
            | Self::WebSocketError(_)
            | Self::IoError(_)
            | Self::EventHandlingError(_)
            | Self::MeterSimulationError(_)
            | Self::FaultInjectionError { .. } => ErrorSeverity::Error,

            Self::ConfigurationError(_)
            | Self::ChargePointError(_)
            | Self::ApiServerError(_)
            | Self::ScenarioExecutionError { .. }
            | Self::ScenarioValidationError { .. }
            | Self::SerializationError(_)
            | Self::AuthenticationError(_)
            | Self::AuthorizationError(_)
            | Self::InternalError(_) => ErrorSeverity::Critical,
        }
    }

    /// Get suggested retry delay in milliseconds
    pub fn suggested_retry_delay(&self) -> Option<u64> {
        match self {
            Self::ConnectionError(_) => Some(5000),           // 5 seconds
            Self::WebSocketError(_) => Some(3000),            // 3 seconds
            Self::TimeoutError { .. } => Some(1000),          // 1 second
            Self::ExternalServiceError { .. } => Some(10000), // 10 seconds
            Self::RateLimitError { .. } => Some(60000),       // 1 minute
            Self::ConcurrentOperationError(_) => Some(500),   // 500ms
            _ => None,
        }
    }
}

/// Error categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Configuration,
    ChargePoint,
    Network,
    Api,
    Scenario,
    FaultInjection,
    MeterSimulation,
    EventHandling,
    Serialization,
    Io,
    Timeout,
    State,
    Resource,
    ExternalService,
    Auth,
    RateLimit,
    Concurrency,
    Internal,
    NotSupported,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::ChargePoint => write!(f, "ChargePoint"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Api => write!(f, "API"),
            ErrorCategory::Scenario => write!(f, "Scenario"),
            ErrorCategory::FaultInjection => write!(f, "FaultInjection"),
            ErrorCategory::MeterSimulation => write!(f, "MeterSimulation"),
            ErrorCategory::EventHandling => write!(f, "EventHandling"),
            ErrorCategory::Serialization => write!(f, "Serialization"),
            ErrorCategory::Io => write!(f, "IO"),
            ErrorCategory::Timeout => write!(f, "Timeout"),
            ErrorCategory::State => write!(f, "State"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::ExternalService => write!(f, "ExternalService"),
            ErrorCategory::Auth => write!(f, "Authentication"),
            ErrorCategory::RateLimit => write!(f, "RateLimit"),
            ErrorCategory::Concurrency => write!(f, "Concurrency"),
            ErrorCategory::Internal => write!(f, "Internal"),
            ErrorCategory::NotSupported => write!(f, "NotSupported"),
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Warning,
    Error,
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

/// Error context for additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Source location where error occurred
    pub source: Option<String>,
    /// Additional context data
    pub context: std::collections::HashMap<String, String>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request ID for tracing (if applicable)
    pub request_id: Option<String>,
    /// User ID associated with operation (if applicable)
    pub user_id: Option<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new() -> Self {
        Self {
            source: None,
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
        }
    }

    /// Create with source location
    pub fn with_source(source: impl Into<String>) -> Self {
        Self {
            source: Some(source.into()),
            context: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
        }
    }

    /// Add context data
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended error with context
#[derive(Debug, Clone)]
pub struct ExtendedSimulatorError {
    /// Base error
    pub error: SimulatorError,
    /// Error context
    pub context: ErrorContext,
    /// Error ID for tracking
    pub id: uuid::Uuid,
}

impl ExtendedSimulatorError {
    /// Create new extended error
    pub fn new(error: SimulatorError) -> Self {
        Self {
            error,
            context: ErrorContext::new(),
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Create with context
    pub fn with_context(error: SimulatorError, context: ErrorContext) -> Self {
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

impl std::fmt::Display for ExtendedSimulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.id, self.error)?;
        if let Some(source) = &self.context.source {
            write!(f, " (source: {})", source)?;
        }
        if let Some(request_id) = &self.context.request_id {
            write!(f, " (request: {})", request_id)?;
        }
        Ok(())
    }
}

impl std::error::Error for ExtendedSimulatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Result type for simulator operations
pub type SimulatorResult<T> = Result<T, SimulatorError>;

/// Result type for extended errors
pub type ExtendedResult<T> = Result<T, ExtendedSimulatorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = SimulatorError::configuration("Invalid config");
        assert!(matches!(error, SimulatorError::ConfigurationError(_)));
        assert_eq!(error.to_string(), "Configuration error: Invalid config");
    }

    #[test]
    fn test_error_properties() {
        let error = SimulatorError::connection("Connection failed");
        assert!(error.is_recoverable());
        assert_eq!(error.category(), ErrorCategory::Network);
        assert_eq!(error.severity(), ErrorSeverity::Error);
        assert_eq!(error.suggested_retry_delay(), Some(5000));

        let critical_error = SimulatorError::internal("System failure");
        assert!(!critical_error.is_recoverable());
        assert_eq!(critical_error.category(), ErrorCategory::Internal);
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);
        assert_eq!(critical_error.suggested_retry_delay(), None);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::with_source("simulator.rs:123")
            .add_context("connector_id", "1")
            .add_context("operation", "plug_in")
            .with_request_id("req-123")
            .with_user_id("user-456");

        assert_eq!(context.source, Some("simulator.rs:123".to_string()));
        assert_eq!(context.context.get("connector_id"), Some(&"1".to_string()));
        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.user_id, Some("user-456".to_string()));
    }

    #[test]
    fn test_extended_error() {
        let error = SimulatorError::fault_injection(1, "Test fault");
        let context = ErrorContext::with_source("simulator.rs:456")
            .add_context("error_code", "OverCurrentFailure");

        let extended =
            ExtendedSimulatorError::with_context(error, context).add_context("retry_count", "3");

        assert!(!extended.id.is_nil());
        assert!(extended.to_string().contains("Test fault"));
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
            SimulatorError::configuration("test"),
            SimulatorError::charge_point("test"),
            SimulatorError::connection("test"),
            SimulatorError::api_server("test"),
            SimulatorError::scenario_execution("scenario", "message"),
            SimulatorError::scenario_validation("scenario", "message"),
            SimulatorError::fault_injection(1, "test"),
            SimulatorError::meter_simulation("test"),
            SimulatorError::event_handling("test"),
            SimulatorError::websocket("test"),
            SimulatorError::serialization("test"),
            SimulatorError::io("test"),
            SimulatorError::timeout("op", 1000),
            SimulatorError::invalid_state("op", "state"),
            SimulatorError::resource_not_found("type", "id"),
            SimulatorError::validation("field", "message"),
            SimulatorError::external_service("service", "message"),
            SimulatorError::authentication("test"),
            SimulatorError::authorization("test"),
            SimulatorError::rate_limit("op", 10, "minute"),
            SimulatorError::concurrent_operation("test"),
            SimulatorError::internal("test"),
            SimulatorError::not_supported("feature"),
        ];

        // Ensure all variants can be created and have string representation
        for error in errors {
            assert!(!error.to_string().is_empty());
            let _category = error.category();
            let _severity = error.severity();
            let _recoverable = error.is_recoverable();
        }
    }
}

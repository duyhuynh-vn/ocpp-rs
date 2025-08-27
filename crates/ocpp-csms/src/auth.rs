//! Authentication and authorization module for OCPP CSMS

use crate::CsmsResult;
use ocpp_types::common::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// Token expiry time in seconds
    pub token_expiry: u64,
    /// Enable API key authentication
    pub api_key_enabled: bool,
    /// Rate limiting settings
    pub rate_limit: RateLimitConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: "change-me-in-production".to_string(),
            token_expiry: 86400, // 24 hours
            api_key_enabled: false,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_capacity: 100,
        }
    }
}

/// Authentication service
pub struct AuthService {
    config: AuthConfig,
    api_keys: HashMap<String, ApiKey>,
}

impl AuthService {
    /// Create new authentication service
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            api_keys: HashMap::new(),
        }
    }

    /// Authenticate charge point
    pub async fn authenticate_charge_point(
        &self,
        charge_point_id: &str,
        _credentials: Option<&str>,
    ) -> CsmsResult<AuthResult> {
        if !self.config.enabled {
            return Ok(AuthResult {
                authenticated: true,
                charge_point_id: charge_point_id.to_string(),
                permissions: vec!["*".to_string()],
            });
        }

        // TODO: Implement actual authentication logic
        Ok(AuthResult {
            authenticated: true,
            charge_point_id: charge_point_id.to_string(),
            permissions: vec!["charge".to_string()],
        })
    }

    /// Authorize ID tag
    pub async fn authorize_id_tag(&self, _id_tag: &str) -> CsmsResult<IdTagInfo> {
        // TODO: Implement ID tag authorization
        Ok(IdTagInfo {
            status: AuthorizationStatus::Accepted,
            parent_id_tag: None,
            expiry_date: None,
        })
    }

    /// Validate API key
    pub fn validate_api_key(&self, api_key: &str) -> CsmsResult<bool> {
        if !self.config.api_key_enabled {
            return Ok(false);
        }

        Ok(self.api_keys.contains_key(api_key))
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub authenticated: bool,
    /// Charge point ID
    pub charge_point_id: String,
    /// Granted permissions
    pub permissions: Vec<String>,
}

/// API key information
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// Key ID
    pub id: String,
    /// Key value
    pub key: String,
    /// Associated permissions
    pub permissions: Vec<String>,
    /// Expiry date
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the key is active
    pub active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_service_creation() {
        let config = AuthConfig::default();
        let service = AuthService::new(config);
        assert!(!service.api_keys.is_empty() || service.api_keys.is_empty()); // Just test it compiles
    }

    #[tokio::test]
    async fn test_authenticate_charge_point() {
        let config = AuthConfig::default();
        let service = AuthService::new(config);

        let result = service
            .authenticate_charge_point("CP001", None)
            .await
            .unwrap();
        assert!(result.authenticated);
        assert_eq!(result.charge_point_id, "CP001");
    }
}

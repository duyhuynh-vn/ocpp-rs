//! Message validation for OCPP messages
//!
//! This module provides validation utilities for OCPP messages to ensure
//! they conform to the specification requirements.

use crate::v16j::*;
use ocpp_types::{OcppError, OcppResult};
use std::collections::HashMap;

/// Validation trait for OCPP messages
pub trait Validate {
    /// Validate the message and return any validation errors
    fn validate(&self) -> OcppResult<()>;
}

/// Configuration key validator
pub struct ConfigurationValidator {
    /// Known configuration keys with their constraints
    known_keys: HashMap<String, KeyConstraints>,
}

/// Constraints for configuration keys
#[derive(Debug, Clone)]
pub struct KeyConstraints {
    /// Whether the key is read-only
    pub readonly: bool,
    /// Minimum length for string values
    pub min_length: Option<usize>,
    /// Maximum length for string values
    pub max_length: Option<usize>,
    /// Valid values (for enumerated types)
    pub valid_values: Option<Vec<String>>,
    /// Numeric range (min, max)
    pub numeric_range: Option<(f64, f64)>,
    /// Data type
    pub data_type: ConfigDataType,
}

/// Configuration data types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigDataType {
    String,
    Integer,
    Decimal,
    Boolean,
    DateTime,
    CSL, // Comma Separated List
}

impl ConfigurationValidator {
    /// Create a new configuration validator with standard OCPP 1.6J keys
    pub fn new() -> Self {
        let mut validator = Self {
            known_keys: HashMap::new(),
        };
        validator.add_standard_keys();
        validator
    }

    /// Add standard OCPP 1.6J configuration keys
    fn add_standard_keys(&mut self) {
        // Core Profile keys
        self.add_key(
            "AllowOfflineTxForUnknownId",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "AuthorizationCacheEnabled",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "AuthorizeRemoteTxRequests",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "BlinkRepeat",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((1.0, 100.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "ClockAlignedDataInterval",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 900.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "ConnectionTimeOut",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((1.0, 600.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "HeartbeatInterval",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 86400.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "LightIntensity",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 100.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "LocalAuthorizeOffline",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "LocalPreAuthorize",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "MeterValuesAlignedData",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: Some(500),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "MeterValuesSampledData",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: Some(500),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "MeterValueSampleInterval",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 86400.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "NumberOfConnectors",
            KeyConstraints {
                readonly: true,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 100.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "ResetRetries",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 10.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "ConnectorPhaseRotation",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: Some(1000),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "StopTransactionOnEVSideDisconnect",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "StopTransactionOnInvalidId",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "StopTxnAlignedData",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: Some(500),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "StopTxnSampledData",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: Some(500),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "SupportedFeatureProfiles",
            KeyConstraints {
                readonly: true,
                min_length: None,
                max_length: Some(1000),
                valid_values: None,
                numeric_range: None,
                data_type: ConfigDataType::CSL,
            },
        );

        self.add_key(
            "TransactionMessageAttempts",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((1.0, 10.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "TransactionMessageRetryInterval",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((1.0, 3600.0)),
                data_type: ConfigDataType::Integer,
            },
        );

        self.add_key(
            "UnlockConnectorOnEVSideDisconnect",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: Some(vec!["true".to_string(), "false".to_string()]),
                numeric_range: None,
                data_type: ConfigDataType::Boolean,
            },
        );

        self.add_key(
            "WebSocketPingInterval",
            KeyConstraints {
                readonly: false,
                min_length: None,
                max_length: None,
                valid_values: None,
                numeric_range: Some((0.0, 3600.0)),
                data_type: ConfigDataType::Integer,
            },
        );
    }

    /// Add a configuration key with its constraints
    pub fn add_key(&mut self, key: &str, constraints: KeyConstraints) {
        self.known_keys.insert(key.to_string(), constraints);
    }

    /// Validate a configuration key and value
    pub fn validate_key_value(&self, key: &str, value: &str) -> OcppResult<()> {
        if let Some(constraints) = self.known_keys.get(key) {
            self.validate_value(key, value, constraints)
        } else {
            // Unknown key, but that's allowed (vendor-specific keys)
            Ok(())
        }
    }

    fn validate_value(
        &self,
        key: &str,
        value: &str,
        constraints: &KeyConstraints,
    ) -> OcppResult<()> {
        // Check length constraints
        if let Some(min_len) = constraints.min_length {
            if value.len() < min_len {
                return Err(OcppError::ValidationError {
                    message: format!("Value for key '{}' is too short (min: {})", key, min_len),
                });
            }
        }

        if let Some(max_len) = constraints.max_length {
            if value.len() > max_len {
                return Err(OcppError::ValidationError {
                    message: format!("Value for key '{}' is too long (max: {})", key, max_len),
                });
            }
        }

        // Check valid values
        if let Some(valid_values) = &constraints.valid_values {
            if !valid_values.contains(&value.to_string()) {
                return Err(OcppError::ValidationError {
                    message: format!("Invalid value '{}' for key '{}'", value, key),
                });
            }
        }

        // Check data type and numeric range
        match constraints.data_type {
            ConfigDataType::Integer => {
                let parsed: i32 = value.parse().map_err(|_| OcppError::ValidationError {
                    message: format!("Value '{}' for key '{}' is not a valid integer", value, key),
                })?;

                if let Some((min, max)) = constraints.numeric_range {
                    let val = parsed as f64;
                    if val < min || val > max {
                        return Err(OcppError::ValidationError {
                            message: format!(
                                "Value {} for key '{}' is out of range ({}-{})",
                                parsed, key, min, max
                            ),
                        });
                    }
                }
            }
            ConfigDataType::Decimal => {
                let parsed: f64 = value.parse().map_err(|_| OcppError::ValidationError {
                    message: format!("Value '{}' for key '{}' is not a valid decimal", value, key),
                })?;

                if let Some((min, max)) = constraints.numeric_range {
                    if parsed < min || parsed > max {
                        return Err(OcppError::ValidationError {
                            message: format!(
                                "Value {} for key '{}' is out of range ({}-{})",
                                parsed, key, min, max
                            ),
                        });
                    }
                }
            }
            ConfigDataType::Boolean => {
                if value != "true" && value != "false" {
                    return Err(OcppError::ValidationError {
                        message: format!(
                            "Value '{}' for key '{}' is not a valid boolean (true/false)",
                            value, key
                        ),
                    });
                }
            }
            ConfigDataType::DateTime => {
                // Validate ISO 8601 format
                chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
                    OcppError::ValidationError {
                        message: format!(
                            "Value '{}' for key '{}' is not a valid ISO 8601 timestamp",
                            value, key
                        ),
                    }
                })?;
            }
            ConfigDataType::String | ConfigDataType::CSL => {
                // String validation already done with length checks
            }
        }

        Ok(())
    }

    /// Check if a key is read-only
    pub fn is_readonly(&self, key: &str) -> bool {
        self.known_keys
            .get(key)
            .map(|c| c.readonly)
            .unwrap_or(false)
    }
}

impl Default for ConfigurationValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Validation implementations for message types
impl Validate for AuthorizeRequest {
    fn validate(&self) -> OcppResult<()> {
        if self.id_tag.is_empty() {
            return Err(OcppError::ValidationError {
                message: "idTag cannot be empty".to_string(),
            });
        }

        if self.id_tag.len() > 20 {
            return Err(OcppError::ValidationError {
                message: "idTag cannot be longer than 20 characters".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for BootNotificationRequest {
    fn validate(&self) -> OcppResult<()> {
        if self.charge_point_vendor.is_empty() || self.charge_point_vendor.len() > 20 {
            return Err(OcppError::ValidationError {
                message: "chargePointVendor must be 1-20 characters".to_string(),
            });
        }

        if self.charge_point_model.is_empty() || self.charge_point_model.len() > 20 {
            return Err(OcppError::ValidationError {
                message: "chargePointModel must be 1-20 characters".to_string(),
            });
        }

        // Validate optional fields
        if let Some(ref serial) = self.charge_point_serial_number {
            if serial.len() > 25 {
                return Err(OcppError::ValidationError {
                    message: "chargePointSerialNumber cannot be longer than 25 characters"
                        .to_string(),
                });
            }
        }

        if let Some(ref serial) = self.charge_box_serial_number {
            if serial.len() > 25 {
                return Err(OcppError::ValidationError {
                    message: "chargeBoxSerialNumber cannot be longer than 25 characters"
                        .to_string(),
                });
            }
        }

        if let Some(ref version) = self.firmware_version {
            if version.len() > 50 {
                return Err(OcppError::ValidationError {
                    message: "firmwareVersion cannot be longer than 50 characters".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Validate for StartTransactionRequest {
    fn validate(&self) -> OcppResult<()> {
        if self.connector_id == 0 {
            return Err(OcppError::ValidationError {
                message: "connectorId must be greater than 0".to_string(),
            });
        }

        if self.id_tag.is_empty() || self.id_tag.len() > 20 {
            return Err(OcppError::ValidationError {
                message: "idTag must be 1-20 characters".to_string(),
            });
        }

        if self.meter_start < 0 {
            return Err(OcppError::ValidationError {
                message: "meterStart cannot be negative".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for StopTransactionRequest {
    fn validate(&self) -> OcppResult<()> {
        if let Some(ref id_tag) = self.id_tag {
            if id_tag.is_empty() || id_tag.len() > 20 {
                return Err(OcppError::ValidationError {
                    message: "idTag must be 1-20 characters".to_string(),
                });
            }
        }

        if self.meter_stop < 0 {
            return Err(OcppError::ValidationError {
                message: "meterStop cannot be negative".to_string(),
            });
        }

        if self.transaction_id < 0 {
            return Err(OcppError::ValidationError {
                message: "transactionId cannot be negative".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for StatusNotificationRequest {
    fn validate(&self) -> OcppResult<()> {
        // Connector ID 0 is valid for charge point status
        if let Some(ref info) = self.info {
            if info.len() > 50 {
                return Err(OcppError::ValidationError {
                    message: "info cannot be longer than 50 characters".to_string(),
                });
            }
        }

        if let Some(ref vendor_code) = self.vendor_error_code {
            if vendor_code.len() > 50 {
                return Err(OcppError::ValidationError {
                    message: "vendorErrorCode cannot be longer than 50 characters".to_string(),
                });
            }
        }

        if let Some(ref vendor_id) = self.vendor_id {
            if vendor_id.len() > 255 {
                return Err(OcppError::ValidationError {
                    message: "vendorId cannot be longer than 255 characters".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Validate for MeterValuesRequest {
    fn validate(&self) -> OcppResult<()> {
        if self.connector_id == 0 {
            return Err(OcppError::ValidationError {
                message: "connectorId must be greater than 0 for MeterValues".to_string(),
            });
        }

        if let Some(tx_id) = self.transaction_id {
            if tx_id < 0 {
                return Err(OcppError::ValidationError {
                    message: "transactionId cannot be negative".to_string(),
                });
            }
        }

        if self.meter_values.is_empty() {
            return Err(OcppError::ValidationError {
                message: "meterValue array cannot be empty".to_string(),
            });
        }

        // Validate each meter value
        for meter_value in &self.meter_values {
            if meter_value.sampled_values.is_empty() {
                return Err(OcppError::ValidationError {
                    message: "sampledValue array in meterValue cannot be empty".to_string(),
                });
            }

            for sampled_value in &meter_value.sampled_values {
                if sampled_value.value.is_empty() {
                    return Err(OcppError::ValidationError {
                        message: "sampledValue value cannot be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Validate a generic string field with length constraints
pub fn validate_string_field(
    field_name: &str,
    value: &str,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> OcppResult<()> {
    if let Some(min) = min_length {
        if value.len() < min {
            return Err(OcppError::ValidationError {
                message: format!("{} must be at least {} characters long", field_name, min),
            });
        }
    }

    if let Some(max) = max_length {
        if value.len() > max {
            return Err(OcppError::ValidationError {
                message: format!("{} cannot be longer than {} characters", field_name, max),
            });
        }
    }

    Ok(())
}

/// Validate a connector ID
pub fn validate_connector_id(connector_id: u32, allow_zero: bool) -> OcppResult<()> {
    if connector_id == 0 && !allow_zero {
        return Err(OcppError::ValidationError {
            message: "Connector ID must be greater than 0".to_string(),
        });
    }

    // OCPP typically supports up to 100 connectors
    if connector_id > 100 {
        return Err(OcppError::ValidationError {
            message: "Connector ID cannot be greater than 100".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_request_validation() {
        let valid_request = AuthorizeRequest {
            id_tag: "TAG123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let empty_tag = AuthorizeRequest {
            id_tag: "".to_string(),
        };
        assert!(empty_tag.validate().is_err());

        let too_long_tag = AuthorizeRequest {
            id_tag: "VERYLONGTAGTHATEXCEEDS20CHARACTERS".to_string(),
        };
        assert!(too_long_tag.validate().is_err());
    }

    #[test]
    fn test_boot_notification_validation() {
        let valid_request = BootNotificationRequest {
            charge_point_vendor: "Vendor".to_string(),
            charge_point_model: "Model".to_string(),
            charge_point_serial_number: Some("SN123".to_string()),
            charge_box_serial_number: None,
            firmware_version: Some("1.0.0".to_string()),
            iccid: None,
            imsi: None,
            meter_type: None,
            meter_serial_number: None,
        };
        assert!(valid_request.validate().is_ok());

        let empty_vendor = BootNotificationRequest {
            charge_point_vendor: "".to_string(),
            charge_point_model: "Model".to_string(),
            charge_point_serial_number: None,
            charge_box_serial_number: None,
            firmware_version: None,
            iccid: None,
            imsi: None,
            meter_type: None,
            meter_serial_number: None,
        };
        assert!(empty_vendor.validate().is_err());
    }

    #[test]
    fn test_configuration_validator() {
        let validator = ConfigurationValidator::new();

        // Test valid boolean
        assert!(validator
            .validate_key_value("AllowOfflineTxForUnknownId", "true")
            .is_ok());
        assert!(validator
            .validate_key_value("AllowOfflineTxForUnknownId", "false")
            .is_ok());

        // Test invalid boolean
        assert!(validator
            .validate_key_value("AllowOfflineTxForUnknownId", "yes")
            .is_err());

        // Test valid integer within range
        assert!(validator
            .validate_key_value("HeartbeatInterval", "300")
            .is_ok());

        // Test integer out of range
        assert!(validator
            .validate_key_value("HeartbeatInterval", "100000")
            .is_err());

        // Test unknown key (should be allowed)
        assert!(validator
            .validate_key_value("UnknownVendorKey", "somevalue")
            .is_ok());

        // Test read-only check
        assert!(validator.is_readonly("NumberOfConnectors"));
        assert!(!validator.is_readonly("HeartbeatInterval"));
    }

    #[test]
    fn test_start_transaction_validation() {
        let valid_request = StartTransactionRequest {
            connector_id: 1,
            id_tag: "USER123".to_string(),
            meter_start: 12345,
            timestamp: chrono::Utc::now(),
            reservation_id: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_connector = StartTransactionRequest {
            connector_id: 0,
            id_tag: "USER123".to_string(),
            meter_start: 12345,
            timestamp: chrono::Utc::now(),
            reservation_id: None,
        };
        assert!(invalid_connector.validate().is_err());

        let negative_meter = StartTransactionRequest {
            connector_id: 1,
            id_tag: "USER123".to_string(),
            meter_start: -100,
            timestamp: chrono::Utc::now(),
            reservation_id: None,
        };
        assert!(negative_meter.validate().is_err());
    }

    #[test]
    fn test_connector_id_validation() {
        assert!(validate_connector_id(1, false).is_ok());
        assert!(validate_connector_id(0, true).is_ok());
        assert!(validate_connector_id(0, false).is_err());
        assert!(validate_connector_id(101, false).is_err());
    }

    #[test]
    fn test_string_field_validation() {
        assert!(validate_string_field("test", "hello", Some(3), Some(10)).is_ok());
        assert!(validate_string_field("test", "hi", Some(3), Some(10)).is_err());
        assert!(validate_string_field("test", "verylongstring", Some(3), Some(10)).is_err());
    }
}

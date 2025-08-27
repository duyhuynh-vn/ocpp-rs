//! State management module for OCPP CSMS

use crate::CsmsResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Global state manager for OCPP CSMS
pub struct StateManager {
    /// Charge point states
    charge_point_states: Arc<DashMap<String, ChargePointState>>,
    /// Connector states
    connector_states: Arc<DashMap<ConnectorKey, ConnectorState>>,
    /// Transaction states
    transaction_states: Arc<DashMap<i32, TransactionState>>,
    /// Configuration cache
    config_cache: Arc<RwLock<DashMap<String, DashMap<String, String>>>>,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            charge_point_states: Arc::new(DashMap::new()),
            connector_states: Arc::new(DashMap::new()),
            transaction_states: Arc::new(DashMap::new()),
            config_cache: Arc::new(RwLock::new(DashMap::new())),
        }
    }

    /// Update charge point state
    pub async fn update_charge_point_state(
        &self,
        charge_point_id: &str,
        state: ChargePointState,
    ) -> CsmsResult<()> {
        debug!(
            "Updating charge point state: {} -> {:?}",
            charge_point_id, state
        );
        self.charge_point_states
            .insert(charge_point_id.to_string(), state);
        Ok(())
    }

    /// Get charge point state
    pub fn get_charge_point_state(&self, charge_point_id: &str) -> Option<ChargePointState> {
        self.charge_point_states
            .get(charge_point_id)
            .map(|s| s.clone())
    }

    /// Update connector state
    pub async fn update_connector_state(
        &self,
        charge_point_id: &str,
        connector_id: u32,
        state: ConnectorState,
    ) -> CsmsResult<()> {
        let key = ConnectorKey {
            charge_point_id: charge_point_id.to_string(),
            connector_id,
        };
        debug!("Updating connector state: {:?} -> {:?}", key, state);
        self.connector_states.insert(key, state);
        Ok(())
    }

    /// Get connector state
    pub fn get_connector_state(
        &self,
        charge_point_id: &str,
        connector_id: u32,
    ) -> Option<ConnectorState> {
        let key = ConnectorKey {
            charge_point_id: charge_point_id.to_string(),
            connector_id,
        };
        self.connector_states.get(&key).map(|s| s.clone())
    }

    /// Start transaction
    pub async fn start_transaction(
        &self,
        transaction_id: i32,
        state: TransactionState,
    ) -> CsmsResult<()> {
        info!("Starting transaction: {}", transaction_id);
        self.transaction_states.insert(transaction_id, state);
        Ok(())
    }

    /// Stop transaction
    pub async fn stop_transaction(
        &self,
        transaction_id: i32,
    ) -> CsmsResult<Option<TransactionState>> {
        info!("Stopping transaction: {}", transaction_id);
        Ok(self
            .transaction_states
            .remove(&transaction_id)
            .map(|(_, state)| state))
    }

    /// Get transaction state
    pub fn get_transaction_state(&self, transaction_id: i32) -> Option<TransactionState> {
        self.transaction_states
            .get(&transaction_id)
            .map(|s| s.clone())
    }

    /// Get all active transactions
    pub fn get_active_transactions(&self) -> Vec<(i32, TransactionState)> {
        self.transaction_states
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Cache configuration value
    pub async fn cache_config_value(
        &self,
        charge_point_id: &str,
        key: &str,
        value: &str,
    ) -> CsmsResult<()> {
        // Try to find existing config first
        {
            let config_cache = self.config_cache.read().await;
            if let Some(cp_config) = config_cache.get(charge_point_id) {
                cp_config.insert(key.to_string(), value.to_string());
                return Ok(());
            };
        }

        // Need to create new config for this charge point
        let config_cache = self.config_cache.write().await;
        let cp_config = DashMap::new();
        cp_config.insert(key.to_string(), value.to_string());
        config_cache.insert(charge_point_id.to_string(), cp_config);
        Ok(())
    }

    /// Get cached configuration value
    pub async fn get_cached_config_value(
        &self,
        charge_point_id: &str,
        key: &str,
    ) -> Option<String> {
        let config_cache = self.config_cache.read().await;
        config_cache
            .get(charge_point_id)
            .and_then(|cp_config| cp_config.get(key).map(|v| v.clone()))
    }

    /// Clear charge point from state
    pub async fn clear_charge_point_state(&self, charge_point_id: &str) -> CsmsResult<()> {
        info!("Clearing state for charge point: {}", charge_point_id);

        // Remove charge point state
        self.charge_point_states.remove(charge_point_id);

        // Remove connector states
        let connectors_to_remove: Vec<ConnectorKey> = self
            .connector_states
            .iter()
            .filter(|entry| entry.key().charge_point_id == charge_point_id)
            .map(|entry| entry.key().clone())
            .collect();

        for key in connectors_to_remove {
            self.connector_states.remove(&key);
        }

        // Remove configuration cache
        let config_cache = self.config_cache.write().await;
        config_cache.remove(charge_point_id);

        Ok(())
    }

    /// Get state statistics
    pub fn get_stats(&self) -> StateStats {
        StateStats {
            charge_points: self.charge_point_states.len(),
            connectors: self.connector_states.len(),
            active_transactions: self.transaction_states.len(),
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Charge point state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargePointState {
    /// Current status
    pub status: ChargePointStatus,
    /// Last heartbeat timestamp
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// Boot notification data
    pub boot_info: Option<BootInfo>,
    /// Connection information
    pub connection_info: Option<ConnectionInfo>,
}

/// Charge point status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargePointStatus {
    /// Connected and operational
    Online,
    /// Disconnected or not responding
    Offline,
    /// Registered but not yet operational
    Pending,
    /// In error state
    Faulted,
}

/// Boot notification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootInfo {
    /// Charge point vendor
    pub vendor: String,
    /// Charge point model
    pub model: String,
    /// Serial number
    pub serial_number: Option<String>,
    /// Firmware version
    pub firmware_version: Option<String>,
    /// Boot timestamp
    pub boot_time: chrono::DateTime<chrono::Utc>,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Connection ID
    pub connection_id: Uuid,
    /// Remote address
    pub remote_address: String,
    /// Connected timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Protocol version
    pub protocol_version: String,
}

/// Connector key for indexing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectorKey {
    /// Charge point ID
    pub charge_point_id: String,
    /// Connector ID
    pub connector_id: u32,
}

/// Connector state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorState {
    /// Current status
    pub status: ConnectorStatus,
    /// Error code
    pub error_code: ErrorCode,
    /// Additional information
    pub info: Option<String>,
    /// Vendor error code
    pub vendor_error_code: Option<String>,
    /// Vendor ID
    pub vendor_id: Option<String>,
    /// Last status update timestamp
    pub last_update: chrono::DateTime<chrono::Utc>,
    /// Current transaction ID
    pub transaction_id: Option<i32>,
}

/// Connector status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorStatus {
    /// Available for new transaction
    Available,
    /// Preparing for transaction
    Preparing,
    /// Charging in progress
    Charging,
    /// Suspended by EV
    SuspendedEV,
    /// Suspended by EVSE
    SuspendedEVSE,
    /// Transaction finishing
    Finishing,
    /// Reserved
    Reserved,
    /// Out of order
    Faulted,
    /// Unavailable
    Unavailable,
}

/// Error code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// No error
    NoError,
    /// Connector lock failure
    ConnectorLockFailure,
    /// EV communication error
    EVCommunicationError,
    /// Ground failure
    GroundFailure,
    /// High temperature
    HighTemperature,
    /// Internal error
    InternalError,
    /// Local list conflict
    LocalListConflict,
    /// Other error
    OtherError,
    /// Over current failure
    OverCurrentFailure,
    /// Over voltage
    OverVoltage,
    /// Power meter failure
    PowerMeterFailure,
    /// Power switch failure
    PowerSwitchFailure,
    /// Reader failure
    ReaderFailure,
    /// Reset failure
    ResetFailure,
    /// Under voltage
    UnderVoltage,
    /// Weak signal
    WeakSignal,
}

/// Transaction state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionState {
    /// Transaction ID
    pub transaction_id: i32,
    /// Charge point ID
    pub charge_point_id: String,
    /// Connector ID
    pub connector_id: u32,
    /// ID tag that started the transaction
    pub id_tag: String,
    /// Start timestamp
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Start meter value in Wh
    pub meter_start: i32,
    /// Reservation ID (if applicable)
    pub reservation_id: Option<i32>,
    /// Current meter value
    pub current_meter_value: Option<i32>,
    /// Last meter value update
    pub last_meter_update: Option<chrono::DateTime<chrono::Utc>>,
}

/// State statistics
#[derive(Debug, Clone)]
pub struct StateStats {
    /// Number of charge points tracked
    pub charge_points: usize,
    /// Number of connectors tracked
    pub connectors: usize,
    /// Number of active transactions
    pub active_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_manager_creation() {
        let manager = StateManager::new();
        let stats = manager.get_stats();
        assert_eq!(stats.charge_points, 0);
        assert_eq!(stats.connectors, 0);
        assert_eq!(stats.active_transactions, 0);
    }

    #[tokio::test]
    async fn test_charge_point_state_management() {
        let manager = StateManager::new();
        let state = ChargePointState {
            status: ChargePointStatus::Online,
            last_heartbeat: Some(chrono::Utc::now()),
            boot_info: None,
            connection_info: None,
        };

        manager
            .update_charge_point_state("CP001", state.clone())
            .await
            .unwrap();

        let retrieved = manager.get_charge_point_state("CP001").unwrap();
        assert_eq!(retrieved.status, ChargePointStatus::Online);
    }

    #[tokio::test]
    async fn test_connector_state_management() {
        let manager = StateManager::new();
        let state = ConnectorState {
            status: ConnectorStatus::Available,
            error_code: ErrorCode::NoError,
            info: None,
            vendor_error_code: None,
            vendor_id: None,
            last_update: chrono::Utc::now(),
            transaction_id: None,
        };

        manager
            .update_connector_state("CP001", 1, state.clone())
            .await
            .unwrap();

        let retrieved = manager.get_connector_state("CP001", 1).unwrap();
        assert_eq!(retrieved.status, ConnectorStatus::Available);
    }

    #[tokio::test]
    async fn test_transaction_management() {
        let manager = StateManager::new();
        let state = TransactionState {
            transaction_id: 123,
            charge_point_id: "CP001".to_string(),
            connector_id: 1,
            id_tag: "TAG001".to_string(),
            start_time: chrono::Utc::now(),
            meter_start: 1000,
            reservation_id: None,
            current_meter_value: Some(1000),
            last_meter_update: Some(chrono::Utc::now()),
        };

        manager.start_transaction(123, state.clone()).await.unwrap();

        let retrieved = manager.get_transaction_state(123).unwrap();
        assert_eq!(retrieved.transaction_id, 123);
        assert_eq!(retrieved.charge_point_id, "CP001");

        let stopped = manager.stop_transaction(123).await.unwrap();
        assert!(stopped.is_some());
        assert!(manager.get_transaction_state(123).is_none());
    }

    #[tokio::test]
    async fn test_config_cache() {
        let manager = StateManager::new();

        manager
            .cache_config_value("CP001", "HeartbeatInterval", "60")
            .await
            .unwrap();

        let value = manager
            .get_cached_config_value("CP001", "HeartbeatInterval")
            .await;
        assert_eq!(value, Some("60".to_string()));

        let missing = manager.get_cached_config_value("CP001", "MissingKey").await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_clear_charge_point_state() {
        let manager = StateManager::new();

        // Add some state
        let cp_state = ChargePointState {
            status: ChargePointStatus::Online,
            last_heartbeat: Some(chrono::Utc::now()),
            boot_info: None,
            connection_info: None,
        };
        manager
            .update_charge_point_state("CP001", cp_state)
            .await
            .unwrap();

        let conn_state = ConnectorState {
            status: ConnectorStatus::Available,
            error_code: ErrorCode::NoError,
            info: None,
            vendor_error_code: None,
            vendor_id: None,
            last_update: chrono::Utc::now(),
            transaction_id: None,
        };
        manager
            .update_connector_state("CP001", 1, conn_state)
            .await
            .unwrap();

        manager
            .cache_config_value("CP001", "TestKey", "TestValue")
            .await
            .unwrap();

        // Clear state
        manager.clear_charge_point_state("CP001").await.unwrap();

        // Verify state is cleared
        assert!(manager.get_charge_point_state("CP001").is_none());
        assert!(manager.get_connector_state("CP001", 1).is_none());
        assert!(manager
            .get_cached_config_value("CP001", "TestKey")
            .await
            .is_none());
    }
}

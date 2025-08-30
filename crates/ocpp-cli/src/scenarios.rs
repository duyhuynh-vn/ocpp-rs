//! # CLI Scenarios Module
//!
//! This module provides scenario definitions and execution for the OCPP CLI tool.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: String,
    /// List of steps to execute
    pub steps: Vec<ScenarioStep>,
    /// Scenario parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Individual scenario step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    /// Step name
    pub name: String,
    /// Action to perform
    pub action: ScenarioAction,
    /// Delay before executing this step
    pub delay_seconds: u64,
    /// Step parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Available scenario actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioAction {
    /// Connect to central system
    Connect,
    /// Disconnect from central system
    Disconnect,
    /// Send boot notification
    BootNotification,
    /// Send heartbeat
    Heartbeat,
    /// Plug in connector
    PlugIn { connector_id: u32 },
    /// Plug out connector
    PlugOut { connector_id: u32 },
    /// Start transaction
    StartTransaction { connector_id: u32, id_tag: String },
    /// Stop transaction
    StopTransaction { connector_id: u32 },
    /// Send status notification
    StatusNotification { connector_id: u32, status: String },
    /// Inject fault
    InjectFault {
        connector_id: u32,
        error_code: String,
    },
    /// Clear fault
    ClearFault { connector_id: u32 },
    /// Wait for specified duration
    Wait { duration_seconds: u64 },
    /// Send meter values
    MeterValues { connector_id: u32 },
}

/// Scenario execution result
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Scenario name
    pub name: String,
    /// Success flag
    pub success: bool,
    /// Execution duration
    pub duration: Duration,
    /// Step results
    pub step_results: Vec<StepResult>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Step execution result
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step name
    pub name: String,
    /// Success flag
    pub success: bool,
    /// Execution duration
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
}

/// Scenario executor
pub struct ScenarioExecutor {
    /// Available scenarios
    scenarios: HashMap<String, Scenario>,
}

impl ScenarioExecutor {
    /// Create new scenario executor
    pub fn new() -> Self {
        let mut executor = Self {
            scenarios: HashMap::new(),
        };
        executor.load_default_scenarios();
        executor
    }

    /// Add scenario
    pub fn add_scenario(&mut self, scenario: Scenario) {
        self.scenarios.insert(scenario.name.clone(), scenario);
    }

    /// Get scenario by name
    pub fn get_scenario(&self, name: &str) -> Option<&Scenario> {
        self.scenarios.get(name)
    }

    /// List available scenarios
    pub fn list_scenarios(&self) -> Vec<&str> {
        self.scenarios.keys().map(|s| s.as_str()).collect()
    }

    /// Execute scenario by name
    pub async fn execute_scenario(&self, name: &str) -> Result<ScenarioResult> {
        let scenario = self
            .scenarios
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Scenario '{}' not found", name))?;

        self.execute_scenario_impl(scenario).await
    }

    /// Execute scenario instance
    async fn execute_scenario_impl(&self, scenario: &Scenario) -> Result<ScenarioResult> {
        info!("Executing scenario: {}", scenario.name);

        let start_time = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut overall_success = true;

        for step in &scenario.steps {
            info!("Executing step: {}", step.name);

            // Apply delay
            if step.delay_seconds > 0 {
                sleep(Duration::from_secs(step.delay_seconds)).await;
            }

            let step_start = std::time::Instant::now();
            let result = self.execute_step(step).await;
            let step_duration = step_start.elapsed();

            let (step_success, error_message) = match &result {
                Ok(_) => (true, None),
                Err(e) => {
                    overall_success = false;
                    warn!("Step '{}' failed: {}", step.name, e);
                    (false, Some(e.to_string()))
                }
            };

            step_results.push(StepResult {
                name: step.name.clone(),
                success: step_success,
                duration: step_duration,
                error: error_message,
            });

            if !step_success {
                break; // Stop on first failure
            }
        }

        let total_duration = start_time.elapsed();

        Ok(ScenarioResult {
            name: scenario.name.clone(),
            success: overall_success,
            duration: total_duration,
            step_results,
            error: if !overall_success {
                Some("One or more steps failed".to_string())
            } else {
                None
            },
        })
    }

    /// Execute individual step
    async fn execute_step(&self, step: &ScenarioStep) -> Result<()> {
        match &step.action {
            ScenarioAction::Connect => {
                info!("Connecting to central system");
                // Simulate connection
                sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            ScenarioAction::Disconnect => {
                info!("Disconnecting from central system");
                sleep(Duration::from_millis(200)).await;
                Ok(())
            }
            ScenarioAction::BootNotification => {
                info!("Sending boot notification");
                sleep(Duration::from_millis(300)).await;
                Ok(())
            }
            ScenarioAction::Heartbeat => {
                info!("Sending heartbeat");
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }
            ScenarioAction::PlugIn { connector_id } => {
                info!("Plugging in connector {}", connector_id);
                sleep(Duration::from_millis(800)).await;
                Ok(())
            }
            ScenarioAction::PlugOut { connector_id } => {
                info!("Plugging out connector {}", connector_id);
                sleep(Duration::from_millis(600)).await;
                Ok(())
            }
            ScenarioAction::StartTransaction {
                connector_id,
                id_tag,
            } => {
                info!(
                    "Starting transaction on connector {} for user {}",
                    connector_id, id_tag
                );
                sleep(Duration::from_millis(1000)).await;
                Ok(())
            }
            ScenarioAction::StopTransaction { connector_id } => {
                info!("Stopping transaction on connector {}", connector_id);
                sleep(Duration::from_millis(800)).await;
                Ok(())
            }
            ScenarioAction::StatusNotification {
                connector_id,
                status,
            } => {
                info!(
                    "Sending status notification: connector {} = {}",
                    connector_id, status
                );
                sleep(Duration::from_millis(200)).await;
                Ok(())
            }
            ScenarioAction::InjectFault {
                connector_id,
                error_code,
            } => {
                info!(
                    "Injecting fault on connector {}: {}",
                    connector_id, error_code
                );
                sleep(Duration::from_millis(400)).await;
                Ok(())
            }
            ScenarioAction::ClearFault { connector_id } => {
                info!("Clearing fault on connector {}", connector_id);
                sleep(Duration::from_millis(300)).await;
                Ok(())
            }
            ScenarioAction::Wait { duration_seconds } => {
                info!("Waiting for {} seconds", duration_seconds);
                sleep(Duration::from_secs(*duration_seconds)).await;
                Ok(())
            }
            ScenarioAction::MeterValues { connector_id } => {
                info!("Sending meter values for connector {}", connector_id);
                sleep(Duration::from_millis(150)).await;
                Ok(())
            }
        }
    }

    /// Load default scenarios
    fn load_default_scenarios(&mut self) {
        // Basic charging scenario
        self.add_scenario(Scenario {
            name: "basic_charging".to_string(),
            description: "Complete basic charging cycle".to_string(),
            steps: vec![
                ScenarioStep {
                    name: "Connect".to_string(),
                    action: ScenarioAction::Connect,
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Boot Notification".to_string(),
                    action: ScenarioAction::BootNotification,
                    delay_seconds: 1,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Plug In".to_string(),
                    action: ScenarioAction::PlugIn { connector_id: 1 },
                    delay_seconds: 2,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Start Transaction".to_string(),
                    action: ScenarioAction::StartTransaction {
                        connector_id: 1,
                        id_tag: "USER001".to_string(),
                    },
                    delay_seconds: 2,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Charging Duration".to_string(),
                    action: ScenarioAction::Wait {
                        duration_seconds: 10,
                    },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Stop Transaction".to_string(),
                    action: ScenarioAction::StopTransaction { connector_id: 1 },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Plug Out".to_string(),
                    action: ScenarioAction::PlugOut { connector_id: 1 },
                    delay_seconds: 2,
                    parameters: HashMap::new(),
                },
            ],
            parameters: HashMap::new(),
        });

        // Fault scenario
        self.add_scenario(Scenario {
            name: "fault_during_charging".to_string(),
            description: "Fault injection during charging session".to_string(),
            steps: vec![
                ScenarioStep {
                    name: "Connect".to_string(),
                    action: ScenarioAction::Connect,
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Plug In".to_string(),
                    action: ScenarioAction::PlugIn { connector_id: 1 },
                    delay_seconds: 1,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Start Transaction".to_string(),
                    action: ScenarioAction::StartTransaction {
                        connector_id: 1,
                        id_tag: "USER002".to_string(),
                    },
                    delay_seconds: 2,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Normal Charging".to_string(),
                    action: ScenarioAction::Wait {
                        duration_seconds: 5,
                    },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Inject Fault".to_string(),
                    action: ScenarioAction::InjectFault {
                        connector_id: 1,
                        error_code: "OverCurrentFailure".to_string(),
                    },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Fault Duration".to_string(),
                    action: ScenarioAction::Wait {
                        duration_seconds: 3,
                    },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Clear Fault".to_string(),
                    action: ScenarioAction::ClearFault { connector_id: 1 },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Plug Out".to_string(),
                    action: ScenarioAction::PlugOut { connector_id: 1 },
                    delay_seconds: 2,
                    parameters: HashMap::new(),
                },
            ],
            parameters: HashMap::new(),
        });

        // Emergency unplug scenario
        self.add_scenario(Scenario {
            name: "emergency_unplug".to_string(),
            description: "Emergency cable disconnection during charging".to_string(),
            steps: vec![
                ScenarioStep {
                    name: "Start Charging".to_string(),
                    action: ScenarioAction::StartTransaction {
                        connector_id: 1,
                        id_tag: "USER003".to_string(),
                    },
                    delay_seconds: 1,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Charging".to_string(),
                    action: ScenarioAction::Wait {
                        duration_seconds: 3,
                    },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
                ScenarioStep {
                    name: "Emergency Unplug".to_string(),
                    action: ScenarioAction::PlugOut { connector_id: 1 },
                    delay_seconds: 0,
                    parameters: HashMap::new(),
                },
            ],
            parameters: HashMap::new(),
        });
    }
}

impl Default for ScenarioExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_executor_creation() {
        let executor = ScenarioExecutor::new();
        assert!(!executor.list_scenarios().is_empty());
        assert!(executor.get_scenario("basic_charging").is_some());
    }

    #[test]
    fn test_scenario_addition() {
        let mut executor = ScenarioExecutor::new();
        let initial_count = executor.list_scenarios().len();

        let custom_scenario = Scenario {
            name: "custom_test".to_string(),
            description: "Custom test scenario".to_string(),
            steps: vec![],
            parameters: HashMap::new(),
        };

        executor.add_scenario(custom_scenario);
        assert_eq!(executor.list_scenarios().len(), initial_count + 1);
        assert!(executor.get_scenario("custom_test").is_some());
    }

    #[tokio::test]
    async fn test_basic_scenario_execution() {
        let executor = ScenarioExecutor::new();

        // This would normally execute against a real connection
        // For testing, we just verify the scenario exists and can be retrieved
        assert!(executor.get_scenario("basic_charging").is_some());
    }

    #[test]
    fn test_scenario_serialization() {
        let scenario = Scenario {
            name: "test_scenario".to_string(),
            description: "Test scenario".to_string(),
            steps: vec![ScenarioStep {
                name: "test_step".to_string(),
                action: ScenarioAction::Wait {
                    duration_seconds: 5,
                },
                delay_seconds: 1,
                parameters: HashMap::new(),
            }],
            parameters: HashMap::new(),
        };

        let json = serde_json::to_string(&scenario).unwrap();
        let deserialized: Scenario = serde_json::from_str(&json).unwrap();

        assert_eq!(scenario.name, deserialized.name);
        assert_eq!(scenario.steps.len(), deserialized.steps.len());
    }
}

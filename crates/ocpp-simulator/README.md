# OCPP Simulator

A comprehensive OCPP (Open Charge Point Protocol) 1.6J charge point simulator with full API control, real-world scenario simulation, and WebSocket connectivity to Central Systems.

## Features

### Complete Charge Point Simulation
- ✅ **Full OCPP 1.6J Protocol Support** - All standard message types and workflows
- ✅ **Multiple Connector Management** - Simulate charge points with 1-100 connectors
- ✅ **State Machine Implementation** - Proper state transitions (Available → Preparing → Charging → Finishing)
- ✅ **Transaction Lifecycle** - Complete transaction management with proper start/stop sequences
- ✅ **WebSocket Connection** - Real WebSocket connection to Central Systems via `wss://` or `ws://`

### Real-World Charging Scenarios
- ✅ **Plugin/Unplug Simulation** - Realistic cable connection/disconnection events
- ✅ **EV Disconnection Events** - Simulate unexpected EV disconnections during charging
- ✅ **Charging Patterns** - Multiple charging patterns (Constant Power, Tapered, Variable)
- ✅ **Power Management** - Realistic power consumption with variations and noise
- ✅ **Battery SoC Simulation** - Simulated EV battery state of charge progression

### Comprehensive Fault Injection
- ✅ **All OCPP Error Codes** - Support for all standard fault types:
  - `ConnectorLockFailure` - Cable lock mechanism failures
  - `EVCommunicationError` - Communication issues with EV
  - `GroundFailure` - Electrical grounding problems
  - `HighTemperature` - Overheating conditions
  - `InternalError` - Internal system failures
  - `OverCurrentFailure` - Electrical overcurrent conditions
  - `OverVoltage` / `UnderVoltage` - Voltage regulation issues
  - `PowerMeterFailure` - Energy meter malfunctions
  - `PowerSwitchFailure` - Contactor/relay failures
  - `ReaderFailure` - RFID reader problems
- ✅ **Random Fault Injection** - Configurable automatic fault generation
- ✅ **Auto-Recovery** - Configurable fault recovery timeouts

### Advanced API Control
- ✅ **RESTful API** - Complete HTTP API for external control
- ✅ **WebSocket Events** - Real-time event streaming
- ✅ **Batch Operations** - Execute multiple operations in sequence
- ✅ **Scenario Execution** - Automated test scenario playback
- ✅ **Real-time Metrics** - Live performance and usage statistics

### Realistic Meter Simulation
- ✅ **Energy Metering** - Accurate energy consumption calculation
- ✅ **Power Variations** - Configurable power fluctuations
- ✅ **Electrical Parameters** - Voltage, current, power factor simulation
- ✅ **Temperature Simulation** - Component temperature based on load
- ✅ **Meter Noise** - Realistic measurement noise and drift

## Quick Start

### 1. Installation

```bash
# Clone the repository
git clone https://github.com/duyhuynh-vn/ocpp-rs
cd ocpp-rs

# Build the simulator
cargo build --release --bin ocpp-simulator
```

### 2. Generate Configuration

```bash
# Generate default configuration
./target/release/ocpp-simulator config --output simulator.toml

# Or with custom settings
./target/release/ocpp-simulator config \
  --connectors 4 \
  --url "wss://your-central-system.com/ocpp" \
  --id "CP001"
```

### 3. Start Simulation

```bash
# Start with default config
./target/release/ocpp-simulator start

# Start with custom config and settings
./target/release/ocpp-simulator start \
  --config simulator.toml \
  --bind 0.0.0.0:8081 \
  --log-level debug
```

### 4. Control via API

The simulator exposes a REST API on port 8081 (configurable):

```bash
# Check status
curl http://localhost:8081/status

# Plug in connector 1
curl -X POST http://localhost:8081/connectors/1/plug-in

# Start transaction
curl -X POST "http://localhost:8081/connectors/1/start-transaction?id_tag=user123"

# Inject fault
curl -X POST http://localhost:8081/connectors/1/fault \
  -H "Content-Type: application/json" \
  -d '{"error_code": "OverCurrentFailure", "info": "Simulated overcurrent"}'

# Stop transaction
curl -X POST http://localhost:8081/connectors/1/stop-transaction

# Unplug connector
curl -X POST http://localhost:8081/connectors/1/plug-out
```

## Configuration

### Basic Configuration

```toml
[simulator]
charge_point_id = "SIM001"
central_system_url = "ws://localhost:8080"
connector_count = 2
heartbeat_interval = 300
meter_values_interval = 60

[simulator.vendor_info]
charge_point_vendor = "OCPP-RS"
charge_point_model = "Simulator"
firmware_version = "1.0.0"

[simulator.api_config]
bind_address = "0.0.0.0"
port = 8081
enable_cors = true

[simulator.meter_config]
max_charging_power_w = 7360.0
voltage_v = 230.0
include_noise = true
simulate_drift = true
```

### Fault Injection Configuration

```toml
[simulator.fault_config]
enable_random_faults = true
random_fault_probability = 0.01  # 1% chance per check
random_fault_interval_seconds = 60
recovery_time_range = [30, 300]  # 30 seconds to 5 minutes

[simulator.fault_config.fault_types]
"OverCurrentFailure" = 0.3
"OverVoltage" = 0.2
"HighTemperature" = 0.1
```

## API Reference

### Connector Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/connectors` | GET | List all connectors |
| `/connectors/{id}` | GET | Get connector details |
| `/connectors/{id}/plug-in` | POST | Plug in cable |
| `/connectors/{id}/plug-out` | POST | Unplug cable |
| `/connectors/{id}/start-transaction` | POST | Start charging transaction |
| `/connectors/{id}/stop-transaction` | POST | Stop charging transaction |
| `/connectors/{id}/fault` | POST | Inject fault |
| `/connectors/{id}/clear-fault` | POST | Clear fault |

### System Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/status` | GET | Get simulator status |
| `/stats` | GET | Get detailed statistics |
| `/connection/connect` | POST | Connect to Central System |
| `/connection/disconnect` | POST | Disconnect from Central System |
| `/scenarios` | GET | List available scenarios |
| `/scenarios/{name}` | POST | Execute scenario |

### Real-time Events

Connect to WebSocket at `ws://localhost:8082/events/stream` for real-time events:

```javascript
const ws = new WebSocket('ws://localhost:8082/events/stream');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Event:', message);
};

// Subscribe to specific events
ws.send(JSON.stringify({
  type: "Subscribe",
  data: {
    filter: {
      min_severity: "info",
      event_types: ["connector_plugged_in", "transaction_started"]
    }
  }
}));
```

## Charging Scenarios

### Basic Charging Flow

```bash
# Complete charging session
curl -X POST http://localhost:8081/connectors/1/plug-in
curl -X POST "http://localhost:8081/connectors/1/start-transaction?id_tag=user123"
# Wait for charging...
curl -X POST http://localhost:8081/connectors/1/stop-transaction
curl -X POST http://localhost:8081/connectors/1/plug-out
```

### Fault During Charging

```bash
# Start charging
curl -X POST http://localhost:8081/connectors/1/plug-in
curl -X POST "http://localhost:8081/connectors/1/start-transaction?id_tag=user123"

# Inject fault during charging
curl -X POST http://localhost:8081/connectors/1/fault \
  -H "Content-Type: application/json" \
  -d '{"error_code": "HighTemperature", "info": "Thermal protection triggered"}'

# Clear fault
curl -X POST http://localhost:8081/connectors/1/clear-fault
curl -X POST http://localhost:8081/connectors/1/plug-out
```

### EV Disconnection Simulation

```bash
# Emergency unplug during charging
curl -X POST http://localhost:8081/connectors/1/plug-in
curl -X POST "http://localhost:8081/connectors/1/start-transaction?id_tag=user123"
# Simulate emergency unplug (transaction will auto-stop)
curl -X POST http://localhost:8081/connectors/1/plug-out
```

## Predefined Scenarios

### Available Scenarios

- **`basic_charging`** - Complete normal charging cycle
- **`interrupted_charging`** - Charging interrupted and resumed
- **`fault_during_charging`** - Fault injection during active charging
- **`multiple_connectors`** - Concurrent charging on multiple connectors
- **`reservation_flow`** - Connector reservation and usage
- **`emergency_stop`** - Emergency stop procedures
- **`network_issues`** - Connection loss and recovery

### Execute Scenarios

```bash
# Run basic charging scenario
curl -X POST http://localhost:8081/scenarios/basic_charging \
  -H "Content-Type: application/json" \
  -d '{"parameters": {"connector_id": 1, "id_tag": "user123"}}'

# List available scenarios
curl http://localhost:8081/scenarios
```

## Monitoring and Debugging

### Real-time Statistics

```bash
# Get current statistics
curl http://localhost:8081/stats

# Example response:
{
  "success": true,
  "data": {
    "uptime_seconds": 3600,
    "connector_count": 2,
    "connector_states": {
      "1": "Charging",
      "2": "Available"
    },
    "active_transactions": [
      {
        "transaction_id": 12345,
        "connector_id": 1,
        "id_tag": "user123",
        "start_time": "2024-01-01T10:00:00Z",
        "energy_wh": 5500,
        "current_power_w": 7200
      }
    ],
    "total_energy_wh": 125000,
    "connected": true
  }
}
```

### Event History

```bash
# Get recent events
curl "http://localhost:8081/events?limit=50&severity=info"

# Filter events by connector
curl "http://localhost:8081/events?connectors=1,2&types=transaction_started,fault_injected"
```

### Logging

```bash
# Enable debug logging
./target/release/ocpp-simulator start --log-level debug

# JSON structured logging
./target/release/ocpp-simulator start --json-logs
```

## Advanced Usage

### Multiple Charge Points

```bash
# Generate configs for multiple CPs
./target/release/ocpp-simulator config --id CP001 --output cp001.toml
./target/release/ocpp-simulator config --id CP002 --output cp002.toml --port 8082

# Start multiple instances
./target/release/ocpp-simulator start --config cp001.toml &
./target/release/ocpp-simulator start --config cp002.toml &
```

### Custom Scenarios

Create custom scenarios in the configuration:

```toml
[[simulator.scenario_config.scenarios.custom_test]]
name = "custom_test"
description = "Custom test scenario"
steps = [
  { name = "plug_in", action = "plug_in", connector_id = 1, delay_seconds = 0 },
  { name = "start_tx", action = "start_transaction", connector_id = 1, delay_seconds = 2, parameters = { id_tag = "test123" } },
  { name = "wait", action = "wait", delay_seconds = 0, parameters = { duration = 30 } },
  { name = "stop_tx", action = "stop_transaction", connector_id = 1, delay_seconds = 0 },
  { name = "plug_out", action = "plug_out", connector_id = 1, delay_seconds = 2 }
]
```

### Integration Testing

```bash
# Validate configuration
./target/release/ocpp-simulator validate --config simulator.toml

# Run specific scenario
./target/release/ocpp-simulator scenario --scenario basic_charging --params '{"connector_id": 1}'
```

## Troubleshooting

### Common Issues

1. **Connection Failed**: Check Central System URL and network connectivity
2. **Boot Notification Rejected**: Verify charge point credentials and configuration
3. **API Not Accessible**: Check bind address and firewall settings
4. **High CPU Usage**: Reduce meter update frequency and disable unnecessary features

### Debug Commands

```bash
# Test connection
curl http://localhost:8081/system/health

# Check WebSocket connection
curl http://localhost:8081/connection/status

# View error logs
journalctl -f -u ocpp-simulator
```

## Development

### Building from Source

```bash
git clone https://github.com/duyhuynh-vn/ocpp-rs
cd ocpp-rs
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run simulator-specific tests
cargo test -p ocpp-simulator

# Integration tests
cargo test --features integration-tests
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

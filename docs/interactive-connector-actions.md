# Interactive Connector Actions

This document describes the interactive connector operations available in the OCPP CLI tool for simulating real-world charging station behavior.

## Overview

The OCPP CLI provides a comprehensive interactive interface for simulating connector operations on OCPP-compliant charging stations. This allows you to test Central System implementations by simulating realistic charge point behavior including cable connections, transaction management, and status changes.

## Getting Started

### Starting the Interactive Mode

```bash
cargo run --bin ocpp-cli -- connect --interactive
```

This will:
1. Connect to the default Central System (ws://localhost:8080/ocpp/CP001)
2. Initialize 2 connectors by default
3. Send a BootNotification message
4. Start the interactive menu

### Custom Configuration

```bash
# Connect to a custom Central System
cargo run --bin ocpp-cli -- connect \
  --url "ws://your-csms.example.com:8080/ocpp/CP001" \
  --id "MY_CHARGE_POINT" \
  --connectors 4 \
  --interactive

# Enable verbose logging
cargo run --bin ocpp-cli -- connect \
  --interactive \
  --verbose \
  --log-level debug
```

## Interactive Menu Structure

### Main Menu

When you start interactive mode, you'll see the main menu:

```
🎮 Interactive Mode - Type 'help' for commands

Choose an option:
  📊 Show Status
  🔌 Connector Operations  ← Focus of this guide
  ⚡ Transaction Operations
  🚨 Fault Injection
  🎯 Run Scenario
  📈 Statistics
  🔍 Monitor Events
  ❌ Exit
```

Select "🔌 Connector Operations" to access the connector management features.

## Connector Operations

### Available Operations

The Connector Operations menu provides these actions:

1. **🔌 Plug In Cable** - Simulate a user connecting an EV cable
2. **🔌 Plug Out Cable** - Simulate a user disconnecting the cable
3. **⚡ Start Transaction** - Begin a charging session
4. **🛑 Stop Transaction** - End an active charging session
5. **🔧 Set Availability** - Change connector availability status
6. **📊 Show Detailed Status** - View comprehensive connector information
7. **⬅️ Back to Main Menu** - Return to the main interactive menu

### Connector Status Display

Before each operation, you'll see the current status of all connectors:

```
📊 Current Connector Status:
   ⚪ Connector 1: Available 
   🔌 Connector 2: Charging (Tx: 1670000001) 7.2kW
   ⚪ Connector 3: Unavailable 
   🔌 Connector 4: Preparing 
```

**Status Icons:**
- ⚪ No cable connected
- 🔌 Cable connected

**Status Colors:**
- 🟢 Green: Available, operational states
- 🟡 Yellow: Charging, transitional states  
- 🔴 Red: Faulted, unavailable states
- 🔵 Cyan: Other states (preparing, suspended, etc.)

## Detailed Operation Guide

### 1. Plug In Cable

**Purpose:** Simulates a user connecting an EV charging cable to the connector.

**Process:**
1. Select "🔌 Plug In Cable"
2. Choose which connector to plug into
3. The system will:
   - Change connector status from `Available` to `Preparing`
   - Set `cable_plugged` to `true`
   - Send a `StatusNotification` message to the Central System
   - Update the last status timestamp

**Prerequisites:** 
- Connector must be in `Available` state
- No cable already connected

**OCPP Messages Sent:**
```json
{
  "messageType": 2,
  "messageId": "1670000001",
  "action": "StatusNotification",
  "payload": {
    "connectorId": 1,
    "errorCode": "NoError",
    "status": "Preparing",
    "timestamp": "2023-12-03T10:30:00Z"
  }
}
```

### 2. Plug Out Cable

**Purpose:** Simulates a user disconnecting the charging cable.

**Process:**
1. Select "🔌 Plug Out Cable"
2. Choose which connector to unplug
3. The system will:
   - Check if any transaction is active (prevents unplugging during charging)
   - Change connector status to `Available`
   - Set `cable_plugged` to `false`
   - Send a `StatusNotification` message

**Prerequisites:**
- Cable must be connected
- No active transaction (transaction must be stopped first)

**Safety Feature:** The system prevents cable disconnection during active transactions, simulating real-world safety mechanisms.

### 3. Start Transaction

**Purpose:** Begins a charging session, simulating RFID authorization and transaction initiation.

**Process:**
1. Select "⚡ Start Transaction"
2. Choose the connector
3. Enter an ID tag (or press Enter for default "DEMO_TAG")
4. The system will:
   - Generate a unique transaction ID
   - Change connector status to `Charging`
   - Set charging power to 7.2kW (default)
   - Start energy delivery simulation
   - Send `StartTransaction` message
   - Send `StatusNotification` for charging state

**Prerequisites:**
- Cable must be plugged in
- Connector must be available (not faulted or unavailable)
- No existing active transaction

**OCPP Messages Sent:**
```json
// StartTransaction
{
  "messageType": 2,
  "messageId": "1670000002",
  "action": "StartTransaction",
  "payload": {
    "connectorId": 1,
    "idTag": "DEMO_TAG",
    "meterStart": 0,
    "timestamp": "2023-12-03T10:31:00Z"
  }
}

// StatusNotification
{
  "messageType": 2,
  "messageId": "1670000003", 
  "action": "StatusNotification",
  "payload": {
    "connectorId": 1,
    "errorCode": "NoError",
    "status": "Charging",
    "timestamp": "2023-12-03T10:31:00Z"
  }
}
```

### 4. Stop Transaction

**Purpose:** Ends an active charging session.

**Process:**
1. Select "🛑 Stop Transaction"
2. Choose the connector with an active transaction
3. The system will:
   - Calculate total energy delivered
   - Change status to `Finishing` temporarily, then to `Preparing`
   - Clear the transaction ID and charging power
   - Send `StopTransaction` message
   - Send `StatusNotification` updates

**Prerequisites:**
- Active transaction must exist on the selected connector

**OCPP Messages Sent:**
```json
// StopTransaction
{
  "messageType": 2,
  "messageId": "1670000004",
  "action": "StopTransaction", 
  "payload": {
    "transactionId": 1670000001,
    "meterStop": 150,
    "timestamp": "2023-12-03T10:45:00Z"
  }
}

// StatusNotification (Finishing)
{
  "messageType": 2,
  "messageId": "1670000005",
  "action": "StatusNotification",
  "payload": {
    "connectorId": 1,
    "errorCode": "NoError", 
    "status": "Finishing",
    "timestamp": "2023-12-03T10:45:00Z"
  }
}

// StatusNotification (Back to Preparing)
{
  "messageType": 2,
  "messageId": "1670000006",
  "action": "StatusNotification",
  "payload": {
    "connectorId": 1,
    "errorCode": "NoError",
    "status": "Preparing", 
    "timestamp": "2023-12-03T10:45:02Z"
  }
}
```

### 5. Set Availability

**Purpose:** Changes connector operational status for maintenance or administrative purposes.

**Available Options:**
- **🟢 Make Available:** Sets connector to operational state
- **🔴 Make Unavailable:** Takes connector out of service
- **🚨 Set Faulted:** Simulates a hardware or software fault

**Process:**
1. Select "🔧 Set Availability"
2. Choose the connector
3. Select the new availability status
4. The system sends appropriate `StatusNotification`

**Prerequisites:**
- Cannot change availability during active transactions (except to Available)

### 6. Show Detailed Status

**Purpose:** Displays comprehensive information about all connectors.

**Information Displayed:**
- Connector ID and current status
- Cable connection state
- Error codes
- Active transaction details (ID, power, energy delivered)
- ID tag information
- Last status update timestamp

**Example Output:**
```
📊 Detailed Connector Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔌 Connector 1:
   Status: Charging
   Cable: Plugged
   Error Code: NoError
   Transaction ID: 1670000001
   Current Power: 7.2 kW
   Energy Delivered: 2.35 kWh
   ID Tag: DEMO_TAG
   Last Update: 14:30:25

🔌 Connector 2:
   Status: Available
   Cable: Not plugged
   Error Code: NoError
   Last Update: 14:25:10
```

## Energy Simulation

The system includes realistic energy delivery simulation:

- **Power Level:** Default 7.2kW during charging
- **Energy Calculation:** Continuously updated every 5 seconds
- **Formula:** `energy_increment = power_kW × time_seconds ÷ 3600`
- **Display:** Shows cumulative energy delivered in kWh

## OCPP Compliance

### Message Types Supported

- **StatusNotification:** Sent for all status changes
- **StartTransaction:** Sent when beginning charging sessions  
- **StopTransaction:** Sent when ending charging sessions
- **BootNotification:** Sent on initial connection

### OCPP 1.6 Compliance

All messages follow OCPP 1.6 JSON specification:
- Proper message structure: `[MessageType, MessageId, Action, Payload]`
- Correct field naming and data types
- RFC3339 timestamp formatting
- Appropriate error codes and status values

### Status Values

Supported OCPP 1.6 status values:
- `Available` - Ready for new transactions
- `Preparing` - Cable connected, ready to charge
- `Charging` - Active energy transfer
- `SuspendedEV` - Suspended by electric vehicle
- `SuspendedEVSE` - Suspended by charging station
- `Finishing` - Transaction ending
- `Reserved` - Reserved for specific user
- `Unavailable` - Out of service
- `Faulted` - Hardware or software fault

## Testing Scenarios

### Basic Charging Session
1. Plug in cable → Status: `Preparing`
2. Start transaction → Status: `Charging`
3. Wait (energy accumulates) → Power: 7.2kW, Energy: increasing
4. Stop transaction → Status: `Finishing` → `Preparing`
5. Plug out cable → Status: `Available`

### Error Conditions
- Try to unplug during charging (should be blocked)
- Try to start transaction without cable (should be blocked)
- Try to start transaction on unavailable connector (should be blocked)

### Maintenance Operations
- Set connector unavailable
- Set connector faulted
- Restore connector to available

## Integration with Central Systems

The interactive connector actions integrate seamlessly with OCPP Central System Management Systems (CSMS). You can:

1. **Test CSMS Response:** See how your CSMS handles different status notifications
2. **Validate Transaction Flow:** Ensure proper StartTransaction/StopTransaction handling
3. **Test Error Scenarios:** Verify CSMS behavior with faulted connectors
4. **Monitor Message Exchange:** Use verbose logging to see all OCPP traffic

## Troubleshooting

### Common Issues

**"Cable must be plugged in before starting transaction"**
- Solution: Use "Plug In Cable" operation first

**"Cannot unplug cable during active transaction"**
- Solution: Stop the transaction first, then unplug

**"No active transaction on connector X"**
- Solution: Start a transaction before trying to stop it

**WebSocket connection issues**
- Check Central System URL and availability
- Verify firewall settings
- Enable debug logging for detailed connection info

### Debug Mode

Enable verbose logging for detailed message tracing:

```bash
cargo run --bin ocpp-cli -- connect \
  --interactive \
  --verbose \
  --log-level debug
```

This shows:
- WebSocket connection details
- All sent and received OCPP messages  
- Internal state changes
- Error conditions and handling

## Advanced Features

### Custom ID Tags
When starting transactions, you can enter custom ID tags to simulate different users or test specific authorization scenarios.

### Realistic Timing
The system includes realistic delays between status changes (e.g., 2-second delay from `Finishing` to `Preparing`) to simulate real hardware behavior.

### State Persistence
Connector states persist throughout the session, allowing for complex multi-step testing scenarios.

### Energy Monitoring
Real-time energy delivery simulation helps test metering and billing systems in the Central System.

## Contributing

To extend the interactive connector functionality:

1. **Add New Operations:** Extend the operations menu in `connector_operations()`
2. **Implement OCPP Messages:** Add new message types in the `send_*` functions
3. **Enhance Status Logic:** Modify connector state transitions in `ConnectorStatus`
4. **Add Validation:** Extend prerequisite checking for new operations

The modular design makes it easy to add new connector behaviors and OCPP message types while maintaining the interactive user experience.
# Testing the OCPP CLI

This document provides examples of how to test the OCPP CLI application.

## Basic Usage Examples

### 1. Connect to Local OCPP Server

```bash
# Connect with default settings
cargo run -- connect

# Connect with verbose logging
cargo run -- connect --verbose

# Connect interactively
cargo run -- connect --interactive
```

### 2. Custom Connection Parameters

```bash
# Connect to custom Central System
cargo run -- connect \
  --url wss://your-central-system.com/ocpp/CP001 \
  --id CP001 \
  --connectors 2 \
  --verbose
```

### 3. Run Test Suites

```bash
# Basic functionality tests
cargo run -- test --suite basic --verbose

# Load testing
cargo run -- test --suite load --count 3 --duration 120

# Full test suite
cargo run -- test --suite full
```

### 4. Scenario Management

```bash
# List available scenarios
cargo run -- scenario --list

# Generate a scenario template
cargo run -- scenario --generate

# Execute a scenario during connection
cargo run -- connect --auto-scenario basic-charging
```

## Interactive Mode Features

When running with `--interactive`, you get access to:

1. **📊 Show Status** - Display current connection and simulator status
2. **🔌 Connector Operations** - Simulate plug in/out, availability changes
3. **⚡ Transaction Operations** - Start/stop charging sessions
4. **🚨 Fault Injection** - Simulate various hardware faults
5. **🎯 Run Scenario** - Execute predefined test scenarios
6. **📈 Statistics** - View message counts and performance metrics
7. **🔍 Monitor Events** - Real-time event monitoring

## Expected Output Examples

### Successful Connection

```
   ____   ____ ____  ____     ____ _     ___
  / __ \ / ___/ __ \|  _ \   / ___| |   |_ _|
 | |  | | |  | |  | | |_) | | |   | |    | |
 | |__| | |__| |__| |  __/  | |___| |___ | |
  \____/ \____\____/|_|      \____|_____|___|

🔌 OCPP Charge Point Simulator

📡 Connection Details:
   URL: ws://localhost:8080/ocpp/CP001
   Charge Point ID: CP001
   Connectors: 2

✅ Connected successfully

🟢 Connection Established
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎮 Interactive Mode - Type 'help' for commands
```

### Connection Failure

```
❌ Connection failed
Error: Failed to connect: WebSocket connection failed: HTTP error: 400 Bad Request
```

### Test Suite Results

```
🧪 Running OCPP Test Suite
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Basic tests completed
```

## Troubleshooting

### Common Connection Issues

1. **400 Bad Request**: The WebSocket URL format may be incorrect
   - Try: `wss://host/path/ChargePointID`
   - Ensure the Charge Point ID matches the URL path

2. **Connection Timeout**: Central System may be unreachable
   - Check network connectivity
   - Verify the server is running
   - Check firewall settings

3. **TLS/SSL Errors**: Certificate issues with HTTPS/WSS
   - Ensure valid certificates on the server
   - Check system time/date

### Debug Mode

Run with debug logging to see detailed WebSocket communication:

```bash
RUST_LOG=debug cargo run -- connect --verbose
```

This will show:
- WebSocket handshake details
- OCPP message exchanges
- Connection state changes
- Error details

## Testing Against Different Central Systems

### Local Server (Default)
```bash
cargo run -- connect
```

### Custom OCPP Central System
```bash
cargo run -- connect \
  --url wss://your-cs.example.com/ocpp/YourChargePointID \
  --id YourChargePointID
```

### Local Development Server
```bash
cargo run -- connect \
  --url ws://localhost:8080/ocpp/TestCP \
  --id TestCP \
  --log-level debug
```

## Performance Testing

### Load Testing Multiple Charge Points
```bash
# Simulate 5 charge points for 10 minutes
cargo run -- test \
  --suite load \
  --count 5 \
  --duration 600 \
  --url wss://your-test-server.com/ocpp
```

### Fault Recovery Testing
```bash
# Test fault injection and recovery
cargo run -- test --suite fault --verbose
```

## Message Flow Examples

### Typical Startup Sequence
1. WebSocket connection established
2. BootNotification sent
3. BootNotification response received
4. Regular Heartbeat messages
5. StatusNotification for connectors

### Interactive Session Flow
1. User selects "Start Transaction"
2. StatusNotification: Preparing → Charging
3. StartTransaction message sent
4. Regular MeterValues during charging
5. StopTransaction when completed
6. StatusNotification: Finishing → Available

## Environment Variables

Set default values using environment variables:

```bash
export OCPP_CLI_URL="wss://your-default-cs.com/ocpp"
export OCPP_CLI_ID="DefaultChargePoint"
export OCPP_CLI_LOG_LEVEL="debug"
export OCPP_CLI_CONNECTORS="2"

cargo run -- connect
```

## Tips for Testing

1. **Start Simple**: Begin with basic connection test
2. **Use Verbose Mode**: Always use `--verbose` when debugging
3. **Interactive First**: Try interactive mode to understand the flow
4. **Check Logs**: Enable debug logging for detailed information
5. **Test Incrementally**: Start with single connector, then add more
6. **Document Issues**: Note any error messages for troubleshooting
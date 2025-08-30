# OCPP CLI

A comprehensive command-line interface for testing and simulating OCPP (Open Charge Point Protocol) implementations.

## Features

- **WebSocket Connection**: Connect to Central Systems via WebSocket
- **Interactive Simulation**: Real-time charge point simulation with user interaction
- **Automated Testing**: Basic test suites for OCPP message validation
- **Scenario Execution**: Predefined charging scenarios
- **Message Monitoring**: Monitor OCPP message traffic
- **Multi-connector Support**: Simulate multiple connectors per charge point

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/duyhuynh-vn/ocpp-rs
cd ocpp-rs

# Build the CLI
cargo build --release --bin ocpp-cli

# The binary will be available at target/release/ocpp-cli
```

## Quick Start

### Connect to Central System

Connect to a local OCPP server:

```bash
ocpp-cli connect
```

Connect with custom parameters:

```bash
ocpp-cli connect \
  --url wss://your-central-system.com/ocpp/ChargePoint001 \
  --id ChargePoint001 \
  --connectors 2 \
  --verbose
```

### Interactive Mode

Run in interactive mode for manual testing:

```bash
ocpp-cli connect --interactive
```

This opens an interactive menu with options for:
- Connector operations (plug in/out, availability)
- Transaction operations (start/stop charging)
- Fault injection and testing
- Scenario execution
- Real-time statistics

### Automated Testing

Run basic OCPP functionality tests:

```bash
ocpp-cli test --url wss://your-cs.com/ocpp --suite basic
```

Available test suites:
- `basic` - Core OCPP message tests (BootNotification, Heartbeat, StatusNotification)
- `load` - Load testing with multiple concurrent connections
- `fault` - Fault injection and recovery testing
- `full` - Complete test suite

### Scenario Execution

List available scenarios:

```bash
ocpp-cli scenario --list
```

Execute a specific scenario:

```bash
ocpp-cli scenario --execute basic-charging
```

Generate a custom scenario template:

```bash
ocpp-cli scenario --generate
```

## Commands

### `connect`

Connect to a Central System and simulate a charge point.

```bash
ocpp-cli connect [OPTIONS]
```

**Options:**
- `-u, --url <URL>` - Central System WebSocket URL (default: ws://localhost:8080/ocpp/CP001)
- `-i, --id <ID>` - Charge Point identifier (default: CP001)
- `-c, --connectors <NUM>` - Number of connectors (default: 2)
- `-I, --interactive` - Enable interactive mode
- `-a, --auto-scenario <NAME>` - Auto-start a scenario
- `--log-level <LEVEL>` - Log level: trace, debug, info, warn, error (default: info)
- `-v, --verbose` - Enable verbose logging

### `test`

Run automated test suites against a Central System.

```bash
ocpp-cli test [OPTIONS]
```

**Options:**
- `-u, --url <URL>` - Central System WebSocket URL
- `-s, --suite <SUITE>` - Test suite to run (default: basic)
- `-c, --count <NUM>` - Number of charge points to simulate (default: 1)
- `-d, --duration <SECONDS>` - Test duration (default: 300)

### `monitor`

Monitor OCPP traffic (planned feature).

```bash
ocpp-cli monitor [OPTIONS]
```

### `scenario`

Manage and execute test scenarios.

```bash
ocpp-cli scenario [OPTIONS]
```

**Options:**
- `-l, --list` - List available scenarios
- `-g, --generate` - Generate scenario template
- `-e, --execute <NAME>` - Execute specific scenario

### `validate`

Validate OCPP message files (planned feature).

```bash
ocpp-cli validate [OPTIONS]
```

## Examples

### Basic Connection Test

```bash
# Connect to local OCPP server
ocpp-cli connect --verbose

# Connect to custom Central System
ocpp-cli connect \
  --url wss://my-cs.example.com/ocpp/CP001 \
  --id CP001 \
  --connectors 1
```

### Interactive Testing Session

```bash
# Start interactive session
ocpp-cli connect --interactive

# Follow the on-screen menu to:
# 1. Check connection status
# 2. Simulate connector operations
# 3. Start/stop charging sessions
# 4. Inject faults for testing
# 5. Run predefined scenarios
```

### Automated Test Suite

```bash
# Run basic tests
ocpp-cli test --suite basic --verbose

# Run load tests with 5 concurrent charge points
ocpp-cli test --suite load --count 5 --duration 600

# Run comprehensive test suite
ocpp-cli test --suite full --url wss://test-cs.com/ocpp
```

### Scenario-based Testing

```bash
# List all available scenarios
ocpp-cli scenario --list

# Run a basic charging scenario
ocpp-cli connect --auto-scenario basic-charging

# Generate custom scenario template
ocpp-cli scenario --generate > my-scenario.json
# Edit my-scenario.json and run:
ocpp-cli scenario --execute my-scenario.json
```

## Configuration

The CLI supports configuration through:

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration files** (planned)

### Environment Variables

- `OCPP_CLI_URL` - Default Central System URL
- `OCPP_CLI_ID` - Default Charge Point ID
- `OCPP_CLI_LOG_LEVEL` - Default log level
- `OCPP_CLI_CONNECTORS` - Default number of connectors

## Supported OCPP Messages

Currently implemented OCPP 1.6J messages:

### Core Profile
- [x] BootNotification
- [x] Heartbeat
- [x] StatusNotification
- [ ] Authorize
- [ ] StartTransaction
- [ ] StopTransaction
- [ ] MeterValues

### Smart Charging Profile
- [ ] SetChargingProfile
- [ ] ClearChargingProfile
- [ ] GetCompositeSchedule

### Remote Trigger Profile
- [ ] TriggerMessage

## Architecture

The CLI is built with:

- **tokio** - Async runtime
- **tokio-tungstenite** - WebSocket client
- **clap** - Command-line parsing
- **dialoguer** - Interactive prompts
- **indicatif** - Progress bars and spinners
- **tracing** - Structured logging
- **serde** - JSON serialization

### Project Structure

```
ocpp-cli/
├── src/
│   ├── bin/
│   │   └── main.rs          # CLI entry point
│   ├── client.rs            # WebSocket client implementation
│   ├── commands.rs          # Command handlers
│   ├── config.rs            # Configuration management
│   ├── scenarios.rs         # Test scenario execution
│   ├── testing.rs           # Test suite implementation
│   ├── ui.rs                # User interface utilities
│   └── lib.rs               # Library exports
└── Cargo.toml
```

## Troubleshooting

### Connection Issues

**Problem**: `WebSocket connection failed: HTTP error: 400 Bad Request`

**Solutions**:
- Verify the WebSocket URL format: `wss://host/path/ChargePointId`
- Check if the Central System expects specific subprotocols
- Ensure the Charge Point ID matches the URL path
- Check network connectivity and firewall settings

**Problem**: `Connection timeout`

**Solutions**:
- Verify the Central System is running and accessible
- Check if the server requires authentication
- Ensure correct port number in URL

### Interactive Mode Issues

**Problem**: Menu not responding

**Solutions**:
- Ensure terminal supports interactive input
- Try running with `--verbose` for debug information
- Check terminal compatibility with dialoguer library

### Testing Issues

**Problem**: Tests failing with timeout

**Solutions**:
- Increase test duration with `--duration` option
- Check Central System response times
- Verify test suite compatibility

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/duyhuynh-vn/ocpp-rs
cd ocpp-rs/crates/ocpp-cli

# Build in development mode
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- connect --verbose
```

### Adding New Features

1. **New Commands**: Add to `src/commands.rs` and update `src/bin/main.rs`
2. **New Messages**: Implement in the `ocpp-messages` crate
3. **New Scenarios**: Add to `src/scenarios.rs`
4. **UI Components**: Update `src/ui.rs`

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Test specific functionality
cargo test test_connection
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests for new functionality
5. Update documentation
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built on top of the OCPP-RS library ecosystem
- Uses the OCPP 1.6J specification
- Inspired by real-world charge point testing needs

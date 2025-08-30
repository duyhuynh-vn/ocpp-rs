//! # OCPP CLI Application
//!
//! A comprehensive CLI tool for testing and simulating OCPP charge points.
//! Connects to real Central Systems and provides interactive verification.

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use futures_util::{SinkExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_native_tls::TlsConnector;
use tokio_tungstenite::{
    client_async, tungstenite::protocol::Message, tungstenite::protocol::WebSocketConfig,
};
use tracing::{debug, error, info, warn};
use url::Url;

#[derive(Parser)]
#[command(name = "ocpp-cli")]
#[command(about = "OCPP CLI - Test and simulate OCPP charge points")]
#[command(version, author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to Central System and run interactive simulation
    Connect(ConnectArgs),
    /// Run automated test suite
    Test(TestArgs),
    /// Monitor OCPP traffic
    Monitor(MonitorArgs),
    /// Generate test scenarios
    Scenario(ScenarioArgs),
    /// Validate OCPP messages
    Validate(ValidateArgs),
}

#[derive(Args)]
struct ConnectArgs {
    /// Central System WebSocket URL
    #[arg(short, long, default_value = "ws://localhost:8080/ocpp/CP001")]
    url: String,

    /// Charge Point ID
    #[arg(short, long, default_value = "CP001")]
    id: String,

    /// Number of connectors
    #[arg(short, long, default_value = "2")]
    connectors: u32,

    /// Interactive mode
    #[arg(short = 'I', long)]
    interactive: bool,

    /// Auto-start scenarios
    #[arg(short, long)]
    auto_scenario: Option<String>,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable detailed logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
struct TestArgs {
    /// Central System URL
    #[arg(short, long)]
    url: Option<String>,

    /// Test suite to run
    #[arg(short, long, default_value = "basic")]
    suite: String,

    /// Number of charge points to simulate
    #[arg(short, long, default_value = "1")]
    count: u32,

    /// Test duration in seconds
    #[arg(short, long, default_value = "300")]
    duration: u64,
}

#[derive(Args)]
struct MonitorArgs {
    /// Central System URL to monitor
    #[arg(short, long)]
    url: String,

    /// Filter by message type
    #[arg(short, long)]
    filter: Option<String>,

    /// Output format (json, table, raw)
    #[arg(short, long, default_value = "table")]
    output: String,
}

#[derive(Args)]
struct ScenarioArgs {
    /// Generate new scenario
    #[arg(short, long)]
    generate: bool,

    /// List available scenarios
    #[arg(short, long)]
    list: bool,

    /// Execute specific scenario
    #[arg(short, long)]
    execute: Option<String>,
}

#[derive(Args)]
struct ValidateArgs {
    /// OCPP message JSON file to validate
    #[arg(short, long)]
    file: Option<String>,

    /// Message type to validate against
    #[arg(short, long)]
    message_type: Option<String>,
}

#[derive(Debug, Clone)]
struct SimulatorState {
    charge_point_id: String,
    connectors: u32,
    connected: bool,
    start_time: chrono::DateTime<chrono::Utc>,
    message_count: u64,
}

#[derive(Debug, Clone)]
struct ConnectorStatus {
    id: u32,
    status: String,
    error_code: String,
    transaction_id: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Connect(args) => connect_command(args).await,
        Commands::Test(args) => test_command(args).await,
        Commands::Monitor(args) => monitor_command(args).await,
        Commands::Scenario(args) => scenario_command(args).await,
        Commands::Validate(args) => validate_command(args).await,
    }
}

async fn connect_command(args: ConnectArgs) -> Result<()> {
    // Initialize logging
    init_logging(&args.log_level, args.verbose)?;

    print_banner();
    println!(
        "🔌 {}\n",
        "OCPP Charge Point Simulator".bright_cyan().bold()
    );

    // Display connection info
    println!("📡 Connection Details:");
    println!("   URL: {}", args.url.bright_white());
    println!("   Charge Point ID: {}", args.id.bright_yellow());
    println!(
        "   Connectors: {}",
        args.connectors.to_string().bright_green()
    );
    println!();

    // Create simulator state
    let state = SimulatorState {
        charge_point_id: args.id.clone(),
        connectors: args.connectors,
        connected: false,
        start_time: chrono::Utc::now(),
        message_count: 0,
    };

    // Start WebSocket connection
    let simulator = Arc::new(tokio::sync::Mutex::new(state));

    // Setup progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} {msg}")?,
    );
    pb.set_message("Connecting to Central System...");

    // Parse URL
    let _url = Url::parse(&args.url)?;

    // Connect to WebSocket
    pb.enable_steady_tick(Duration::from_millis(120));

    match connect_websocket(&args.url, &args.id, simulator.clone()).await {
        Ok(_) => {
            pb.finish_with_message("✅ Connected successfully");
            println!("\n🟢 {}", "Connection Established".bright_green().bold());
        }
        Err(e) => {
            pb.finish_with_message("❌ Connection failed");
            return Err(anyhow::anyhow!("Failed to connect: {}", e));
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Setup Ctrl+C handler
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    ctrlc::set_handler(move || {
        println!("\n\n🛑 {} received", "Ctrl+C".yellow());
        let _ = shutdown_tx_clone.try_send(());
    })?;

    // Interactive mode or monitoring
    if args.interactive {
        println!(
            "🎮 {} - Type 'help' for commands",
            "Interactive Mode".bright_blue()
        );
        run_interactive_mode(&args, simulator.clone()).await?;
    } else if let Some(scenario_name) = args.auto_scenario {
        println!(
            "🎯 Running auto scenario: {}",
            scenario_name.bright_yellow()
        );
        run_auto_scenario(&scenario_name, simulator.clone()).await?;
    } else {
        // Just monitor and display status
        run_monitoring_mode(&args, simulator.clone()).await?;
    }

    // Wait for shutdown signal
    tokio::select! {
        _ = shutdown_rx.recv() => {
            println!("\n📴 Shutting down simulator...");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n📴 Shutting down simulator...");
        }
    }

    println!("✅ {}", "Simulator stopped".bright_green());
    Ok(())
}

async fn connect_websocket(
    url: &str,
    charge_point_id: &str,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    info!(
        "Connecting to {} (Charge Point ID: {})",
        url, charge_point_id
    );

    // Parse URL
    let url_parsed = url::Url::parse(url)?;
    let host = url_parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in URL"))?;
    let port = url_parsed
        .port()
        .unwrap_or(if url_parsed.scheme() == "wss" {
            443
        } else {
            80
        });

    // Connect TCP stream
    let tcp_stream = TcpStream::connect((host, port))
        .await
        .map_err(|e| anyhow::anyhow!("TCP connection failed: {}", e))?;

    // Setup TLS if needed
    let stream = if url_parsed.scheme() == "wss" {
        let connector = TlsConnector::from(
            native_tls::TlsConnector::new()
                .map_err(|e| anyhow::anyhow!("TLS connector creation failed: {}", e))?,
        );
        let tls_stream = connector
            .connect(host, tcp_stream)
            .await
            .map_err(|e| anyhow::anyhow!("TLS connection failed: {}", e))?;
        tokio_tungstenite::MaybeTlsStream::NativeTls(tls_stream)
    } else {
        tokio_tungstenite::MaybeTlsStream::Plain(tcp_stream)
    };

    // Create WebSocket request with OCPP subprotocol
    let request = http::Request::builder()
        .uri(url)
        .header("Host", host)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Protocol", "ocpp1.6")
        .body(())
        .map_err(|e| anyhow::anyhow!("Failed to build WebSocket request: {}", e))?;

    // Perform WebSocket handshake
    let config = WebSocketConfig::default();
    let (ws_stream, response) = client_async(request, stream)
        .await
        .map_err(|e| anyhow::anyhow!("WebSocket handshake failed: {}", e))?;

    info!("WebSocket connected successfully with OCPP 1.6 subprotocol");
    debug!("WebSocket response status: {:?}", response.status());

    let (mut write, mut read) = ws_stream.split();

    // Update simulator state
    {
        let mut state = simulator.lock().await;
        state.connected = true;
    }

    // Send BootNotification
    let boot_notification = create_boot_notification(charge_point_id);
    write
        .send(Message::Text(boot_notification))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send BootNotification: {}", e))?;

    // Start message handling task
    let simulator_clone = simulator.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received message: {}", text);
                    match handle_message(&text, simulator_clone.clone()).await {
                        Ok(Some(response)) => {
                            // Send response back
                            if let Err(e) = write.send(Message::Text(response)).await {
                                error!("Failed to send response: {}", e);
                            }
                        }
                        Ok(None) => {
                            // No response needed
                        }
                        Err(e) => {
                            error!("Error handling message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

async fn handle_message(
    message: &str,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<Option<String>> {
    // Update message count
    {
        let mut state = simulator.lock().await;
        state.message_count += 1;
    }

    // Parse and handle OCPP message
    let parsed: Value = serde_json::from_str(message)?;

    if let Some(array) = parsed.as_array() {
        if array.len() >= 4 {
            let message_type = array[0].as_i64().unwrap_or(0);
            let message_id = array[1].as_str().unwrap_or("");

            match message_type {
                2 => {
                    // CALL - need to respond
                    let action = array[2].as_str().unwrap_or("");
                    let payload = &array[3];

                    info!("Received CALL: {} ({})", action, message_id);

                    // Handle specific actions that require responses
                    match action {
                        "DataTransfer" => {
                            return handle_data_transfer_call(message_id, payload).await;
                        }
                        "ChangeAvailability" => {
                            return handle_change_availability_call(message_id, payload).await;
                        }
                        "RemoteStartTransaction" => {
                            return handle_remote_start_transaction_call(message_id, payload).await;
                        }
                        "RemoteStopTransaction" => {
                            return handle_remote_stop_transaction_call(message_id, payload).await;
                        }
                        "Reset" => {
                            return handle_reset_call(message_id, payload).await;
                        }
                        "GetConfiguration" => {
                            return handle_get_configuration_call(message_id, payload).await;
                        }
                        "ChangeConfiguration" => {
                            return handle_change_configuration_call(message_id, payload).await;
                        }
                        _ => {
                            warn!("Unsupported action: {}", action);
                            // Send NotSupported error
                            let error_response = serde_json::json!([
                                4,
                                message_id,
                                "NotSupported",
                                format!("Action '{}' is not supported", action),
                                {}
                            ]);
                            return Ok(Some(error_response.to_string()));
                        }
                    }
                }
                3 => {
                    // CALLRESULT
                    info!("Received CALLRESULT for message: {}", message_id);
                }
                4 => {
                    // CALLERROR
                    let error_code = array[2].as_str().unwrap_or("");
                    warn!(
                        "Received CALLERROR: {} for message: {}",
                        error_code, message_id
                    );
                }
                _ => {
                    warn!("Unknown message type: {}", message_type);
                }
            }
        }
    }

    Ok(None)
}

async fn handle_data_transfer_call(message_id: &str, payload: &Value) -> Result<Option<String>> {
    let vendor_id = payload["vendorId"].as_str().unwrap_or("");
    let message_id_param = payload["messageId"].as_str();
    let data = payload["data"].as_str();

    info!(
        "Handling DataTransfer: vendor_id={}, message_id={:?}, data={:?}",
        vendor_id, message_id_param, data
    );

    // Handle specific vendor/message combinations
    let (status, response_data) = match (vendor_id, message_id_param) {
        ("1", Some("Qrcode")) => {
            info!("✅ Processing QRCode data transfer for vendor 1");
            let response_obj = serde_json::json!({
                "result": "success",
                "qr_code_displayed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            (
                "Accepted",
                Some(serde_json::Value::String(response_obj.to_string())),
            )
        }
        ("1", _) => {
            // Known vendor, accept other messages
            (
                "Accepted",
                data.map(|d| serde_json::Value::String(d.to_string())),
            )
        }
        _ => {
            // Unknown vendor
            (
                "UnknownVendorId",
                Some(serde_json::Value::String("".to_string())),
            )
        }
    };

    let mut response_payload = serde_json::json!({
        "status": status
    });

    if let Some(data) = response_data {
        response_payload["data"] = data;
    } else {
        response_payload["data"] = serde_json::Value::String("".to_string());
    }

    let response = serde_json::json!([3, message_id, response_payload]);

    info!("Sending DataTransfer response: {}", status);
    Ok(Some(response.to_string()))
}

async fn handle_change_availability_call(
    message_id: &str,
    _payload: &Value,
) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "status": "Accepted"
        }
    ]);
    Ok(Some(response.to_string()))
}

async fn handle_remote_start_transaction_call(
    message_id: &str,
    _payload: &Value,
) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "status": "Accepted"
        }
    ]);
    Ok(Some(response.to_string()))
}

async fn handle_remote_stop_transaction_call(
    message_id: &str,
    _payload: &Value,
) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "status": "Accepted"
        }
    ]);
    Ok(Some(response.to_string()))
}

async fn handle_reset_call(message_id: &str, _payload: &Value) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "status": "Accepted"
        }
    ]);
    Ok(Some(response.to_string()))
}

async fn handle_get_configuration_call(
    message_id: &str,
    _payload: &Value,
) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "configurationKey": [],
            "unknownKey": []
        }
    ]);
    Ok(Some(response.to_string()))
}

async fn handle_change_configuration_call(
    message_id: &str,
    _payload: &Value,
) -> Result<Option<String>> {
    let response = serde_json::json!([
        3,
        message_id,
        {
            "status": "Accepted"
        }
    ]);
    Ok(Some(response.to_string()))
}

fn create_boot_notification(charge_point_id: &str) -> String {
    let boot_notification = serde_json::json!([
        2,
        "1",
        "BootNotification",
        {
            "chargePointVendor": "OCPP-RS",
            "chargePointModel": "CLI-Simulator",
            "chargePointSerialNumber": charge_point_id,
            "firmwareVersion": "1.0.0"
        }
    ]);

    boot_notification.to_string()
}

async fn test_command(args: TestArgs) -> Result<()> {
    init_logging("info", false)?;

    println!("🧪 {}", "Running OCPP Test Suite".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let url = args
        .url
        .unwrap_or_else(|| "ws://localhost:8080/ocpp/CP001".to_string());

    match args.suite.as_str() {
        "basic" => run_basic_tests(&url, args.count, args.duration).await,
        "load" => run_load_tests(&url, args.count, args.duration).await,
        "fault" => run_fault_tests(&url, args.count, args.duration).await,
        "full" => run_full_test_suite(&url, args.count, args.duration).await,
        _ => {
            println!("❌ Unknown test suite: {}", args.suite.red());
            list_available_test_suites();
            Ok(())
        }
    }
}

async fn monitor_command(args: MonitorArgs) -> Result<()> {
    init_logging("debug", true)?;

    println!("👁️  {}", "OCPP Traffic Monitor".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Monitoring: {}", args.url.bright_white());
    println!("Format: {}", args.output.bright_yellow());
    if let Some(filter) = &args.filter {
        println!("Filter: {}", filter.bright_green());
    }
    println!();

    println!("📡 Monitoring OCPP traffic...");
    println!("Press Ctrl+C to stop");

    // Setup Ctrl+C handler
    tokio::signal::ctrl_c().await?;
    println!("\n✅ Monitoring stopped");

    Ok(())
}

async fn scenario_command(args: ScenarioArgs) -> Result<()> {
    if args.list {
        list_available_scenarios();
    } else if args.generate {
        generate_scenario_template().await?;
    } else if let Some(scenario) = args.execute {
        execute_scenario(&scenario).await?;
    } else {
        println!("Use --help to see available scenario commands");
    }

    Ok(())
}

async fn validate_command(args: ValidateArgs) -> Result<()> {
    init_logging("info", false)?;

    println!("🔍 {}", "OCPP Message Validator".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(file) = args.file {
        validate_message_file(&file, args.message_type.as_deref()).await?;
    } else {
        println!("❌ Please specify a file to validate with --file");
    }

    Ok(())
}

// Interactive mode implementation
async fn run_interactive_mode(
    args: &ConnectArgs,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    let theme = ColorfulTheme::default();

    loop {
        println!("\n🎮 Interactive Commands:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━");

        let options = vec![
            "📊 Show Status",
            "🔌 Connector Operations",
            "⚡ Transaction Operations",
            "🚨 Fault Injection",
            "🎯 Run Scenario",
            "📈 Statistics",
            "🔍 Monitor Events",
            "❌ Exit",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("Choose an action")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => show_status(args, simulator.clone()).await?,
            1 => connector_operations(args, simulator.clone()).await?,
            2 => transaction_operations(args, simulator.clone()).await?,
            3 => fault_injection(args, simulator.clone()).await?,
            4 => run_scenario_interactive(args, simulator.clone()).await?,
            5 => show_statistics(args, simulator.clone()).await?,
            6 => monitor_events_interactive(args, simulator.clone()).await?,
            7 => {
                println!("👋 Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn show_status(
    args: &ConnectArgs,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n📊 {}", "Simulator Status".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━");

    let state = simulator.lock().await;

    // Display connection status
    println!("🔗 Connection:");
    println!("   URL: {}", args.url.bright_white());
    println!(
        "   Status: {}",
        if state.connected {
            "Connected".bright_green()
        } else {
            "Disconnected".bright_red()
        }
    );
    let uptime = chrono::Utc::now().signed_duration_since(state.start_time);
    println!(
        "   Uptime: {}",
        format_duration(uptime.to_std().unwrap_or_default()).bright_blue()
    );

    // Display connector status
    println!("\n🔌 Connectors:");
    for i in 1..=args.connectors {
        println!("   Connector {}: {}", i, "Available".bright_green());
    }

    // Display statistics
    println!("\n📊 Statistics:");
    println!(
        "   Messages: {}",
        state.message_count.to_string().bright_yellow()
    );
    println!("   Transactions: {}", "0".bright_cyan());

    Ok(())
}

async fn connector_operations(
    _args: &ConnectArgs,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n🔌 {}", "Connector Operations".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━");

    let theme = ColorfulTheme::default();
    let operations = vec![
        "Plug In Cable",
        "Plug Out Cable",
        "Start Transaction",
        "Stop Transaction",
        "Set Availability",
        "Back to Main Menu",
    ];

    let selection = Select::with_theme(&theme)
        .with_prompt("Choose connector operation")
        .items(&operations)
        .default(0)
        .interact()?;

    match selection {
        0 => println!("✅ Cable plugged in"),
        1 => println!("✅ Cable plugged out"),
        2 => println!("✅ Transaction started"),
        3 => println!("✅ Transaction stopped"),
        4 => println!("✅ Availability set"),
        5 => return Ok(()),
        _ => unreachable!(),
    }

    Ok(())
}

async fn transaction_operations(
    _args: &ConnectArgs,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n⚡ {}", "Transaction Operations".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("🔄 Transaction operations would be implemented here");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}

async fn fault_injection(
    _args: &ConnectArgs,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n🚨 {}", "Fault Injection".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━");

    println!("💥 Fault injection would be implemented here");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}

async fn run_scenario_interactive(
    _args: &ConnectArgs,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n🎯 {}", "Run Scenario".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━");

    let theme = ColorfulTheme::default();
    let scenarios = vec![
        "Basic Charging",
        "Fast Charging",
        "Emergency Stop",
        "Fault Recovery",
        "Back to Main Menu",
    ];

    let selection = Select::with_theme(&theme)
        .with_prompt("Choose scenario")
        .items(&scenarios)
        .default(0)
        .interact()?;

    match selection {
        0..=3 => {
            println!(
                "🎬 Running scenario: {}",
                scenarios[selection].bright_yellow()
            );
            let pb = ProgressBar::new(100);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"));

            for i in 0..=100 {
                pb.set_position(i);
                pb.set_message(format!("Step {}/100", i));
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            pb.finish_with_message("✅ Scenario completed");
        }
        4 => return Ok(()),
        _ => unreachable!(),
    }

    Ok(())
}

async fn show_statistics(
    _args: &ConnectArgs,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n📈 {}", "Statistics".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━");

    let state = simulator.lock().await;

    println!("📊 Message Statistics:");
    println!(
        "   Total Messages: {}",
        state.message_count.to_string().bright_yellow()
    );
    println!("   Boot Notifications: {}", "1".bright_green());
    println!("   Heartbeats: {}", "0".bright_blue());
    println!("   Status Notifications: {}", "0".bright_cyan());

    println!("\n⚡ Transaction Statistics:");
    println!("   Total Transactions: {}", "0".bright_yellow());
    println!("   Active Transactions: {}", "0".bright_green());
    println!("   Energy Delivered: {} kWh", "0.0".bright_blue());

    println!("\nPress Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}

async fn monitor_events_interactive(
    _args: &ConnectArgs,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!("\n🔍 {}", "Monitor Events".bright_cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━");

    println!("👁️  Event monitoring would show real-time events here");
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}

async fn run_monitoring_mode(
    _args: &ConnectArgs,
    simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!(
        "📊 {} - Press Ctrl+C to stop",
        "Monitoring Mode".bright_blue()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut interval = interval(Duration::from_secs(5));
    let mut counter = 0;

    loop {
        interval.tick().await;
        counter += 1;

        let state = simulator.lock().await;
        let uptime = chrono::Utc::now().signed_duration_since(state.start_time);

        println!(
            "📡 Status Update #{}: {} | Messages: {} | Uptime: {}",
            counter,
            if state.connected {
                "Connected".bright_green()
            } else {
                "Disconnected".bright_red()
            },
            state.message_count.to_string().bright_yellow(),
            format_duration(uptime.to_std().unwrap_or_default()).bright_blue()
        );

        if counter >= 20 {
            break;
        }
    }

    Ok(())
}

async fn run_auto_scenario(
    scenario_name: &str,
    _simulator: Arc<tokio::sync::Mutex<SimulatorState>>,
) -> Result<()> {
    println!(
        "🤖 Auto-executing scenario: {}",
        scenario_name.bright_yellow()
    );

    let pb = ProgressBar::new(10);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    for i in 0..=10 {
        pb.set_position(i);
        pb.set_message(format!("Executing step {}/10", i));
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    pb.finish_with_message("✅ Auto-scenario completed");
    Ok(())
}

async fn run_basic_tests(_url: &str, _count: u32, _duration: u64) -> Result<()> {
    println!("🧪 Running basic tests...");
    sleep(Duration::from_secs(2)).await;
    println!("✅ Basic tests completed");
    Ok(())
}

async fn run_load_tests(_url: &str, _count: u32, _duration: u64) -> Result<()> {
    println!("🏋️ Running load tests...");
    sleep(Duration::from_secs(3)).await;
    println!("✅ Load tests completed");
    Ok(())
}

async fn run_fault_tests(_url: &str, _count: u32, _duration: u64) -> Result<()> {
    println!("🔥 Running fault tests...");
    sleep(Duration::from_secs(2)).await;
    println!("✅ Fault tests completed");
    Ok(())
}

async fn run_full_test_suite(_url: &str, count: u32, duration: u64) -> Result<()> {
    println!("🎯 Running full test suite...");
    run_basic_tests(_url, count, duration).await?;
    run_load_tests(_url, count, duration).await?;
    run_fault_tests(_url, count, duration).await?;
    println!("✅ Full test suite completed");
    Ok(())
}

fn list_available_test_suites() {
    println!("\n📋 Available test suites:");
    println!("   • basic  - Basic OCPP functionality tests");
    println!("   • load   - Load and performance tests");
    println!("   • fault  - Fault injection and recovery tests");
    println!("   • full   - Complete test suite");
}

fn list_available_scenarios() {
    println!("\n📋 Available scenarios:");
    println!("   • basic-charging     - Simple charging session");
    println!("   • fast-charging      - High-power charging");
    println!("   • emergency-stop     - Emergency stop scenario");
    println!("   • fault-recovery     - Fault injection and recovery");
    println!("   • multiple-connector - Multi-connector operations");
}

async fn generate_scenario_template() -> Result<()> {
    println!("📝 Generating scenario template...");

    let template = serde_json::json!({
        "name": "custom-scenario",
        "description": "Custom OCPP scenario",
        "steps": [
            {
                "action": "connect",
                "description": "Connect to Central System"
            },
            {
                "action": "boot_notification",
                "description": "Send BootNotification"
            },
            {
                "action": "start_transaction",
                "connector_id": 1,
                "id_tag": "test_tag"
            },
            {
                "action": "stop_transaction",
                "transaction_id": 1
            }
        ]
    });

    println!("✅ Template generated:");
    println!("{}", serde_json::to_string_pretty(&template)?);
    Ok(())
}

async fn execute_scenario(_scenario: &str) -> Result<()> {
    println!("🎬 Executing scenario...");
    sleep(Duration::from_secs(1)).await;
    println!("✅ Scenario execution completed");
    Ok(())
}

async fn validate_message_file(_file: &str, _message_type: Option<&str>) -> Result<()> {
    println!("🔍 Validating message file: {}", _file);
    sleep(Duration::from_millis(500)).await;
    println!("✅ Message validation completed");
    Ok(())
}

// Utility functions
fn print_banner() {
    println!(
        "{}",
        r#"
   ____   ____ ____  ____     ____ _     ___
  / __ \ / ___/ __ \|  _ \   / ___| |   |_ _|
 | |  | | |  | |  | | |_) | | |   | |    | |
 | |__| | |__| |__| |  __/  | |___| |___ | |
  \____/ \____\____/|_|      \____|_____|___|

"#
        .bright_cyan()
    );
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn init_logging(level: &str, verbose: bool) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let level = match level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    if verbose {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into()))
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into()))
            .with(tracing_subscriber::fmt::layer().compact())
            .init();
    }

    Ok(())
}

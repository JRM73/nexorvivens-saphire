// =============================================================================
// main.rs -- Entry point for Saphire Lite
//
// The HTTP/WebSocket route handlers live in the saphire::api module (src/api/).
// This file is responsible for:
//   1. Parsing CLI arguments (--demo, --config)
//   2. Loading configuration from saphire.toml
//   3. Connecting to PostgreSQL (main DB + logs DB)
//   4. Bootstrapping the cognitive agent (SaphireAgent)
//   5. Spawning the web server and signal handlers
//   6. Running the main cognitive life-loop (autonomous thought, user
//      message handling, control message dispatch, mortality checks)
// =============================================================================

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio::time::sleep;

use saphire::config::SaphireConfig;
use saphire::db::SaphireDb;
use saphire::llm;
use saphire::agent::SaphireAgent;
use saphire::agent::lifecycle::UserMessage;
use saphire::logging::SaphireLogger;
use saphire::logging::db::LogsDb;

use saphire::api::{AppState, ControlMessage};

/// Asynchronous entry point for the Saphire Lite application.
///
/// The `#[tokio::main]` macro creates the Tokio async runtime and drives
/// this function to completion. All subsystem initialization, the web
/// server, and the main cognitive loop run within this runtime.
#[tokio::main]
async fn main() {
    // Initialize the structured tracing/logging subscriber.
    // Falls back to the "info" log level if the RUST_LOG environment
    // variable is not set.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         SAPHIRE LITE — Agent Cognitif Autonome              ║");
    println!("║                    Version 1.0.0                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Parse command-line arguments:
    //   --demo    : run in demonstration mode (no LLM, no DB)
    //   --config  : path to the TOML configuration file (default: saphire.toml)
    let args: Vec<String> = std::env::args().collect();
    let demo_mode = args.iter().any(|a| a == "--demo");

    let config_path = args.iter()
        .position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("saphire.toml");

    // Load configuration from the TOML file; fall back to defaults on error.
    let config = match SaphireConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Configuration error: {}. Using default values.", e);
            SaphireConfig::default()
        }
    };

    tracing::info!("Configuration loaded");
    println!("  Configuration loaded");
    println!("  LLM model: {}", config.llm.model);
    println!("  Web interface: {}:{}", config.web_ui.host, config.web_ui.port);

    // --- Demonstration mode --------------------------------------------------
    // Uses a mock LLM backend and skips all database connections.
    // Runs 8 predefined scenarios and exits.
    if demo_mode {
        println!("  Demonstration mode (no DB, no LLM)");
        let mock_llm = llm::create_backend(&saphire::llm::LlmConfig {
            backend: "mock".into(),
            ..Default::default()
        });
        let mut agent = SaphireAgent::new(config, mock_llm);
        agent.boot().await;
        saphire::pipeline::run_demo(&mut agent);
        agent.shutdown().await;
        return;
    }

    // --- Normal mode: create the real LLM backend ----------------------------
    let llm_backend = llm::create_backend(&config.llm);
    println!("  LLM backend: {} ({})", config.llm.backend, config.llm.model);

    let mut agent = SaphireAgent::new(config.clone(), llm_backend);

    // Connect to the main PostgreSQL database (persistent memory, identity).
    println!("  Connecting to PostgreSQL ({}:{})...", config.database.host, config.database.port);
    match SaphireDb::connect(&config.database).await {
        Ok(db) => {
            println!("  PostgreSQL connected, migrations applied");
            tracing::info!("PostgreSQL connected");
            agent.set_db(db);
        },
        Err(e) => {
            tracing::error!("PostgreSQL unavailable: {}. Degraded mode (no persistence).", e);
            println!("  PostgreSQL unavailable: {}. Degraded mode.", e);
        }
    }

    // Connect to the separate logs database (structured event logs).
    println!("  Connecting to logs database ({}:{})...", config.logs_database.host, config.logs_database.port);
    let logs_db: Option<Arc<LogsDb>> = match LogsDb::connect(&config.logs_database).await {
        Ok(db) => {
            println!("  Logs database connected, migrations applied");
            tracing::info!("LogsDb connected");
            Some(Arc::new(db))
        },
        Err(e) => {
            tracing::warn!("LogsDb unavailable: {}. Logging to terminal only.", e);
            println!("  Logs database unavailable: {}. Terminal-only mode.", e);
            None
        }
    };

    // Create a broadcast channel for real-time dashboard updates.
    let (dashboard_tx, _) = broadcast::channel::<String>(100);
    let dashboard_tx = Arc::new(dashboard_tx);

    // Initialize the structured logger with optional DB sink and dashboard broadcast.
    let logger = Arc::new(Mutex::new(SaphireLogger::new(
        logs_db.clone(),
        Some(dashboard_tx.clone()),
    )));

    agent.set_logger(logger.clone());
    if let Some(ref ldb) = logs_db {
        agent.set_logs_db(ldb.clone());
    }

    // Configure the mortality subsystem if enabled in the configuration.
    // When mortality is active, the agent can enter an agony phase and
    // eventually "die" after a specified number of cycles.
    if config.mortality.enabled {
        agent.body.set_mortality_config(config.mortality.agony_duration_cycles);
        println!("  Mortality enabled (agony duration: {} cycles)", config.mortality.agony_duration_cycles);
    }

    // Boot the agent: initialize all subsystems, load founding memories,
    // restore identity from DB if available, and fire the boot stimulus.
    agent.boot().await;

    // --- Communication channels ----------------------------------------------
    // ws_tx   : broadcast channel for WebSocket clients (chat + dashboard)
    // user_tx : mpsc channel for incoming user messages from the web UI
    // ctrl_tx : mpsc channel for control messages (parameter changes, resets)
    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_tx = Arc::new(ws_tx);
    let (user_tx, mut user_rx) = mpsc::channel::<UserMessage>(32);
    let (ctrl_tx, mut ctrl_rx) = mpsc::channel::<ControlMessage>(64);
    // Atomic flag signaling a graceful shutdown request.
    let shutdown = Arc::new(AtomicBool::new(false));

    agent.set_ws_tx(ws_tx.clone());
    // Wrap the agent in Arc<Mutex<>> for safe shared access across async tasks.
    let agent = Arc::new(Mutex::new(agent));

    // Create a per-IP rate limiter for the HTTP API.
    let rate_limiter = std::sync::Arc::new(
        saphire::api::middleware::RateLimiter::new(config.web_ui.rate_limit_per_minute)
    );

    // Build the shared application state passed to all Axum handlers.
    let app_state = AppState {
        ws_tx: ws_tx.clone(),
        user_tx: user_tx.clone(),
        ctrl_tx: ctrl_tx.clone(),
        shutdown: shutdown.clone(),
        agent: agent.clone(),
        dashboard_tx: dashboard_tx.clone(),
        logger: Some(logger.clone()),
        logs_db: logs_db.clone(),
        api_key: config.web_ui.api_key.clone(),
        allowed_origins: config.web_ui.allowed_origins.clone(),
        rate_limiter,
    };

    // Spawn the Axum web server on a background Tokio task.
    let web_host = config.web_ui.host.clone();
    let web_port = config.web_ui.port;
    let state_for_web = app_state.clone();
    tokio::spawn(async move {
        start_web_server(&web_host, web_port, state_for_web).await;
    });

    // Spawn a signal handler task that listens for SIGINT (Ctrl+C) and
    // SIGTERM, then sets the shutdown flag to gracefully terminate the
    // main cognitive loop.
    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = sigterm.recv() => {},
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await.ok();
        }
        println!("\n  Shutdown signal received...");
        shutdown_signal.store(true, Ordering::Relaxed);
    });

    if config.web_ui.api_key.is_some() {
        println!("  API authentication enabled (Bearer token)");
    } else {
        println!("  No API key configured -- endpoints are unprotected");
    }
    println!("  Rate limit: {} req/min", config.web_ui.rate_limit_per_minute);

    println!("\n  Saphire is active. Interface: http://{}:{}\n",
        config.web_ui.host, config.web_ui.port);

    // =========================================================================
    // Main cognitive life-loop
    //
    // Each iteration performs the following steps:
    //   1. Check the shutdown flag
    //   2. Check mortality state (if enabled) -- break if dead
    //   3. Drain all pending control messages (parameter changes, resets)
    //   4. Drain all pending user messages and send responses via WebSocket
    //   5. Perform one autonomous thought cycle (if the interval has elapsed)
    //   6. Wait for the next event (user message, control message, or timeout)
    // =========================================================================
    loop {
        // Step 1: Check for graceful shutdown request.
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        // Step 2: Mortality check -- if the agent has died, broadcast the
        // death event over WebSocket and terminate the loop.
        if config.mortality.enabled {
            let agent = agent.lock().await;
            if agent.body.mortality.state.is_dead() {
                let death_json = agent.body.mortality.to_json();
                println!("\n  Saphire has died. Cause: {}",
                    death_json["cause"].as_str().unwrap_or("unknown"));
                if let Some(thought) = death_json["last_thought"].as_str() {
                    println!("  Last thought: \"{}\"", thought);
                }
                let _ = ws_tx.send(serde_json::json!({
                    "type": "death",
                    "cause": death_json["cause"],
                    "last_thought": death_json["last_thought"],
                    "death_cycle": death_json["death_cycle"],
                }).to_string());
                drop(agent);
                break;
            }
            drop(agent);
        }

        // Step 3: Drain all pending control messages (non-blocking).
        while let Ok(ctrl) = ctrl_rx.try_recv() {
            let mut agent = agent.lock().await;
            handle_control_message(&mut agent, ctrl).await;
        }

        // Step 4: Drain all pending user messages (non-blocking) and
        // broadcast the agent's responses over WebSocket.
        while let Ok(msg) = user_rx.try_recv() {
            let mut agent = agent.lock().await;
            let response = agent.handle_human_message(&msg.text, &msg.username).await;
            let _ = ws_tx.send(serde_json::json!({
                "type": "chat_response",
                "content": response,
            }).to_string());
        }

        // Step 5: Autonomous thought -- the agent generates an internal
        // reflection if the configured thought interval has elapsed.
        {
            let mut agent = agent.lock().await;
            if let Some(thought) = agent.autonomous_think().await {
                if config.saphire.show_thoughts_in_terminal {
                    println!("  [Thought] {}", truncate(&thought, 80));
                }
            }
        }

        // Retrieve the current thought interval from the agent (it may
        // vary depending on arousal and sleep pressure).
        let interval = {
            let agent = agent.lock().await;
            agent.thought_interval()
        };

        // Step 6: Wait for the next event -- whichever comes first:
        //   - A new user message
        //   - A new control message
        //   - The thought interval timer expiring
        tokio::select! {
            Some(msg) = user_rx.recv() => {
                let mut agent = agent.lock().await;
                let response = agent.handle_human_message(&msg.text, &msg.username).await;
                let _ = ws_tx.send(serde_json::json!({
                    "type": "chat_response",
                    "content": response,
                }).to_string());
            },
            Some(ctrl) = ctrl_rx.recv() => {
                let mut agent = agent.lock().await;
                handle_control_message(&mut agent, ctrl).await;
            },
            _ = sleep(interval) => {},
        }
    }

    // Graceful shutdown: flush pending writes, persist state, release resources.
    let mut agent = agent.lock().await;
    agent.shutdown().await;
}

/// Truncates a string to at most `max` characters, appending "..." if truncated.
///
/// # Parameters
/// - `s`: the input string to truncate.
/// - `max`: the maximum number of Unicode characters to retain.
///
/// # Returns
/// The original string if it fits within `max` characters, otherwise the
/// first `max` characters followed by "...".
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}

/// Dispatches a control message to the appropriate handler on the agent.
///
/// Control messages are sent from the HTTP API or WebSocket interface and
/// allow runtime modification of the agent's parameters (neurochemical
/// baselines, decision thresholds, factory resets, etc.).
///
/// # Parameters
/// - `agent`: mutable reference to the cognitive agent.
/// - `ctrl`: the control message to process.
async fn handle_control_message(agent: &mut SaphireAgent, ctrl: ControlMessage) {
    match ctrl {
        ControlMessage::SetBaseline { molecule, value } => {
            // Adjust the homeostatic baseline for a specific neurotransmitter.
            agent.set_baseline(&molecule, value);
        },
        ControlMessage::SetModuleWeight { module: _, value: _ } => {
            // Not implemented in the lite version -- module weight adjustment
            // is omitted to keep the codebase minimal.
        },
        ControlMessage::SetThreshold { which, value } => {
            // Adjust a decision threshold (e.g., threshold_yes, threshold_no).
            agent.set_threshold(&which, value);
        },
        ControlMessage::SetParam { param, value } => {
            // Set a generic named parameter on the agent.
            agent.set_param(&param, value);
        },
        ControlMessage::EmergencyStabilize => {
            // Immediately clamp all neurochemical levels to safe ranges
            // and reset arousal to baseline. Used as a panic button.
            agent.emergency_stabilize();
        },
        ControlMessage::SuggestTopic { topic: _ } => {
            // Not implemented in the lite version -- the knowledge module
            // that handles topic suggestions is absent.
        },
        ControlMessage::FactoryReset { level } => {
            // Apply a factory reset at the specified level (chemistry-only,
            // parameters-only, or full reset).
            agent.apply_factory_reset(level).await;
        },
        ControlMessage::GetConfig { response_tx } => {
            // Return the current agent configuration as JSON via a oneshot channel.
            let _ = response_tx.send(agent.config_json());
        },
        ControlMessage::GetChemistry { response_tx } => {
            // Return the current neurochemical state as JSON via a oneshot channel.
            let _ = response_tx.send(agent.chemistry_json());
        },
    }
}

/// Starts the Axum HTTP/WebSocket server.
///
/// Binds to the specified host and port, then serves requests using the
/// router built by `saphire::api::build_router`.
///
/// # Parameters
/// - `host`: the IP address or hostname to bind to (e.g., "0.0.0.0").
/// - `port`: the TCP port number (e.g., 8080).
/// - `state`: shared application state injected into all route handlers.
async fn start_web_server(host: &str, port: u16, state: AppState) {
    let app = saphire::api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("Failed to bind the web server to the specified address");
    tracing::info!("Web server started on {}", addr);
    axum::serve(listener, app).await.ok();
}

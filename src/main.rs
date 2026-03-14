// main.rs — Saphire entry point
// HTTP/WebSocket handlers are in the saphire::api module (src/api/).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, broadcast, Mutex};
use tokio::time::sleep;

use saphire::config::SaphireConfig;
use saphire::db::SaphireDb;
use saphire::llm;
use saphire::plugins::PluginManager;
use saphire::plugins::web_ui::WebUiPlugin;
use saphire::plugins::micro_nn::MicroNNPlugin;
use saphire::plugins::vector_memory::VectorMemoryPlugin;
use saphire::agent::SaphireAgent;
use saphire::agent::lifecycle::UserMessage;
use saphire::logging::SaphireLogger;
use saphire::logging::db::LogsDb;

use saphire::api::{AppState, ControlMessage};

/// Main asynchronous entry point.
/// The `#[tokio::main]` macro creates the tokio runtime and executes this function.
#[tokio::main]
async fn main() {
    // Initialize the tracing system (structured logs).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           💎 SAPHIRE — Agent Cognitif Autonome              ║");
    println!("║                    Version 1.0.0                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Check command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let demo_mode = args.iter().any(|a| a == "--demo");

    let config_path = args.iter()
        .position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("saphire.toml");

    let config = match SaphireConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Erreur config : {}. Utilisation des valeurs par défaut.", e);
            SaphireConfig::default()
        }
    };

    tracing::info!("Configuration chargée");
    println!("  ⚙️  Configuration chargée");
    println!("  🧠 Modèle LLM : {}", config.llm.model);
    println!("  🌐 Interface web : {}:{}", config.plugins.web_ui.host, config.plugins.web_ui.port);

    let mut plugins = PluginManager::new();
    plugins.register(Box::new(WebUiPlugin::new()));
    plugins.register(Box::new(MicroNNPlugin::new(config.plugins.micro_nn.learning_rate)));
    plugins.register(Box::new(VectorMemoryPlugin::new(
        config.plugins.vector_memory.embedding_dimensions,
        config.plugins.vector_memory.max_memories,
    )));

    // Demo mode: mock LLM backend, no DB
    if demo_mode {
        println!("  🎬 Mode démonstration (sans DB, sans LLM)");
        let mock_llm = llm::create_backend(&saphire::llm::LlmConfig {
            backend: "mock".into(),
            ..Default::default()
        });
        let mut agent = SaphireAgent::new(config, mock_llm, plugins);
        agent.boot().await;
        saphire::pipeline::run_demo(&mut agent);
        agent.shutdown().await;
        return;
    }

    let llm_backend = llm::create_backend(&config.llm);
    println!("  🧠 Backend LLM : {} ({})", config.llm.backend, config.llm.model);

    let mut agent = SaphireAgent::new(config.clone(), llm_backend, plugins);

    // Connect to the PostgreSQL database
    println!("  🗄️  Connexion à PostgreSQL ({}:{})...", config.database.host, config.database.port);
    match SaphireDb::connect(&config.database).await {
        Ok(db) => {
            println!("  ✅ PostgreSQL connecté, migrations exécutées");
            tracing::info!("PostgreSQL connecté");
            agent.set_db(db);
        },
        Err(e) => {
            tracing::error!("PostgreSQL indisponible : {}. Mode dégradé (sans persistance).", e);
            println!("  ⚠️  PostgreSQL indisponible : {}. Mode dégradé.", e);
        }
    }

    // Connect to the logs database (separate)
    println!("  🗄️  Connexion à la base de logs ({}:{})...", config.logs_database.host, config.logs_database.port);
    let logs_db: Option<Arc<LogsDb>> = match LogsDb::connect(&config.logs_database).await {
        Ok(db) => {
            println!("  ✅ Base de logs connectée, migrations exécutées");
            tracing::info!("LogsDb connectée");
            Some(Arc::new(db))
        },
        Err(e) => {
            tracing::warn!("LogsDb indisponible : {}. Logs en mode terminal uniquement.", e);
            println!("  ⚠️  Base de logs indisponible : {}. Mode terminal.", e);
            None
        }
    };

    let (dashboard_tx, _) = broadcast::channel::<String>(100);
    let dashboard_tx = Arc::new(dashboard_tx);

    let logger = Arc::new(Mutex::new(SaphireLogger::new(
        logs_db.clone(),
        Some(dashboard_tx.clone()),
    )));

    agent.set_logger(logger.clone());
    if let Some(ref ldb) = logs_db {
        agent.set_logs_db(ldb.clone());
    }

    // Hardware detection
    if config.hardware.auto_detect {
        // Extract Ollama base URL from base_url (remove /v1 if present)
        let ollama_url = config.llm.base_url.trim_end_matches("/v1").to_string();
        let hw = saphire::hardware::HardwareProfile::detect(&ollama_url);
        if config.hardware.log_profile {
            hw.log_summary();
        }
        let rec = hw.recommend(&config.llm.model);
        for warning in &rec.warnings {
            println!("  ⚠️  Hardware: {}", warning);
        }
        agent.hardware_profile = Some(hw);
    }

    // Deterministic genome generation
    if config.genome.enabled {
        let genome = saphire::genome::Genome::from_seed(config.genome.seed);
        genome.log_summary();

        // Apply chemical genes to baselines
        if config.genome.apply_at_boot {
            let chem = &genome.chemical;
            agent.adjust_baseline("dopamine", chem.baseline_dopamine_offset);
            agent.adjust_baseline("serotonin", chem.baseline_serotonin_offset);
            agent.adjust_baseline("cortisol", chem.baseline_cortisol_offset);
            println!("  🧬 Gènes chimiques appliqués aux baselines");
        }

        agent.genome = Some(genome);
    }

    // Mortality configuration
    if config.mortality.enabled {
        agent.body.set_mortality_config(config.mortality.agony_duration_cycles);
        println!("  {} Mortalite activee (agonie: {} cycles)", '\u{1F480}', config.mortality.agony_duration_cycles);
    }

    agent.boot().await;

    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_tx = Arc::new(ws_tx);
    let (user_tx, mut user_rx) = mpsc::channel::<UserMessage>(32);
    let (ctrl_tx, mut ctrl_rx) = mpsc::channel::<ControlMessage>(64);
    let shutdown = Arc::new(AtomicBool::new(false));

    agent.set_ws_tx(ws_tx.clone());
    let agent = Arc::new(Mutex::new(agent));

    let rate_limiter = std::sync::Arc::new(
        saphire::api::middleware::RateLimiter::new(config.plugins.web_ui.rate_limit_per_minute)
    );

    let app_state = AppState {
        ws_tx: ws_tx.clone(),
        user_tx: user_tx.clone(),
        ctrl_tx: ctrl_tx.clone(),
        shutdown: shutdown.clone(),
        agent: agent.clone(),
        dashboard_tx: dashboard_tx.clone(),
        logger: Some(logger.clone()),
        logs_db: logs_db.clone(),
        api_key: config.plugins.web_ui.api_key.clone(),
        allowed_origins: config.plugins.web_ui.allowed_origins.clone(),
        rate_limiter,
    };

    // Start the web server
    let web_host = config.plugins.web_ui.host.clone();
    let web_port = config.plugins.web_ui.port;
    let state_for_web = app_state.clone();
    tokio::spawn(async move {
        start_web_server(&web_host, web_port, state_for_web).await;
    });

    // Handler for shutdown signals (SIGINT + SIGTERM)
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
        println!("\n  ⚡ Signal d'arrêt reçu...");
        shutdown_signal.store(true, Ordering::Relaxed);
    });

    if config.plugins.web_ui.api_key.is_some() {
        println!("  🔐 Authentification API activée (Bearer token)");
    } else {
        println!("  ⚠️  Pas de clé API configurée — endpoints non protégés");
    }
    println!("  🛡️  Rate limit : {} req/min", config.plugins.web_ui.rate_limit_per_minute);

    println!("\n  🚀 Saphire est active. Interface : http://{}:{}\n",
        config.plugins.web_ui.host, config.plugins.web_ui.port);

    // ─── Main life loop ───────────────────────────
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        // Mortality check: if Saphire is dead, stop the loop
        if config.mortality.enabled {
            let agent = agent.lock().await;
            if agent.body.mortality.state.is_dead() {
                let mort_json = agent.body.mortality.to_json();
                println!("\n  {} Saphire est morte. Cause: {}", '\u{1F480}',
                    mort_json["cause"].as_str().unwrap_or("inconnue"));
                if let Some(thought) = mort_json["last_thought"].as_str() {
                    println!("  Derniere pensee: \"{}\"", thought);
                }
                let _ = ws_tx.send(serde_json::json!({
                    "type": "death",
                    "cause": mort_json["cause"],
                    "last_thought": mort_json["last_thought"],
                    "death_cycle": mort_json["death_cycle"],
                }).to_string());
                drop(agent);
                break;
            }
            drop(agent);
        }

        // Drain all pending control messages
        while let Ok(ctrl) = ctrl_rx.try_recv() {
            let mut agent = agent.lock().await;
            handle_control_message(&mut agent, ctrl).await;
        }

        // Process pending user messages
        while let Ok(msg) = user_rx.try_recv() {
            let mut agent = agent.lock().await;
            let chat_resp = agent.handle_human_message(&msg.text, &msg.username).await;
            let _ = ws_tx.send(serde_json::json!({
                "type": "chat_response",
                "content": chat_resp.text,
                "markers": {
                    "emotion": chat_resp.emotion,
                    "consciousness": chat_resp.consciousness,
                    "reflexes": chat_resp.reflexes,
                    "register": chat_resp.register,
                    "involves_memory": chat_resp.involves_memory,
                    "confidence": chat_resp.confidence,
                }
            }).to_string());

            // Vocalize the response via Sensoria (TTS)
            speak_via_sensoria(chat_resp.text.clone(), chat_resp.emotion.clone());
        }

        // Autonomous thought or sleep tick
        {
            let mut agent = agent.lock().await;
            if agent.sleep.is_sleeping {
                // Saphire is sleeping: execute sleep tick
                agent.sleep_tick().await;
            } else {
                // Update sleep pressure + subconscious
                agent.update_sleep_pressure().await;

                if agent.should_initiate_sleep() {
                    agent.initiate_sleep().await;
                } else if let Some(thought) = agent.autonomous_think().await {
                    if config.saphire.show_thoughts_in_terminal {
                        println!("  💭 [Pensée] {}", truncate(&thought, 80));
                    }
                }
            }
        }

        let interval = {
            let agent = agent.lock().await;
            agent.thought_interval()
        };

        tokio::select! {
            Some(msg) = user_rx.recv() => {
                let mut agent = agent.lock().await;
                let chat_resp = agent.handle_human_message(&msg.text, &msg.username).await;
                let _ = ws_tx.send(serde_json::json!({
                    "type": "chat_response",
                    "content": chat_resp.text,
                    "markers": {
                        "emotion": chat_resp.emotion,
                        "consciousness": chat_resp.consciousness,
                        "reflexes": chat_resp.reflexes,
                        "register": chat_resp.register,
                        "involves_memory": chat_resp.involves_memory,
                        "confidence": chat_resp.confidence,
                    }
                }).to_string());

                // Vocalize the response via Sensoria (TTS)
                speak_via_sensoria(chat_resp.text.clone(), chat_resp.emotion.clone());
            },
            Some(ctrl) = ctrl_rx.recv() => {
                let mut agent = agent.lock().await;
                handle_control_message(&mut agent, ctrl).await;
            },
            _ = sleep(interval) => {},
        }
    }

    // Graceful shutdown
    let mut agent = agent.lock().await;
    agent.shutdown().await;
}

/// Sends text to Sensoria for speech synthesis (non-blocking).
fn speak_via_sensoria(text: String, emotion: String) {
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            ureq::post("http://192.168.1.129:9090/api/speak")
                .set("Content-Type", "application/json")
                .send_string(&serde_json::json!({
                    "text": text,
                    "emotion": emotion,
                }).to_string())
        }).await;

        match result {
            Ok(Ok(resp)) if resp.status() == 200 => {
                tracing::debug!("[SENSORIA] TTS envoyé avec succès");
            }
            Ok(Ok(resp)) => {
                tracing::warn!("[SENSORIA] TTS réponse {}", resp.status());
            }
            Ok(Err(e)) => {
                tracing::debug!("[SENSORIA] TTS injoignable : {}", e);
            }
            Err(e) => {
                tracing::debug!("[SENSORIA] TTS erreur interne : {}", e);
            }
        }
    });
}

/// Truncates a string to `max` characters.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}

/// Processes a control message by applying the corresponding action on the agent.
async fn handle_control_message(agent: &mut SaphireAgent, ctrl: ControlMessage) {
    match ctrl {
        ControlMessage::SetBaseline { molecule, value } => {
            agent.set_baseline(&molecule, value);
        },
        ControlMessage::SetModuleWeight { module, value } => {
            agent.set_module_weight(&module, value);
        },
        ControlMessage::SetThreshold { which, value } => {
            agent.set_threshold(&which, value);
        },
        ControlMessage::SetParam { param, value } => {
            agent.set_param(&param, value);
        },
        ControlMessage::EmergencyStabilize => {
            agent.emergency_stabilize();
        },
        ControlMessage::SuggestTopic { topic } => {
            agent.suggest_topic(topic);
        },
        ControlMessage::FactoryReset { level } => {
            agent.apply_factory_reset(level).await;
        },
        ControlMessage::GetConfig { response_tx } => {
            let _ = response_tx.send(agent.config_json());
        },
        ControlMessage::GetChemistry { response_tx } => {
            let _ = response_tx.send(agent.chemistry_json());
        },
    }
}

/// Starts the axum web server using the router built by the api module.
async fn start_web_server(host: &str, port: u16, state: AppState) {
    let app = saphire::api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("Impossible de bind le serveur web");
    tracing::info!("Serveur web démarré sur {}", addr);
    axum::serve(listener, app).await.ok();
}

// =============================================================================
// config/loader.rs — Configuration loading (TOML file + environment overrides)
// =============================================================================

use std::path::Path;
use super::SaphireConfig;

impl SaphireConfig {
    /// Loads the configuration from a TOML file at the given `path`.
    ///
    /// If the file does not exist, returns the default configuration.
    /// After loading, environment variable overrides are applied on top
    /// (useful for Docker and containerized deployments where the TOML
    /// file may not be editable).
    ///
    /// # Parameters
    /// - `path`: filesystem path to the `saphire.toml` configuration file.
    ///
    /// # Returns
    /// The fully resolved `SaphireConfig`, or a human-readable error string.
    pub fn load(path: &str) -> Result<Self, String> {
        if !Path::new(path).exists() {
            tracing::info!("Pas de fichier config '{}', utilisation des valeurs par défaut.", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Lecture config: {}", e))?;

        let mut config: SaphireConfig = toml::from_str(&content)
            .map_err(|e| format!("Parse config: {}", e))?;

        // Apply environment variable overrides (highest priority)
        config.apply_env_overrides();

        Ok(config)
    }

    /// Applies overrides from environment variables onto the current configuration.
    ///
    /// This allows modifying the configuration without touching the TOML file,
    /// which is particularly useful in Docker containers, CI/CD pipelines,
    /// and other environments where configuration files are baked into images.
    ///
    /// Each supported environment variable maps to a specific configuration field.
    /// If a variable is not set or cannot be parsed, the corresponding field
    /// retains its value from the TOML file (or its default).
    fn apply_env_overrides(&mut self) {
        // Primary database connection
        if let Ok(host) = std::env::var("SAPHIRE_DB_HOST") {
            self.database.host = host;
        }
        if let Ok(port) = std::env::var("SAPHIRE_DB_PORT") {
            if let Ok(p) = port.parse() {
                self.database.port = p;
            }
        }
        if let Ok(user) = std::env::var("SAPHIRE_DB_USER") {
            self.database.user = user;
        }
        if let Ok(pass) = std::env::var("SAPHIRE_DB_PASSWORD") {
            self.database.password = pass;
        }
        if let Ok(name) = std::env::var("SAPHIRE_DB_NAME") {
            self.database.dbname = name;
        }

        // Logs database connection (separate database for audit/telemetry)
        if let Ok(host) = std::env::var("SAPHIRE_LOGS_DB_HOST") {
            self.logs_database.host = host;
        }
        if let Ok(port) = std::env::var("SAPHIRE_LOGS_DB_PORT") {
            if let Ok(p) = port.parse() {
                self.logs_database.port = p;
            }
        }
        if let Ok(user) = std::env::var("SAPHIRE_LOGS_DB_USER") {
            self.logs_database.user = user;
        }
        if let Ok(pass) = std::env::var("SAPHIRE_LOGS_DB_PASSWORD") {
            self.logs_database.password = pass;
        }
        if let Ok(name) = std::env::var("SAPHIRE_LOGS_DB_NAME") {
            self.logs_database.dbname = name;
        }

        // LLM (Large Language Model) backend
        if let Ok(url) = std::env::var("SAPHIRE_LLM_URL") {
            self.llm.base_url = url;
        }
        if let Ok(model) = std::env::var("SAPHIRE_LLM_MODEL") {
            self.llm.model = model;
        }

        // Web server (HTTP + WebSocket UI)
        if let Ok(host) = std::env::var("SAPHIRE_WEB_HOST") {
            self.web_ui.host = host;
        }
        if let Ok(port) = std::env::var("SAPHIRE_WEB_PORT") {
            if let Ok(p) = port.parse() {
                self.web_ui.port = p;
            }
        }

        // API key for endpoint authentication
        if let Ok(key) = std::env::var("SAPHIRE_API_KEY") {
            if !key.is_empty() {
                self.web_ui.api_key = Some(key);
            }
        }
    }
}

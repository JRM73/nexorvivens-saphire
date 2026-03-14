// =============================================================================
// config/loader.rs — Configuration loading (TOML file + env vars)
// =============================================================================

use std::path::Path;
use super::SaphireConfig;

impl SaphireConfig {
    /// Loads configuration from a TOML file.
    /// If the file does not exist, returns the default configuration.
    /// After loading, environment variables are applied as overrides
    /// (useful for Docker/containerized deployments).
    pub fn load(path: &str) -> Result<Self, String> {
        if !Path::new(path).exists() {
            tracing::info!("Pas de fichier config '{}', utilisation des valeurs par défaut.", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Lecture config: {}", e))?;

        let mut config: SaphireConfig = toml::from_str(&content)
            .map_err(|e| format!("Parse config: {}", e))?;

        // Override with environment variables (highest priority)
        config.apply_env_overrides();

        Ok(config)
    }

    /// Applies overrides from environment variables.
    /// This allows modifying the configuration without touching the TOML file,
    /// which is particularly useful in Docker or CI/CD environments.
    fn apply_env_overrides(&mut self) {
        // Database
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

        // Logs database
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

        // LLM (Large Language Model)
        if let Ok(url) = std::env::var("SAPHIRE_LLM_URL") {
            self.llm.base_url = url;
        }
        if let Ok(model) = std::env::var("SAPHIRE_LLM_MODEL") {
            self.llm.model = model;
        }
        if let Ok(url) = std::env::var("SAPHIRE_EMBED_URL") {
            self.llm.embed_base_url = Some(url);
        }

        // Web server
        if let Ok(host) = std::env::var("SAPHIRE_WEB_HOST") {
            self.plugins.web_ui.host = host;
        }
        if let Ok(port) = std::env::var("SAPHIRE_WEB_PORT") {
            if let Ok(p) = port.parse() {
                self.plugins.web_ui.port = p;
            }
        }

        // API key for endpoint authentication
        if let Ok(key) = std::env::var("SAPHIRE_API_KEY") {
            if !key.is_empty() {
                self.plugins.web_ui.api_key = Some(key);
            }
        }
    }
}

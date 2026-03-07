// =============================================================================
// config/mod.rs — Saphire configuration (saphire.toml + environment variables)
//
// Purpose: This module defines all configuration structures for Saphire.
// It loads parameters from a TOML file and supports overriding via
// environment variables (useful for Docker and containerized deployments).
// =============================================================================

mod structures;
mod loader;

pub use structures::*;

// =============================================================================
// config/mod.rs — Saphire configuration (saphire.toml + environment variables)
//
// Purpose: Defines all configuration structures for Saphire.
// Loads parameters from a TOML file and allows overrides via
// environment variables (useful for Docker deployments).
// =============================================================================

mod structures;
mod loader;

pub use structures::*;

// =============================================================================
// config/mod.rs — Configuration de Saphire (saphire.toml + variables d'env)
//
// Role : Ce module definit toutes les structures de configuration de Saphire.
// Il charge les parametres depuis un fichier TOML et permet la surcharge par
// des variables d'environnement (utile pour les deploiements Docker).
// =============================================================================

mod structures;
mod loader;

pub use structures::*;

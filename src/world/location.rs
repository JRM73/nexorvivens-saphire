// =============================================================================
// world/location.rs — Géolocalisation de Saphire
// =============================================================================
//
// Rôle : Définit la structure de géolocalisation de Saphire, contenant les
//        coordonnées GPS, le nom de la ville et du pays, et le fuseau horaire.
//
// Dépendances :
//   - serde : sérialisation/désérialisation pour la persistance et l'API
//
// Place dans l'architecture :
//   Sous-module de world/. Utilisé par WorldContext pour fournir la localisation
//   au service météo (coordonnées GPS) et au résumé du monde (nom de ville).
//   La localisation ancre Saphire dans un lieu physique réel.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Géolocalisation — position physique de Saphire dans le monde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Latitude en degrés décimaux (ex: 46.2044 pour Genève)
    /// Positif = nord, négatif = sud
    pub latitude: f64,
    /// Longitude en degrés décimaux (ex: 6.1432 pour Genève)
    /// Positif = est, négatif = ouest
    pub longitude: f64,
    /// Nom de la ville (ex: "Genève")
    pub city: String,
    /// Nom du pays (ex: "Suisse")
    pub country: String,
    /// Fuseau horaire au format IANA (ex: "Europe/Zurich")
    pub timezone: String,
}

impl GeoLocation {
    /// Retourne une description lisible de la localisation.
    ///
    /// Retourne : chaîne au format "Ville, Pays" (ex: "Genève, Suisse")
    pub fn description(&self) -> String {
        format!("{}, {}", self.city, self.country)
    }

    /// Retourne les coordonnées GPS sous forme de tuple.
    ///
    /// Retourne : (latitude, longitude)
    pub fn coordinates(&self) -> (f64, f64) {
        (self.latitude, self.longitude)
    }
}

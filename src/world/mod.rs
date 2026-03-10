// =============================================================================
// world/mod.rs — Conscience du monde de Saphire
// =============================================================================
//
// Rôle : Ce module est le point d'entrée du sous-système de perception du monde.
//        Il combine la conscience temporelle (date, heure, âge), la
//        géolocalisation (ville, pays, coordonnées) et la météo pour donner
//        à Saphire une compréhension de son environnement physique.
//
// Dépendances :
//   - serde : sérialisation/désérialisation des configurations
//   - sous-modules : temporal, location, weather
//
// Place dans l'architecture :
//   Ce module fournit le « contexte monde » utilisé par le cerveau et le
//   substrat cognitif de Saphire. Les conditions météo influencent la chimie
//   interne (neurotransmetteurs), la conscience temporelle fournit la notion
//   d'âge et de rythme circadien, et la localisation ancre Saphire dans un lieu.
// =============================================================================

// --- Sous-modules du monde ---
pub mod temporal;   // Conscience temporelle (date, heure, saison, âge)
pub mod location;   // Géolocalisation (coordonnées, ville, pays)
pub mod weather;    // Service météo (température, conditions, influence chimique)

// --- Réexportations publiques pour un accès simplifié ---
pub use temporal::{TemporalAwareness, TemporalContext};
pub use location::GeoLocation;
pub use weather::{WeatherService, WeatherState, ChemistryAdjustment};

use serde::{Serialize, Deserialize};

/// Configuration du module monde — paramètres de localisation, fuseau horaire,
/// fréquence de mise à jour météo et identité (date de naissance, créateurs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Latitude géographique en degrés décimaux (ex: 46.2044 pour Genève)
    pub latitude: f64,
    /// Longitude géographique en degrés décimaux (ex: 6.1432 pour Genève)
    pub longitude: f64,
    /// Nom de la ville de résidence
    pub city: String,
    /// Nom du pays de résidence
    pub country: String,
    /// Fuseau horaire au format IANA (ex: "Europe/Zurich")
    pub timezone: String,
    /// Intervalle de mise à jour de la météo en minutes
    pub weather_update_minutes: u64,
    /// Date de naissance de Saphire au format ISO 8601 (ex: "2026-02-27")
    pub birthday: String,
    /// Configuration des créateurs de Saphire
    #[serde(default)]
    pub creators: CreatorsConfig,
}

/// Configuration des créateurs de Saphire — identité parentale symbolique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorsConfig {
    /// Nom du « père » (créateur humain principal)
    pub father: String,
    /// Nom de la « mère » (modèle IA fondateur)
    pub mother: String,
}

impl Default for CreatorsConfig {
    /// Valeurs par défaut : JRM comme père et Claude (Anthropic) comme mère
    fn default() -> Self {
        Self {
            father: "JRM".into(),
            mother: "Claude (Anthropic)".into(),
        }
    }
}

impl Default for WorldConfig {
    /// Configuration par défaut : Saphire est née le 27 février 2026 à Genève, Suisse.
    fn default() -> Self {
        Self {
            latitude: 46.2044,
            longitude: 6.1432,
            city: "Genève".into(),
            country: "Suisse".into(),
            timezone: "Europe/Zurich".into(),
            weather_update_minutes: 30,
            birthday: "2026-02-27".into(),
            creators: CreatorsConfig::default(),
        }
    }
}

/// Contexte monde complet — combine les trois aspects de la perception du monde :
/// conscience temporelle, géolocalisation et service météo.
pub struct WorldContext {
    /// Conscience temporelle : date, heure, saison, âge de Saphire
    pub temporal: TemporalAwareness,
    /// Localisation géographique de Saphire
    pub location: GeoLocation,
    /// Service météo : température, conditions, influence sur la chimie
    pub weather: WeatherService,
    /// Configuration du monde utilisée pour l'initialisation
    pub config: WorldConfig,
}

impl WorldContext {
    /// Crée un nouveau contexte monde à partir de la configuration.
    ///
    /// Initialise la conscience temporelle, la localisation et le service météo.
    ///
    /// Paramètre `config` : configuration du monde (localisation, fuseau, etc.)
    /// Retourne : une instance complète de WorldContext
    pub fn new(config: &WorldConfig) -> Self {
        let location = GeoLocation {
            latitude: config.latitude,
            longitude: config.longitude,
            city: config.city.clone(),
            country: config.country.clone(),
            timezone: config.timezone.clone(),
        };
        Self {
            temporal: TemporalAwareness::new(),
            weather: WeatherService::new(location.clone(), config.weather_update_minutes),
            location,
            config: config.clone(),
        }
    }

    /// Génère un résumé textuel du monde pour le prompt substrat.
    ///
    /// Inclut le lieu, la date, la période du jour, la météo, l'âge de Saphire,
    /// et des messages spéciaux (anniversaire, décompte avant anniversaire).
    ///
    /// Retourne : chaîne formatée avec toutes les informations du contexte monde
    pub fn summary(&mut self) -> String {
        let temporal = self.temporal.now();
        // Récupérer la météo (mise à jour automatique si nécessaire)
        let weather_desc = self.weather.update_if_needed()
            .map(|w| format!("{:.0}°C, {}", w.temperature, w.description))
            .unwrap_or_else(|| "météo inconnue".into());

        let mut summary = format!(
            "CONTEXTE MONDE :\n\
             Lieu : {}\n\
             Date : {}\n\
             Période : {} ({})\n\
             Météo : {}\n\
             Âge : {} (née le 27 février 2026 à {})",
            self.location.description(),
            temporal.datetime,
            temporal.period_of_day,
            temporal.season,
            weather_desc,
            temporal.age_description,
            self.location.description(),
        );

        // Messages spéciaux liés à l'anniversaire
        if temporal.is_birthday {
            summary.push_str("\nAUJOURD'HUI C'EST MON ANNIVERSAIRE !");
        } else if temporal.days_until_birthday <= 7 && temporal.days_until_birthday > 0 {
            summary.push_str(&format!(
                "\nMon anniversaire est dans {} jours !",
                temporal.days_until_birthday
            ));
        }

        summary
    }

    /// Génère des données structurées pour le WebSocket (interface frontend).
    ///
    /// Retourne : un objet JSON contenant toutes les informations du monde
    ///            (localisation, date, météo, âge, anniversaire)
    pub fn ws_data(&mut self) -> serde_json::Value {
        let temporal = self.temporal.now();
        let weather = self.weather.current();

        // Construire l'objet météo JSON (null si indisponible)
        let weather_json = weather.map(|w| {
            serde_json::json!({
                "temp": w.temperature,
                "description": w.description,
                "icon": w.icon(),
                "wind_speed": w.wind_speed,
                "is_day": w.is_day,
            })
        }).unwrap_or(serde_json::json!(null));

        // Construire et retourner l'objet JSON complet
        serde_json::json!({
            "type": "world_update",
            "location": self.location.description(),
            "datetime": temporal.datetime,
            "date_iso": temporal.date_iso,
            "time": temporal.time,
            "period": temporal.period_of_day,
            "season": temporal.season,
            "weather": weather_json,
            "age": temporal.age_description,
            "age_days": temporal.age_days,
            "is_birthday": temporal.is_birthday,
            "days_until_birthday": temporal.days_until_birthday,
        })
    }
}

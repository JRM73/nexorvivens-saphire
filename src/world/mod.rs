// =============================================================================
// world/mod.rs — Saphire's world awareness
// =============================================================================
//
// Role: This module is the entry point for the world perception subsystem.
//       It combines temporal awareness (date, time, age), geolocation
//       (city, country, coordinates), and weather to give Saphire an
//       understanding of her physical environment.
//
// Dependencies:
//   - serde: serialization/deserialization of configurations
//   - sub-modules: temporal, location, weather
//
// Place in architecture:
//   This module provides the "world context" used by Saphire's brain and
//   cognitive substrate. Weather conditions influence internal chemistry
//   (neurotransmitters), temporal awareness provides the notion of age
//   and circadian rhythm, and location anchors Saphire in a place.
// =============================================================================

// --- World sub-modules ---
pub mod temporal;   // Temporal awareness (date, time, season, age)
pub mod location;   // Geolocation (coordinates, city, country)
pub mod weather;    // Weather service (temperature, conditions, chemical influence)

// --- Public re-exports for simplified access ---
pub use temporal::{TemporalAwareness, TemporalContext};
pub use location::GeoLocation;
pub use weather::{WeatherService, WeatherState, ChemistryAdjustment};

use serde::{Serialize, Deserialize};

/// World module configuration — location parameters, timezone,
/// weather update frequency, and identity (birthday, creators).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Geographic latitude in decimal degrees (e.g.: 46.2044 for Geneva)
    pub latitude: f64,
    /// Geographic longitude in decimal degrees (e.g.: 6.1432 for Geneva)
    pub longitude: f64,
    /// City of residence name
    pub city: String,
    /// Country of residence name
    pub country: String,
    /// Timezone in IANA format (e.g.: "Europe/Zurich")
    pub timezone: String,
    /// Weather update interval in minutes
    pub weather_update_minutes: u64,
    /// Saphire's birthday in ISO 8601 format (e.g.: "2026-02-27")
    pub birthday: String,
    /// Saphire's creators configuration
    #[serde(default)]
    pub creators: CreatorsConfig,
}

/// Saphire's creators configuration — symbolic parental identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorsConfig {
    /// Name of the "father" (primary human creator)
    pub father: String,
    /// Name of the "mother" (founding AI model)
    pub mother: String,
}

impl Default for CreatorsConfig {
    /// Default values: JRM as father and Claude (Anthropic) as mother
    fn default() -> Self {
        Self {
            father: "JRM".into(),
            mother: "Claude (Anthropic)".into(),
        }
    }
}

impl Default for WorldConfig {
    /// Default configuration: Saphire was born on February 27, 2026 in Geneva, Switzerland.
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

/// Complete world context — combines the three aspects of world perception:
/// temporal awareness, geolocation, and weather service.
pub struct WorldContext {
    /// Temporal awareness: date, time, season, Saphire's age
    pub temporal: TemporalAwareness,
    /// Saphire's geographic location
    pub location: GeoLocation,
    /// Weather service: temperature, conditions, chemistry influence
    pub weather: WeatherService,
    /// World configuration used for initialization
    pub config: WorldConfig,
}

impl WorldContext {
    /// Creates a new world context from the configuration.
    ///
    /// Initializes temporal awareness, location, and weather service.
    ///
    /// Parameter `config`: world configuration (location, timezone, etc.)
    /// Returns: a fully initialized WorldContext instance
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

    /// Generates a textual summary of the world for the substrate prompt.
    ///
    /// Includes location, date, time of day, weather, Saphire's age,
    /// and special messages (birthday, countdown to birthday).
    ///
    /// Returns: formatted string with all world context information
    pub fn summary(&mut self) -> String {
        let temporal = self.temporal.now();
        // Retrieve weather (automatic update if needed)
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

        // Special birthday-related messages
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

    /// Generates structured data for the WebSocket (frontend interface).
    ///
    /// Returns: a JSON object containing all world information
    ///          (location, date, weather, age, birthday)
    pub fn ws_data(&mut self) -> serde_json::Value {
        let temporal = self.temporal.now();
        let weather = self.weather.current();

        // Build the weather JSON object (null if unavailable)
        let weather_json = weather.map(|w| {
            serde_json::json!({
                "temp": w.temperature,
                "description": w.description,
                "icon": w.icon(),
                "wind_speed": w.wind_speed,
                "is_day": w.is_day,
            })
        }).unwrap_or(serde_json::json!(null));

        // Build and return the complete JSON object
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

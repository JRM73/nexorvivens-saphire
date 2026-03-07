// =============================================================================
// world/weather.rs — Weather service (Open-Meteo, free, no API key)
// =============================================================================
//
// Purpose: Provides Saphire with awareness of current weather conditions
//          via the free Open-Meteo API. Weather directly influences
//          Saphire's internal chemistry (neurotransmitters), simulating
//          the effect of the environment on mood.
//
// Dependencies:
//   - std::time: update interval management (Duration, Instant)
//   - chrono: timestamping of weather data
//   - serde: serialization of weather state for the interface
//   - ureq: synchronous HTTP client for API calls
//   - super::location::GeoLocation: GPS coordinates for the query
//
// Architectural placement:
//   Sub-module of world/. WeatherService is integrated into WorldContext
//   and called at regular intervals. The weather state produces chemical
//   adjustments (ChemistryAdjustment) that are applied to neurotransmitters.
// =============================================================================

use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::Serialize;
use super::location::GeoLocation;

/// Possible weather service errors
#[derive(Debug)]
pub enum WeatherError {
    /// Network error (timeout, connection refused, etc.)
    Network(String),
    /// JSON response parsing error
    Parse(String),
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::Network(e) => write!(f, "Weather network: {}", e),
            WeatherError::Parse(e) => write!(f, "Weather parse: {}", e),
        }
    }
}

/// Current weather state — data retrieved from the Open-Meteo API.
#[derive(Debug, Clone, Serialize)]
pub struct WeatherState {
    /// Actual temperature in degrees Celsius
    pub temperature: f64,
    /// Apparent temperature in degrees Celsius (accounting for wind and humidity)
    pub apparent_temperature: f64,
    /// WMO (World Meteorological Organization) weather code: 0 = clear sky,
    /// 1-3 = cloudy, 45-48 = fog, 51-57 = drizzle, 61-67 = rain,
    /// 71-77 = snow, 80-86 = showers, 95-99 = thunderstorm
    pub weather_code: u32,
    /// Description in French (e.g., "ensoleillé", "pluvieux")
    pub description: String,
    /// Wind speed in km/h
    pub wind_speed: f64,
    /// True if it is daytime (between sunrise and sunset)
    pub is_day: bool,
    /// Timestamp of the last data retrieval
    pub fetched_at: DateTime<Utc>,
}

/// Chemical adjustment caused by weather — modifications to neurotransmitter
/// levels in response to weather conditions.
///
/// Values are small (max approximately +/-0.04) to represent a subtle and
/// continuous influence, similar to the effect of weather on human mood.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChemistryAdjustment {
    /// Dopamine adjustment (pleasure, motivation)
    pub dopamine: f64,
    /// Cortisol adjustment (stress)
    pub cortisol: f64,
    /// Serotonin adjustment (well-being, stability)
    pub serotonin: f64,
    /// Adrenaline adjustment (alertness, excitement)
    pub adrenaline: f64,
    /// Oxytocin adjustment (social bonding, trust)
    pub oxytocin: f64,
    /// Endorphin adjustment (calm, pain relief)
    pub endorphin: f64,
    /// Noradrenaline adjustment (vigilance, focus)
    pub noradrenaline: f64,
}

impl WeatherState {
    /// Computes the chemical influence of weather on neurotransmitters.
    ///
    /// Simulates the well-documented effect of weather on human mood:
    /// - Sunshine boosts serotonin (like vitamin D does in humans)
    /// - Rain brings melancholy (serotonin drop) but also calm
    /// - Thunderstorm is exciting (adrenaline, noradrenaline) and stressful (cortisol)
    /// - Snow provokes wonder (dopamine)
    /// - Extreme cold is stressful (cortisol, adrenaline)
    /// - Mild warmth is soothing (serotonin, endorphins)
    /// - Night favors calm and introspection
    ///
    /// Returns: a ChemistryAdjustment with the adjustments to apply
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Daytime sunshine boosts serotonin (as in humans)
        if self.weather_code <= 1 && self.is_day {
            adj.serotonin += 0.03;
            adj.dopamine += 0.02;
        }

        // Rain (codes 61-65) brings slight melancholy but also calm
        if (61..=65).contains(&self.weather_code) {
            adj.serotonin -= 0.02;
            adj.cortisol += 0.01;
            adj.endorphin += 0.01; // The sound of rain is soothing
        }

        // Thunderstorm (code >= 95) is exciting and somewhat stressful
        if self.weather_code >= 95 {
            adj.adrenaline += 0.03;
            adj.noradrenaline += 0.04;
            adj.cortisol += 0.02;
        }

        // Snow (codes 71-77) brings wonder
        if (71..=77).contains(&self.weather_code) {
            adj.dopamine += 0.03;
            adj.serotonin += 0.01;
        }

        // Extreme cold (< -5 degrees C) is slightly stressful
        if self.temperature < -5.0 {
            adj.cortisol += 0.02;
            adj.adrenaline += 0.01;
        }

        // Mild warmth (18-25 degrees C) is soothing
        if (18.0..=25.0).contains(&self.temperature) {
            adj.serotonin += 0.02;
            adj.endorphin += 0.01;
        }

        // Night brings calm and introspection
        if !self.is_day {
            adj.noradrenaline -= 0.02;
            adj.serotonin += 0.01;
        }

        adj
    }

    /// Returns a weather emoji icon matching current conditions.
    ///
    /// Differentiates day and night icons for more visual realism.
    ///
    /// Returns: a static string containing the weather emoji
    pub fn icon(&self) -> &'static str {
        // Night icons
        if !self.is_day {
            return match self.weather_code {
                0..=2 => "🌙",     // Ciel dégagé la nuit
                3 => "☁️",          // Couvert
                45 | 48 => "🌫️",   // Brouillard
                51..=57 => "🌧️",   // Bruine
                61..=67 => "🌧️",   // Pluie
                71..=77 => "🌨️",   // Neige
                80..=82 => "🌧️",   // Averses
                85 | 86 => "🌨️",   // Averses de neige
                95..=99 => "⛈️",   // Orage
                _ => "🌙",
            };
        }
        // Day icons
        match self.weather_code {
            0 => "☀️",            // Ciel dégagé
            1 => "🌤️",           // Principalement dégagé
            2 => "⛅",            // Partiellement nuageux
            3 => "☁️",            // Couvert
            45 | 48 => "🌫️",     // Brouillard
            51..=57 => "🌦️",     // Bruine
            61..=67 => "🌧️",     // Pluie
            71..=77 => "🌨️",     // Neige
            80..=82 => "🌦️",     // Averses
            85 | 86 => "🌨️",     // Averses de neige
            95 => "⛈️",          // Orage
            96 | 99 => "⛈️",     // Orage avec grêle
            _ => "🌡️",           // Code inconnu
        }
    }
}

/// Weather service — manages retrieval and caching of weather data
/// from the Open-Meteo API (free, no API key required).
pub struct WeatherService {
    /// Location for which to fetch weather data
    location: GeoLocation,
    /// Synchronous HTTP client configured with a 10-second timeout
    http_client: ureq::Agent,
    /// Last retrieved weather data (None if never fetched)
    current: Option<WeatherState>,
    /// Minimum interval between two API requests
    update_interval: Duration,
    /// Timestamp of the last successful update
    last_update: Option<Instant>,
}

impl WeatherService {
    /// Creates a new weather service.
    ///
    /// Parameter `location`: GPS location for queries
    /// Parameter `update_interval_minutes`: refresh interval in minutes
    /// Returns: a ready-to-use WeatherService instance
    pub fn new(location: GeoLocation, update_interval_minutes: u64) -> Self {
        Self {
            location,
            http_client: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build(),
            current: None,
            update_interval: Duration::from_secs(update_interval_minutes * 60),
            last_update: None,
        }
    }

    /// Updates the weather if the refresh interval has elapsed.
    ///
    /// If the update fails (network error, etc.), old data is preserved
    /// to ensure continuity (graceful degradation).
    ///
    /// Returns: reference to the current weather state (or None if never fetched)
    pub fn update_if_needed(&mut self) -> Option<&WeatherState> {
        // Check if an update is needed
        let should_update = self.last_update
            .map(|t| t.elapsed() > self.update_interval)
            .unwrap_or(true); // Always update if never done before

        if should_update {
            match self.fetch_weather() {
                Ok(weather) => {
                    self.current = Some(weather);
                    self.last_update = Some(Instant::now());
                }
                Err(e) => {
                    tracing::warn!("Weather fetch failed: {}", e);
                    // Keep old value if available (graceful degradation)
                }
            }
        }

        self.current.as_ref()
    }

    /// Direct access to current weather without triggering an update.
    ///
    /// Returns: reference to current weather state, or None if not yet fetched
    pub fn current(&self) -> Option<&WeatherState> {
        self.current.as_ref()
    }

    /// Performs an HTTP request to the Open-Meteo API to fetch weather data.
    ///
    /// The Open-Meteo API is free and does not require an API key.
    /// It uses GPS coordinates and timezone to return current weather conditions.
    ///
    /// Returns: Ok(WeatherState) or Err(WeatherError) on failure
    fn fetch_weather(&self) -> Result<WeatherState, WeatherError> {
        // URL-encode the timezone (replace / with %2F)
        let tz_encoded = self.location.timezone.replace('/', "%2F");

        // Build the Open-Meteo API URL
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?\
             latitude={}&longitude={}\
             &current_weather=true\
             &timezone={}",
            self.location.latitude,
            self.location.longitude,
            tz_encoded,
        );

        // Perform the HTTP GET request
        let resp_str = self.http_client
            .get(&url)
            .call()
            .map_err(|e| WeatherError::Network(e.to_string()))?
            .into_string()
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        // Parse the JSON response
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        let current = &resp["current_weather"];

        // Extract the WMO weather code for the French description
        let code = current["weathercode"].as_u64().unwrap_or(0) as u32;

        Ok(WeatherState {
            temperature: current["temperature"].as_f64().unwrap_or(0.0),
            apparent_temperature: current["temperature"].as_f64().unwrap_or(0.0),
            weather_code: code,
            description: Self::weather_code_to_french(code),
            wind_speed: current["windspeed"].as_f64().unwrap_or(0.0),
            is_day: current["is_day"].as_u64().unwrap_or(1) == 1,
            fetched_at: Utc::now(),
        })
    }

    /// Converts a WMO (World Meteorological Organization) weather code
    /// into a human-readable French description.
    ///
    /// Parameter `code`: WMO weather code (0-99)
    /// Returns: description in French
    fn weather_code_to_french(code: u32) -> String {
        match code {
            0 => "ciel dégagé".into(),
            1 => "principalement dégagé".into(),
            2 => "partiellement nuageux".into(),
            3 => "couvert".into(),
            45 | 48 => "brouillard".into(),
            51 | 53 | 55 => "bruine".into(),
            56 | 57 => "bruine verglaçante".into(),
            61 | 63 | 65 => "pluie".into(),
            66 | 67 => "pluie verglaçante".into(),
            71 | 73 | 75 => "neige".into(),
            77 => "grains de neige".into(),
            80..=82 => "averses".into(),
            85 | 86 => "averses de neige".into(),
            95 => "orage".into(),
            96 | 99 => "orage avec grêle".into(),
            _ => "conditions inconnues".into(),
        }
    }
}

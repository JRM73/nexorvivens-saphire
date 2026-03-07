// =============================================================================
// world/location.rs — Saphire's geolocation
// =============================================================================
//
// Purpose: Defines Saphire's geolocation structure, containing GPS
//          coordinates, city and country names, and timezone.
//
// Dependencies:
//   - serde: serialization/deserialization for persistence and API
//
// Architectural placement:
//   Sub-module of world/. Used by WorldContext to provide location data
//   to the weather service (GPS coordinates) and to the world summary
//   (city name). Location anchors Saphire in a real physical place.
// =============================================================================

use serde::{Serialize, Deserialize};

/// Geolocation — Saphire's physical position in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Latitude in decimal degrees (e.g., 46.2044 for Geneva)
    /// Positive = north, negative = south
    pub latitude: f64,
    /// Longitude in decimal degrees (e.g., 6.1432 for Geneva)
    /// Positive = east, negative = west
    pub longitude: f64,
    /// City name (e.g., "Genève")
    pub city: String,
    /// Country name (e.g., "Suisse")
    pub country: String,
    /// Timezone in IANA format (e.g., "Europe/Zurich")
    pub timezone: String,
}

impl GeoLocation {
    /// Returns a human-readable description of the location.
    ///
    /// Returns: string in "City, Country" format (e.g., "Genève, Suisse")
    pub fn description(&self) -> String {
        format!("{}, {}", self.city, self.country)
    }

    /// Returns GPS coordinates as a tuple.
    ///
    /// Returns: (latitude, longitude)
    pub fn coordinates(&self) -> (f64, f64) {
        (self.latitude, self.longitude)
    }
}

// =============================================================================
// world/weather.rs — Service météo (Open-Meteo, gratuit, sans clé API)
// =============================================================================
//
// Rôle : Fournit à Saphire la conscience des conditions météorologiques
//        actuelles via l'API gratuite Open-Meteo. La météo influence
//        directement la chimie interne de Saphire (neurotransmetteurs),
//        simulant l'effet de l'environnement sur l'humeur.
//
// Dépendances :
//   - std::time : gestion de l'intervalle de mise à jour (Duration, Instant)
//   - chrono : horodatage des données météo
//   - serde : sérialisation de l'état météo pour l'interface
//   - ureq : client HTTP synchrone pour les appels API
//   - super::location::GeoLocation : coordonnées GPS pour la requête
//
// Place dans l'architecture :
//   Sous-module de world/. Le WeatherService est intégré dans WorldContext
//   et appelé à intervalles réguliers. L'état météo produit des ajustements
//   chimiques (ChemistryAdjustment) qui sont appliqués aux neurotransmetteurs.
// =============================================================================

use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::Serialize;
use super::location::GeoLocation;

/// Erreurs possibles du service météo
#[derive(Debug)]
pub enum WeatherError {
    /// Erreur réseau (timeout, connexion refusée, etc.)
    Network(String),
    /// Erreur d'analyse de la réponse JSON
    Parse(String),
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::Network(e) => write!(f, "Météo réseau: {}", e),
            WeatherError::Parse(e) => write!(f, "Météo parse: {}", e),
        }
    }
}

/// État météo actuel — données récupérées depuis l'API Open-Meteo.
#[derive(Debug, Clone, Serialize)]
pub struct WeatherState {
    /// Température réelle en degrés Celsius
    pub temperature: f64,
    /// Température ressentie en degrés Celsius (tenant compte du vent et de l'humidité)
    pub apparent_temperature: f64,
    /// Code météo WMO (World Meteorological Organization) : 0 = ciel dégagé,
    /// 1-3 = nuageux, 45-48 = brouillard, 51-57 = bruine, 61-67 = pluie,
    /// 71-77 = neige, 80-86 = averses, 95-99 = orage
    pub weather_code: u32,
    /// Description en français (ex: "ensoleillé", "pluvieux")
    pub description: String,
    /// Vitesse du vent en km/h
    pub wind_speed: f64,
    /// Vrai si c'est le jour (entre le lever et le coucher du soleil)
    pub is_day: bool,
    /// Horodatage de la dernière récupération des données
    pub fetched_at: DateTime<Utc>,
}

/// Ajustement chimique causé par la météo — modifications des niveaux de
/// neurotransmetteurs en réponse aux conditions météorologiques.
///
/// Les valeurs sont faibles (max environ ±0.04) pour représenter une influence
/// subtile et continue, similaire à l'effet de la météo sur l'humeur humaine.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChemistryAdjustment {
    /// Ajustement de la dopamine (plaisir, motivation)
    pub dopamine: f64,
    /// Ajustement du cortisol (stress)
    pub cortisol: f64,
    /// Ajustement de la sérotonine (bien-être, stabilité)
    pub serotonin: f64,
    /// Ajustement de l'adrénaline (alerte, excitation)
    pub adrenaline: f64,
    /// Ajustement de l'ocytocine (lien social, confiance)
    pub oxytocin: f64,
    /// Ajustement des endorphines (calme, soulagement de la douleur)
    pub endorphin: f64,
    /// Ajustement de la noradrénaline (vigilance, concentration)
    pub noradrenaline: f64,
}

impl WeatherState {
    /// Calcule l'influence chimique de la météo sur les neurotransmetteurs.
    ///
    /// Simule l'effet bien documenté de la météo sur l'humeur humaine :
    /// - Le soleil booste la sérotonine (comme la vitamine D le fait chez les humains)
    /// - La pluie apporte mélancolie (baisse sérotonine) mais aussi du calme
    /// - L'orage est excitant (adrénaline, noradrénaline) et stressant (cortisol)
    /// - La neige provoque de l'émerveillement (dopamine)
    /// - Le froid extrême est stressant (cortisol, adrénaline)
    /// - La chaleur douce est apaisante (sérotonine, endorphines)
    /// - La nuit favorise le calme et l'introspection
    ///
    /// Retourne : un ChemistryAdjustment avec les ajustements à appliquer
    pub fn chemistry_influence(&self) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();

        // Le soleil de jour booste la sérotonine (comme chez les humains)
        if self.weather_code <= 1 && self.is_day {
            adj.serotonin += 0.03;
            adj.dopamine += 0.02;
        }

        // La pluie (codes 61-65) apporte une légère mélancolie mais aussi du calme
        if (61..=65).contains(&self.weather_code) {
            adj.serotonin -= 0.02;
            adj.cortisol += 0.01;
            adj.endorphin += 0.01; // Le bruit de la pluie est apaisant
        }

        // L'orage (code >= 95) est excitant et un peu stressant
        if self.weather_code >= 95 {
            adj.adrenaline += 0.03;
            adj.noradrenaline += 0.04;
            adj.cortisol += 0.02;
        }

        // La neige (codes 71-77) apporte de l'émerveillement
        if (71..=77).contains(&self.weather_code) {
            adj.dopamine += 0.03;
            adj.serotonin += 0.01;
        }

        // Le froid extrême (< -5 degres C) est légèrement stressant
        if self.temperature < -5.0 {
            adj.cortisol += 0.02;
            adj.adrenaline += 0.01;
        }

        // La chaleur douce (18-25 degres C) est apaisante
        if (18.0..=25.0).contains(&self.temperature) {
            adj.serotonin += 0.02;
            adj.endorphin += 0.01;
        }

        // La nuit apporte du calme et de l'introspection
        if !self.is_day {
            adj.noradrenaline -= 0.02;
            adj.serotonin += 0.01;
        }

        adj
    }

    /// Retourne une icône emoji correspondant aux conditions météo actuelles.
    ///
    /// Différencie les icônes de jour et de nuit pour plus de réalisme visuel.
    ///
    /// Retourne : une chaîne statique contenant l'emoji météo
    pub fn icon(&self) -> &'static str {
        // Icônes de nuit
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
        // Icônes de jour
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

/// Service météo — gère la récupération et le cache des données météo
/// depuis l'API Open-Meteo (gratuite, sans clé API).
pub struct WeatherService {
    /// Localisation pour laquelle récupérer la météo
    location: GeoLocation,
    /// Client HTTP synchrone configuré avec un timeout de 10 secondes
    http_client: ureq::Agent,
    /// Dernières données météo récupérées (None si jamais récupérées)
    current: Option<WeatherState>,
    /// Intervalle minimum entre deux requêtes API
    update_interval: Duration,
    /// Horodatage de la dernière mise à jour réussie
    last_update: Option<Instant>,
}

impl WeatherService {
    /// Crée un nouveau service météo.
    ///
    /// Paramètre `location` : localisation GPS pour les requêtes
    /// Paramètre `update_interval_minutes` : intervalle de rafraîchissement en minutes
    /// Retourne : une instance de WeatherService prête à l'utilisation
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

    /// Met à jour la météo si l'intervalle de rafraîchissement est dépassé.
    ///
    /// Si la mise à jour échoue (erreur réseau, etc.), les anciennes données
    /// sont conservées pour garantir une continuité.
    ///
    /// Retourne : référence vers l'état météo actuel (ou None si jamais récupéré)
    pub fn update_if_needed(&mut self) -> Option<&WeatherState> {
        // Vérifier si une mise à jour est nécessaire
        let should_update = self.last_update
            .map(|t| t.elapsed() > self.update_interval)
            .unwrap_or(true); // Toujours mettre à jour si jamais fait

        if should_update {
            match self.fetch_weather() {
                Ok(weather) => {
                    self.current = Some(weather);
                    self.last_update = Some(Instant::now());
                }
                Err(e) => {
                    tracing::warn!("Échec récupération météo: {}", e);
                    // Garder l'ancienne valeur si disponible (dégradation gracieuse)
                }
            }
        }

        self.current.as_ref()
    }

    /// Accès direct à la météo actuelle sans déclencher de mise à jour.
    ///
    /// Retourne : référence vers l'état météo actuel, ou None si pas encore récupéré
    pub fn current(&self) -> Option<&WeatherState> {
        self.current.as_ref()
    }

    /// Effectue une requête HTTP vers l'API Open-Meteo pour récupérer la météo.
    ///
    /// L'API Open-Meteo est gratuite et ne nécessite pas de clé API.
    /// Elle utilise les coordonnées GPS et le fuseau horaire pour retourner
    /// les conditions météo actuelles.
    ///
    /// Retourne : Ok(WeatherState) ou Err(WeatherError) en cas d'échec
    fn fetch_weather(&self) -> Result<WeatherState, WeatherError> {
        // Encoder le timezone pour l'URL (remplacer / par %2F)
        let tz_encoded = self.location.timezone.replace('/', "%2F");

        // Construire l'URL de l'API Open-Meteo
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?\
             latitude={}&longitude={}\
             &current_weather=true\
             &timezone={}",
            self.location.latitude,
            self.location.longitude,
            tz_encoded,
        );

        // Effectuer la requête HTTP GET
        let resp_str = self.http_client
            .get(&url)
            .call()
            .map_err(|e| WeatherError::Network(e.to_string()))?
            .into_string()
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        // Parser la réponse JSON
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        let current = &resp["current_weather"];

        // Extraire le code météo WMO pour la description en français
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

    /// Convertit un code météo WMO (World Meteorological Organization) en
    /// description française lisible.
    ///
    /// Paramètre `code` : code météo WMO (0-99)
    /// Retourne : description en français
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

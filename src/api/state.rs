// =============================================================================
// api/state.rs — Etat partage de l'application et messages de controle
//
// Role : Definit AppState (etat partage entre handlers HTTP/WebSocket et la
// boucle principale) et ControlMessage (commandes de pilotage envoyees
// depuis l'interface web vers la boucle de vie).
// =============================================================================

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{mpsc, broadcast, Mutex};

use crate::agent::SaphireAgent;
use crate::agent::lifecycle::{SharedState, UserMessage};
use crate::logging::SaphireLogger;
use crate::logging::db::LogsDb;

/// Messages de controle envoyes depuis l'interface web vers la boucle principale.
/// Chaque variante correspond a une action de configuration ou de pilotage
/// que l'operateur peut declencher via l'IU (Interface Utilisateur) web.
#[derive(Debug)]
pub enum ControlMessage {
    /// Definir la valeur de base d'une molecule (ex: dopamine, cortisol).
    /// `molecule` : nom du neurotransmetteur, `value` : nouvelle valeur de reference.
    SetBaseline { molecule: String, value: f64 },
    /// Modifier le poids d'un module cerebral (reptilien, limbique, neocortex).
    /// `module` : nom du module, `value` : nouveau poids.
    SetModuleWeight { module: String, value: f64 },
    /// Ajuster un seuil de decision (oui/non).
    /// `which` : identifiant du seuil, `value` : nouvelle valeur.
    SetThreshold { which: String, value: f64 },
    /// Modifier un parametre general (temperature, intervalle de pensee, etc.).
    /// `param` : nom du parametre, `value` : nouvelle valeur.
    SetParam { param: String, value: f64 },
    /// Stabilisation d'urgence : remet toute la neurochimie aux valeurs de base.
    EmergencyStabilize,
    /// Suggerer un sujet de reflexion a l'agent.
    /// `topic` : le sujet propose.
    SuggestTopic { topic: String },
    /// Reset aux valeurs d'usine.
    /// `level` : niveau de reset (ChemistryOnly, ParametersOnly, FullReset)
    FactoryReset { level: crate::factory::ResetLevel },
    /// Demander la configuration actuelle. La reponse est envoyee via le canal oneshot.
    /// `response_tx` : canal de reponse unique pour renvoyer le JSON de configuration.
    GetConfig { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
    /// Demander l'etat neurochimique actuel. La reponse est envoyee via le canal oneshot.
    /// `response_tx` : canal de reponse unique pour renvoyer le JSON de la chimie.
    GetChemistry { response_tx: tokio::sync::oneshot::Sender<serde_json::Value> },
}

/// Etat partage de l'application, accessible par le serveur web et la boucle principale.
/// Ce struct est clone et partage entre les handlers HTTP/WebSocket et le coeur de l'agent.
#[derive(Clone)]
pub struct AppState {
    /// Emetteur broadcast pour diffuser des messages JSON aux clients WebSocket.
    pub ws_tx: Arc<broadcast::Sender<String>>,
    /// Canal d'envoi des messages utilisateur (chat) vers la boucle principale.
    pub user_tx: mpsc::Sender<UserMessage>,
    /// Canal d'envoi des messages de controle (configuration) vers la boucle principale.
    pub ctrl_tx: mpsc::Sender<ControlMessage>,
    /// Drapeau atomique d'arret : quand il passe a true, la boucle s'arrete.
    pub shutdown: Arc<AtomicBool>,
    /// Reference partagee vers l'agent (protegee par un Mutex asynchrone).
    pub agent: Arc<Mutex<SaphireAgent>>,
    /// Broadcast channel dedie au dashboard (separe du WS principal).
    pub dashboard_tx: Arc<broadcast::Sender<String>>,
    /// Logger centralise partage.
    pub logger: Option<Arc<Mutex<SaphireLogger>>>,
    /// Base de donnees de logs (optionnelle).
    pub logs_db: Option<Arc<LogsDb>>,
    /// Cle API pour l'authentification (None = pas d'auth)
    pub api_key: Option<String>,
    /// Origines autorisees pour CORS et WebSocket
    pub allowed_origins: Vec<String>,
    /// Rate limiter par IP
    pub rate_limiter: Arc<crate::api::middleware::RateLimiter>,
}

// Conversion de AppState vers SharedState pour compatibilite avec le module lifecycle.
// SharedState est une version reduite qui ne contient que les canaux essentiels.
impl From<AppState> for SharedState {
    fn from(s: AppState) -> Self {
        SharedState {
            ws_tx: s.ws_tx,
            user_tx: s.user_tx,
            shutdown: s.shutdown,
        }
    }
}

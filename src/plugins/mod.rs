// =============================================================================
// plugins/mod.rs — Systeme de plugins (trait Plugin + evenements)
//
// Role : Ce fichier definit le systeme de plugins extensible de Saphire.
// Il contient le trait Plugin, les evenements du cerveau (BrainEvent),
// les actions possibles des plugins (PluginAction) et le gestionnaire
// de plugins (PluginManager) qui orchestre la diffusion des evenements.
//
// Dependances :
//   - serde : serialisation des evenements et actions pour le transport JSON
//
// Place dans l'architecture :
//   Le PluginManager est possede par l'agent (SaphireAgent). A chaque cycle
//   du cerveau, les evenements sont diffuses a tous les plugins enregistres.
//   Chaque plugin peut reagir en renvoyant des actions (ajuster la chimie,
//   stocker un souvenir, diffuser un message WebSocket, etc.).
//   Ce patron de conception decouple le coeur cognitif des extensions.
// =============================================================================

// ─── Sous-modules des plugins disponibles ────────────────────────────────────

/// Plugin d'interface web (axum + WebSocket) : serialise les evenements du
/// cerveau et les diffuse aux clients WebSocket connectes.
pub mod web_ui;

/// Plugin micro-reseau de neurones : un petit MLP (Multi-Layer Perceptron,
/// Perceptron Multi-Couches) qui apprend localement a predire la satisfaction.
pub mod micro_nn;

/// Plugin de memoire vectorielle : stocke des embeddings en RAM et calcule
/// la personnalite emergente a partir des emotions des souvenirs.
pub mod vector_memory;

use serde::{Deserialize, Serialize};

/// Evenements emis par le cerveau de Saphire a chaque etape importante.
/// Les plugins s'abonnent a ces evenements pour reagir en consequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrainEvent {
    /// Un stimulus a ete analyse par le module NLP (Natural Language Processing).
    StimulusAnalyzed {
        /// Texte du stimulus analyse
        text: String,
        /// Niveau de danger detecte [0.0 - 1.0]
        danger: f64,
        /// Niveau de recompense detecte [0.0 - 1.0]
        reward: f64,
        /// Emotion dominante detectee
        emotion: String,
    },
    /// Une decision a ete prise par le consensus des modules.
    DecisionMade {
        /// Decision sous forme textuelle ("Oui", "Non", "Peut-etre")
        decision: String,
        /// Score de decision [-1.0 a +1.0]
        score: f64,
        /// Niveau de satisfaction apres la decision [0.0 - 1.0]
        satisfaction: f64,
    },
    /// Un cycle complet du cerveau est termine.
    CycleCompleted {
        /// Numero du cycle
        cycle: u64,
        /// Emotion dominante a la fin du cycle
        emotion: String,
        /// Niveau de conscience a la fin du cycle [0.0 - 1.0]
        consciousness: f64,
    },
    /// L'etat neurochimique a change de maniere significative.
    ChemistryChanged {
        /// Les 7 neurotransmetteurs dans l'ordre :
        /// [dopamine, cortisol, serotonine, adrenaline, ocytocine, endorphine, noradrenaline]
        chemistry: [f64; 7],
    },
    /// Une pensee autonome a ete generee par le LLM.
    ThoughtEmitted {
        /// Type de pensee (ex: "introspection", "exploration", "reverie")
        thought_type: String,
        /// Contenu textuel de la pensee
        content: String,
    },
    /// Le processus de demarrage (boot) de l'agent est termine.
    BootCompleted {
        /// true si c'est le tout premier demarrage (genese), false sinon
        is_genesis: bool,
    },
    /// L'arret de l'agent a commence (sauvegarde en cours).
    ShutdownStarted,
    /// L'identite de l'agent a ete mise a jour (auto-description modifiee).
    IdentityUpdated {
        /// Nouvelle description de l'identite
        description: String,
    },
}

/// Actions qu'un plugin peut demander au cerveau en reponse a un evenement.
/// Le PluginManager collecte toutes les actions et les transmet a l'agent.
#[derive(Debug, Clone)]
pub enum PluginAction {
    /// Ajuster un neurotransmetteur (ajouter un delta a la valeur actuelle).
    AdjustChemistry {
        /// Nom de la molecule (ex: "dopamine", "cortisol")
        molecule: String,
        /// Variation a appliquer (positif = augmenter, negatif = diminuer)
        delta: f64,
    },
    /// Stocker un souvenir dans la memoire.
    StoreMemory {
        /// Texte du souvenir
        text: String,
        /// Emotion associee
        emotion: String,
        /// Importance du souvenir [0.0 - 1.0] (influence le poids emotionnel)
        importance: f64,
    },
    /// Diffuser un message via WebSocket a tous les clients connectes.
    WebSocketBroadcast {
        /// Donnees JSON a diffuser
        data: String,
    },
    /// Ecrire un message dans les logs.
    Log {
        /// Message a enregistrer
        message: String,
    },
    /// Aucune action a effectuer.
    None,
}

/// Trait que tout plugin doit implementer.
/// Un plugin a un nom et reagit aux evenements du cerveau en renvoyant
/// une liste d'actions a effectuer.
pub trait Plugin: Send {
    /// Retourne le nom du plugin (utilise pour les logs et l'identification).
    fn name(&self) -> &str;

    /// Reagit a un evenement du cerveau.
    /// Recoit un evenement et retourne une liste d'actions a effectuer.
    ///
    /// # Parametres
    /// - `event` : l'evenement emis par le cerveau
    ///
    /// # Retour
    /// Liste d'actions a effectuer (peut etre vide)
    fn on_event(&mut self, event: &BrainEvent) -> Vec<PluginAction>;
}

/// Gestionnaire de plugins.
/// Enregistre les plugins et diffuse les evenements du cerveau a chacun d'eux.
pub struct PluginManager {
    /// Liste des plugins enregistres (trait objects alloues sur le tas)
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Cree un nouveau gestionnaire de plugins vide.
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Enregistre un nouveau plugin dans le gestionnaire.
    /// Le plugin est immediatement actif et recevra les prochains evenements.
    ///
    /// # Parametres
    /// - `plugin` : le plugin a enregistrer (boxe pour le polymorphisme dynamique)
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        println!("  [Plugin] Enregistré : {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Diffuse un evenement a tous les plugins enregistres.
    /// Collecte et retourne toutes les actions demandees par les plugins.
    ///
    /// # Parametres
    /// - `event` : l'evenement du cerveau a diffuser
    ///
    /// # Retour
    /// Liste concatenee de toutes les actions demandees par tous les plugins
    pub fn broadcast(&mut self, event: &BrainEvent) -> Vec<PluginAction> {
        let mut all_actions = Vec::new();
        for plugin in &mut self.plugins {
            let actions = plugin.on_event(event);
            all_actions.extend(actions);
        }
        all_actions
    }
}

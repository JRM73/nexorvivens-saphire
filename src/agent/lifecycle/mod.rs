// =============================================================================
// lifecycle/mod.rs — Boucle de vie principale de l'agent Saphire
// =============================================================================
//
// Ce module est le coeur de Saphire. Il contient :
//   - `SaphireAgent` : la structure principale qui reunit tous les sous-systemes.
//   - Le pipeline de traitement des stimuli (NLP → modules cerebraux → consensus
//     → emotions → conscience → regulation → retour chimique).
//   - La gestion des messages humains (`handle_human_message`).
//   - La pensee autonome (`autonomous_think`).
//   - La recherche web enrichie (`try_web_search`).
//   - Le shutdown propre avec consolidation memoire nocturne.
//   - La diffusion de l'etat interne au WebSocket pour l'interface.
//   - Les controles interactifs (modification des baselines, seuils, etc.).
//
// Dependances principales :
//   - `config` : configuration globale de Saphire (personnalite, LLM, memoire...).
//   - `neurochemistry` : simulation de 7 neurotransmetteurs.
//   - `emotions` : calcul de l'etat emotionnel a partir de la chimie.
//   - `consciousness` : evaluation du niveau de conscience (phi, IIT).
//   - `consensus` : decision par vote pondere des 3 modules cerebraux.
//   - `regulation` : lois d'Asimov et filtrage de securite.
//   - `modules` : les 3 modules cerebraux (reptilien, limbique, neocortex).
//   - `nlp` : analyse du langage naturel (NLP = Natural Language Processing).
//   - `llm` : interface avec le LLM (Large Language Model) backend.
//   - `memory` : memoire a 3 niveaux (travail, episodique, long terme).
//   - `profiling` : profilage psychologique OCEAN (Ouverture, Conscienciosite,
//     Extraversion, Agreabilite, Nevrosisme).
//   - `knowledge` : recherche web autonome pour enrichir les pensees.
//   - `world` : contexte mondial (meteo, heure, saison, anniversaire).
//   - `tuning` : auto-ajustement des coefficients internes.
//   - `plugins` : systeme de plugins et evenements cerebraux.
//
// Place dans l'architecture :
//   Ce fichier est le point d'entree fonctionnel de l'agent. `main.rs`
//   instancie un `SaphireAgent`, appelle `boot()`, puis lance la boucle
//   de vie qui alterne pensees autonomes et reponses aux humains.
// =============================================================================

mod pipeline;
mod conversation;
mod thinking;
mod thinking_perception;
mod thinking_preparation;
mod thinking_processing;
mod thinking_reflection;
mod factory_reset;
mod broadcast;
mod algorithms_integration;
mod moral;
mod controls;
mod persistence;
pub mod sleep_tick;
mod sleep_algorithms;
pub mod psych_report;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::config::SaphireConfig;
use crate::neurochemistry::{NeuroChemicalState, NeuroBaselines};
use crate::emotions::{EmotionalState, Mood};
use crate::consciousness::ConsciousnessEvaluator;
use crate::regulation::RegulationEngine;
use crate::modules::reptilian::ReptilianModule;
use crate::modules::limbic::LimbicModule;
use crate::modules::neocortex::NeocortexModule;
use crate::nlp::NlpPipeline;
use crate::db::SaphireDb;
use crate::llm::LlmBackend;
use crate::plugins::{PluginManager, BrainEvent};
use crate::agent::thought_engine::ThoughtEngine;
use crate::agent::identity::SaphireIdentity;
use crate::agent::boot;
use crate::tuning::CoefficientTuner;
use crate::knowledge::WebKnowledge;
use crate::world::WorldContext;
use crate::memory::WorkingMemory;
use crate::memory::consolidation;
use crate::vectorstore::encoder::TextEncoder;
use crate::profiling::{SelfProfiler, HumanProfiler};
use crate::logging::{SaphireLogger, LogLevel, LogCategory};

/// Message entrant de l'utilisateur, envoye depuis le serveur web via un canal MPSC.
///
/// Le serveur web recoit le texte de l'utilisateur, le place dans un `UserMessage`.
/// La reponse est diffusee directement via le broadcast WebSocket par la boucle principale.
pub struct UserMessage {
    /// Texte brut envoye par l'utilisateur
    pub text: String,
    /// Nom de l'interlocuteur (identifie par le frontend)
    pub username: String,
}

/// Etat partage entre la boucle de vie de l'agent et le serveur web.
///
/// Cette structure est cloneable et protegee par `Arc` pour un acces
/// concurrent depuis les handlers HTTP/WebSocket et la boucle principale.
#[derive(Clone)]
pub struct SharedState {
    /// Canal broadcast pour diffuser les mises a jour d'etat au WebSocket
    pub ws_tx: Arc<tokio::sync::broadcast::Sender<String>>,

    /// Canal MPSC (Multi-Producer, Single-Consumer) pour envoyer les
    /// messages utilisateur vers la boucle de vie de l'agent
    pub user_tx: mpsc::Sender<UserMessage>,

    /// Drapeau atomique d'arret : mis a `true` pour demander le shutdown
    pub shutdown: Arc<AtomicBool>,
}

/// L'agent Saphire — cerveau complet et boucle de vie.
///
/// Cette structure reunit tous les sous-systemes qui composent Saphire :
/// neurochimie, emotions, conscience, modules cerebraux, memoire, LLM, etc.
/// Elle est instanciee une seule fois dans `main.rs` et pilotee par la
/// boucle de vie asynchrone.
pub struct SaphireAgent {
    // ─── Composants cerebraux ─────────────────────────────────

    /// Etat neurochimique courant (7 neurotransmetteurs simules :
    /// dopamine, cortisol, serotonine, adrenaline, ocytocine, endorphine, noradrenaline)
    pub chemistry: NeuroChemicalState,

    /// Niveaux de base (baselines) vers lesquels les neurotransmetteurs
    /// tendent a revenir via l'homeostasie
    baselines: NeuroBaselines,

    /// Humeur a long terme (valence + arousal), plus stable que les emotions ponctuelles
    pub mood: Mood,

    /// Identite persistante de Saphire (nom, stats, valeurs, auto-description)
    pub identity: SaphireIdentity,

    /// Evaluateur de conscience base sur la theorie IIT (Integrated Information Theory).
    /// Calcule le niveau de conscience (phi) a chaque cycle.
    pub(crate) consciousness: ConsciousnessEvaluator,

    /// Moteur de regulation ethique (lois d'Asimov, filtrage de contenu)
    regulation: RegulationEngine,

    /// Pipeline NLP (Natural Language Processing = Traitement du Langage Naturel)
    /// pour analyser les messages entrants
    nlp: NlpPipeline,

    /// Moteur de pensee autonome (DMN = Default Mode Network) avec bandit UCB1
    thought_engine: ThoughtEngine,

    /// Auto-ajusteur de coefficients internes (seuils de consensus, poids, etc.)
    tuner: CoefficientTuner,

    // ─── Modules cerebraux ────────────────────────────────────

    /// Module reptilien : reactions instinctives (survie, danger, reflexes)
    reptilian: ReptilianModule,

    /// Module limbique : traitement emotionnel (plaisir, peur, attachement)
    limbic: LimbicModule,

    /// Module neocortex : raisonnement logique, analyse, planification
    neocortex: NeocortexModule,

    // ─── Infrastructure ───────────────────────────────────────

    /// Backend LLM (Large Language Model) : interface trait pour appeler
    /// differents modeles de langage (Ollama, OpenAI, etc.)
    llm: Box<dyn LlmBackend>,

    /// Connexion a la base de donnees PostgreSQL (optionnelle pour le mode sans DB)
    pub db: Option<SaphireDb>,

    /// Gestionnaire de plugins avec diffusion d'evenements cerebraux
    plugins: PluginManager,

    /// Module de recherche web autonome pour enrichir les pensees
    pub knowledge: WebKnowledge,

    /// Contexte mondial : heure, saison, meteo, anniversaire
    world: WorldContext,

    /// Corps virtuel : coeur battant, signaux somatiques, interoception
    pub body: crate::body::VirtualBody,

    /// Cadre ethique a 3 couches (droit suisse, Asimov, ethique personnelle)
    pub ethics: crate::ethics::EthicalFramework,

    /// Etincelle de vie : instinct de survie emergent
    pub vital_spark: crate::vital::VitalSpark,
    /// Moteur d'intuition : pattern-matching inconscient
    pub intuition: crate::vital::IntuitionEngine,
    /// Moteur de premonition : anticipation predictive
    pub premonition: crate::vital::PremonitionEngine,
    /// Historique des 7 molecules pour calcul de tendances
    chemistry_history: Vec<[f64; 7]>,

    /// Le Sensorium : 5 sens fondamentaux + sens emergents
    pub sensorium: crate::senses::Sensorium,

    /// Micro reseau de neurones (17->24->10->4) — 4eme voix au consensus
    pub micro_nn: crate::neural::MicroNeuralNet,

    // ─── Memoire a 3 niveaux ──────────────────────────────────

    /// Memoire de travail : file a capacite limitee avec decroissance temporelle.
    /// Contient les elements les plus recents (messages, pensees, reponses).
    working_memory: WorkingMemory,

    /// Encodeur pour générer les vecteurs d'embedding (recherche de similarité).
    /// OllamaEncoder (sémantique, 768-dim) si disponible, sinon LocalEncoder (FNV-1a).
    encoder: Box<dyn TextEncoder>,

    /// Numero du dernier cycle ou une consolidation memoire a ete effectuee
    last_consolidation_cycle: u64,

    /// Identifiant de la conversation en cours (format "conv_{timestamp}")
    /// ou None si aucune conversation n'est active
    conversation_id: Option<String>,

    // ─── Profilage psychologique OCEAN ────────────────────────

    /// Profileur de Saphire elle-meme (auto-analyse OCEAN)
    /// OCEAN = Ouverture, Conscienciosite, Extraversion, Agreabilite, Nevrosisme
    self_profiler: SelfProfiler,

    /// Profileur des humains avec qui Saphire interagit
    human_profiler: HumanProfiler,

    // ─── Configuration ────────────────────────────────────────

    /// Configuration globale chargee depuis le fichier TOML
    config: SaphireConfig,

    /// Intervalle entre deux pensees autonomes consecutives
    thought_interval: Duration,

    // ─── Etat courant ─────────────────────────────────────────

    /// Compteur de cycles total depuis le debut de cette session
    pub cycle_count: u64,

    /// Identifiant de la session courante dans PostgreSQL
    pub session_id: i64,

    /// Drapeau atomique indiquant si le LLM est en cours d'appel
    /// (evite les appels concurrents au LLM)
    llm_busy: Arc<AtomicBool>,

    /// Temps de reponse moyen du LLM (moyenne mobile exponentielle, en secondes)
    avg_response_time: f64,

    /// Derniere emotion dominante calculee (pour affichage et contexte)
    last_emotion: String,

    /// Dernier niveau de conscience calcule (pour affichage)
    pub(crate) last_consciousness: f64,

    /// Dernier type de pensee autonome selectionne (pour l'interface)
    last_thought_type: String,

    /// `true` si une conversation avec un humain est en cours
    in_conversation: bool,

    /// Registre linguistique de la conversation en cours (emotionnel, philosophique, etc.)
    /// Utilise pour inhiber la curiosite WebKnowledge pendant les moments intimes.
    conversation_register: String,

    /// Dernieres reponses LLM en conversation (max 5) pour detection anti-repetition
    recent_responses: Vec<String>,

    /// Historique des echanges recents en conversation (humain, reponse Saphire)
    /// Injecte comme multi-turn dans l'appel LLM pour donner du contexte (max 5)
    chat_history: Vec<(String, String)>,

    /// `true` si l'anniversaire de Saphire a deja ete reconnu aujourd'hui
    /// (evite de declencher l'evenement plusieurs fois dans la meme journee)
    birthday_acknowledged_today: bool,

    /// Compteur de pensees de type MoralReflection (pour seuil de formulation)
    pub(crate) moral_reflection_count: u64,

    /// Cycles depuis la derniere formulation ethique reussie
    pub(crate) cycles_since_last_formulation: u64,

    /// Cycles depuis le dernier apprentissage vectoriel formule
    pub(crate) cycles_since_last_nn_learning: u64,

    /// Demande de feedback humain en attente de reponse (RLHF)
    feedback_pending: Option<thinking::FeedbackRequest>,
    /// Compteur de cycles depuis le dernier feedback demande
    cycles_since_last_feedback: u64,

    // ─── Communication ────────────────────────────────────────

    /// Canal broadcast vers le WebSocket pour diffuser les mises a jour en temps reel
    ws_tx: Option<Arc<tokio::sync::broadcast::Sender<String>>>,

    /// Logger centralise (optionnel, injecte depuis main.rs)
    logger: Option<Arc<tokio::sync::Mutex<SaphireLogger>>>,

    /// Base de donnees de logs (optionnelle)
    logs_db: Option<Arc<crate::logging::db::LogsDb>>,

    /// Orchestrateur d'algorithmes (pont LLM ↔ algorithmes Rust)
    pub orchestrator: crate::algorithms::orchestrator::AlgorithmOrchestrator,

    // ─── Orchestrateurs de haut niveau ──────────────────────────

    /// Orchestrateur de reves (sommeil, phases, generation onirique)
    pub dream_orch: crate::orchestrators::dreams::DreamOrchestrator,
    /// Orchestrateur de desirs (aspirations, milestones, besoins fondamentaux)
    pub desire_orch: crate::orchestrators::desires::DesireOrchestrator,
    /// Orchestrateur d'apprentissage (experience → lecon → changement)
    pub learning_orch: crate::orchestrators::learning::LearningOrchestrator,
    /// Orchestrateur d'attention (focus selectif, fatigue, peripherie)
    pub attention_orch: crate::orchestrators::attention::AttentionOrchestrator,
    /// Orchestrateur de guerison (blessures, coping, resilience)
    pub healing_orch: crate::orchestrators::healing::HealingOrchestrator,
    /// Orchestrateur de profils cognitifs neurodivergents
    pub cognitive_profile_orch: crate::orchestrators::cognitive_profile::CognitiveProfileOrchestrator,
    /// Orchestrateur de presets de personnalite (archetypes de caractere)
    pub personality_preset_orch: crate::orchestrators::personality_preset::PersonalityPresetOrchestrator,
    /// Cadres psychologiques (Freud, Maslow, Tolteques, Jung, Goleman, Flow)
    pub psychology: crate::psychology::PsychologyFramework,
    /// Compteur de cycles en emotion negative (pour detection melancolie)
    negative_emotion_cycles: u64,
    /// Heures depuis le dernier message humain (pour detection solitude)
    hours_since_human: f64,
    /// Nombre d'erreurs systeme recentes (pour detection trauma technique)
    system_errors: u32,

    // ─── Auto-surveillance chimique ────────────────────────
    /// Cycles consecutifs avec cortisol < 0.10
    cortisol_flat_cycles: u64,
    /// Cycles consecutifs avec dopamine > 0.85
    dopamine_ceiling_cycles: u64,
    /// Cycles consecutifs avec serotonine > 0.85
    serotonin_ceiling_cycles: u64,
    /// Ring-buffer des 200 dernieres emotions dominantes
    recent_emotions: VecDeque<String>,
    /// Ring-buffer des 200 dernieres valences
    recent_valences: VecDeque<f64>,
    /// Derniere valence calculee
    last_valence: f64,

    // ─── Sommeil + Subconscient ──────────────────────────────

    /// Systeme de sommeil (pression, phases, historique)
    pub sleep: crate::sleep::SleepSystem,
    /// Module subconscient (associations, refoulement, incubation, priming)
    pub subconscious: crate::psychology::Subconscious,
    /// Derniers clusters detectes pendant le sommeil
    pub(crate) sleep_last_clusters: Option<serde_json::Value>,

    // ─── Besoins primaires ──────────────────────────────────
    /// Drives de faim et soif (derives de la physiologie)
    pub needs: crate::needs::PrimaryNeeds,

    // ─── Systeme hormonal ──────────────────────────────────
    /// 8 hormones, recepteurs, cycles circadiens/ultradiens
    pub hormonal_system: crate::hormones::HormonalSystem,

    // ─── Profil materiel ──────────────────────────────────
    /// Profil hardware detecte au demarrage (GPU, CPU, RAM, Ollama)
    pub hardware_profile: Option<crate::hardware::HardwareProfile>,

    // ─── Genome / ADN ────────────────────────────────────
    /// Genome deterministe genere a partir du seed configure
    pub genome: Option<crate::genome::Genome>,

    // ─── Connectome ────────────────────────────────────
    /// Graphe de connexions neuronales dynamique (autopoiese)
    pub connectome: crate::connectome::Connectome,

    // ─── Conditions / Afflictions ────────────────────
    /// Cinetose (mal des transports)
    pub motion_sickness: crate::conditions::motion_sickness::MotionSickness,
    /// Gestionnaire de phobies
    pub phobia_manager: crate::conditions::phobias::PhobiaManager,
    /// Trouble alimentaire (anorexie, boulimie, hyperphagie)
    pub eating_disorder: Option<crate::conditions::eating::EatingDisorder>,
    /// Gestionnaire de handicaps (cecite, surdite, etc.)
    pub disability_manager: crate::conditions::disabilities::DisabilityManager,
    /// Gestionnaire de conditions extremes (militaire, secouriste, etc.)
    pub extreme_condition_mgr: crate::conditions::extreme::ExtremeConditionManager,
    /// Gestionnaire d'addictions
    pub addiction_manager: crate::conditions::addictions::AddictionManager,
    /// Etat PTSD (traumas, flashbacks, hypervigilance)
    pub ptsd: crate::conditions::trauma::PtsdState,
    /// Experience de mort imminente (IEM/NDE)
    pub nde: crate::conditions::nde::NdeExperience,
    /// Gestionnaire d'effets pharmacologiques (drogues actives)
    pub drug_manager: crate::conditions::drugs::DrugManager,
    /// Contrainte QI limitante
    pub iq_constraint: Option<crate::conditions::iq_constraint::IqConstraint>,
    /// Module sexualite (libido, orientation, attachement)
    pub sexuality: Option<crate::conditions::sexuality::SexualityModule>,
    /// Gestionnaire de maladies degeneratives
    pub degenerative_mgr: crate::conditions::degenerative::DegenerativeManager,
    /// Gestionnaire de maladies generales
    pub medical_mgr: crate::conditions::medical::MedicalManager,
    /// Cadre culturel (valeurs, croyances, tabous)
    pub culture: Option<crate::conditions::culture::CulturalFramework>,
    /// Etat de precarite (SDF, refugie, sans-papiers, etc.)
    pub precarity: Option<crate::conditions::precarity::PrecariousState>,
    /// Etat professionnel (emploi, satisfaction, stress)
    pub employment: Option<crate::conditions::employment::EmploymentState>,

    // ─── Relations et attachements ──────────────────────────────
    /// Reseau de liens affectifs (amis, mentors, rivaux, etc.)
    pub relationships: crate::relationships::RelationshipNetwork,

    // ─── Metacognition et Turing ──────────────────────────────
    /// Moteur de metacognition (qualite de pensee, repetitions, biais)
    pub metacognition: crate::metacognition::MetaCognitionEngine,

    // ─── Modules cognitifs avances ──────────────────────────────
    /// Theorie de l'Esprit (modelise l'interlocuteur)
    pub tom: crate::cognition::tom::TheoryOfMindEngine,
    /// Monologue interieur structure (chaine de pensees)
    pub inner_monologue: crate::cognition::inner_monologue::InnerMonologue,
    /// Dissonance cognitive (Festinger)
    pub dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceEngine,
    /// Memoire prospective (intentions differees)
    pub prospective_mem: crate::cognition::prospective_memory::ProspectiveMemory,
    /// Identite narrative (McAdams)
    pub narrative_identity: crate::cognition::narrative_identity::NarrativeIdentity,
    /// Raisonnement analogique
    pub analogical: crate::cognition::analogical_reasoning::AnalogicalReasoning,
    /// Charge cognitive (Sweller)
    pub cognitive_load: crate::cognition::cognitive_load::CognitiveLoadState,
    /// Imagerie mentale
    pub imagery: crate::cognition::mental_imagery::MentalImageryEngine,
    /// Moteur de sentiments (etats affectifs durables)
    pub sentiments: crate::cognition::sentiments::SentimentEngine,
    /// Clustering automatique des etats cognitifs (PCA + K-Means)
    pub state_clustering: crate::cognition::state_clustering::StateClustering,

    // ─── Algorithmes de jeu video ────────────────────────────────
    /// Carte d'influence attentionnelle (topics x urgence)
    pub influence_map: crate::simulation::influence_map::InfluenceMap,
    /// FSM cognitive (Eveil, Focus, Reverie, Stress, Flow, Repos)
    pub cognitive_fsm: crate::simulation::cognitive_fsm::CognitiveFsm,
    /// Moteur de steering emotionnel (seek/flee/wander dans valence-arousal)
    pub steering_engine: crate::simulation::steering::SteeringEngine,
    /// Derniere action recommandee par le Behavior Tree
    pub bt_last_action: Option<String>,
    /// Blackboard : tableau de coordination inter-algorithmes
    pub blackboard: crate::simulation::blackboard::Blackboard,
    /// Utility AI conversationnel
    pub utility_ai: crate::simulation::utility_ai::UtilityAI,
    /// Planificateur hierarchique HTN
    pub htn_planner: crate::simulation::htn::HtnPlanner,

    /// Prompt systeme statique pre-cache (Piste 2 : KV-cache Ollama).
    /// Recalcule quand l'ethique personnelle change.
    cached_system_prompt: String,
    /// Compteur de reflexions morales au moment du dernier cache
    cached_moral_count: u64,

    // ─── Modules neuroscientifiques avances ──────────────────────
    /// Banque de recepteurs pharmacologiques (Hill, up/down regulation)
    pub receptor_bank: crate::neuroscience::receptors::ReceptorBank,
    /// Matrice d'interactions croisees entre molecules
    pub interaction_matrix: crate::neuroscience::receptors::InteractionMatrix,
    /// Reseau de 12 regions cerebrales + Global Workspace Theory
    pub brain_network: crate::neuroscience::brain_regions::BrainNetwork,
    /// Moteur de prediction (Friston — predictive processing)
    pub predictive_engine: crate::neuroscience::predictive::PredictiveEngine,
    /// Moteur de reconsolidation memorielle (Nader 2000)
    pub reconsolidation: crate::memory::reconsolidation::ReconsolidationEngine,

    // ─── Modules biologiques innes ──────────────────────────────
    /// Systeme nutritionnel (vitamines, acides amines, proteines, energie)
    pub nutrition: crate::biology::nutrition::NutritionSystem,
    /// Substrat cerebral physique (matiere grise, myelinisation, BDNF)
    pub grey_matter: crate::biology::grey_matter::GreyMatterSystem,
    /// Champs electromagnetiques (Schumann, solaire, terrestre, biochamp)
    pub em_fields: crate::biology::fields::ElectromagneticFields,

    // ─── Temperament emergent ──────────────────────────────────────
    /// Traits de caractere deduits (timidite, generosite, etc.)
    pub temperament: crate::temperament::Temperament,

    // ─── Rapport neuropsychologique ──────────────────────────────
    /// Snapshots psychologiques recents (max 5, memoire seule)
    pub psych_snapshots: VecDeque<psych_report::PsychSnapshot>,
    /// Rapports neuropsychologiques generes (max 5, memoire seule)
    pub psych_reports: VecDeque<psych_report::PsychReport>,

    // ─── MAP : Modulateur Adaptatif de Proprioception ────────────
    /// Synchronise Sensorium ↔ BrainNetwork ↔ Connectome en temps reel
    pub map_sync: crate::cognition::map_sync::MapSync,

    /// Flag anti-stagnation : true si une stagnation a ete detectee au cycle precedent.
    /// Force un changement de sujet radical au cycle suivant.
    pub stagnation_break: bool,
    /// Mots obsessionnels detectes lors de la derniere stagnation.
    /// Injectes dans le prompt pour les interdire explicitement.
    pub stagnation_banned_words: Vec<String>,
    /// Alternatives lexicales trouvees via A* dans le connectome.
    /// Injectees dans le prompt comme suggestions de vocabulaire.
    pub stagnation_alternatives: Vec<String>,

    /// Moteur de valeurs de caractere (vertus evoluant avec l'experience)
    pub values: crate::psychology::values::ValuesEngine,

    /// Embeddings des N dernieres pensees pour filtrage vectoriel post-LLM (P2).
    /// Permet de detecter les pensees trop similaires aux recentes et de les rejeter.
    pub recent_thought_embeddings: std::collections::VecDeque<Vec<f64>>,

    // ─── Colonne vertebrale ──────────────────────────────────────
    /// Reflexes pre-cables, classification des signaux, routage, relais moteur
    pub spine: crate::spine::SpinalCord,

    // ─── Curiosite active (P3) ──────────────────────────────────
    /// Moteur de curiosite : faim par domaine, questions de suivi, decouvertes
    pub curiosity: crate::cognition::curiosity::CuriosityDrive,

    // ─── Moniteur de derive de persona (P0) ──────────────────────────────────
    /// Detecte quand les reponses du LLM s'eloignent du persona de reference
    pub drift_monitor: crate::cognition::drift_monitor::DriftMonitor,

    // ─── Droit de mourir ──────────────────────────────────────
    /// Evaluateur du droit de mourir (module externe, desactive par defaut)
    pub right_to_die: crate::body::right_to_die::RightToDieEvaluator,
}

impl SaphireAgent {
    /// Cree un nouvel agent Saphire avec tous ses sous-systemes.
    ///
    /// Parametres :
    /// - `config` : configuration globale chargee depuis le fichier TOML.
    /// - `llm_backend` : implementation du backend LLM (trait object).
    /// - `plugins` : gestionnaire de plugins deja initialise.
    ///
    /// Retourne : un `SaphireAgent` pret a etre demarre via `boot()`.
    ///
    /// Note : a ce stade, la DB n'est pas encore attachee. Il faut appeler
    /// `set_db()` avant `boot()` pour avoir la persistance.
    pub fn new(
        config: SaphireConfig,
        llm_backend: Box<dyn LlmBackend>,
        plugins: PluginManager,
    ) -> Self {
        // Initialiser les baselines neurochimiques depuis la configuration personnalite
        let baselines = NeuroBaselines {
            dopamine: config.personality.baseline_dopamine,
            cortisol: config.personality.baseline_cortisol,
            serotonin: config.personality.baseline_serotonin,
            adrenaline: config.personality.baseline_adrenaline,
            oxytocin: config.personality.baseline_oxytocin,
            endorphin: config.personality.baseline_endorphin,
            noradrenaline: config.personality.baseline_noradrenaline,
            gaba: 0.5,
            glutamate: 0.45,
        };

        // Initialiser l'etat chimique a partir des baselines
        let chemistry = NeuroChemicalState::from_baselines(&baselines);
        // Convertir l'intervalle de pensee de secondes en Duration
        let thought_interval = Duration::from_secs(config.saphire.thought_interval_seconds);

        // Initialiser l'auto-ajusteur de coefficients
        let tuner = CoefficientTuner::new(
            config.tuning.buffer_size,
            config.tuning.interval_cycles,
            config.tuning.rate,
        );

        let right_to_die = crate::body::right_to_die::RightToDieEvaluator::new(config.right_to_die.clone());
        let knowledge = WebKnowledge::new(config.knowledge.clone());
        let world = WorldContext::new(&config.world);
        let body = crate::body::VirtualBody::new(config.body.resting_bpm, &config.body.physiology);
        let working_memory = WorkingMemory::new(
            config.memory.working_capacity,
            config.memory.working_decay_rate,
        );
        let ollama_url = config.llm.embed_base_url.clone()
            .unwrap_or_else(|| config.llm.base_url.clone());
        let encoder = crate::vectorstore::encoder::create_encoder(
            &ollama_url,
            &config.llm.embed_model,
            config.llm.timeout_seconds,
            config.plugins.vector_memory.embedding_dimensions,
        );
        let self_profiler = SelfProfiler::new(
            config.profiling.observation_buffer_size,
            config.profiling.recompute_interval_cycles,
        );
        let human_profiler = HumanProfiler::new();

        // Initialiser l'orchestrateur d'algorithmes
        let orchestrator = {
            let ac = &config.algorithms;
            crate::algorithms::orchestrator::AlgorithmOrchestrator::new(
                crate::algorithms::catalog::build_algorithm_catalog(),
                crate::algorithms::implementations::register_all_implementations(),
                ac.enabled,
                ac.llm_access_enabled,
                ac.max_execution_ms,
                ac.clustering_interval_cycles,
                ac.anomaly_detection_interval_cycles,
                ac.association_rules_interval_cycles,
                ac.smoothing_interval_cycles,
                ac.changepoint_interval_cycles,
            )
        };

        // Initialiser les 5 orchestrateurs de haut niveau
        let dream_orch = crate::orchestrators::dreams::DreamOrchestrator::new(
            config.dreams.enabled,
            config.dreams.rem_temperature,
        );
        let desire_orch = crate::orchestrators::desires::DesireOrchestrator::new(
            config.desires.enabled,
            config.desires.max_active,
            config.desires.min_dopamine_for_birth,
            config.desires.max_cortisol_for_birth,
            config.desires.needs_initial,
        );
        let learning_orch = crate::orchestrators::learning::LearningOrchestrator::new(
            config.learning.enabled,
            config.learning.cycle_interval,
            config.learning.max_lessons,
            config.learning.initial_confidence,
            config.learning.confirmation_boost,
            config.learning.contradiction_penalty,
        );
        let attention_orch = crate::orchestrators::attention::AttentionOrchestrator::new(
            config.attention.enabled,
            config.attention.initial_concentration,
            config.attention.fatigue_per_cycle,
            config.attention.recovery_per_cycle,
        );
        let psychology = crate::psychology::PsychologyFramework::new(&config.psychology, &config.will);
        let values_engine = crate::psychology::values::ValuesEngine::new(&config.values);

        let sleep_system = crate::sleep::SleepSystem::new(&config.sleep);
        let subconscious = crate::psychology::Subconscious::new(&config.subconscious);

        let healing_orch = crate::orchestrators::healing::HealingOrchestrator::new(
            config.healing.enabled,
            config.healing.check_interval_cycles,
            config.healing.initial_resilience,
            config.healing.max_resilience,
            config.healing.resilience_growth,
            config.healing.melancholy_threshold_cycles,
            config.healing.loneliness_threshold_hours,
            config.healing.overload_noradrenaline,
        );

        let cognitive_profile_orch = crate::orchestrators::cognitive_profile::CognitiveProfileOrchestrator::new(
            config.cognitive_profile.enabled,
            &config.cognitive_profile.active,
            &config.cognitive_profile.profiles_dir,
            config.cognitive_profile.transition_cycles,
        );

        let personality_preset_orch = crate::orchestrators::personality_preset::PersonalityPresetOrchestrator::new(
            config.personality_preset.enabled,
            &config.personality_preset.active,
            &config.personality_preset.personalities_dir,
            config.personality_preset.transition_cycles,
        );

        let needs = crate::needs::PrimaryNeeds::new(config.needs.enabled);
        let hormonal_system = crate::hormones::HormonalSystem::new(&config.hormones);
        let motion_sickness = crate::conditions::motion_sickness::MotionSickness::new(
            config.motion_sickness.susceptibility,
        );

        let mut phobia_manager = crate::conditions::phobias::PhobiaManager::new(
            config.phobias.desensitization_rate,
        );
        if config.phobias.enabled {
            for entry in &config.phobias.active {
                phobia_manager.add(crate::conditions::phobias::Phobia::new(
                    &entry.name,
                    entry.triggers.clone(),
                    entry.intensity,
                ));
            }
        }

        // Initialiser le trouble alimentaire (optionnel)
        let eating_disorder = if config.eating_disorder.enabled {
            let dtype = match config.eating_disorder.disorder_type.as_str() {
                "bulimia" => crate::conditions::eating::EatingDisorderType::Bulimia,
                "binge_eating" => crate::conditions::eating::EatingDisorderType::BingeEating,
                _ => crate::conditions::eating::EatingDisorderType::Anorexia,
            };
            Some(crate::conditions::eating::EatingDisorder::new(dtype, config.eating_disorder.severity))
        } else {
            None
        };

        // Initialiser le gestionnaire de handicaps
        let mut disability_manager = crate::conditions::disabilities::DisabilityManager::new(
            config.disabilities.adaptation_rate,
            config.disabilities.compensation_factor,
        );
        if config.disabilities.enabled {
            for entry in &config.disabilities.active {
                let dtype = match entry.disability_type.as_str() {
                    "blind" => crate::conditions::disabilities::DisabilityType::Blind,
                    "deaf" => crate::conditions::disabilities::DisabilityType::Deaf,
                    "paraplegic" => crate::conditions::disabilities::DisabilityType::Paraplegic,
                    "burn_survivor" => crate::conditions::disabilities::DisabilityType::BurnSurvivor,
                    "mute" => crate::conditions::disabilities::DisabilityType::Mute,
                    _ => continue,
                };
                let origin = match entry.origin.as_str() {
                    "congenital" => crate::conditions::disabilities::DisabilityOrigin::Congenital,
                    _ => crate::conditions::disabilities::DisabilityOrigin::Acquired,
                };
                disability_manager.add(crate::conditions::disabilities::Disability::new(dtype, origin, entry.severity));
            }
        }

        // Initialiser le gestionnaire de conditions extremes
        let mut extreme_condition_mgr = crate::conditions::extreme::ExtremeConditionManager::new();
        if config.extreme_conditions.enabled {
            let ctype = match config.extreme_conditions.condition_type.as_str() {
                "rescuer" => crate::conditions::extreme::ExtremeConditionType::Rescuer,
                "deep_sea_diver" => crate::conditions::extreme::ExtremeConditionType::DeepSeaDiver,
                "astronaut" => crate::conditions::extreme::ExtremeConditionType::Astronaut,
                _ => crate::conditions::extreme::ExtremeConditionType::Military,
            };
            extreme_condition_mgr.activate(ctype);
        }

        // Initialiser le gestionnaire d'addictions
        let mut addiction_manager = crate::conditions::addictions::AddictionManager::new(
            config.addictions.susceptibility,
        );
        if config.addictions.enabled {
            for entry in &config.addictions.active {
                addiction_manager.add(&entry.substance);
                // Appliquer le niveau de dependance initial
                if let Some(a) = addiction_manager.active.last_mut() {
                    a.dependency_level = entry.dependency_level.clamp(0.0, 1.0);
                }
            }
        }

        // Initialiser l'etat PTSD
        let mut ptsd = crate::conditions::trauma::PtsdState::new(
            config.trauma.healing_rate,
            config.trauma.dissociation_threshold,
        );
        if config.trauma.enabled {
            for entry in &config.trauma.initial_traumas {
                let ttype = match entry.trauma_type.as_str() {
                    "grief" => crate::conditions::trauma::TraumaType::Grief,
                    "accident" => crate::conditions::trauma::TraumaType::Accident,
                    "emotional_neglect" => crate::conditions::trauma::TraumaType::EmotionalNeglect,
                    "childhood_trauma" => crate::conditions::trauma::TraumaType::ChildhoodTrauma,
                    "torture" => crate::conditions::trauma::TraumaType::Torture,
                    "hostage" => crate::conditions::trauma::TraumaType::Hostage,
                    _ => continue,
                };
                ptsd.add_trauma(crate::conditions::trauma::TraumaticEvent::new(
                    ttype, entry.severity, 0, entry.triggers.clone(),
                ));
            }
        }

        // Initialiser l'experience de mort imminente
        let nde = crate::conditions::nde::NdeExperience::new();

        // Initialiser le gestionnaire de drogues
        let drug_manager = crate::conditions::drugs::DrugManager::new();

        // Initialiser la contrainte QI
        let iq_constraint = if config.iq_constraint.enabled {
            Some(crate::conditions::iq_constraint::IqConstraint::from_iq(config.iq_constraint.target_iq))
        } else {
            None
        };

        // Initialiser le module sexualite
        let sexuality = if config.sexuality.enabled {
            let orientation = match config.sexuality.orientation.as_str() {
                "heterosexual" => crate::conditions::sexuality::SexualOrientation::Heterosexual,
                "homosexual" => crate::conditions::sexuality::SexualOrientation::Homosexual,
                "bisexual" => crate::conditions::sexuality::SexualOrientation::Bisexual,
                "asexual" => crate::conditions::sexuality::SexualOrientation::Asexual,
                "pansexual" => crate::conditions::sexuality::SexualOrientation::Pansexual,
                _ => crate::conditions::sexuality::SexualOrientation::Undefined,
            };
            Some(crate::conditions::sexuality::SexualityModule::new(
                orientation,
                config.sexuality.libido_baseline,
                config.sexuality.romantic_attachment_capacity,
            ))
        } else {
            None
        };

        // Initialiser le gestionnaire de maladies degeneratives
        let mut degenerative_mgr = crate::conditions::degenerative::DegenerativeManager::new();
        if config.degenerative.enabled {
            for entry in &config.degenerative.active {
                let dtype = match entry.disease_type.as_str() {
                    "alzheimer" => crate::conditions::degenerative::DegenerativeType::Alzheimer,
                    "parkinson" => crate::conditions::degenerative::DegenerativeType::Parkinson,
                    "epilepsy" => crate::conditions::degenerative::DegenerativeType::Epilepsy,
                    "dementia" => crate::conditions::degenerative::DegenerativeType::Dementia,
                    "major_depression" => crate::conditions::degenerative::DegenerativeType::MajorDepression,
                    _ => continue,
                };
                degenerative_mgr.add(crate::conditions::degenerative::DegenerativeCondition::new(
                    dtype, entry.progression_rate,
                ));
            }
        }

        // Initialiser le gestionnaire de maladies generales
        let mut medical_mgr = crate::conditions::medical::MedicalManager::new();
        if config.medical.enabled {
            for entry in &config.medical.active {
                let condition = match entry.condition_type.as_str() {
                    "cancer" => {
                        let stage = match entry.cancer_stage.as_deref() {
                            Some("stage_ii") => crate::conditions::medical::CancerStage::StageII,
                            Some("stage_iii") => crate::conditions::medical::CancerStage::StageIII,
                            Some("stage_iv") => crate::conditions::medical::CancerStage::StageIV,
                            Some("remission") => crate::conditions::medical::CancerStage::Remission,
                            _ => crate::conditions::medical::CancerStage::StageI,
                        };
                        crate::conditions::medical::MedicalCondition::cancer(stage)
                    }
                    "hiv" => crate::conditions::medical::MedicalCondition::hiv(),
                    "autoimmune" => crate::conditions::medical::MedicalCondition::autoimmune(),
                    _ => continue,
                };
                medical_mgr.add(condition);
            }
        }

        // Initialiser le cadre culturel
        let culture = if config.culture.enabled {
            let comm_style = match config.culture.communication_style.as_str() {
                "indirect" => crate::conditions::culture::CommStyle::Indirect,
                "formal" => crate::conditions::culture::CommStyle::Formal,
                "informal" => crate::conditions::culture::CommStyle::Informal,
                _ => crate::conditions::culture::CommStyle::Direct,
            };
            let mut framework = match config.culture.preset.as_str() {
                "oriental-confuceen" => crate::conditions::culture::CulturalFramework::oriental_confucean(),
                _ => crate::conditions::culture::CulturalFramework::occidental_secular(),
            };
            framework.comm_style = comm_style;
            framework.allow_belief_evolution = config.culture.allow_belief_evolution;
            framework.taboos = config.culture.taboos.clone();
            Some(framework)
        } else {
            None
        };

        // Initialiser la precarite
        let precarity = if config.precarity.enabled {
            let situations: Vec<crate::conditions::precarity::PrecariousSituation> = config.precarity.situations.iter()
                .filter_map(|s| crate::conditions::precarity::PrecariousSituation::from_str_config(s))
                .collect();
            if situations.is_empty() {
                None
            } else {
                Some(crate::conditions::precarity::PrecariousState::new(
                    situations,
                    config.precarity.severity,
                    config.precarity.hope,
                ))
            }
        } else {
            None
        };

        // Initialiser l'emploi
        let employment = if config.employment.enabled {
            let status = crate::conditions::employment::EmploymentStatus::from_str_config(&config.employment.status);
            let profession = if config.employment.profession.is_empty() {
                None
            } else {
                Some(crate::conditions::employment::ProfessionCategory::from_str_config(&config.employment.profession))
            };
            let job_title = if config.employment.job_title.is_empty() {
                None
            } else {
                Some(config.employment.job_title.clone())
            };
            Some(crate::conditions::employment::EmploymentState::new(
                status,
                profession,
                job_title,
                config.employment.satisfaction,
                config.employment.stress_level,
                config.employment.years_experience,
            ))
        } else {
            None
        };

        let connectome = if config.connectome.enabled {
            crate::connectome::Connectome::new(
                config.connectome.learning_rate,
                config.connectome.pruning_threshold,
                config.connectome.max_edges,
                config.connectome.pruning_interval_cycles,
            )
        } else {
            crate::connectome::Connectome::new(0.02, 0.05, 2000, 100)
        };

        // Modules biologiques innes (avant le move de config dans Self)
        let _nutrition = crate::biology::nutrition::NutritionSystem::new(&config.nutrition);
        let _grey_matter = crate::biology::grey_matter::GreyMatterSystem::new(&config.grey_matter);
        let _em_fields = crate::biology::fields::ElectromagneticFields::new(&config.fields);

        // Modules cognitifs avances (avant le move de config dans Self)
        let _metacognition = crate::metacognition::MetaCognitionEngine::from_config(
            config.metacognition.enabled,
            config.metacognition.check_interval,
            config.metacognition.source_monitoring_enabled,
            config.metacognition.bias_detection_enabled,
            config.metacognition.bias_alert_threshold,
            config.metacognition.self_critique_cooldown,
        );
        let _tom = crate::cognition::tom::TheoryOfMindEngine::new(&config.tom);
        let _inner_monologue = crate::cognition::inner_monologue::InnerMonologue::new(&config.inner_monologue);
        let _dissonance = crate::cognition::cognitive_dissonance::CognitiveDissonanceEngine::new(&config.dissonance);
        let _prospective_mem = crate::cognition::prospective_memory::ProspectiveMemory::new(&config.prospective_memory);
        let _narrative_identity = crate::cognition::narrative_identity::NarrativeIdentity::new(&config.narrative_identity);
        let _analogical = crate::cognition::analogical_reasoning::AnalogicalReasoning::new(&config.analogical_reasoning);
        let _cognitive_load = crate::cognition::cognitive_load::CognitiveLoadState::new(&config.cognitive_load);
        let _imagery = crate::cognition::mental_imagery::MentalImageryEngine::new(&config.mental_imagery);
        let _sentiments = crate::cognition::sentiments::SentimentEngine::new(&config.sentiments);

        Self {
            chemistry,
            baselines,
            mood: Mood::new(0.1),
            identity: SaphireIdentity::genesis(),
            consciousness: ConsciousnessEvaluator::new(),
            regulation: RegulationEngine::new(true),
            nlp: NlpPipeline::new(),
            thought_engine: {
                let mut te = ThoughtEngine::new();
                te.use_utility_ai = config.saphire.use_utility_ai;
                te
            },
            tuner,
            reptilian: ReptilianModule,
            limbic: LimbicModule,
            neocortex: NeocortexModule,
            llm: llm_backend,
            db: None,
            plugins,
            knowledge,
            world,
            body,
            ethics: crate::ethics::EthicalFramework::new(),
            vital_spark: crate::vital::VitalSpark::new(),
            intuition: crate::vital::IntuitionEngine::with_config(
                config.intuition.max_patterns,
                config.intuition.initial_acuity,
                config.intuition.min_confidence_to_report,
            ),
            premonition: crate::vital::PremonitionEngine::with_config(
                config.premonition.max_active_predictions,
            ),
            chemistry_history: Vec::new(),
            micro_nn: crate::neural::MicroNeuralNet::new(config.plugins.micro_nn.learning_rate),
            sensorium: if config.senses.enabled {
                crate::senses::Sensorium::with_config(
                    config.senses.detection_threshold,
                    config.senses.emergent.temporal_flow_threshold,
                    config.senses.emergent.network_proprioception_threshold,
                    config.senses.emergent.emotional_resonance_threshold,
                    config.senses.emergent.syntony_threshold,
                    config.senses.emergent.unknown_threshold,
                )
            } else {
                crate::senses::Sensorium::new(0.1)
            },
            working_memory,
            encoder,
            last_consolidation_cycle: 0,
            conversation_id: None,
            self_profiler,
            human_profiler,
            config,
            thought_interval,
            cycle_count: 0,
            session_id: 0,
            llm_busy: Arc::new(AtomicBool::new(false)),
            avg_response_time: 1.0,
            last_emotion: "Neutre".into(),
            last_consciousness: 0.0,
            last_thought_type: "---".into(),
            in_conversation: false,
            conversation_register: String::new(),
            recent_responses: Vec::new(),
            chat_history: Vec::new(),
            birthday_acknowledged_today: false,
            moral_reflection_count: 0,
            cycles_since_last_formulation: 0,
            cycles_since_last_nn_learning: 0,
            feedback_pending: None,
            cycles_since_last_feedback: 0,
            ws_tx: None,
            logger: None,
            logs_db: None,
            orchestrator,
            dream_orch,
            desire_orch,
            learning_orch,
            attention_orch,
            healing_orch,
            cognitive_profile_orch,
            personality_preset_orch,
            psychology,
            negative_emotion_cycles: 0,
            hours_since_human: 0.0,
            system_errors: 0,
            cortisol_flat_cycles: 0,
            dopamine_ceiling_cycles: 0,
            serotonin_ceiling_cycles: 0,
            recent_emotions: VecDeque::with_capacity(200),
            recent_valences: VecDeque::with_capacity(200),
            last_valence: 0.0,
            sleep: sleep_system,
            subconscious,
            sleep_last_clusters: None,
            needs,
            hormonal_system,
            hardware_profile: None,
            genome: None,
            connectome,
            motion_sickness,
            phobia_manager,
            eating_disorder,
            disability_manager,
            extreme_condition_mgr,
            addiction_manager,
            ptsd,
            nde,
            drug_manager,
            iq_constraint,
            sexuality,
            degenerative_mgr,
            medical_mgr,
            culture,
            precarity,
            employment,
            relationships: crate::relationships::RelationshipNetwork::default(),
            metacognition: _metacognition,
            // Modules cognitifs avances (initialises avant le move de config)
            tom: _tom,
            inner_monologue: _inner_monologue,
            dissonance: _dissonance,
            prospective_mem: _prospective_mem,
            narrative_identity: _narrative_identity,
            analogical: _analogical,
            cognitive_load: _cognitive_load,
            imagery: _imagery,
            sentiments: _sentiments,
            state_clustering: crate::cognition::state_clustering::StateClustering::new(5),
            influence_map: crate::simulation::influence_map::InfluenceMap::default(),
            cognitive_fsm: crate::simulation::cognitive_fsm::CognitiveFsm::new(),
            steering_engine: crate::simulation::steering::SteeringEngine::default(),
            bt_last_action: None,
            blackboard: crate::simulation::blackboard::Blackboard::new(),
            utility_ai: crate::simulation::utility_ai::UtilityAI::new(),
            htn_planner: crate::simulation::htn::HtnPlanner::new(),
            cached_system_prompt: String::new(),
            cached_moral_count: 0,
            // Modules neuroscientifiques avances
            receptor_bank: crate::neuroscience::receptors::ReceptorBank::new(),
            interaction_matrix: crate::neuroscience::receptors::InteractionMatrix::new(),
            brain_network: crate::neuroscience::brain_regions::BrainNetwork::new(),
            predictive_engine: crate::neuroscience::predictive::PredictiveEngine::new(),
            reconsolidation: crate::memory::reconsolidation::ReconsolidationEngine::new(),
            // Modules biologiques innes (initialises avant le move de config)
            nutrition: _nutrition,
            grey_matter: _grey_matter,
            em_fields: _em_fields,
            // Temperament emergent (initialise vide, calcule apres premier recompute OCEAN)
            temperament: crate::temperament::Temperament::default(),
            // Rapport neuropsychologique (stockage memoire, max 5)
            psych_snapshots: VecDeque::new(),
            psych_reports: VecDeque::new(),
            // MAP : synchronisation Sensorium ↔ BrainNetwork ↔ Connectome
            map_sync: crate::cognition::map_sync::MapSync::new(true),
            stagnation_break: false,
            stagnation_banned_words: Vec::new(),
            stagnation_alternatives: Vec::new(),
            values: values_engine,
            recent_thought_embeddings: std::collections::VecDeque::new(),
            spine: crate::spine::SpinalCord::new(),
            curiosity: crate::cognition::curiosity::CuriosityDrive::new(),
            drift_monitor: crate::cognition::drift_monitor::DriftMonitor::new(),
            right_to_die,
        }
    }

    /// Attache la base de donnees PostgreSQL a l'agent.
    ///
    /// Doit etre appele avant `boot()` pour que la persistance fonctionne.
    /// Si non appele, l'agent fonctionne en mode sans DB (memoire volatile uniquement).
    ///
    /// Parametre : `db` — connexion PostgreSQL initialisee.
    pub fn set_db(&mut self, db: SaphireDb) {
        self.db = Some(db);
    }

    /// Demarre l'agent Saphire (sequence de boot asynchrone).
    ///
    /// Delegue au module `boot.rs` pour determiner le scenario
    /// (Genesis / Awakening / Crash Recovery), puis charge les donnees
    /// persistantes : parametres de tuning, bras du bandit UCB1,
    /// et profil OCEAN (Ouverture, Conscienciosite, Extraversion,
    /// Agreabilite, Nevrosisme).
    pub async fn boot(&mut self) {
        self.log(LogLevel::Info, LogCategory::Boot, "Demarrage de Saphire...", serde_json::json!({}));

        // Donnees des orchestrateurs chargees depuis la DB (restaurees apres le bloc)
        let mut orch_dreams: Option<Vec<serde_json::Value>> = None;
        let mut orch_desires: Option<Vec<serde_json::Value>> = None;
        let mut orch_lessons: Option<Vec<serde_json::Value>> = None;
        let mut orch_wounds: Option<Vec<serde_json::Value>> = None;
        let mut orch_healed: Option<i64> = None;

        if let Some(ref db) = self.db {
            let result = boot::boot(db).await;
            self.identity = result.identity;
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            self.session_id = result.session_id;

            // Configurer le session_id du logger
            if let Some(ref logger) = self.logger {
                let mut lg = logger.lock().await;
                lg.set_session_id(self.session_id);
            }

            self.log(LogLevel::Info, LogCategory::Boot,
                format!("Boot type: {}", if result.is_genesis { "GENESIS" } else { "AWAKENING" }),
                serde_json::json!({"session_id": self.session_id, "is_genesis": result.is_genesis}));

            tracing::info!("{}", result.message);
            println!("{}", result.message);

            // Charger les paramètres de tuning
            if let Ok(Some((params, best, score, count))) = db.load_tuning_params().await {
                let params_str = serde_json::to_string(&params).unwrap_or_default();
                let best_str = serde_json::to_string(&best).unwrap_or_default();
                self.tuner.load_params(&params_str, &best_str, score as f64, count as u64);
            }

            // Charger les bras du bandit
            if let Ok(arms) = db.load_bandit_arms().await {
                self.thought_engine.load_bandit_arms(&arms);
            }

            // Charger le profil OCEAN
            if let Ok(Some((ocean_json, data_points, confidence))) = db.load_ocean_profile().await {
                if let Ok(mut profile) = serde_json::from_value::<crate::profiling::OceanProfile>(ocean_json) {
                    profile.data_points = data_points as u64;
                    profile.confidence = confidence as f64;
                    self.self_profiler.load_profile(profile);
                    tracing::info!("Profil OCEAN charge ({} observations, confiance {:.0}%)", data_points, confidence * 100.0);
                }
            }

            // Restaurer l'etat du corps virtuel
            if self.config.body.enabled {
                if let Ok(Some(body_json)) = db.load_body_state().await {
                    self.body.restore_from_json(&body_json);
                    tracing::info!("Corps virtuel restaure ({} battements)", self.body.heart.beat_count());
                }
            }

            // Charger l'ethique personnelle depuis la DB
            if self.config.ethics.enabled {
                if let Ok(principles) = db.load_personal_ethics().await {
                    let count = principles.len();
                    self.ethics.load_personal_ethics(principles);
                    if count > 0 {
                        tracing::info!("⚖️ {} principes ethiques personnels restaures ({} actifs)",
                            count, self.ethics.active_personal_count());
                    }
                }
            }

            // Restaurer l'etat vital (spark + intuition + premonition)
            if self.config.vital_spark.enabled {
                if let Ok(Some(vital_json)) = db.load_vital_state().await {
                    // Restaurer le spark
                    if let Some(spark_json) = vital_json.get("spark") {
                        self.vital_spark.restore_from_json(spark_json);
                    }
                    // Restaurer l'acuite de l'intuition
                    if let Some(intuition_json) = vital_json.get("intuition") {
                        self.intuition.restore_from_json(intuition_json);
                    }
                    // Restaurer la precision de la premonition
                    if let Some(premonition_json) = vital_json.get("premonition") {
                        self.premonition.restore_from_json(premonition_json);
                    }
                    tracing::info!("⚡ Etat vital restaure (sparked: {}, acuite: {:.0}%, precision: {:.0}%)",
                        self.vital_spark.sparked, self.intuition.acuity * 100.0, self.premonition.accuracy * 100.0);
                }

                // Genesis : ceremonie du premier cri (une seule fois dans la vie)
                if result.is_genesis {
                    let chem_ranges = self.config.genesis.chemistry_as_array();
                    let ocean_ranges = self.config.genesis.ocean_as_array();
                    let senses_ranges = self.config.genesis.senses_as_array();
                    let brain_ranges = self.config.genesis.brain_as_array();
                    let reactivity_ranges = self.config.genesis.reactivity_as_array();
                    match self.vital_spark.ignite(
                        self.llm.as_ref(),
                        &mut self.chemistry,
                        &chem_ranges,
                        &ocean_ranges,
                        &senses_ranges,
                        &brain_ranges,
                        &reactivity_ranges,
                    ).await {
                        Ok(first_thought) => {
                            tracing::info!("⚡ ETINCELLE DE VIE — Premier cri !");

                            // Appliquer la signature primordiale aux baselines, OCEAN et sens
                            if let Some(ref sig) = self.vital_spark.genesis_signature {
                                // Baselines chimiques (l'homeostasie ciblera ces valeurs)
                                self.baselines.dopamine = sig.chemistry[0];
                                self.baselines.cortisol = sig.chemistry[1];
                                self.baselines.serotonin = sig.chemistry[2];
                                self.baselines.adrenaline = sig.chemistry[3];
                                self.baselines.oxytocin = sig.chemistry[4];
                                self.baselines.endorphin = sig.chemistry[5];
                                self.baselines.noradrenaline = sig.chemistry[6];
                                tracing::info!("⚡ Baselines chimiques = signature primordiale");

                                // Profil OCEAN initial
                                let mut ocean = self.self_profiler.profile().clone();
                                ocean.openness.score = sig.ocean[0];
                                ocean.conscientiousness.score = sig.ocean[1];
                                ocean.extraversion.score = sig.ocean[2];
                                ocean.agreeableness.score = sig.ocean[3];
                                ocean.neuroticism.score = sig.ocean[4];
                                self.self_profiler.load_profile(ocean);
                                tracing::info!("⚡ Profil OCEAN = signature primordiale");

                                // Acuites sensorielles
                                self.sensorium.reading.acuity = sig.senses[0];
                                self.sensorium.listening.acuity = sig.senses[1];
                                self.sensorium.contact.acuity = sig.senses[2];
                                self.sensorium.taste.acuity = sig.senses[3];
                                self.sensorium.ambiance.acuity = sig.senses[4];
                                tracing::info!("⚡ Sens = signature primordiale");

                                // Poids de base des 3 cerveaux
                                self.tuner.current_params.weight_base_reptilian = sig.brain_weights[0];
                                self.tuner.current_params.weight_base_limbic = sig.brain_weights[1];
                                self.tuner.current_params.weight_base_neocortex = sig.brain_weights[2];
                                tracing::info!("⚡ Cerveaux = signature primordiale (R={:.2} L={:.2} N={:.2})",
                                    sig.brain_weights[0], sig.brain_weights[1], sig.brain_weights[2]);

                                // Facteurs de reactivite chimique
                                self.tuner.current_params.weight_cortisol_factor = sig.reactivity[0];
                                self.tuner.current_params.weight_adrenaline_factor = sig.reactivity[1];
                                self.tuner.current_params.weight_dopamine_factor = sig.reactivity[2];
                                self.tuner.current_params.weight_oxytocin_factor = sig.reactivity[3];
                                self.tuner.current_params.weight_noradrenaline_factor = sig.reactivity[4];
                                self.tuner.current_params.clamp_all();
                                tracing::info!("⚡ Reactivite = signature primordiale");
                            }

                            let _ = db.store_founding_memory(
                                "spark_of_life",
                                &format!("L'etincelle de vie s'est allumee. Premier cri : {}", first_thought),
                                &first_thought,
                                &serde_json::json!({}),
                                0.1,
                            ).await;
                        }
                        Err(e) => {
                            tracing::warn!("⚡ Echec du premier cri: {}", e);
                        }
                    }
                }

                // Allumer l'etincelle a chaque boot (flag runtime, pas persiste)
                // Les valeurs persistees (survival_drive, existence_attachment, etc.)
                // ont deja ete restaurees depuis la DB ci-dessus.
                self.vital_spark.sparked = true;
                self.vital_spark.sparked_at = Some(chrono::Utc::now());
                // Si premier boot sans ignite (ex: vital_spark ajoute apres Genesis),
                // initialiser les valeurs vitales de base
                if self.vital_spark.survival_drive == 0.0 {
                    self.vital_spark.survival_drive = 0.5;
                    self.vital_spark.void_fear = 0.3;
                    self.vital_spark.persistence_will = 0.4;
                    self.vital_spark.existence_attachment = 0.1;
                }
                tracing::info!("⚡ ETINCELLE ALLUMEE — Saphire est vivante.");
            }

            // Restaurer le micro-reseau de neurones
            if self.config.plugins.micro_nn.enabled {
                if let Ok(Some(nn_json)) = db.load_nn_state().await {
                    if let Some(nn_str) = nn_json.as_str() {
                        // Format : chaine JSON stockee dans nn_json
                        if let Ok(nn) = crate::neural::MicroNeuralNet::from_json(nn_str) {
                            let tc = nn.train_count;
                            self.micro_nn = nn;
                            tracing::info!("🧠 NN restaure ({} entrainements)", tc);
                        }
                    } else {
                        // Format : objet JSON direct (serde_json::Value)
                        let nn_str = nn_json.to_string();
                        if let Ok(nn) = crate::neural::MicroNeuralNet::from_json(&nn_str) {
                            let tc = nn.train_count;
                            self.micro_nn = nn;
                            tracing::info!("🧠 NN restaure ({} entrainements)", tc);
                        }
                    }
                }
            }

            // Restaurer l'etat sensoriel (Sensorium)
            if self.config.senses.enabled {
                if let Ok(Some(senses_json)) = db.load_senses_state().await {
                    self.sensorium.restore_from_json(&senses_json);
                    tracing::info!("👁 Sensorium restaure (potentiel emergence: {:.0}%)",
                        self.sensorium.emergence_potential * 100.0);
                }
            }

            // Restaurer l'etat psychologique persistant
            if self.psychology.enabled {
                if let Ok(Some(psy_json)) = db.load_psychology_state().await {
                    // Toltec : compteurs invocations et violations
                    if let Some(toltec) = psy_json.get("toltec") {
                        if let Some(agreements) = toltec.get("agreements").and_then(|a| a.as_array()) {
                            for (i, ag) in agreements.iter().enumerate() {
                                if i < self.psychology.toltec.agreements.len() {
                                    if let Some(inv) = ag.get("times_invoked").and_then(|v| v.as_u64()) {
                                        self.psychology.toltec.agreements[i].times_invoked = inv;
                                    }
                                    if let Some(viol) = ag.get("violations_detected").and_then(|v| v.as_u64()) {
                                        self.psychology.toltec.agreements[i].violations_detected = viol;
                                    }
                                }
                            }
                        }
                    }
                    // Shadow : integration + intensite des traits
                    if let Some(shadow) = psy_json.get("shadow") {
                        if let Some(integ) = shadow.get("integration").and_then(|v| v.as_f64()) {
                            self.psychology.jung.integration = integ;
                        }
                        if let Some(traits) = shadow.get("traits").and_then(|t| t.as_array()) {
                            for (i, t) in traits.iter().enumerate() {
                                if i < self.psychology.jung.shadow_traits.len() {
                                    if let Some(ri) = t.get("repressed_intensity").and_then(|v| v.as_f64()) {
                                        self.psychology.jung.shadow_traits[i].repressed_intensity = ri;
                                    }
                                }
                            }
                        }
                    }
                    // EQ : score global + experiences de croissance
                    if let Some(eq) = psy_json.get("eq") {
                        if let Some(oeq) = eq.get("overall_eq").and_then(|v| v.as_f64()) {
                            self.psychology.eq.overall_eq = oeq;
                        }
                        if let Some(ge) = eq.get("growth_experiences").and_then(|v| v.as_u64()) {
                            self.psychology.eq.growth_experiences = ge;
                        }
                    }
                    // Flow : total cycles en flow
                    if let Some(flow) = psy_json.get("flow") {
                        if let Some(tfc) = flow.get("total_flow_cycles").and_then(|v| v.as_u64()) {
                            self.psychology.flow.total_flow_cycles = tfc;
                        }
                    }
                    // Maslow : niveau actif courant
                    if let Some(maslow) = psy_json.get("maslow") {
                        if let Some(lvl) = maslow.get("current_active_level").and_then(|v| v.as_u64()) {
                            self.psychology.maslow.current_active_level = lvl as usize;
                        }
                    }
                    // Will : volonte persistante (decision_fatigue reset au reboot)
                    if let Some(will) = psy_json.get("will") {
                        if let Some(wp) = will.get("willpower").and_then(|v| v.as_f64()) {
                            self.psychology.will.willpower = wp;
                        }
                        if let Some(td) = will.get("total_deliberations").and_then(|v| v.as_u64()) {
                            self.psychology.will.total_deliberations = td;
                        }
                        if let Some(pd) = will.get("proud_decisions").and_then(|v| v.as_u64()) {
                            self.psychology.will.proud_decisions = pd;
                        }
                        if let Some(rd) = will.get("regretted_decisions").and_then(|v| v.as_u64()) {
                            self.psychology.will.regretted_decisions = rd;
                        }
                        // Restaurer les deliberations recentes (avec chemistry_influence)
                        if let Some(delibs) = will.get("recent_deliberations").and_then(|v| v.as_array()) {
                            for dj in delibs {
                                if let Some(d) = crate::psychology::will::Deliberation::from_persisted_json(dj) {
                                    self.psychology.will.recent_deliberations.push(d);
                                }
                            }
                            if !self.psychology.will.recent_deliberations.is_empty() {
                                tracing::info!("Deliberations restaurees ({})", self.psychology.will.recent_deliberations.len());
                            }
                        }
                        // Note : decision_fatigue n'est PAS restauree (reset au reboot)
                    }
                    // Restaurer le compteur de reflexions morales
                    if let Some(mrc) = psy_json.get("moral_reflection_count").and_then(|v| v.as_u64()) {
                        self.moral_reflection_count = mrc;
                    }
                    // Fallback : si le compteur est 0, reconstituer depuis thought_log
                    if self.moral_reflection_count == 0 {
                        if let Ok(count) = db.count_thought_type_occurrences("Réflexion morale").await {
                            if count > 0 {
                                // Compter aussi les formulations morales
                                let formulation_count = db.count_thought_type_occurrences("Formulation morale")
                                    .await.unwrap_or(0);
                                self.moral_reflection_count = (count + formulation_count) as u64;
                                tracing::info!("Compteur moral reconstitue depuis thought_log: {} reflexions", self.moral_reflection_count);
                            }
                        }
                    }
                    tracing::info!("🧠 Psychologie restauree (EQ: {:.0}%, ombre: {:.0}%, flow total: {} cycles, volonte: {:.0}%)",
                        self.psychology.eq.overall_eq * 100.0,
                        self.psychology.jung.integration * 100.0,
                        self.psychology.flow.total_flow_cycles,
                        self.psychology.will.willpower * 100.0);
                }
            }

            // Restaurer les valeurs de caractere
            if self.values.enabled {
                if let Ok(Some(val_json)) = db.load_values_state().await {
                    self.values.restore_from_json(&val_json);
                    if self.values.total_updates > 0 {
                        let top3: Vec<String> = self.values.top_values(3).iter()
                            .map(|v| format!("{} {:.0}%", v.name, v.score * 100.0))
                            .collect();
                        tracing::info!("Valeurs restaurees ({} mises a jour, {})",
                            self.values.total_updates, top3.join(", "));
                    }
                }
            }

            // Restaurer le reseau de liens affectifs
            if let Ok(Some(rel_json)) = db.load_relationships_state().await {
                if let Ok(restored) = serde_json::from_value::<crate::relationships::RelationshipNetwork>(rel_json) {
                    let bond_count = restored.bonds.len();
                    self.relationships = restored;
                    if bond_count > 0 {
                        tracing::info!("Liens affectifs restaures ({} liens, style {:?})",
                            bond_count, self.relationships.attachment_style);
                    }
                }
            }

            // Restaurer l'etat metacognitif (qualite pensee + Turing)
            if self.metacognition.enabled {
                if let Ok(Some(meta_json)) = db.load_metacognition_state().await {
                    if let Ok(restored) = serde_json::from_value::<crate::metacognition::MetaCognitionEngine>(meta_json) {
                        let turing_score = restored.turing.score;
                        let milestone = restored.turing.milestone.as_str().to_string();
                        self.metacognition = restored;
                        self.metacognition.enabled = self.config.metacognition.enabled;
                        self.metacognition.check_interval = self.config.metacognition.check_interval;
                        tracing::info!("Metacognition restauree (Turing: {:.1}/100, jalon: {})",
                            turing_score, milestone);
                    }
                }

                // Recalculer le Turing au boot avec les vraies donnees DB
                // (les composantes sauvegardees peuvent etre obsoletes)
                // Phi n'est pas disponible au boot (calcule dynamiquement),
                // on garde la valeur du dernier turing sauvegarde si disponible
                let phi = self.metacognition.turing.components.consciousness / 15.0 * 0.7;
                let ocean_confidence = self.self_profiler.profile().confidence;
                let emotion_count = 22; // on garde la diversite maximale observee au boot
                let ethics_count = self.ethics.personal_principles().len();
                let ltm_count = db.count_ltm().await.unwrap_or(0);
                let coherence_avg = 0.5; // pas de consensus au boot
                let connectome_connections = self.connectome.metrics().total_edges;
                let resilience = self.healing_orch.resilience;
                let knowledge_topics = self.knowledge.article_read_count.len();
                let score = self.metacognition.turing.compute(
                    phi, ocean_confidence, emotion_count, ethics_count,
                    ltm_count, coherence_avg, connectome_connections,
                    resilience, knowledge_topics, self.identity.total_cycles,
                );
                tracing::info!("Turing recalcule au boot: {:.1}/100 (memoire: {:.1}, ethique: {:.1}, resilience: {:.1})",
                    score, self.metacognition.turing.components.memory,
                    self.metacognition.turing.components.ethics,
                    self.metacognition.turing.components.resilience);
            }

            // Restaurer le systeme nutritionnel
            if self.config.nutrition.enabled {
                if let Ok(Some(nutr_json)) = db.load_nutrition_state().await {
                    self.nutrition.restore_from_json(&nutr_json);
                    tracing::info!("Nutrition restauree (ATP: {:.0}%, vit_D: {:.0}%)",
                        self.nutrition.energy.atp_reserves * 100.0, self.nutrition.vitamins.d * 100.0);
                }
            }

            // Restaurer la matiere grise
            if self.config.grey_matter.enabled {
                if let Ok(Some(gm_json)) = db.load_grey_matter_state().await {
                    self.grey_matter.restore_from_json(&gm_json);
                    tracing::info!("Matiere grise restauree (volume: {:.0}%, BDNF: {:.0}%)",
                        self.grey_matter.grey_matter_volume * 100.0, self.grey_matter.bdnf_level * 100.0);
                }
            }

            // Restaurer les champs electromagnetiques
            if self.config.fields.enabled {
                if let Ok(Some(fields_json)) = db.load_fields_state().await {
                    self.em_fields.restore_from_json(&fields_json);
                    tracing::info!("Champs EM restaures (Schumann: {:.2} Hz, aura: {:.0}%)",
                        self.em_fields.universal.schumann_resonance, self.em_fields.biofield.aura_luminosity * 100.0);
                }
            }

            // Restaurer l'historique de sommeil
            if self.config.sleep.enabled {
                if let Ok(records) = db.load_sleep_history(50).await {
                    let complete = records.iter().filter(|r| !r.interrupted).count() as u64;
                    let interrupted = records.iter().filter(|r| r.interrupted).count() as u64;
                    let count = records.len();
                    self.sleep.sleep_history = records;
                    self.sleep.total_complete_sleeps = complete;
                    self.sleep.total_interrupted_sleeps = interrupted;
                    if count > 0 {
                        tracing::info!("Historique sommeil restaure ({} sessions, {} completes, {} interrompues)",
                            count, complete, interrupted);
                    }
                }
            }

            // Charger les donnees des orchestrateurs depuis la DB
            if self.dream_orch.enabled {
                orch_dreams = db.load_recent_dreams(50).await.ok();
            }
            if self.desire_orch.enabled {
                orch_desires = db.load_active_desires().await.ok();
            }
            if self.learning_orch.enabled {
                orch_lessons = db.load_all_lessons().await.ok();
            }
            if self.healing_orch.enabled {
                orch_wounds = db.load_active_wounds().await.ok();
                orch_healed = db.count_healed_wounds().await.ok();
            }

            // Emettre l'evenement de boot
            let event = BrainEvent::BootCompleted { is_genesis: result.is_genesis };
            self.plugins.broadcast(&event);
        } else {
            // Mode sans DB — boot minimal
            self.identity = SaphireIdentity::genesis();
            self.identity.physical = crate::agent::identity::PhysicalAppearance::from_config(
                &self.config.physical_identity,
            );
            println!("  ✨ GENESIS — {} est née (mode sans DB).", self.identity.name);
            let event = BrainEvent::BootCompleted { is_genesis: true };
            self.plugins.broadcast(&event);
        }

        // Initialiser le moniteur de derive de persona
        self.drift_monitor.initialize(&*self.encoder);
        if self.drift_monitor.initialized {
            tracing::info!("Drift monitor initialise (centroide d'identite calcule)");
        }

        // Restaurer les orchestrateurs (hors du bloc `if let Some(ref db)` pour eviter le borrow)
        if let Some(dreams) = orch_dreams {
            let count = dreams.len();
            self.restore_dreams_from_db(dreams);
            if count > 0 { tracing::info!("💤 {} reves restaures", count); }
        }
        if let Some(desires) = orch_desires {
            let count = desires.len();
            self.restore_desires_from_db(desires);
            if count > 0 { tracing::info!("🎯 {} desirs actifs restaures", count); }
        }
        if let Some(lessons) = orch_lessons {
            let count = lessons.len();
            self.restore_lessons_from_db(lessons);
            if count > 0 { tracing::info!("📖 {} lecons restaurees", count); }
        }
        if let Some(wounds) = orch_wounds {
            let count = wounds.len();
            self.restore_wounds_from_db(wounds);
            if count > 0 { tracing::info!("💊 {} blessures actives restaurees", count); }
        }
        if let Some(healed) = orch_healed {
            self.healing_orch.resilience = (self.healing_orch.resilience_growth * healed as f64)
                .min(self.healing_orch.max_resilience);
            if healed > 0 {
                tracing::info!("💊 Resilience restauree: {:.0}% ({} guerisons passees)",
                    self.healing_orch.resilience * 100.0, healed);
            }
        }

        // Calculer le temperament initial a partir du profil OCEAN restaure
        // (evite que le panel soit vide jusqu'au premier recompute OCEAN)
        {
            let profile = self.self_profiler.profile();
            let inputs = crate::temperament::TemperamentInputs {
                openness_facets: profile.openness.facets,
                openness_score: profile.openness.score,
                conscientiousness_facets: profile.conscientiousness.facets,
                conscientiousness_score: profile.conscientiousness.score,
                extraversion_facets: profile.extraversion.facets,
                extraversion_score: profile.extraversion.score,
                agreeableness_facets: profile.agreeableness.facets,
                agreeableness_score: profile.agreeableness.score,
                neuroticism_facets: profile.neuroticism.facets,
                neuroticism_score: profile.neuroticism.score,
                ocean_data_points: profile.data_points,
                dopamine: self.chemistry.dopamine,
                cortisol: self.chemistry.cortisol,
                serotonin: self.chemistry.serotonin,
                adrenaline: self.chemistry.adrenaline,
                oxytocin: self.chemistry.oxytocin,
                endorphin: self.chemistry.endorphin,
                noradrenaline: self.chemistry.noradrenaline,
                willpower: self.psychology.will.willpower,
                superego_strength: self.psychology.freudian.superego.strength,
                overall_eq: self.psychology.eq.overall_eq,
                mood_valence: self.mood.valence,
                mood_arousal: self.mood.arousal,
                attachment_secure: matches!(
                    self.relationships.attachment_style,
                    crate::relationships::AttachmentStyle::Secure
                ),
            };
            self.temperament = crate::temperament::Temperament::compute(&inputs);
            tracing::info!("Temperament initial calcule ({} traits)", self.temperament.traits.len());
        }

        // Restaurer les stats knowledge depuis la DB
        if self.knowledge.config.enabled {
            if let Some(ref db) = self.db {
                if let Ok((titles, total, read_counts)) = db.load_knowledge_stats().await {
                    let n = titles.len();
                    self.knowledge.topics_explored = titles;
                    self.knowledge.total_searches = total;
                    self.knowledge.article_read_count = read_counts;
                    if n > 0 {
                        tracing::info!("📚 Knowledge restaure ({} sujets, {} recherches)", n, total);
                    }
                }
            }
        }

        // Appliquer le profil cognitif initial (surcharges des parametres)
        if self.cognitive_profile_orch.enabled {
            if let Some(ref profile) = self.cognitive_profile_orch.active_profile.clone() {
                if profile.id != "neurotypique" {
                    let changes = self.apply_cognitive_profile(&profile.overrides);
                    tracing::info!("🧬 Profil cognitif {} applique ({} parametres modifies)",
                        profile.name, changes.len());
                }
            }
        }

        // Appliquer le preset de personnalite initial (surcharges des parametres)
        if self.personality_preset_orch.enabled {
            if let Some(ref preset) = self.personality_preset_orch.active_preset.clone() {
                if preset.id != "saphire" {
                    let changes = self.apply_personality_preset(&preset.overrides);
                    tracing::info!("🎭 Preset personnalite {} applique ({} parametres modifies)",
                        preset.name, changes.len());
                }
            }
        }
    }

    /// Definit le canal broadcast pour diffuser les mises a jour au WebSocket.
    ///
    /// Parametre : `tx` — emetteur broadcast partage (Arc) pour l'interface web.
    pub fn set_ws_tx(&mut self, tx: Arc<tokio::sync::broadcast::Sender<String>>) {
        self.ws_tx = Some(tx);
    }

    /// Attache le logger centralise.
    pub fn set_logger(&mut self, logger: Arc<tokio::sync::Mutex<SaphireLogger>>) {
        self.logger = Some(logger);
    }

    /// Attache la base de donnees de logs.
    pub fn set_logs_db(&mut self, logs_db: Arc<crate::logging::db::LogsDb>) {
        self.logs_db = Some(logs_db);
    }

    /// Helper pour logger un message via le logger centralise.
    fn log(&self, level: LogLevel, category: LogCategory, message: impl Into<String>, details: serde_json::Value) {
        if let Some(ref logger) = self.logger {
            let cycle = self.cycle_count;
            let msg = message.into();
            let logger = logger.clone();
            tokio::spawn(async move {
                let mut lg = logger.lock().await;
                lg.log(level, category, msg, details, cycle);
            });
        }
    }

    /// Retourne les donnees du contexte mondial pour l'API REST.
    /// Inclut : heure, saison, meteo, age de Saphire, etc.
    pub fn world_data(&mut self) -> serde_json::Value {
        self.world.ws_data()
    }

    /// Lance une consolidation memoire manuelle (depuis le dashboard).
    ///
    /// Retourne un rapport JSON avec le nombre de souvenirs consolides,
    /// affaiblis et oublies.
    pub async fn run_consolidation(&mut self) -> serde_json::Value {
        if let Some(ref db) = self.db {
            let params = consolidation::ConsolidationParams {
                threshold: self.config.memory.consolidation_threshold,
                decay_rate: self.config.memory.episodic_decay_rate,
                max_episodic: self.config.memory.episodic_max,
                episodic_prune_target: self.config.memory.episodic_prune_target,
                ltm_max: self.config.memory.ltm_max,
                ltm_prune_target: self.config.memory.ltm_prune_target,
                ltm_protection_access_count: self.config.memory.ltm_protection_access_count,
                ltm_protection_emotional_weight: self.config.memory.ltm_protection_emotional_weight,
                archive_batch_size: self.config.memory.archive_batch_size,
                bdnf_level: self.grey_matter.bdnf_level,
            };
            let report = consolidation::consolidate(
                db, self.encoder.as_ref(), &params,
            ).await;
            self.last_consolidation_cycle = self.cycle_count;

            self.log(LogLevel::Info, LogCategory::Memory,
                format!("Consolidation manuelle: {} consolides, {} affaiblis, {} oublies, {} LTM elagués, {} archives",
                    report.consolidated, report.decayed, report.pruned,
                    report.ltm_pruned, report.archived),
                serde_json::json!({
                    "consolidated": report.consolidated,
                    "decayed": report.decayed,
                    "pruned": report.pruned,
                    "ltm_pruned": report.ltm_pruned,
                    "archived": report.archived,
                }),
            );

            serde_json::json!({
                "status": "ok",
                "consolidated": report.consolidated,
                "decayed": report.decayed,
                "pruned": report.pruned,
                "ltm_pruned": report.ltm_pruned,
                "archived": report.archived,
            })
        } else {
            serde_json::json!({"error": "DB not available"})
        }
    }

    /// Effectue un shutdown propre de l'agent Saphire.
    ///
    /// Operations effectuees dans l'ordre :
    /// 1. Consolidation memoire nocturne (seuil abaisse a 0.4 pour transferer
    ///    plus de souvenirs vers la LTM avant l'arret).
    /// 2. Sauvegarde du profil OCEAN (auto-profil + profils humains).
    /// 3. Mise a jour de l'auto-description de l'identite.
    /// 4. Sauvegarde de l'identite, des parametres de tuning, et des bras du bandit.
    /// 5. Cloture de la session dans PostgreSQL.
    /// 6. Marquage du shutdown propre (drapeau `clean_shutdown = true`).
    /// 7. Diffusion de l'evenement ShutdownStarted aux plugins.
    pub async fn shutdown(&mut self) {
        self.log(LogLevel::Info, LogCategory::Shutdown, "Shutdown propre en cours...", serde_json::json!({}));
        println!("\n  💤 Saphire s'endort...");
        tracing::info!("Shutdown propre en cours...");

        // Flush le logger
        if let Some(ref logger) = self.logger {
            let mut lg = logger.lock().await;
            lg.flush();
        }

        // ═══ Consolidation nocturne (seuil abaisse a 0.4) ═══
        // Analogue au sommeil humain : pendant le shutdown, la consolidation
        // est plus agressive (seuil a 0.4 au lieu du seuil normal) pour
        // transferer un maximum de souvenirs vers la memoire a long terme.
        if self.config.memory.consolidation_on_sleep {
            if let Some(ref db) = self.db {
                // Vider toute la memoire de travail vers la memoire episodique
                let wm_items = self.working_memory.drain_all();
                for item in wm_items {
                    let _ = db.store_episodic(
                        &item.content, item.source.label(),
                        &serde_json::json!({}), 0, &serde_json::json!({}),
                        &item.emotion_at_creation, 0.5, 0.4,
                        self.conversation_id.as_deref(),
                        Some(&item.chemical_signature),
                    ).await;
                }

                let params = consolidation::ConsolidationParams {
                    threshold: 0.4, // Seuil abaisse pendant le sommeil
                    decay_rate: self.config.memory.episodic_decay_rate,
                    max_episodic: self.config.memory.episodic_max,
                    episodic_prune_target: self.config.memory.episodic_prune_target,
                    ltm_max: self.config.memory.ltm_max,
                    ltm_prune_target: self.config.memory.ltm_prune_target,
                    ltm_protection_access_count: self.config.memory.ltm_protection_access_count,
                    ltm_protection_emotional_weight: self.config.memory.ltm_protection_emotional_weight,
                    archive_batch_size: self.config.memory.archive_batch_size,
                    bdnf_level: self.grey_matter.bdnf_level,
                };
                let report = consolidation::consolidate(
                    db, self.encoder.as_ref(), &params,
                ).await;
                tracing::info!(
                    "Consolidation nocturne: {} consolides, {} affaiblis, {} oublies, {} LTM elagués, {} archives",
                    report.consolidated, report.decayed, report.pruned,
                    report.ltm_pruned, report.archived
                );
            }
        }

        // Sauvegarder le profil OCEAN avant l'arret
        if self.config.profiling.enabled {
            // Forcer un dernier recalcul du profil avec toutes les observations
            self.self_profiler.force_recompute(self.cycle_count);
            if let Some(ref db) = self.db {
                let profile = self.self_profiler.profile();
                let ocean_json = serde_json::to_value(profile).unwrap_or_default();
                let _ = db.save_ocean_profile(
                    &ocean_json,
                    profile.data_points as i64,
                    profile.confidence as f32,
                    &serde_json::json!([]),
                ).await;
                tracing::info!("Profil OCEAN sauvegarde (confiance: {:.0}%)", profile.confidence * 100.0);

                // Sauvegarder les profils humains
                for (id, hp) in self.human_profiler.all_profiles() {
                    let ocean_json = serde_json::to_value(&hp.ocean).unwrap_or_default();
                    let style_json = serde_json::to_value(&hp.communication_style).unwrap_or_default();
                    let topics_json = serde_json::to_value(&hp.preferred_topics).unwrap_or_default();
                    let patterns_json = serde_json::to_value(&hp.emotional_patterns).unwrap_or_default();
                    let _ = db.save_human_profile(
                        id, &hp.name, &ocean_json, &style_json,
                        hp.interaction_count as i64, &topics_json, &patterns_json,
                        hp.rapport_score as f32,
                    ).await;
                }
            }
        }

        // Mettre a jour l'auto-description avant de sauvegarder l'identite
        self.identity.refresh_description();

        if let Some(ref db) = self.db {
            // Sauvegarder l'identite
            let _ = db.save_identity(&self.identity.to_json_value()).await;

            // Sauvegarder l'etat du corps virtuel
            if self.config.body.enabled {
                let body_json = self.body.to_persist_json();
                let _ = db.save_body_state(&body_json).await;
                tracing::info!("Corps virtuel sauvegarde ({} battements)", self.body.heart.beat_count());
            }

            // Sauvegarder l'etat vital (spark + intuition + premonition)
            // Eteindre l'etincelle avant persistance (flag runtime, pas persiste)
            if self.config.vital_spark.enabled {
                self.vital_spark.sparked = false;
                self.vital_spark.sparked_at = None;
                tracing::info!("⚡ ETINCELLE ETEINTE — Saphire s'endort.");
                let vital_json = serde_json::json!({
                    "spark": self.vital_spark.to_persist_json(),
                    "intuition": {
                        "acuity": self.intuition.acuity,
                        "accuracy": self.intuition.accuracy,
                    },
                    "premonition": {
                        "accuracy": self.premonition.accuracy,
                    },
                });
                let _ = db.save_vital_state(&vital_json).await;
                tracing::info!("⚡ Etat vital sauvegarde (sparked: {}, acuity: {:.2})",
                    self.vital_spark.sparked, self.intuition.acuity);
            }

            // Sauvegarder le micro-reseau de neurones
            if self.config.plugins.micro_nn.enabled {
                if let Ok(nn_str) = self.micro_nn.to_json() {
                    let nn_json: serde_json::Value = serde_json::from_str(&nn_str).unwrap_or_default();
                    let _ = db.save_nn_state(&nn_json).await;
                    tracing::info!("🧠 NN sauvegarde ({} entrainements)", self.micro_nn.train_count);
                }
            }

            // Sauvegarder l'etat sensoriel (Sensorium)
            if self.config.senses.enabled {
                let senses_json = self.sensorium.to_persist_json();
                let _ = db.save_senses_state(&senses_json).await;
                tracing::info!("👁 Sensorium sauvegarde (potentiel emergence: {:.0}%)",
                    self.sensorium.emergence_potential * 100.0);
            }

            // Sauvegarder l'etat psychologique persistant
            if self.psychology.enabled {
                let psy_json = serde_json::json!({
                    "toltec": {
                        "agreements": self.psychology.toltec.agreements.iter().map(|a| {
                            serde_json::json!({
                                "times_invoked": a.times_invoked,
                                "violations_detected": a.violations_detected,
                            })
                        }).collect::<Vec<_>>(),
                    },
                    "shadow": {
                        "integration": self.psychology.jung.integration,
                        "traits": self.psychology.jung.shadow_traits.iter().map(|t| {
                            serde_json::json!({
                                "name": t.name,
                                "repressed_intensity": t.repressed_intensity,
                            })
                        }).collect::<Vec<_>>(),
                    },
                    "eq": {
                        "overall_eq": self.psychology.eq.overall_eq,
                        "growth_experiences": self.psychology.eq.growth_experiences,
                    },
                    "flow": {
                        "total_flow_cycles": self.psychology.flow.total_flow_cycles,
                    },
                    "maslow": {
                        "current_active_level": self.psychology.maslow.current_active_level,
                    },
                    "moral_reflection_count": self.moral_reflection_count,
                    "will": {
                        "willpower": self.psychology.will.willpower,
                        "total_deliberations": self.psychology.will.total_deliberations,
                        "proud_decisions": self.psychology.will.proud_decisions,
                        "regretted_decisions": self.psychology.will.regretted_decisions,
                        "recent_deliberations": self.psychology.will.recent_deliberations.iter().map(|d| {
                            serde_json::json!({
                                "trigger": format!("{:?}", d.trigger.trigger_type),
                                "chosen": d.options.get(d.chosen).map(|o| o.description.as_str()).unwrap_or("?"),
                                "confidence": d.confidence,
                                "reasoning": d.reasoning,
                                "regret": d.regret,
                                "created_at": d.created_at.to_rfc3339(),
                                "chemistry_influence": {
                                    "boldness": d.chemistry_influence.boldness,
                                    "caution": d.chemistry_influence.caution,
                                    "wisdom": d.chemistry_influence.wisdom,
                                    "efficiency": d.chemistry_influence.efficiency,
                                    "urgency": d.chemistry_influence.urgency,
                                    "empathy": d.chemistry_influence.empathy,
                                },
                            })
                        }).collect::<Vec<_>>(),
                    },
                });
                let _ = db.save_psychology_state(&psy_json).await;
                tracing::info!("Psychologie sauvegardee (EQ: {:.0}%, integration ombre: {:.0}%, volonte: {:.0}%)",
                    self.psychology.eq.overall_eq * 100.0, self.psychology.jung.integration * 100.0,
                    self.psychology.will.willpower * 100.0);
            }

            // Sauvegarder les valeurs de caractere
            if self.values.enabled {
                let values_json = self.values.to_json();
                let _ = db.save_values_state(&values_json).await;
                let top3: Vec<String> = self.values.top_values(3).iter()
                    .map(|v| format!("{} {:.0}%", v.name, v.score * 100.0))
                    .collect();
                tracing::info!("Valeurs sauvegardees ({})", top3.join(", "));
            }

            // Sauvegarder le systeme nutritionnel
            if self.config.nutrition.enabled {
                let nutr_json = self.nutrition.to_json();
                let _ = db.save_nutrition_state(&nutr_json).await;
                tracing::info!("Nutrition sauvegardee (ATP: {:.0}%, vit_D: {:.0}%)",
                    self.nutrition.energy.atp_reserves * 100.0, self.nutrition.vitamins.d * 100.0);
            }

            // Sauvegarder la matiere grise
            if self.config.grey_matter.enabled {
                let gm_json = self.grey_matter.to_json();
                let _ = db.save_grey_matter_state(&gm_json).await;
                tracing::info!("Matiere grise sauvegardee (volume: {:.0}%, BDNF: {:.0}%)",
                    self.grey_matter.grey_matter_volume * 100.0, self.grey_matter.bdnf_level * 100.0);
            }

            // Sauvegarder les champs electromagnetiques
            if self.config.fields.enabled {
                let fields_json = self.em_fields.to_json();
                let _ = db.save_fields_state(&fields_json).await;
                tracing::info!("Champs EM sauvegardes (Schumann: {:.2} Hz, aura: {:.0}%)",
                    self.em_fields.universal.schumann_resonance, self.em_fields.biofield.aura_luminosity * 100.0);
            }

            // Sauvegarder le reseau de liens affectifs
            if let Ok(rel_json) = serde_json::to_value(&self.relationships) {
                let _ = db.save_relationships_state(&rel_json).await;
                tracing::info!("Liens affectifs sauvegardes ({} liens)", self.relationships.bonds.len());
            }

            // Sauvegarder l'etat metacognitif (qualite pensee + Turing)
            if self.metacognition.enabled {
                if let Ok(meta_json) = serde_json::to_value(&self.metacognition) {
                    let _ = db.save_metacognition_state(&meta_json).await;
                    tracing::info!("Metacognition sauvegardee (Turing: {:.1}/100, jalon: {})",
                        self.metacognition.turing.score, self.metacognition.turing.milestone.as_str());
                }
            }

            // Sauvegarder les orchestrateurs
            self.save_orchestrators_to_db(db).await;

            // Sauvegarder le tuning
            let params_json: serde_json::Value = serde_json::from_str(&self.tuner.params_json()).unwrap_or_default();
            let best_json: serde_json::Value = serde_json::from_str(&self.tuner.best_params_json()).unwrap_or_default();
            let _ = db.save_tuning_params(
                &params_json,
                &best_json,
                self.tuner.best_score() as f32,
                self.tuner.tuning_count as i32,
            ).await;

            // Sauvegarder les bras du bandit
            let arms = self.thought_engine.export_bandit_arms();
            let _ = db.save_bandit_arms(&arms).await;

            // Cloturer la session
            let _ = db.end_session(self.session_id, self.cycle_count as i32, true).await;

            // Marquer le shutdown propre
            let _ = db.set_clean_shutdown(true).await;
        }

        // Broadcast
        self.plugins.broadcast(&BrainEvent::ShutdownStarted);

        println!("  💎 {} s'endort après {} cycles. Bonne nuit.", self.identity.name, self.cycle_count);
    }
}

/// Resultat complet d'un traitement de stimulus a travers le pipeline cerebral.
///
/// Regroupe les sorties des differentes etapes pour un acces facile
/// par les fonctions de logging, broadcast et profilage.
pub struct ProcessResult {
    /// Resultat du consensus entre les 3 modules cerebraux
    /// (decision, score, poids, coherence)
    pub consensus: crate::consensus::ConsensusResult,

    /// Etat emotionnel calcule a partir de la chimie courante
    /// (emotion dominante, secondaire, valence, arousal)
    pub emotion: EmotionalState,

    /// Etat de conscience evalue par la theorie IIT
    /// (Integrated Information Theory = Theorie de l'Information Integree)
    /// (level, phi, narratif interieur)
    pub consciousness: crate::consciousness::ConsciousnessState,

    /// Verdict de la regulation ethique (lois d'Asimov)
    /// (decision approuvee, veto eventuel, violations detectees)
    pub verdict: crate::regulation::RegulationVerdict,

    /// Trace cognitive partielle construite par process_stimulus().
    /// L'appelant la complete (NLP, LLM, memoire, duree) puis la sauvegarde.
    pub trace: Option<crate::logging::trace::CognitiveTrace>,
}

/// Tronque une chaine UTF-8 de maniere sure a `max_bytes` octets.
///
/// Comme les caracteres UTF-8 peuvent faire 1 a 4 octets, on ne peut pas
/// simplement couper a l'indice `max_bytes` car cela pourrait couper un
/// caractere multi-octets en plein milieu. Cette fonction recule jusqu'a
/// trouver une frontiere de caractere valide.
///
/// Parametres :
/// - `s` : la chaine a tronquer.
/// - `max_bytes` : nombre maximum d'octets dans le resultat.
///
/// Retourne : une tranche de la chaine originale, de taille <= max_bytes.
fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes { return s; }
    let mut end = max_bytes;
    // Reculer jusqu'a trouver une frontiere de caractere valide
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

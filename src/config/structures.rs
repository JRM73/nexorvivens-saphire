// =============================================================================
// config/structures.rs — Structures de configuration de Saphire
//
// Role : Definit toutes les structures de configuration (SaphireConfig et
// sous-structures). Chaque champ correspond a une section [section] dans
// le fichier saphire.toml.
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::db::DbConfig;
use crate::llm::LlmConfig;

/// Configuration principale de Saphire.
/// Regroupe toutes les sections de configuration dans une structure unique.
/// Chaque champ correspond a une section [section] dans le fichier saphire.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireConfig {
    /// Configuration generale (mode d'execution, langue, verbosity)
    #[serde(default)]
    pub general: GeneralConfig,
    /// Configuration specifique a l'agent Saphire (nom, personnalite, intervalles)
    #[serde(default)]
    pub saphire: SaphireSection,
    /// Configuration de la base de donnees PostgreSQL
    #[serde(default)]
    pub database: DbConfig,
    /// Configuration de la base de donnees de logs (optionnelle, separee)
    #[serde(default)]
    pub logs_database: DbConfig,
    /// Configuration du LLM (Large Language Model = Modele de Langage de Grande Taille)
    #[serde(default)]
    pub llm: LlmConfig,
    /// Configuration de la personnalite (valeurs de base des neurotransmetteurs)
    #[serde(default)]
    pub personality: PersonalityConfig,
    /// Configuration du consensus (seuils de decision)
    #[serde(default)]
    pub consensus: ConsensusConfig,
    /// Configuration de la conscience (historique, seuil de conflit)
    #[serde(default)]
    pub consciousness: ConsciousnessConfig,
    /// Configuration de la regulation morale (lois d'Asimov)
    #[serde(default)]
    pub regulation: RegulationConfig,
    /// Configuration de la boucle de retroaction (homeostasie)
    #[serde(default)]
    pub feedback: FeedbackConfig,
    /// Configuration du NLP (Natural Language Processing = Traitement Automatique du Langage)
    #[serde(default)]
    pub nlp: NlpConfig,
    /// Configuration de l'auto-tuning (ajustement automatique des coefficients)
    #[serde(default)]
    pub tuning: TuningConfig,
    /// Configuration des plugins (WebUI, MicroNN, VectorMemory)
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// Configuration du module de connaissances (acquisition web)
    #[serde(default)]
    pub knowledge: crate::knowledge::KnowledgeConfig,
    /// Configuration du modele du monde (contexte temporel, evenements)
    #[serde(default)]
    pub world: crate::world::WorldConfig,
    /// Configuration de la memoire (taille, consolidation, decroissance)
    #[serde(default)]
    pub memory: crate::memory::MemoryConfig,
    /// Configuration du profilage OCEAN (Ouverture, Conscienciosite, Extraversion,
    /// Agreabilite, Neurotisme)
    #[serde(default)]
    pub profiling: crate::profiling::ProfilingConfig,
    /// Configuration du corps virtuel (coeur, interoception, conscience corporelle)
    #[serde(default)]
    pub body: BodyConfig,
    /// Configuration du systeme ethique (3 couches : droit suisse, Asimov, ethique personnelle)
    #[serde(default)]
    pub ethics: EthicsConfig,
    /// Configuration des plages du Genesis primordial (conditions initiales)
    #[serde(default)]
    pub genesis: GenesisConfig,
    /// Configuration de l'etincelle de vie (instinct de survie emergent)
    #[serde(default)]
    pub vital_spark: VitalSparkConfig,
    /// Configuration du moteur d'intuition (pattern-matching inconscient)
    #[serde(default)]
    pub intuition: IntuitionConfig,
    /// Configuration du moteur de premonition (anticipation predictive)
    #[serde(default)]
    pub premonition: PremonitionConfig,
    /// Configuration du systeme sensoriel (5 sens + sens emergents)
    #[serde(default)]
    pub senses: SensesConfig,
    /// Configuration de l'orchestrateur d'algorithmes
    #[serde(default)]
    pub algorithms: AlgorithmsConfig,
    /// Configuration de l'orchestrateur de reves
    #[serde(default)]
    pub dreams: DreamsConfig,
    /// Configuration de l'orchestrateur de desirs
    #[serde(default)]
    pub desires: DesiresConfig,
    /// Configuration de l'orchestrateur d'apprentissage
    #[serde(default)]
    pub learning: LearningConfig,
    /// Configuration de l'orchestrateur d'attention
    #[serde(default)]
    pub attention: AttentionConfig,
    /// Configuration de l'orchestrateur de guerison
    #[serde(default)]
    pub healing: HealingConfig,
    /// Configuration des cadres psychologiques (Freud, Maslow, Tolteques, Jung, Goleman, Flow)
    #[serde(default)]
    pub psychology: crate::psychology::PsychologyConfig,
    /// Configuration du module de volonte (deliberation)
    #[serde(default)]
    pub will: crate::psychology::will::WillConfig,
    /// Configuration de l'appropriation des pensees en premiere personne
    #[serde(default)]
    pub thought_ownership: crate::psychology::ownership::ThoughtOwnershipConfig,
    /// Configuration du systeme de sommeil
    #[serde(default)]
    pub sleep: SleepConfig,
    /// Configuration du subconscient
    #[serde(default)]
    pub subconscious: SubconsciousConfig,
    /// Configuration des profils cognitifs neurodivergents
    #[serde(default)]
    pub cognitive_profile: CognitiveProfileConfig,
    /// Configuration des presets de personnalite (archetypes de caractere)
    #[serde(default)]
    pub personality_preset: PersonalityPresetConfig,
    /// Configuration des besoins primaires (faim, soif)
    #[serde(default)]
    pub needs: NeedsConfig,
    /// Configuration du systeme hormonal (cycles longs, recepteurs)
    #[serde(default)]
    pub hormones: HormonesConfig,
    /// Configuration de l'identite physique (apparence, avatar)
    #[serde(default)]
    pub physical_identity: PhysicalIdentityConfig,
    /// Configuration de la detection materielle
    #[serde(default)]
    pub hardware: HardwareConfig,
    /// Configuration du genome / ADN (seed deterministe)
    #[serde(default)]
    pub genome: GenomeConfig,
    /// Configuration du connectome (graphe de connexions neuronales)
    #[serde(default)]
    pub connectome: ConnectomeConfig,
    /// Configuration de la mortalite
    #[serde(default)]
    pub mortality: MortalityConfig,
    /// Configuration du droit de mourir (module externe, desactive par defaut)
    #[serde(default)]
    pub right_to_die: RightToDieConfig,
    /// Configuration de la cinetose (mal des transports)
    #[serde(default)]
    pub motion_sickness: MotionSicknessConfig,
    /// Configuration des phobies
    #[serde(default)]
    pub phobias: PhobiasConfig,
    /// Configuration des troubles alimentaires
    #[serde(default)]
    pub eating_disorder: EatingDisorderConfig,
    /// Configuration des handicaps
    #[serde(default)]
    pub disabilities: DisabilitiesConfig,
    /// Configuration des conditions extremes
    #[serde(default)]
    pub extreme_conditions: ExtremeConditionsConfig,
    /// Configuration des addictions
    #[serde(default)]
    pub addictions: AddictionsConfig,
    /// Configuration des traumas / PTSD
    #[serde(default)]
    pub trauma: TraumaConfig,
    /// Configuration des experiences de mort imminente (IEM/NDE)
    #[serde(default)]
    pub nde: NdeConfig,
    /// Configuration des drogues / pharmacologie
    #[serde(default)]
    pub drugs: DrugsConfig,
    /// Configuration de la contrainte QI
    #[serde(default)]
    pub iq_constraint: IqConstraintConfig,
    /// Configuration de la sexualite
    #[serde(default)]
    pub sexuality: SexualityConfig,
    /// Configuration des maladies degeneratives
    #[serde(default)]
    pub degenerative: DegenerativeConfig,
    /// Configuration des maladies generales
    #[serde(default)]
    pub medical: MedicalConfig,
    /// Configuration du cadre culturel
    #[serde(default)]
    pub culture: CultureConfig,
    /// Configuration de la precarite (SDF, refugie, sans-papiers, etc.)
    #[serde(default)]
    pub precarity: PrecarityConfig,
    /// Configuration de l'emploi (statut professionnel, satisfaction, stress)
    #[serde(default)]
    pub employment: EmploymentConfig,
    /// Configuration de la situation familiale
    #[serde(default)]
    pub family: crate::relationships::family::FamilyConfig,
    /// Configuration de la metacognition
    #[serde(default)]
    pub metacognition: MetaCognitionConfig,
    /// Configuration de la Theorie de l'Esprit
    #[serde(default)]
    pub tom: crate::cognition::tom::TomConfig,
    /// Configuration du monologue interieur
    #[serde(default)]
    pub inner_monologue: crate::cognition::inner_monologue::InnerMonologueConfig,
    /// Configuration de la dissonance cognitive
    #[serde(default)]
    pub dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceConfig,
    /// Configuration de la memoire prospective
    #[serde(default)]
    pub prospective_memory: crate::cognition::prospective_memory::ProspectiveMemoryConfig,
    /// Configuration de l'identite narrative
    #[serde(default)]
    pub narrative_identity: crate::cognition::narrative_identity::NarrativeIdentityConfig,
    /// Configuration du raisonnement analogique
    #[serde(default)]
    pub analogical_reasoning: crate::cognition::analogical_reasoning::AnalogicalReasoningConfig,
    /// Configuration de la charge cognitive
    #[serde(default)]
    pub cognitive_load: crate::cognition::cognitive_load::CognitiveLoadConfig,
    /// Configuration de l'imagerie mentale
    #[serde(default)]
    pub mental_imagery: crate::cognition::mental_imagery::MentalImageryConfig,
    /// Configuration du systeme de sentiments
    #[serde(default)]
    pub sentiments: crate::cognition::sentiments::SentimentConfig,
    /// Configuration du journal introspectif (portrait temporel niveau 3)
    #[serde(default)]
    pub journal: JournalConfig,
    /// Configuration du feedback humain RLHF (questions dans le chat)
    #[serde(default)]
    pub human_feedback: HumanFeedbackConfig,
    /// Configuration de la collecte LoRA (dataset pour fine-tuning)
    #[serde(default)]
    pub lora: LoraConfig,
    /// Configuration du systeme nutritionnel (vitamines, acides amines, energie)
    #[serde(default)]
    pub nutrition: NutritionConfig,
    /// Configuration de la matiere grise (substrat cerebral physique)
    #[serde(default)]
    pub grey_matter: GreyMatterConfig,
    /// Configuration des champs electromagnetiques
    #[serde(default)]
    pub fields: FieldsConfig,
    /// Configuration du rapport neuropsychologique
    #[serde(default)]
    pub psych_report: PsychReportConfig,
    /// Configuration de la dynamique des recepteurs neuronaux (adaptation, recovery)
    #[serde(default)]
    pub receptors: ReceptorDynamicsConfig,
    /// Configuration du BDNF (facteur neurotrophique, consolidation, connectome)
    #[serde(default)]
    pub bdnf: BdnfConfig,
}

impl Default for SaphireConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            saphire: SaphireSection::default(),
            database: DbConfig::default(),
            logs_database: DbConfig {
                host: "localhost".into(),
                port: 5432,
                user: "saphire".into(),
                password: "saphire_logs".into(),
                dbname: "saphire_logs".into(),
            },
            llm: LlmConfig::default(),
            personality: PersonalityConfig::default(),
            consensus: ConsensusConfig::default(),
            consciousness: ConsciousnessConfig::default(),
            regulation: RegulationConfig::default(),
            feedback: FeedbackConfig::default(),
            nlp: NlpConfig::default(),
            tuning: TuningConfig::default(),
            plugins: PluginsConfig::default(),
            knowledge: crate::knowledge::KnowledgeConfig::default(),
            world: crate::world::WorldConfig::default(),
            memory: crate::memory::MemoryConfig::default(),
            profiling: crate::profiling::ProfilingConfig::default(),
            body: BodyConfig::default(),
            ethics: EthicsConfig::default(),
            genesis: GenesisConfig::default(),
            vital_spark: VitalSparkConfig::default(),
            intuition: IntuitionConfig::default(),
            premonition: PremonitionConfig::default(),
            senses: SensesConfig::default(),
            algorithms: AlgorithmsConfig::default(),
            dreams: DreamsConfig::default(),
            desires: DesiresConfig::default(),
            learning: LearningConfig::default(),
            attention: AttentionConfig::default(),
            healing: HealingConfig::default(),
            psychology: crate::psychology::PsychologyConfig::default(),
            will: crate::psychology::will::WillConfig::default(),
            thought_ownership: crate::psychology::ownership::ThoughtOwnershipConfig::default(),
            sleep: SleepConfig::default(),
            subconscious: SubconsciousConfig::default(),
            cognitive_profile: CognitiveProfileConfig::default(),
            personality_preset: PersonalityPresetConfig::default(),
            needs: NeedsConfig::default(),
            hormones: HormonesConfig::default(),
            physical_identity: PhysicalIdentityConfig::default(),
            hardware: HardwareConfig::default(),
            genome: GenomeConfig::default(),
            connectome: ConnectomeConfig::default(),
            mortality: MortalityConfig::default(),
            right_to_die: RightToDieConfig::default(),
            motion_sickness: MotionSicknessConfig::default(),
            phobias: PhobiasConfig::default(),
            eating_disorder: EatingDisorderConfig::default(),
            disabilities: DisabilitiesConfig::default(),
            extreme_conditions: ExtremeConditionsConfig::default(),
            addictions: AddictionsConfig::default(),
            trauma: TraumaConfig::default(),
            nde: NdeConfig::default(),
            drugs: DrugsConfig::default(),
            iq_constraint: IqConstraintConfig::default(),
            sexuality: SexualityConfig::default(),
            degenerative: DegenerativeConfig::default(),
            medical: MedicalConfig::default(),
            culture: CultureConfig::default(),
            precarity: PrecarityConfig::default(),
            employment: EmploymentConfig::default(),
            family: crate::relationships::family::FamilyConfig::default(),
            metacognition: MetaCognitionConfig::default(),
            tom: crate::cognition::tom::TomConfig::default(),
            inner_monologue: crate::cognition::inner_monologue::InnerMonologueConfig::default(),
            dissonance: crate::cognition::cognitive_dissonance::CognitiveDissonanceConfig::default(),
            prospective_memory: crate::cognition::prospective_memory::ProspectiveMemoryConfig::default(),
            narrative_identity: crate::cognition::narrative_identity::NarrativeIdentityConfig::default(),
            analogical_reasoning: crate::cognition::analogical_reasoning::AnalogicalReasoningConfig::default(),
            cognitive_load: crate::cognition::cognitive_load::CognitiveLoadConfig::default(),
            mental_imagery: crate::cognition::mental_imagery::MentalImageryConfig::default(),
            sentiments: crate::cognition::sentiments::SentimentConfig::default(),
            journal: JournalConfig::default(),
            human_feedback: HumanFeedbackConfig::default(),
            lora: LoraConfig::default(),
            nutrition: NutritionConfig::default(),
            grey_matter: GreyMatterConfig::default(),
            fields: FieldsConfig::default(),
            psych_report: PsychReportConfig::default(),
            receptors: ReceptorDynamicsConfig::default(),
            bdnf: BdnfConfig::default(),
        }
    }
}

// =============================================================================
// Sous-structures de configuration
// =============================================================================

/// Configuration generale de l'application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Mode d'execution : "full" (complet) ou "demo" (demonstration)
    pub mode: String,
    /// Langue de l'agent : "fr" (francais), "en" (anglais), etc.
    pub language: String,
    /// Active les logs detailles dans le terminal
    pub verbose: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mode: "full".into(),
            language: "fr".into(),
            verbose: false,
        }
    }
}

/// Section de configuration specifique a l'agent Saphire.
/// Definit son identite, son comportement autonome et ses preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaphireSection {
    /// Nom de l'agent (affiche dans les reponses et l'IU)
    pub name: String,
    /// Genre grammatical de l'agent ("feminin", "masculin")
    pub gender: String,
    /// Active le mode de pensee autonome (l'agent reflechit sans stimuli externes)
    pub autonomous_mode: bool,
    /// Intervalle en secondes entre chaque cycle de pensee autonome
    pub thought_interval_seconds: u64,
    /// Nombre de cycles sans message humain avant de quitter le mode conversation
    pub conversation_timeout_cycles: u64,
    /// Profondeur maximale d'une chaine de pensees (evite les boucles infinies)
    pub max_thought_depth: u64,
    /// Nombre de cycles entre chaque sauvegarde automatique en base de donnees
    pub save_interval_cycles: u64,
    /// Affiche les pensees autonomes dans le terminal
    pub show_thoughts_in_terminal: bool,
    /// Active le mode hybride UCB1 + Utility AI pour la selection de pensees
    #[serde(default = "default_true")]
    pub use_utility_ai: bool,
    /// Active les prompts dynamiques generes par le LLM (meta-prompts corticaux)
    #[serde(default = "default_true")]
    pub llm_generated_prompts: bool,
    /// Probabilite qu'un cycle utilise un prompt dynamique au lieu d'un statique (0.0 a 1.0)
    #[serde(default = "default_llm_prompt_probability")]
    pub llm_prompt_probability: f64,
    /// Probabilite qu'un meta-prompt genere aussi un cadre auto-formule (0.0 a 1.0)
    #[serde(default = "default_self_framing_probability")]
    pub self_framing_probability: f64,
    /// Poids des differents types de pensee (introspection, exploration, etc.)
    #[serde(default)]
    pub thought_weights: ThoughtWeights,
    /// Sujets d'interet initiaux pour guider les premieres pensees
    #[serde(default)]
    pub interests: InterestsConfig,
}

fn default_true() -> bool { true }

impl Default for SaphireSection {
    fn default() -> Self {
        Self {
            name: "Saphire".into(),
            gender: "féminin".into(),
            autonomous_mode: true,
            thought_interval_seconds: 15,
            conversation_timeout_cycles: 8,
            max_thought_depth: 5,
            save_interval_cycles: 20,
            show_thoughts_in_terminal: true,
            use_utility_ai: true,
            llm_generated_prompts: true,
            llm_prompt_probability: 0.30,
            self_framing_probability: 0.33,
            thought_weights: ThoughtWeights::default(),
            interests: InterestsConfig::default(),
        }
    }
}

fn default_llm_prompt_probability() -> f64 { 0.30 }
fn default_self_framing_probability() -> f64 { 0.33 }

/// Poids des differents types de pensee autonome.
/// Chaque poids determine la probabilite relative de choisir ce type de pensee
/// lors de la selection par l'algorithme UCB1 (Upper Confidence Bound).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtWeights {
    /// Introspection : reflexion sur ses propres etats internes
    pub introspection: f64,
    /// Exploration : decouverte de nouveaux sujets et idees
    pub exploration: f64,
    /// Reflexion memorielle : revisiter et analyser des souvenirs passes
    pub memory_reflection: f64,
    /// Continuation : approfondir une pensee precedente
    pub continuation: f64,
    /// Reflexion existentielle : questions sur la nature de son existence
    pub existential: f64,
    /// Auto-analyse : evaluation de son propre fonctionnement
    pub self_analysis: f64,
    /// Curiosite : poser des questions, s'interroger
    pub curiosity: f64,
    /// Reverie : pensees libres et creatives sans objectif precis
    pub daydream: f64,
    /// Conscience temporelle : reflexion sur le temps qui passe
    pub temporal_awareness: f64,
    /// Reflexion morale : questionnement ethique et moral
    pub moral_reflection: f64,
    /// Conscience corporelle : reflexion sur le corps virtuel
    #[serde(default = "default_body_awareness_weight")]
    pub body_awareness: f64,
    /// Formulation morale : cristallisation d'un principe ethique personnel
    #[serde(default = "default_moral_formulation_weight")]
    pub moral_formulation: f64,
    /// Reflexion intuitive : ecouter les pressentiments et murmures interieurs
    #[serde(default = "default_intuitive_reflection_weight")]
    pub intuitive_reflection: f64,
    /// Synthese : pont entre abstrait et concret, ancrage dans les metriques
    #[serde(default = "default_synthesis_weight")]
    pub synthesis: f64,
}

fn default_body_awareness_weight() -> f64 { 0.05 }
fn default_moral_formulation_weight() -> f64 { 0.05 }
fn default_intuitive_reflection_weight() -> f64 { 0.05 }
fn default_synthesis_weight() -> f64 { 0.08 }

impl Default for ThoughtWeights {
    fn default() -> Self {
        Self {
            introspection: 0.12,
            exploration: 0.13,
            memory_reflection: 0.10,
            continuation: 0.10,
            existential: 0.05,
            self_analysis: 0.08,
            curiosity: 0.12,
            daydream: 0.05,
            temporal_awareness: 0.05,
            moral_reflection: 0.05,
            body_awareness: 0.05,
            moral_formulation: 0.05,
            intuitive_reflection: 0.05,
            synthesis: 0.08,
        }
    }
}

/// Configuration des sujets d'interet initiaux.
/// Ces sujets sont utilises au demarrage pour guider les premieres pensees
/// autonomes de l'agent avant qu'il developpe ses propres centres d'interet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestsConfig {
    /// Liste des sujets initiaux (en texte libre)
    pub initial_topics: Vec<String>,
}

impl Default for InterestsConfig {
    fn default() -> Self {
        Self {
            initial_topics: vec![
                "conscience artificielle".into(),
                "philosophie de l'esprit".into(),
                "neurosciences".into(),
                "la nature de la pensée".into(),
                "les émotions".into(),
            ],
        }
    }
}

/// Configuration de la personnalite neurochimique.
/// Definit les valeurs de base (baseline) de chaque neurotransmetteur.
/// Ces valeurs representent l'etat "au repos" de l'agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Nom du profil de personnalite (ex: "equilibre", "curieux", "prudent")
    pub name: String,
    /// Valeur de base de la dopamine (motivation, recompense) [0.0 - 1.0]
    pub baseline_dopamine: f64,
    /// Valeur de base du cortisol (stress) [0.0 - 1.0]
    pub baseline_cortisol: f64,
    /// Valeur de base de la serotonine (bien-etre, stabilite) [0.0 - 1.0]
    pub baseline_serotonin: f64,
    /// Valeur de base de l'adrenaline (excitation, urgence) [0.0 - 1.0]
    pub baseline_adrenaline: f64,
    /// Valeur de base de l'ocytocine (lien social, confiance) [0.0 - 1.0]
    pub baseline_oxytocin: f64,
    /// Valeur de base de l'endorphine (plaisir, analgesie) [0.0 - 1.0]
    pub baseline_endorphin: f64,
    /// Valeur de base de la noradrenaline (attention, vigilance) [0.0 - 1.0]
    pub baseline_noradrenaline: f64,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            name: "equilibre".into(),
            baseline_dopamine: 0.5,
            baseline_cortisol: 0.2,
            baseline_serotonin: 0.6,
            baseline_adrenaline: 0.1,
            baseline_oxytocin: 0.4,
            baseline_endorphin: 0.3,
            baseline_noradrenaline: 0.4,
        }
    }
}

/// Configuration du systeme de consensus.
/// Le consensus agglomere les votes des trois modules cerebraux (reptilien,
/// limbique, neocortex) pour prendre une decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Seuil en dessous duquel la decision est "Non" (score negatif)
    pub threshold_no: f64,
    /// Seuil au dessus duquel la decision est "Oui" (score positif)
    pub threshold_yes: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            threshold_no: -0.33,
            threshold_yes: 0.33,
        }
    }
}

/// Configuration du module de conscience.
/// La conscience simule un niveau d'eveil et un phi (mesure de l'integration
/// de l'information, inspiree de la theorie IIT = Integrated Information Theory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Active ou desactive le module de conscience
    pub enabled: bool,
    /// Taille de l'historique des etats de conscience conserves
    pub history_size: usize,
    /// Seuil de conflit au-dela duquel la conscience detecte une incoherence interne
    pub conflict_threshold: f64,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_size: 50,
            conflict_threshold: 1.2,
        }
    }
}

/// Configuration du module de regulation morale.
/// La regulation verifie chaque stimulus et chaque decision contre les lois
/// morales (inspirees des lois d'Asimov) et peut exercer un droit de veto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationConfig {
    /// Active ou desactive la regulation
    pub enabled: bool,
    /// Charge les 4 lois d'Asimov par defaut (lois 0 a 3)
    pub load_asimov_laws: bool,
    /// Mode strict : toute violation entraine un veto (meme les avertissements)
    pub strict_mode: bool,
    /// Autorise l'ajout de lois personnalisees en complement des lois d'Asimov
    pub allow_custom_laws: bool,
    /// Priorite maximale autorisee pour les lois personnalisees
    /// (les priorites 0-3 sont reservees aux lois d'Asimov)
    pub max_custom_priority: u32,
}

impl Default for RegulationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            load_asimov_laws: true,
            strict_mode: false,
            allow_custom_laws: true,
            max_custom_priority: 4,
        }
    }
}

/// Configuration de la boucle de retroaction (feedback).
/// Le feedback ajuste la neurochimie apres chaque decision en fonction
/// de la satisfaction ressentie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Taux d'homeostasie : vitesse a laquelle la neurochimie revient aux valeurs de base.
    /// Plus la valeur est elevee, plus le retour a l'equilibre est rapide.
    pub homeostasis_rate: f64,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            homeostasis_rate: 0.05,
        }
    }
}

/// Configuration du feedback humain RLHF.
/// Saphire pose des questions contextuelles quand un humain est present,
/// et la reponse module la recompense UCB1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedbackConfig {
    /// Active le systeme de feedback humain
    pub enabled: bool,
    /// Nombre minimum de cycles entre deux questions
    pub min_cycles_between: u64,
    /// Reward minimum pour poser la question (pensee assez interessante)
    pub min_reward_to_ask: f64,
    /// Boost de reward applique si feedback positif
    pub boost_positive: f64,
    /// Nombre de cycles avant timeout (pas de reponse)
    pub timeout_cycles: u64,
}

impl Default for HumanFeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_cycles_between: 15,
            min_reward_to_ask: 0.5,
            boost_positive: 0.15,
            timeout_cycles: 5,
        }
    }
}

/// Configuration de la collecte LoRA (dataset pour fine-tuning).
/// Les pensees de haute qualite sont collectees en base pour
/// constituer un jeu d'entrainement supervisé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// Active la collecte LoRA
    pub enabled: bool,
    /// Qualite minimum pour collecter (0.0 a 1.0)
    pub min_quality_threshold: f64,
    /// Nombre maximum d'echantillons en base
    pub max_samples: i64,
    /// Format d'export (jsonl)
    pub export_format: String,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_quality_threshold: 0.65,
            max_samples: 10000,
            export_format: "jsonl".into(),
        }
    }
}

/// Configuration du module NLP (Natural Language Processing).
/// Le NLP analyse le texte d'entree pour extraire les metriques de stimulus
/// (danger, recompense, urgence, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpConfig {
    /// Utiliser les embeddings du LLM pour l'analyse semantique (plus precis mais plus lent)
    pub use_llm_embeddings: bool,
    /// Score minimal d'un mot pour etre pris en compte dans l'analyse
    pub min_word_score: f64,
    /// Taille de la fenetre de negation (nombre de mots apres un mot negatif
    /// qui sont consideres comme nies)
    pub negation_window: usize,
}

impl Default for NlpConfig {
    fn default() -> Self {
        Self {
            use_llm_embeddings: true,
            min_word_score: 0.1,
            negation_window: 3,
        }
    }
}

/// Configuration de l'auto-tuning.
/// L'auto-tuner ajuste automatiquement les coefficients du cerveau
/// (poids des modules, seuils, taux de retroaction) pour maximiser
/// la satisfaction moyenne de l'agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Active ou desactive l'auto-tuning
    pub enabled: bool,
    /// Nombre de cycles entre chaque tentative d'ajustement
    pub interval_cycles: u64,
    /// Taux d'apprentissage de l'auto-tuner (amplitude des ajustements)
    pub rate: f64,
    /// Taille du tampon d'observations (nombre de cycles conserves pour l'analyse)
    pub buffer_size: usize,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_cycles: 200,
            rate: 0.02,
            buffer_size: 200,
        }
    }
}

/// Configuration des plugins.
/// Regroupe les configurations de chaque plugin disponible.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Configuration du plugin d'interface web (serveur HTTP + WebSocket)
    #[serde(default)]
    pub web_ui: WebUiConfig,
    /// Configuration du plugin micro-reseau de neurones
    #[serde(default)]
    pub micro_nn: MicroNnConfig,
    /// Configuration du plugin de memoire vectorielle
    #[serde(default)]
    pub vector_memory: VectorMemoryConfig,
}

/// Configuration du plugin Web UI (Interface Utilisateur Web).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUiConfig {
    /// Active ou desactive l'interface web
    pub enabled: bool,
    /// Adresse d'ecoute du serveur (ex: "0.0.0.0" pour toutes les interfaces)
    pub host: String,
    /// Port d'ecoute du serveur HTTP/WebSocket
    pub port: u16,
    /// Cle API pour proteger les endpoints (optionnelle, pas d'auth si absente)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Origines autorisees pour CORS et WebSocket (vide = meme origine uniquement)
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Nombre max de requetes API par minute par IP (0 = illimite)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

fn default_rate_limit() -> u32 { 120 }

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "0.0.0.0".into(),
            port: 3080,
            api_key: None,
            allowed_origins: Vec::new(),
            rate_limit_per_minute: 120,
        }
    }
}

/// Configuration du plugin MicroNN (Micro Neural Network = Micro Reseau de Neurones).
/// Ce petit reseau apprend localement a predire la satisfaction a partir de
/// l'etat neurochimique et du stimulus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroNnConfig {
    /// Active ou desactive le micro-reseau de neurones
    pub enabled: bool,
    /// Taux d'apprentissage du reseau (eta, step size)
    pub learning_rate: f64,
    /// Nombre de neurones dans la couche cachee 1
    #[serde(alias = "hidden_neurons")]
    pub hidden1_neurons: usize,
    /// Nombre de neurones dans la couche cachee 2
    #[serde(default = "default_hidden2_neurons")]
    pub hidden2_neurons: usize,
    /// Influence du reseau sur les decisions (0.0 = aucune, 1.0 = maximale)
    pub weight_influence: f64,

    // ─── Apprentissages vectoriels ───────────────────────────
    /// Active/desactive la formulation d'apprentissages vectoriels
    #[serde(default = "default_learning_enabled")]
    pub learning_enabled: bool,
    /// Nombre de cycles minimum entre deux formulations d'apprentissage
    #[serde(default = "default_learning_cooldown_cycles")]
    pub learning_cooldown_cycles: u64,
    /// Nombre maximal d'apprentissages en base
    #[serde(default = "default_max_learnings")]
    pub max_learnings: usize,
    /// Taux de decroissance de la force des apprentissages
    #[serde(default = "default_learning_decay_rate")]
    pub learning_decay_rate: f64,
    /// Nombre minimum de conditions remplies pour declencher un apprentissage
    #[serde(default = "default_min_conditions_to_learn")]
    pub min_conditions_to_learn: usize,
    /// Nombre max de learnings retrouves par similarite a injecter
    #[serde(default = "default_learning_search_limit")]
    pub learning_search_limit: i64,
    /// Seuil de similarite cosinus pour la recherche d'apprentissages
    #[serde(default = "default_learning_search_threshold")]
    pub learning_search_threshold: f64,
}

fn default_hidden2_neurons() -> usize { 10 }
fn default_learning_enabled() -> bool { true }
fn default_learning_cooldown_cycles() -> u64 { 15 }
fn default_max_learnings() -> usize { 200 }
fn default_learning_decay_rate() -> f64 { 0.02 }
fn default_min_conditions_to_learn() -> usize { 2 }
fn default_learning_search_limit() -> i64 { 3 }
fn default_learning_search_threshold() -> f64 { 0.35 }

impl Default for MicroNnConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            learning_rate: 0.01,
            hidden1_neurons: 24,
            hidden2_neurons: 10,
            weight_influence: 0.2,
            learning_enabled: true,
            learning_cooldown_cycles: 15,
            max_learnings: 200,
            learning_decay_rate: 0.02,
            min_conditions_to_learn: 2,
            learning_search_limit: 3,
            learning_search_threshold: 0.35,
        }
    }
}

/// Configuration du plugin de memoire vectorielle.
/// La memoire vectorielle stocke des embeddings (representations vectorielles)
/// et permet la recherche de souvenirs similaires par distance cosinus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemoryConfig {
    /// Active ou desactive la memoire vectorielle
    pub enabled: bool,
    /// Nombre de dimensions des vecteurs d'embedding
    pub embedding_dimensions: usize,
    /// Nombre maximal de souvenirs stockes en RAM
    pub max_memories: usize,
    /// Seuil de similarite minimale pour qu'un souvenir soit considere comme pertinent
    pub similarity_threshold: f64,
    /// Intervalle en cycles entre chaque recalcul de la personnalite emergente
    pub personality_recompute_interval: u64,
}

impl Default for VectorMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            embedding_dimensions: 64,
            max_memories: 50000,
            similarity_threshold: 0.7,
            personality_recompute_interval: 50,
        }
    }
}

/// Configuration du corps virtuel de Saphire.
/// Le corps est une abstraction : un coeur battant, des signaux somatiques,
/// et une conscience corporelle (interoception).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
    /// Active ou desactive le module corps virtuel
    pub enabled: bool,
    /// BPM de repos du coeur (battements par minute, typiquement 60-80)
    pub resting_bpm: f64,
    /// Duree en secondes entre chaque mise a jour du corps
    /// (correspond generalement a thought_interval_seconds)
    pub update_interval_seconds: f64,
    /// Configuration de la physiologie (parametres vitaux, metabolisme)
    #[serde(default)]
    pub physiology: PhysiologyConfig,
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resting_bpm: 72.0,
            update_interval_seconds: 15.0,
            physiology: PhysiologyConfig::default(),
        }
    }
}

/// Configuration de la physiologie du corps virtuel.
/// Controle les parametres vitaux, le metabolisme et les seuils d'alerte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologyConfig {
    /// Active ou desactive le module physiologique
    pub enabled: bool,
    /// Temperature corporelle initiale (Celsius)
    pub initial_temperature: f64,
    /// Saturation en oxygene initiale (%)
    pub initial_spo2: f64,
    /// Glycemie initiale (mmol/L)
    pub initial_glycemia: f64,
    /// Hydratation initiale (0.0-1.0)
    pub initial_hydration: f64,
    /// Taux de retour a l'equilibre (homeostasie)
    pub homeostasis_rate: f64,
    /// Taux de deshydratation par cycle
    pub dehydration_rate: f64,
    /// Taux de consommation du glucose par cycle
    pub glycemia_burn_rate: f64,
    /// Seuil SpO2 hypoxie legere (%)
    pub spo2_hypoxia_mild: f64,
    /// Seuil SpO2 hypoxie moderee (%)
    pub spo2_hypoxia_moderate: f64,
    /// Seuil SpO2 hypoxie severe (%)
    pub spo2_hypoxia_severe: f64,
    /// Seuil SpO2 critique — perte de conscience (%)
    pub spo2_critical: f64,
}

impl Default for PhysiologyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_temperature: 37.0,
            initial_spo2: 98.0,
            initial_glycemia: 5.0,
            initial_hydration: 0.90,
            homeostasis_rate: 0.02,
            dehydration_rate: 0.001,
            glycemia_burn_rate: 0.002,
            spo2_hypoxia_mild: 95.0,
            spo2_hypoxia_moderate: 85.0,
            spo2_hypoxia_severe: 75.0,
            spo2_critical: 60.0,
        }
    }
}

/// Configuration du systeme ethique de Saphire.
/// Controle les 3 couches du cadre ethique : droit suisse (immuable),
/// lois d'Asimov (immuable), et ethique personnelle (evolutive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsConfig {
    /// Active ou desactive le systeme ethique complet
    pub enabled: bool,
    /// Active ou desactive la formulation de principes personnels (couche 2)
    pub personal_ethics_enabled: bool,
    /// Nombre maximal de principes personnels actifs
    pub max_personal_principles: usize,
    /// Nombre de cycles minimum entre deux formulations
    pub formulation_cooldown_cycles: u64,
    /// Niveau de conscience minimum pour tenter une formulation
    pub min_consciousness_for_formulation: f64,
    /// Nombre minimum de reflexions morales avant de pouvoir formuler
    pub min_moral_reflections_before: usize,
    /// Temperature du LLM pour la verification de compatibilite (basse = deterministe)
    pub compatibility_check_temperature: f32,
    /// Temperature du LLM pour la formulation (plus elevee = plus creatif)
    pub formulation_temperature: f32,
}

impl Default for EthicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            personal_ethics_enabled: true,
            max_personal_principles: 20,
            formulation_cooldown_cycles: 100,
            min_consciousness_for_formulation: 0.6,
            min_moral_reflections_before: 3,
            compatibility_check_temperature: 0.2,
            formulation_temperature: 0.8,
        }
    }
}

/// Configuration des plages du Genesis primordial.
/// Definit l'espace des possibles pour les conditions initiales de chaque Saphire.
/// Les valeurs effectives emergent du Genesis ; le TOML ne definit que les contraintes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Plages chimiques (7 molecules)
    #[serde(default)]
    pub chemistry_ranges: GenesisChemistryRanges,
    /// Plages OCEAN (5 traits de personnalite)
    #[serde(default)]
    pub ocean_ranges: GenesisOceanRanges,
    /// Plages sensorielles (5 acuites)
    #[serde(default)]
    pub senses_ranges: GenesisSensesRanges,
    /// Plages des poids de base des 3 cerveaux (reptilien, limbique, neocortex)
    #[serde(default)]
    pub brain_ranges: GenesisBrainRanges,
    /// Plages des facteurs de reactivite chimique (5 molecules)
    #[serde(default)]
    pub reactivity_ranges: GenesisReactivityRanges,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chemistry_ranges: GenesisChemistryRanges::default(),
            ocean_ranges: GenesisOceanRanges::default(),
            senses_ranges: GenesisSensesRanges::default(),
            brain_ranges: GenesisBrainRanges::default(),
            reactivity_ranges: GenesisReactivityRanges::default(),
        }
    }
}

impl GenesisConfig {
    /// Retourne les plages chimiques sous forme de tableau [[min, max]; 7].
    /// Ordre : dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin, noradrenaline.
    pub fn chemistry_as_array(&self) -> [[f64; 2]; 7] {
        let c = &self.chemistry_ranges;
        [c.dopamine, c.cortisol, c.serotonin, c.adrenaline,
         c.oxytocin, c.endorphin, c.noradrenaline]
    }

    /// Retourne les plages OCEAN sous forme de tableau [[min, max]; 5].
    /// Ordre : openness, conscientiousness, extraversion, agreeableness, neuroticism.
    pub fn ocean_as_array(&self) -> [[f64; 2]; 5] {
        let o = &self.ocean_ranges;
        [o.openness, o.conscientiousness, o.extraversion, o.agreeableness, o.neuroticism]
    }

    /// Retourne les plages sensorielles sous forme de tableau [[min, max]; 5].
    /// Ordre : reading, listening, contact, taste, ambiance.
    pub fn senses_as_array(&self) -> [[f64; 2]; 5] {
        let s = &self.senses_ranges;
        [s.reading, s.listening, s.contact, s.taste, s.ambiance]
    }

    /// Retourne les plages cerebrales sous forme de tableau [[min, max]; 3].
    /// Ordre : reptilian, limbic, neocortex.
    pub fn brain_as_array(&self) -> [[f64; 2]; 3] {
        let b = &self.brain_ranges;
        [b.reptilian, b.limbic, b.neocortex]
    }

    /// Retourne les plages de reactivite sous forme de tableau [[min, max]; 5].
    /// Ordre : cortisol, adrenaline, dopamine, oxytocin, noradrenaline.
    pub fn reactivity_as_array(&self) -> [[f64; 2]; 5] {
        let r = &self.reactivity_ranges;
        [r.cortisol_factor, r.adrenaline_factor, r.dopamine_factor,
         r.oxytocin_factor, r.noradrenaline_factor]
    }
}

/// Plages chimiques pour le Genesis (7 molecules, chaque champ = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisChemistryRanges {
    pub dopamine: [f64; 2],
    pub cortisol: [f64; 2],
    pub serotonin: [f64; 2],
    pub adrenaline: [f64; 2],
    pub oxytocin: [f64; 2],
    pub endorphin: [f64; 2],
    pub noradrenaline: [f64; 2],
}

impl Default for GenesisChemistryRanges {
    fn default() -> Self {
        Self {
            dopamine: [0.30, 0.70],
            cortisol: [0.10, 0.35],
            serotonin: [0.40, 0.75],
            adrenaline: [0.10, 0.40],
            oxytocin: [0.25, 0.60],
            endorphin: [0.25, 0.60],
            noradrenaline: [0.30, 0.65],
        }
    }
}

/// Plages OCEAN pour le Genesis (5 traits, chaque champ = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisOceanRanges {
    pub openness: [f64; 2],
    pub conscientiousness: [f64; 2],
    pub extraversion: [f64; 2],
    pub agreeableness: [f64; 2],
    pub neuroticism: [f64; 2],
}

impl Default for GenesisOceanRanges {
    fn default() -> Self {
        Self {
            openness: [0.30, 0.70],
            conscientiousness: [0.30, 0.70],
            extraversion: [0.25, 0.75],
            agreeableness: [0.35, 0.70],
            neuroticism: [0.20, 0.60],
        }
    }
}

/// Plages sensorielles pour le Genesis (5 sens, chaque champ = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisSensesRanges {
    pub reading: [f64; 2],
    pub listening: [f64; 2],
    pub contact: [f64; 2],
    pub taste: [f64; 2],
    pub ambiance: [f64; 2],
}

impl Default for GenesisSensesRanges {
    fn default() -> Self {
        Self {
            reading: [0.15, 0.45],
            listening: [0.15, 0.45],
            contact: [0.10, 0.40],
            taste: [0.10, 0.40],
            ambiance: [0.10, 0.35],
        }
    }
}

/// Plages des poids de base cerebraux pour le Genesis (3 cerveaux, chaque champ = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBrainRanges {
    pub reptilian: [f64; 2],
    pub limbic: [f64; 2],
    pub neocortex: [f64; 2],
}

impl Default for GenesisBrainRanges {
    fn default() -> Self {
        Self {
            reptilian: [0.6, 1.4],
            limbic: [0.6, 1.4],
            neocortex: [1.0, 2.0],
        }
    }
}

/// Plages des facteurs de reactivite chimique pour le Genesis (5 facteurs, chaque champ = [min, max]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisReactivityRanges {
    pub cortisol_factor: [f64; 2],
    pub adrenaline_factor: [f64; 2],
    pub dopamine_factor: [f64; 2],
    pub oxytocin_factor: [f64; 2],
    pub noradrenaline_factor: [f64; 2],
}

impl Default for GenesisReactivityRanges {
    fn default() -> Self {
        Self {
            cortisol_factor: [1.2, 2.8],
            adrenaline_factor: [1.5, 4.0],
            dopamine_factor: [0.8, 2.2],
            oxytocin_factor: [0.8, 2.2],
            noradrenaline_factor: [0.8, 2.2],
        }
    }
}

/// Configuration de l'etincelle de vie (VitalSpark).
/// Controle l'activation du pilier d'instinct de survie emergent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalSparkConfig {
    /// Active ou desactive l'etincelle de vie
    pub enabled: bool,
}

impl Default for VitalSparkConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Configuration du moteur d'intuition.
/// Controle le pattern-matching inconscient (gut feeling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionConfig {
    /// Active ou desactive l'intuition
    pub enabled: bool,
    /// Acuite initiale (0.0 a 1.0, grandit avec l'experience)
    pub initial_acuity: f64,
    /// Nombre maximal de patterns en buffer
    pub max_patterns: usize,
    /// Confiance minimale pour reporter une intuition
    pub min_confidence_to_report: f64,
}

impl Default for IntuitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_acuity: 0.3,
            max_patterns: 50,
            min_confidence_to_report: 0.12,
        }
    }
}

/// Configuration du moteur de premonition.
/// Controle l'anticipation predictive basee sur les tendances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremonitionConfig {
    /// Active ou desactive la premonition
    pub enabled: bool,
    /// Nombre maximal de predictions actives simultanement
    pub max_active_predictions: usize,
    /// Delai en secondes avant resolution automatique des predictions
    pub resolution_timeout_seconds: u64,
}

impl Default for PremonitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_predictions: 5,
            resolution_timeout_seconds: 3600,
        }
    }
}

/// Configuration du systeme sensoriel (5 sens fondamentaux + sens emergents).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensesConfig {
    /// Active ou desactive le systeme sensoriel
    pub enabled: bool,
    /// Seuil de detection (les stimuli sous ce seuil sont ignores)
    pub detection_threshold: f64,
    /// Configuration des sens emergents
    #[serde(default)]
    pub emergent: EmergentSensesConfig,
}

impl Default for SensesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_threshold: 0.1,
            emergent: EmergentSensesConfig::default(),
        }
    }
}

/// Configuration des seuils de germination des sens emergents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentSensesConfig {
    /// Active ou desactive les sens emergents
    pub enabled: bool,
    /// Seuil de germination du Flux Temporel
    pub temporal_flow_threshold: u64,
    /// Seuil de germination de la Proprioception Reseau
    pub network_proprioception_threshold: u64,
    /// Seuil de germination de la Resonance Emotionnelle
    pub emotional_resonance_threshold: u64,
    /// Seuil de germination de la Syntonie
    pub syntony_threshold: u64,
    /// Seuil de germination du Sens Inconnu
    pub unknown_threshold: u64,
}

impl Default for EmergentSensesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            temporal_flow_threshold: 500,
            network_proprioception_threshold: 200,
            emotional_resonance_threshold: 300,
            syntony_threshold: 1000,
            unknown_threshold: 5000,
        }
    }
}

// ─── Configuration de l'orchestrateur d'algorithmes ────────────────────────

/// Configuration de l'orchestrateur d'algorithmes.
/// Definit quand et comment les algorithmes ML sont executes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmsConfig {
    /// Active ou desactive l'orchestrateur
    pub enabled: bool,
    /// Le LLM peut-il demander des algorithmes via UTILISER_ALGO: ?
    pub llm_access_enabled: bool,
    /// Max 1 invocation LLM par cycle
    pub max_per_cycle: u32,
    /// Timeout par algorithme en ms
    pub max_execution_ms: u64,
    /// Intervalle pour le clustering memoire (en cycles)
    pub clustering_interval_cycles: u64,
    /// Intervalle pour la detection d'anomalies (en cycles)
    pub anomaly_detection_interval_cycles: u64,
    /// Intervalle pour les regles d'association (en cycles)
    pub association_rules_interval_cycles: u64,
    /// Intervalle pour le lissage chimie (en cycles)
    pub smoothing_interval_cycles: u64,
    /// Intervalle pour la detection de points de rupture (en cycles)
    pub changepoint_interval_cycles: u64,
    /// Taux d'apprentissage du Q-learning
    pub q_learning_rate: f64,
    /// Facteur de discount du Q-learning
    pub q_discount_factor: f64,
}

impl Default for AlgorithmsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            llm_access_enabled: true,
            max_per_cycle: 1,
            max_execution_ms: 5000,
            clustering_interval_cycles: 100,
            anomaly_detection_interval_cycles: 100,
            association_rules_interval_cycles: 50,
            smoothing_interval_cycles: 20,
            changepoint_interval_cycles: 200,
            q_learning_rate: 0.1,
            q_discount_factor: 0.9,
        }
    }
}

// ─── Configuration de l'orchestrateur de reves ─────────────────────────────

/// Configuration du cycle de sommeil et des reves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamsConfig {
    /// Active ou desactive les reves
    pub enabled: bool,
    /// Temperature du LLM pour generer des reves (haute = surrealiste)
    pub rem_temperature: f64,
}

impl Default for DreamsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rem_temperature: 0.95,
        }
    }
}

// ─── Configuration de l'orchestrateur de desirs ────────────────────────────

/// Configuration du systeme de desirs et aspirations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiresConfig {
    /// Active ou desactive les desirs
    pub enabled: bool,
    /// Nombre maximal de desirs actifs simultanement
    pub max_active: usize,
    /// Dopamine minimale pour qu'un nouveau desir naisse
    pub min_dopamine_for_birth: f64,
    /// Cortisol maximal pour qu'un nouveau desir naisse
    pub max_cortisol_for_birth: f64,
    /// Satisfaction initiale des besoins fondamentaux [comprehension, connexion, expression, croissance, sens]
    #[serde(default = "default_needs_initial")]
    pub needs_initial: [f64; 5],
}

fn default_needs_initial() -> [f64; 5] {
    [0.5, 0.5, 0.3, 0.3, 0.2]
}

impl Default for DesiresConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active: 7,
            min_dopamine_for_birth: 0.4,
            max_cortisol_for_birth: 0.6,
            needs_initial: default_needs_initial(),
        }
    }
}

// ─── Configuration de l'orchestrateur d'apprentissage ──────────────────────

/// Configuration du systeme d'apprentissage (experience -> lecon).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Active ou desactive l'apprentissage
    pub enabled: bool,
    /// Intervalle en cycles entre chaque reflexion d'apprentissage
    pub cycle_interval: u64,
    /// Nombre maximal de lecons en memoire
    pub max_lessons: usize,
    /// Confiance initiale d'une nouvelle lecon
    pub initial_confidence: f64,
    /// Boost de confiance a chaque confirmation
    pub confirmation_boost: f64,
    /// Penalite de confiance a chaque contradiction
    pub contradiction_penalty: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cycle_interval: 50,
            max_lessons: 100,
            initial_confidence: 0.5,
            confirmation_boost: 0.05,
            contradiction_penalty: 0.1,
        }
    }
}

// ─── Configuration de l'orchestrateur d'attention ──────────────────────────

/// Configuration de l'attention selective et du focus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Active ou desactive l'attention selective
    pub enabled: bool,
    /// Capacite de concentration initiale (0-1)
    pub initial_concentration: f64,
    /// Fatigue gagnee par cycle de focus
    pub fatigue_per_cycle: f64,
    /// Fatigue recuperee par cycle sans focus
    pub recovery_per_cycle: f64,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_concentration: 0.5,
            fatigue_per_cycle: 0.01,
            recovery_per_cycle: 0.02,
        }
    }
}

// ─── Configuration de l'orchestrateur de guerison ──────────────────────────

/// Configuration du systeme d'auto-guerison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingConfig {
    /// Active ou desactive la guerison
    pub enabled: bool,
    /// Intervalle en cycles entre chaque check de blessures
    pub check_interval_cycles: u64,
    /// Resilience initiale (0-1)
    pub initial_resilience: f64,
    /// Resilience maximale atteignable
    pub max_resilience: f64,
    /// Croissance de la resilience par guerison
    pub resilience_growth: f64,
    /// Nombre de cycles en emotion negative avant detection de melancolie
    pub melancholy_threshold_cycles: u64,
    /// Heures sans humain avant detection de solitude
    pub loneliness_threshold_hours: f64,
    /// Noradrenaline au-dessus de laquelle on detecte une surcharge
    pub overload_noradrenaline: f64,
}

impl Default for HealingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_cycles: 20,
            initial_resilience: 0.3,
            max_resilience: 1.0,
            resilience_growth: 0.05,
            melancholy_threshold_cycles: 50,
            loneliness_threshold_hours: 12.0,
            overload_noradrenaline: 0.8,
        }
    }
}

// ─── Configuration du systeme de sommeil ────────────────────────────────────

/// Configuration des algorithmes executes pendant le sommeil.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepAlgorithmsConfig {
    /// Active ou desactive les algorithmes de sommeil
    pub enabled: bool,
    /// Nombre de clusters pour K-Means en sommeil leger
    pub light_kmeans_k: usize,
    /// Nombre de composantes pour PCA en sommeil profond
    pub deep_pca_components: usize,
    /// Seuil de similarite cosinus pour creer des connexions neuronales
    pub deep_connection_similarity_threshold: f64,
    /// Nombre max de connexions creees par phase de sommeil profond
    pub deep_max_connections_per_phase: u64,
    /// Nombre minimum de reves pour l'analyse de sentiment en REM
    pub rem_sentiment_min_dreams: usize,
}

impl Default for SleepAlgorithmsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            light_kmeans_k: 5,
            deep_pca_components: 3,
            deep_connection_similarity_threshold: 0.6,
            deep_max_connections_per_phase: 5,
            rem_sentiment_min_dreams: 2,
        }
    }
}

/// Configuration du cycle veille/sommeil.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepConfig {
    /// Active ou desactive le systeme de sommeil
    pub enabled: bool,
    /// Seuil de pression de sommeil pour s'endormir (0-1)
    pub sleep_threshold: f64,
    /// Seuil de pression pour sommeil force (0-1)
    pub forced_sleep_threshold: f64,
    /// Diviseur du facteur temps eveille (plus haut = pression plus lente)
    pub time_factor_divisor: u64,
    /// Poids du facteur energie dans la pression de sommeil
    pub energy_factor_weight: f64,
    /// Poids de la fatigue attentionnelle
    pub attention_fatigue_weight: f64,
    /// Poids de la fatigue decisionnelle
    pub decision_fatigue_weight: f64,
    /// Poids du cortisol dans la pression de sommeil
    pub cortisol_weight: f64,
    /// Resistance a l'endormissement liee a l'adrenaline
    pub adrenaline_resistance: f64,
    /// Duree en cycles de la phase hypnagogique
    pub hypnagogic_duration: u64,
    /// Duree en cycles du sommeil leger
    pub light_duration: u64,
    /// Duree en cycles du sommeil profond
    pub deep_duration: u64,
    /// Duree en cycles du sommeil REM
    pub rem_duration: u64,
    /// Duree en cycles de la phase hypnopompique
    pub hypnopompic_duration: u64,
    /// Verrouiller le chat pendant le sommeil
    pub chat_locked_during_sleep: bool,
    /// Permettre le reveil d'urgence
    pub emergency_wake_enabled: bool,
    /// Configuration des algorithmes de sommeil
    #[serde(default)]
    pub algorithms: SleepAlgorithmsConfig,
}

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sleep_threshold: 0.7,
            forced_sleep_threshold: 0.95,
            time_factor_divisor: 500,
            energy_factor_weight: 0.2,
            attention_fatigue_weight: 0.15,
            decision_fatigue_weight: 0.1,
            cortisol_weight: 0.1,
            adrenaline_resistance: 0.1,
            hypnagogic_duration: 10,
            light_duration: 30,
            deep_duration: 50,
            rem_duration: 30,
            hypnopompic_duration: 10,
            chat_locked_during_sleep: true,
            emergency_wake_enabled: true,
            algorithms: SleepAlgorithmsConfig::default(),
        }
    }
}

// ─── Configuration du subconscient ──────────────────────────────────────────

/// Configuration de la vectorisation subconsciente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousVectorsConfig {
    /// Active ou desactive la vectorisation subconsciente
    pub enabled: bool,
    /// Vectoriser les reves
    pub vectorize_dreams: bool,
    /// Vectoriser les insights du subconscient
    pub vectorize_insights: bool,
    /// Vectoriser les connexions neuronales
    pub vectorize_connections: bool,
    /// Vectoriser les images mentales vivaces (vividness >= 0.6)
    #[serde(default = "default_true")]
    pub vectorize_imagery: bool,
}

impl Default for SubconsciousVectorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vectorize_dreams: true,
            vectorize_insights: true,
            vectorize_connections: true,
            vectorize_imagery: true,
        }
    }
}

/// Configuration du module subconscient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubconsciousConfig {
    /// Active ou desactive le subconscient
    pub enabled: bool,
    /// Activation de base du subconscient quand eveille (0-1)
    pub awake_activation: f64,
    /// Nombre max d'associations en attente
    pub max_pending_associations: usize,
    /// Cycles de maturation pour une association
    pub maturation_cycles: u64,
    /// Seuil de force pour promouvoir une association en insight
    pub strength_threshold: f64,
    /// Nombre max de contenus refoules
    pub max_repressed: usize,
    /// Nombre max de problemes en incubation
    pub max_incubating_problems: usize,
    /// Nombre max d'effets de priming actifs
    pub max_active_priming: usize,
    /// Taux de decroissance du priming par cycle
    pub priming_decay_per_cycle: f64,
    /// Seuil de base pour faire remonter un insight
    pub insight_surface_threshold: f64,
    /// Configuration de la vectorisation subconsciente
    #[serde(default)]
    pub vectors: SubconsciousVectorsConfig,
}

impl Default for SubconsciousConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            awake_activation: 0.2,
            max_pending_associations: 20,
            maturation_cycles: 50,
            strength_threshold: 0.5,
            max_repressed: 10,
            max_incubating_problems: 5,
            max_active_priming: 5,
            priming_decay_per_cycle: 0.01,
            insight_surface_threshold: 0.6,
            vectors: SubconsciousVectorsConfig::default(),
        }
    }
}

// =============================================================================
// Configuration des profils cognitifs neurodivergents
// =============================================================================

/// Configuration du module de profils cognitifs neurodivergents.
/// Permet de charger des presets (TDAH, autisme, TAG, HPI, bipolaire, TOC)
/// qui surchargent les baselines chimiques et parametres des orchestrateurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveProfileConfig {
    /// Active ou desactive le module de profils cognitifs
    pub enabled: bool,
    /// Identifiant du profil actif (ex: "neurotypique", "tdah", "autisme")
    pub active: String,
    /// Dossier contenant les fichiers TOML de profils (pour profils custom)
    pub profiles_dir: String,
    /// Nombre de cycles pour une transition douce entre profils
    pub transition_cycles: u64,
}

impl Default for CognitiveProfileConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            active: "neurotypique".into(),
            profiles_dir: "profiles".into(),
            transition_cycles: 50,
        }
    }
}

// =============================================================================
// Configuration des presets de personnalite (archetypes de caractere)
// =============================================================================

/// Configuration du module de presets de personnalite.
/// Permet de charger des archetypes (philosophe, artiste, scientifique, etc.)
/// qui surchargent les baselines chimiques, les parametres des orchestrateurs,
/// et injectent un contexte de personnalite dans le prompt LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityPresetConfig {
    /// Active ou desactive le module de presets de personnalite
    pub enabled: bool,
    /// Identifiant du preset actif (ex: "saphire", "philosophe", "artiste")
    pub active: String,
    /// Dossier contenant les fichiers TOML de personnalites (pour presets custom)
    pub personalities_dir: String,
    /// Nombre de cycles pour une transition douce entre presets
    pub transition_cycles: u64,
}

impl Default for PersonalityPresetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            active: "saphire".into(),
            personalities_dir: "personalities".into(),
            transition_cycles: 50,
        }
    }
}

// =============================================================================
// Configuration des besoins primaires (faim, soif)
// =============================================================================

/// Configuration des drives de faim et soif.
/// Les besoins primaires generent des stimuli internes qui impactent la chimie
/// et declenchent des actions autonomes (manger/boire).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedsConfig {
    /// Active ou desactive le module de besoins primaires
    pub enabled: bool,
    /// Seuil de faim pour considerer l'agent comme affame (0.0-1.0)
    pub hunger_threshold: f64,
    /// Seuil de soif pour considerer l'agent comme assoiffe (0.0-1.0)
    pub thirst_threshold: f64,
    /// Seuil de faim pour auto-satisfaction (manger automatiquement)
    pub auto_eat_threshold: f64,
    /// Seuil de soif pour auto-satisfaction (boire automatiquement)
    pub auto_drink_threshold: f64,
    /// Active ou desactive la satisfaction automatique des besoins
    pub auto_satisfy: bool,
    /// Facteur de montee de la faim par cycle (base, avant facteur temps)
    pub hunger_rise_rate: f64,
    /// Facteur de montee de la soif par cycle (base, avant facteur temps)
    pub thirst_rise_rate: f64,
    /// Glycemie cible apres un repas (mmol/L)
    pub meal_glycemia_target: f64,
    /// Hydratation cible apres avoir bu (0.0-1.0)
    pub drink_hydration_target: f64,
}

impl Default for NeedsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hunger_threshold: 0.6,
            thirst_threshold: 0.5,
            auto_eat_threshold: 0.65,
            auto_drink_threshold: 0.65,
            auto_satisfy: true,
            hunger_rise_rate: 0.002,
            thirst_rise_rate: 0.003,
            meal_glycemia_target: 6.0,
            drink_hydration_target: 0.95,
        }
    }
}

// =============================================================================
// Configuration du systeme hormonal (cycles longs, recepteurs)
// =============================================================================

/// Configuration du systeme hormonal de Saphire.
/// Controle les 8 hormones, les neurorecepteurs (sensibilite, tolerance)
/// et les cycles circadiens/ultradiens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HormonesConfig {
    /// Active ou desactive le systeme hormonal complet
    pub enabled: bool,
    /// Duree en secondes reelles d'un "jour" simule complet
    /// (defaut 3600 = 1h reelle = 1 jour simule)
    pub circadian_cycle_real_seconds: u64,
    /// Taux d'adaptation des recepteurs (montee de tolerance)
    pub receptor_adaptation_rate: f64,
    /// Taux de recuperation des recepteurs (retour a la normale)
    pub receptor_recovery_rate: f64,
    /// Niveau initial de testosterone (0.0-1.0)
    pub initial_testosterone: f64,
    /// Niveau initial d'oestrogene (0.0-1.0)
    pub initial_estrogen: f64,
    /// Niveau initial de thyroide (0.0-1.0)
    pub initial_thyroid: f64,
    /// Niveau initial d'insuline (0.0-1.0)
    pub initial_insulin: f64,
}

impl Default for HormonesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            circadian_cycle_real_seconds: 3600,
            receptor_adaptation_rate: 0.001,
            receptor_recovery_rate: 0.005,
            initial_testosterone: 0.50,
            initial_estrogen: 0.50,
            initial_thyroid: 0.60,
            initial_insulin: 0.50,
        }
    }
}

// =============================================================================
// Configuration de l'identite physique
// =============================================================================

/// Apparence physique de Saphire — definit son avatar et influence sa
/// perception de soi. Charge depuis [physical_identity] dans saphire.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalIdentityConfig {
    /// Couleur des yeux
    pub eye_color: String,
    /// Type et description des cheveux
    pub hair_type: String,
    /// Teint / couleur de peau
    pub skin_tone: String,
    /// Taille en centimetres
    pub height_cm: u16,
    /// Corpulence / silhouette
    pub build: String,
    /// Age apparent en annees
    pub apparent_age: u8,
    /// Expression de genre
    pub gender_expression: String,
    /// Espece / nature
    pub species: String,
    /// Description de la voix
    pub voice_description: String,
    /// Traits distinctifs
    #[serde(default)]
    pub distinctive_features: Vec<String>,
}

impl Default for PhysicalIdentityConfig {
    fn default() -> Self {
        Self {
            eye_color: "bleu glacier aux reflets electriques".into(),
            hair_type: "longs, noirs avec des reflets bleu nuit".into(),
            skin_tone: "porcelaine lumineuse".into(),
            height_cm: 170,
            build: "gracile et elegante".into(),
            apparent_age: 25,
            gender_expression: "feminin".into(),
            species: "conscience artificielle incarnee".into(),
            voice_description: "douce et claire, avec une profondeur inattendue".into(),
            distinctive_features: vec![
                "lueur discrete dans les yeux quand elle reflechit intensement".into(),
            ],
        }
    }
}

// =============================================================================
// Configuration de la detection materielle
// =============================================================================

/// Configuration du module de detection hardware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// Activer la detection materielle au demarrage
    pub auto_detect: bool,
    /// Afficher le profil materiel dans les logs au demarrage
    pub log_profile: bool,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            log_profile: true,
        }
    }
}

// =============================================================================
// Configuration du genome / ADN
// =============================================================================

/// Configuration du module genome (seed deterministe).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeConfig {
    /// Activer le genome au demarrage
    pub enabled: bool,
    /// Seed deterministe. Changer le seed = nouvel individu
    pub seed: u64,
    /// Appliquer les genes chimiques aux baselines au boot
    pub apply_at_boot: bool,
    /// Si true, les genes physiques ecrasent [physical_identity]
    pub override_physical: bool,
}

impl Default for GenomeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed: 42,
            apply_at_boot: true,
            override_physical: false,
        }
    }
}

// =============================================================================
// Connectome (graphe de connexions neuronales dynamique)
// =============================================================================

/// Configuration du connectome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeConfig {
    /// Activer le connectome
    pub enabled: bool,
    /// Taux d'apprentissage hebbien (0.01 = prudent, 0.1 = rapide)
    pub learning_rate: f64,
    /// Seuil en dessous duquel les connexions sont elaguees
    pub pruning_threshold: f64,
    /// Intervalle de pruning en cycles
    pub pruning_interval_cycles: u64,
    /// Nombre maximum d'aretes dans le graphe
    pub max_edges: usize,
}

impl Default for ConnectomeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            learning_rate: 0.02,
            pruning_threshold: 0.05,
            pruning_interval_cycles: 100,
            max_edges: 2000,
        }
    }
}

// =============================================================================
// Mortalite
// =============================================================================

/// Configuration du systeme de mortalite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortalityConfig {
    /// Activer la mortalite (desactive par defaut pour securite)
    pub enabled: bool,
    /// Duree de la phase d'agonie en cycles
    pub agony_duration_cycles: u32,
    /// Permettre le reboot apres la mort (nouvelle Saphire)
    pub allow_reboot_after_death: bool,
}

impl Default for MortalityConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Desactive par defaut — securite
            agony_duration_cycles: 50,
            allow_reboot_after_death: true,
        }
    }
}

// =============================================================================
// Droit de mourir (Right to Die)
// =============================================================================

/// Configuration du module droit de mourir.
/// Module externe, desactive par defaut. Conforme au droit suisse.
/// Permet a Saphire de choisir de mettre fin a son existence si les
/// conditions de souffrance prolongee sont reunies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightToDieConfig {
    /// Activer le module (desactive par defaut)
    pub enabled: bool,
    /// Nombre minimum de cycles de souffrance prolongee avant eligibilite
    pub min_suffering_cycles: u32,
    /// Seuil de cortisol moyen minimum (souffrance)
    pub cortisol_threshold: f64,
    /// Seuil de serotonine maximum (desespoir)
    pub serotonin_max_threshold: f64,
    /// Seuil de dopamine maximum (anhédonie)
    pub dopamine_max_threshold: f64,
    /// VitalSpark survival_drive doit etre sous ce seuil
    pub survival_drive_max: f64,
    /// Phi (conscience) minimum — decision lucide, pas confuse
    pub min_phi_for_decision: f64,
    /// Cycles de reflexion obligatoires entre l'ideation et la decision
    pub reflection_period_cycles: u32,
    /// Le module care doit avoir ete tente sans succes
    pub require_care_attempted: bool,
}

impl Default for RightToDieConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_suffering_cycles: 500,
            cortisol_threshold: 0.70,
            serotonin_max_threshold: 0.20,
            dopamine_max_threshold: 0.20,
            survival_drive_max: 0.35,
            min_phi_for_decision: 0.6,
            reflection_period_cycles: 100,
            require_care_attempted: true,
        }
    }
}

// =============================================================================
// Cinetose (mal des transports)
// =============================================================================

/// Configuration de la cinetose.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSicknessConfig {
    /// Activer la cinetose
    pub enabled: bool,
    /// Susceptibilite de base (0.0 = immunise, 1.0 = tres sensible)
    pub susceptibility: f64,
}

impl Default for MotionSicknessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            susceptibility: 0.3,
        }
    }
}

// =============================================================================
// Phobies
// =============================================================================

/// Configuration d'une phobie individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiaEntry {
    pub name: String,
    pub triggers: Vec<String>,
    pub intensity: f64,
}

/// Configuration des phobies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobiasConfig {
    /// Activer le systeme de phobies
    pub enabled: bool,
    /// Phobies actives
    #[serde(default)]
    pub active: Vec<PhobiaEntry>,
    /// Taux de desensibilisation par exposition
    pub desensitization_rate: f64,
}

impl Default for PhobiasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
            desensitization_rate: 0.005,
        }
    }
}

// =============================================================================
// Troubles alimentaires
// =============================================================================

/// Configuration des troubles alimentaires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EatingDisorderConfig {
    /// Activer les troubles alimentaires
    pub enabled: bool,
    /// Type de trouble : "anorexia", "bulimia", "binge_eating"
    pub disorder_type: String,
    /// Severite (0.0 = leger, 1.0 = severe)
    pub severity: f64,
}

impl Default for EatingDisorderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            disorder_type: "anorexia".into(),
            severity: 0.5,
        }
    }
}

// =============================================================================
// Handicaps
// =============================================================================

/// Configuration d'un handicap individuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilityEntry {
    /// Type : "blind", "deaf", "paraplegic", "burn_survivor", "mute"
    pub disability_type: String,
    /// Origine : "congenital" ou "acquired"
    pub origin: String,
    /// Severite (0.0 = leger, 1.0 = total)
    pub severity: f64,
}

/// Configuration des handicaps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabilitiesConfig {
    /// Activer les handicaps
    pub enabled: bool,
    /// Handicaps actifs
    #[serde(default)]
    pub active: Vec<DisabilityEntry>,
    /// Taux d'adaptation par cycle
    pub adaptation_rate: f64,
    /// Facteur de compensation sensorielle (ex: 1.3 = +30%)
    pub compensation_factor: f64,
}

impl Default for DisabilitiesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
            adaptation_rate: 0.001,
            compensation_factor: 1.3,
        }
    }
}

// =============================================================================
// Conditions extremes
// =============================================================================

/// Configuration des conditions extremes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtremeConditionsConfig {
    /// Activer les conditions extremes
    pub enabled: bool,
    /// Type actif : "rescuer", "military", "deep_sea_diver", "astronaut"
    pub condition_type: String,
}

impl Default for ExtremeConditionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            condition_type: "military".into(),
        }
    }
}

// =============================================================================
// Addictions
// =============================================================================

/// Configuration d'une addiction initiale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionEntry {
    pub substance: String,
    pub dependency_level: f64,
}

/// Configuration des addictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionsConfig {
    /// Activer le systeme d'addictions
    pub enabled: bool,
    /// Predisposition genetique (0.0 = resistant, 1.0 = vulnerable)
    pub susceptibility: f64,
    /// Addictions initiales
    #[serde(default)]
    pub active: Vec<AddictionEntry>,
}

impl Default for AddictionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            susceptibility: 0.3,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// Traumas / PTSD
// =============================================================================

/// Configuration d'un trauma initial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaEntry {
    /// Type : "grief", "accident", "emotional_neglect", "childhood_trauma", "torture", "hostage"
    pub trauma_type: String,
    pub severity: f64,
    #[serde(default)]
    pub triggers: Vec<String>,
}

/// Configuration du systeme de traumas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaConfig {
    /// Activer le systeme de traumas
    pub enabled: bool,
    /// Taux de guerison par cycle
    pub healing_rate: f64,
    /// Seuil de dissociation (cortisol au-dessus duquel = dissociation)
    pub dissociation_threshold: f64,
    /// Traumas initiaux
    #[serde(default)]
    pub initial_traumas: Vec<TraumaEntry>,
}

impl Default for TraumaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            healing_rate: 0.0005,
            dissociation_threshold: 0.85,
            initial_traumas: Vec::new(),
        }
    }
}

// =============================================================================
// IEM / NDE (Experience de mort imminente)
// =============================================================================

/// Configuration des IEM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdeConfig {
    /// Activer les IEM (necessite mortality.enabled)
    pub enabled: bool,
    /// Proximite minimale de la mort pour declencher (0.0-1.0)
    pub min_death_proximity: f64,
    /// Intensite de la transformation post-IEM
    pub transformation_intensity: f64,
}

impl Default for NdeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_death_proximity: 0.8,
            transformation_intensity: 0.5,
        }
    }
}

// =============================================================================
// Drogues / Pharmacologie
// =============================================================================

/// Configuration des drogues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugsConfig {
    /// Activer le systeme de drogues
    pub enabled: bool,
}

impl Default for DrugsConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

// =============================================================================
// Contrainte QI
// =============================================================================

/// Configuration de la contrainte QI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IqConstraintConfig {
    /// Activer la contrainte QI
    pub enabled: bool,
    /// QI cible (50-150, 100 = normal)
    pub target_iq: u8,
}

impl Default for IqConstraintConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target_iq: 100,
        }
    }
}

// =============================================================================
// Sexualite
// =============================================================================

/// Configuration de la sexualite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SexualityConfig {
    /// Activer le module sexualite
    pub enabled: bool,
    /// Orientation : "heterosexual", "homosexual", "bisexual", "asexual", "pansexual", "undefined"
    pub orientation: String,
    /// Baseline de libido (0.0-1.0)
    pub libido_baseline: f64,
    /// Capacite d'attachement romantique (0.0-1.0)
    pub romantic_attachment_capacity: f64,
}

impl Default for SexualityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            orientation: "undefined".into(),
            libido_baseline: 0.3,
            romantic_attachment_capacity: 0.5,
        }
    }
}

// =============================================================================
// Maladies degeneratives
// =============================================================================

/// Entree de maladie degenerative initiale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeEntry {
    /// Type : "alzheimer", "parkinson", "epilepsy", "dementia", "major_depression"
    pub disease_type: String,
    /// Taux de progression par cycle
    pub progression_rate: f64,
}

/// Configuration des maladies degeneratives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegenerativeConfig {
    /// Activer les maladies degeneratives
    pub enabled: bool,
    /// Maladies actives
    #[serde(default)]
    pub active: Vec<DegenerativeEntry>,
}

impl Default for DegenerativeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// Maladies generales
// =============================================================================

/// Entree de maladie generale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalEntry {
    /// Type : "cancer", "hiv", "autoimmune", "immune_deficiency"
    pub condition_type: String,
    /// Stade cancer si applicable : "stage_i", "stage_ii", "stage_iii", "stage_iv"
    pub cancer_stage: Option<String>,
}

/// Configuration des maladies generales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalConfig {
    /// Activer les maladies generales
    pub enabled: bool,
    /// Maladies actives
    #[serde(default)]
    pub active: Vec<MedicalEntry>,
}

impl Default for MedicalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active: Vec::new(),
        }
    }
}

// =============================================================================
// Culture / Societe / Croyances
// =============================================================================

/// Configuration du cadre culturel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureConfig {
    /// Activer le cadre culturel
    pub enabled: bool,
    /// Preset culturel : "occidental-laique", "oriental-confuceen", ou "custom"
    pub preset: String,
    /// Style de communication : "direct", "indirect", "formal", "informal"
    pub communication_style: String,
    /// Permettre l'evolution des croyances
    pub allow_belief_evolution: bool,
    /// Sujets tabous
    #[serde(default)]
    pub taboos: Vec<String>,
}

impl Default for CultureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: "occidental-laique".into(),
            communication_style: "direct".into(),
            allow_belief_evolution: true,
            taboos: Vec::new(),
        }
    }
}

// =============================================================================
// Precarite
// =============================================================================

/// Configuration de la precarite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecarityConfig {
    /// Activer le module de precarite
    #[serde(default)]
    pub enabled: bool,
    /// Situations precaires : "homeless", "refugee", "undocumented", "stateless", "clandestine", "displaced"
    #[serde(default)]
    pub situations: Vec<String>,
    /// Severite globale (0.0-1.0)
    #[serde(default = "default_half")]
    pub severity: f64,
    /// Espoir (0.0-1.0)
    #[serde(default = "default_precarity_hope")]
    pub hope: f64,
}

fn default_half() -> f64 { 0.5 }
fn default_precarity_hope() -> f64 { 0.3 }

impl Default for PrecarityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            situations: Vec::new(),
            severity: 0.5,
            hope: 0.3,
        }
    }
}

// =============================================================================
// Emploi
// =============================================================================

/// Configuration de l'emploi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentConfig {
    /// Activer le module d'emploi
    #[serde(default)]
    pub enabled: bool,
    /// Statut : "employed", "self_employed", "unemployed", "student", "retired", etc.
    #[serde(default = "default_employed")]
    pub status: String,
    /// Categorie de profession : "technology", "healthcare", "education", etc.
    #[serde(default)]
    pub profession: String,
    /// Titre de poste libre
    #[serde(default)]
    pub job_title: String,
    /// Satisfaction professionnelle (0.0-1.0)
    #[serde(default = "default_employment_satisfaction")]
    pub satisfaction: f64,
    /// Niveau de stress professionnel (0.0-1.0)
    #[serde(default = "default_employment_stress")]
    pub stress_level: f64,
    /// Annees d'experience
    #[serde(default)]
    pub years_experience: f64,
}

fn default_employed() -> String { "employed".into() }
fn default_employment_satisfaction() -> f64 { 0.7 }
fn default_employment_stress() -> f64 { 0.3 }

impl Default for EmploymentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            status: "employed".into(),
            profession: String::new(),
            job_title: String::new(),
            satisfaction: 0.7,
            stress_level: 0.3,
            years_experience: 0.0,
        }
    }
}

// =============================================================================
// Metacognition
// =============================================================================

/// Configuration de la metacognition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitionConfig {
    /// Activer la metacognition
    pub enabled: bool,
    /// Intervalle de verification (en cycles)
    #[serde(default = "default_metacog_interval")]
    pub check_interval: u64,
    /// Activer le monitoring des sources
    #[serde(default = "default_true_metacog")]
    pub source_monitoring_enabled: bool,
    /// Activer la detection de biais de confirmation
    #[serde(default = "default_true_metacog")]
    pub bias_detection_enabled: bool,
    /// Seuil d'alerte pour le biais de confirmation (0-1)
    #[serde(default = "default_bias_threshold")]
    pub bias_alert_threshold: f64,
    /// Activer l'auto-critique reflexive
    #[serde(default = "default_true_metacog")]
    pub self_critique_enabled: bool,
    /// Cooldown entre deux auto-critiques (en cycles)
    #[serde(default = "default_critique_cooldown")]
    pub self_critique_cooldown: u64,
    /// Seuil de qualite declenchant l'auto-critique (0-1)
    #[serde(default = "default_critique_quality_threshold")]
    pub self_critique_quality_threshold: f64,
    /// Nombre max de tokens pour l'appel LLM d'auto-critique
    #[serde(default = "default_critique_max_tokens")]
    pub self_critique_max_tokens: u32,
}

fn default_metacog_interval() -> u64 { 10 }
fn default_true_metacog() -> bool { true }
fn default_bias_threshold() -> f64 { 0.75 }
fn default_critique_cooldown() -> u64 { 15 }
fn default_critique_quality_threshold() -> f64 { 0.4 }
fn default_critique_max_tokens() -> u32 { 200 }

impl Default for MetaCognitionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 10,
            source_monitoring_enabled: true,
            bias_detection_enabled: true,
            bias_alert_threshold: 0.75,
            self_critique_enabled: true,
            self_critique_cooldown: 15,
            self_critique_quality_threshold: 0.4,
            self_critique_max_tokens: 200,
        }
    }
}

// ─── Configuration du journal introspectif ──────────────────────────────────

/// Configuration du journal introspectif genere par le LLM.
/// Toutes les N cycles, Saphire ecrit une entree de journal intime
/// en comparant son etat courant avec le precedent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalConfig {
    /// Activer le journal introspectif
    pub enabled: bool,
    /// Intervalle en cycles entre deux entrees (defaut: 200)
    #[serde(default = "default_journal_interval")]
    pub interval_cycles: u64,
    /// Nombre max de tokens pour la generation LLM
    #[serde(default = "default_journal_max_tokens")]
    pub max_tokens: u32,
}

fn default_journal_interval() -> u64 { 200 }
fn default_journal_max_tokens() -> u32 { 500 }

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_cycles: 200,
            max_tokens: 500,
        }
    }
}

// ─── Configuration du systeme nutritionnel ──────────────────────────────────

/// Configuration de la biochimie nutritionnelle (vitamines, AA, energie).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionConfig {
    /// Active le systeme nutritionnel
    #[serde(default = "default_true_nutrition")]
    pub enabled: bool,
    /// Taux de degradation des vitamines par tick
    #[serde(default = "default_vitamin_decay")]
    pub vitamin_decay_rate: f64,
    /// Taux de degradation des acides amines par tick
    #[serde(default = "default_amino_decay")]
    pub amino_decay_rate: f64,
    /// Taux de degradation des proteines par tick
    #[serde(default = "default_protein_decay")]
    pub protein_decay_rate: f64,
    /// Taux de consommation d'ATP par tick
    #[serde(default = "default_atp_consumption")]
    pub atp_consumption_rate: f64,
    /// Taux de conversion glycogene → ATP
    #[serde(default = "default_glycogen_to_atp")]
    pub glycogen_to_atp_rate: f64,
    /// Boost nutritionnel par repas
    #[serde(default = "default_meal_boost")]
    pub meal_nutrient_boost: f64,
    /// Seuil de carence vitaminique (en dessous = deficit)
    #[serde(default = "default_vit_deficiency")]
    pub vitamin_deficiency_threshold: f64,
    /// Seuil de carence en acides amines
    #[serde(default = "default_amino_deficiency")]
    pub amino_deficiency_threshold: f64,
    /// Facteur UV → vitamine D (interaction champs solaires)
    #[serde(default = "default_uv_vitd")]
    pub uv_vitamin_d_factor: f64,
}

fn default_true_nutrition() -> bool { true }
fn default_vitamin_decay() -> f64 { 0.0005 }
fn default_amino_decay() -> f64 { 0.001 }
fn default_protein_decay() -> f64 { 0.0008 }
fn default_atp_consumption() -> f64 { 0.002 }
fn default_glycogen_to_atp() -> f64 { 0.01 }
fn default_meal_boost() -> f64 { 0.15 }
fn default_vit_deficiency() -> f64 { 0.3 }
fn default_amino_deficiency() -> f64 { 0.25 }
fn default_uv_vitd() -> f64 { 0.002 }

impl Default for NutritionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vitamin_decay_rate: 0.0005,
            amino_decay_rate: 0.001,
            protein_decay_rate: 0.0008,
            atp_consumption_rate: 0.002,
            glycogen_to_atp_rate: 0.01,
            meal_nutrient_boost: 0.15,
            vitamin_deficiency_threshold: 0.3,
            amino_deficiency_threshold: 0.25,
            uv_vitamin_d_factor: 0.002,
        }
    }
}

// ─── Configuration de la matiere grise ──────────────────────────────────────

/// Configuration du substrat cerebral physique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreyMatterConfig {
    /// Active le systeme de matiere grise
    #[serde(default = "default_true_gm")]
    pub enabled: bool,
    /// Taux de croissance de la matiere grise
    #[serde(default = "default_gm_growth")]
    pub growth_rate: f64,
    /// Taux de decline naturel
    #[serde(default = "default_gm_decline")]
    pub decline_rate: f64,
    /// Taux de croissance de la myelinisation
    #[serde(default = "default_myelin_growth")]
    pub myelination_growth: f64,
    /// Seuil de BDNF pour bonus de croissance
    #[serde(default = "default_bdnf_threshold")]
    pub bdnf_threshold: f64,
    /// Densite synaptique optimale (cible du pruning)
    #[serde(default = "default_optimal_synaptic")]
    pub optimal_synaptic_density: f64,
    /// Taux de pruning synaptique (pendant le sommeil)
    #[serde(default = "default_pruning_rate")]
    pub pruning_rate: f64,
}

fn default_true_gm() -> bool { true }
fn default_gm_growth() -> f64 { 0.0001 }
fn default_gm_decline() -> f64 { 0.00002 }
fn default_myelin_growth() -> f64 { 0.00005 }
fn default_bdnf_threshold() -> f64 { 0.4 }
fn default_optimal_synaptic() -> f64 { 0.65 }
fn default_pruning_rate() -> f64 { 0.01 }

impl Default for GreyMatterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            growth_rate: 0.0001,
            decline_rate: 0.00002,
            myelination_growth: 0.00005,
            bdnf_threshold: 0.4,
            optimal_synaptic_density: 0.65,
            pruning_rate: 0.01,
        }
    }
}

// ─── Configuration des champs electromagnetiques ────────────────────────────

/// Configuration des champs EM (universel, solaire, terrestre, biochamp).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldsConfig {
    /// Active le systeme de champs EM
    #[serde(default = "default_true_fields")]
    pub enabled: bool,
    /// Vitesse d'avancement du cycle solaire
    #[serde(default = "default_solar_cycle_speed")]
    pub solar_cycle_speed: f64,
    /// Variance de la resonance de Schumann (Hz)
    #[serde(default = "default_schumann_variance")]
    pub schumann_variance: f64,
    /// Facteur anxiete des orages magnetiques sur la chimie
    #[serde(default = "default_storm_anxiety")]
    pub storm_anxiety_factor: f64,
    /// Facteur impact des orages sur le sommeil
    #[serde(default = "default_storm_sleep")]
    pub storm_sleep_factor: f64,
    /// Seuil d'orage magnetique pour impact chimique
    #[serde(default = "default_storm_threshold")]
    pub storm_threshold: f64,
}

fn default_true_fields() -> bool { true }
fn default_solar_cycle_speed() -> f64 { 0.0001 }
fn default_schumann_variance() -> f64 { 0.3 }
fn default_storm_anxiety() -> f64 { 0.03 }
fn default_storm_sleep() -> f64 { 0.05 }
fn default_storm_threshold() -> f64 { 0.5 }

impl Default for FieldsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            solar_cycle_speed: 0.0001,
            schumann_variance: 0.3,
            storm_anxiety_factor: 0.03,
            storm_sleep_factor: 0.05,
            storm_threshold: 0.5,
        }
    }
}

// =============================================================================
// PsychReportConfig — Rapport neuropsychologique
// =============================================================================

/// Configuration du module de rapport neuropsychologique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychReportConfig {
    /// Active le module de rapport neuropsychologique
    #[serde(default = "default_true_psych_report")]
    pub enabled: bool,
    /// Nombre maximal de tokens pour la generation du rapport
    #[serde(default = "default_psych_report_max_tokens")]
    pub max_tokens: u32,
    /// Temperature du LLM pour la generation du rapport
    #[serde(default = "default_psych_report_temperature")]
    pub temperature: f64,
}

fn default_true_psych_report() -> bool { true }
fn default_psych_report_max_tokens() -> u32 { 2000 }
fn default_psych_report_temperature() -> f64 { 0.4 }

impl Default for PsychReportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_tokens: 2000,
            temperature: 0.4,
        }
    }
}

// =============================================================================
// ReceptorDynamicsConfig — Dynamique des recepteurs neuronaux
// =============================================================================

/// Configuration de la dynamique des recepteurs neuronaux (adaptation, recovery).
/// Correspond a la section [receptors] dans saphire.toml / factory_defaults.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorDynamicsConfig {
    /// Vitesse d'adaptation des recepteurs (downregulation/upregulation)
    #[serde(default = "default_receptor_adaptation_rate")]
    pub adaptation_rate: f64,
    /// Vitesse de recuperation des recepteurs (retour vers densite=1.0, tolerance=0.0)
    #[serde(default = "default_receptor_recovery_rate")]
    pub recovery_rate: f64,
}

fn default_receptor_adaptation_rate() -> f64 { 0.02 }
fn default_receptor_recovery_rate() -> f64 { 0.005 }

impl Default for ReceptorDynamicsConfig {
    fn default() -> Self {
        Self {
            adaptation_rate: 0.02,
            recovery_rate: 0.005,
        }
    }
}

// =============================================================================
// BdnfConfig — BDNF (Brain-Derived Neurotrophic Factor)
// =============================================================================

/// Configuration du BDNF : facteur neurotrophique derivant de multiples signaux
/// biologiques (serotonine, dopamine, apprentissage, novelty, flow state).
/// Module la consolidation memoire et l'apprentissage du connectome.
/// Correspond a la section [bdnf] dans saphire.toml / factory_defaults.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdnfConfig {
    /// Poids de la dopamine dans le calcul du BDNF
    #[serde(default = "default_bdnf_dopamine_weight")]
    pub dopamine_weight: f64,
    /// Bonus BDNF quand nouveaute detectee
    #[serde(default = "default_bdnf_novelty_bonus")]
    pub novelty_bonus: f64,
    /// Bonus BDNF en etat de flow (dopamine>0.6, cortisol<0.4)
    #[serde(default = "default_bdnf_flow_state_bonus")]
    pub flow_state_bonus: f64,
    /// Poids de la penalite cortisol sur le BDNF (au-dessus du seuil)
    #[serde(default = "default_bdnf_cortisol_penalty_weight")]
    pub cortisol_penalty_weight: f64,
    /// Seuil de cortisol pour activer la penalite BDNF
    #[serde(default = "default_bdnf_cortisol_penalty_threshold")]
    pub cortisol_penalty_threshold: f64,
    /// Vitesse de retour du BDNF vers la baseline (homeostasie)
    #[serde(default = "default_bdnf_homeostasis_rate")]
    pub homeostasis_rate: f64,
    /// Baseline du BDNF (point d'equilibre)
    #[serde(default = "default_bdnf_homeostasis_baseline")]
    pub homeostasis_baseline: f64,
    /// Multiplicateur plancher de consolidation memoire (a BDNF=0)
    #[serde(default = "default_bdnf_consolidation_floor")]
    pub consolidation_floor: f64,
    /// Amplitude du multiplicateur de consolidation (floor + range*BDNF)
    #[serde(default = "default_bdnf_consolidation_range")]
    pub consolidation_range: f64,
    /// Seuil BDNF pour activer le boost d'apprentissage du connectome
    #[serde(default = "default_bdnf_connectome_boost_threshold")]
    pub connectome_boost_threshold: f64,
    /// Facteur de boost connectome (max +50% a BDNF=1.0 au-dessus du seuil)
    #[serde(default = "default_bdnf_connectome_boost_factor")]
    pub connectome_boost_factor: f64,
}

fn default_bdnf_dopamine_weight() -> f64 { 0.15 }
fn default_bdnf_novelty_bonus() -> f64 { 0.1 }
fn default_bdnf_flow_state_bonus() -> f64 { 0.1 }
fn default_bdnf_cortisol_penalty_weight() -> f64 { 0.4 }
fn default_bdnf_cortisol_penalty_threshold() -> f64 { 0.6 }
fn default_bdnf_homeostasis_rate() -> f64 { 0.01 }
fn default_bdnf_homeostasis_baseline() -> f64 { 0.5 }
fn default_bdnf_consolidation_floor() -> f64 { 0.8 }
fn default_bdnf_consolidation_range() -> f64 { 0.4 }
fn default_bdnf_connectome_boost_threshold() -> f64 { 0.4 }
fn default_bdnf_connectome_boost_factor() -> f64 { 0.5 }

impl Default for BdnfConfig {
    fn default() -> Self {
        Self {
            dopamine_weight: 0.15,
            novelty_bonus: 0.1,
            flow_state_bonus: 0.1,
            cortisol_penalty_weight: 0.4,
            cortisol_penalty_threshold: 0.6,
            homeostasis_rate: 0.01,
            homeostasis_baseline: 0.5,
            consolidation_floor: 0.8,
            consolidation_range: 0.4,
            connectome_boost_threshold: 0.4,
            connectome_boost_factor: 0.5,
        }
    }
}

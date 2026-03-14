// =============================================================================
// lib.rs — Saphire : Agent cognitif autonome
//
// Role : Ce fichier est la racine de la bibliotheque (crate) Saphire.
// Il declare et expose tous les modules publics qui composent l'architecture
// de l'agent cognitif autonome.
//
// Dependances : Aucune directe (les dependances sont dans chaque module).
//
// Place dans l'architecture :
//   Ce fichier est le point d'entree de la crate "saphire" (library).
//   Le binaire (main.rs) et les tests importent les modules via cette racine.
//   L'architecture est modulaire : chaque module gere un aspect precis de la
//   cognition artificielle (chimie, emotions, conscience, regulation, etc.).
// =============================================================================

// ─── Module de configuration ─────────────────────────────────────────────────
// Charge et gere les parametres depuis saphire.toml et les variables d'environnement.
pub mod config;

// ─── Module de stimulus ──────────────────────────────────────────────────────
// Definit la structure Stimulus : l'entree perceptuelle de l'agent (texte + metriques).
pub mod stimulus;

// ─── Module de neurochimie ───────────────────────────────────────────────────
// Simule 7 neurotransmetteurs (dopamine, cortisol, serotonine, adrenaline,
// ocytocine, endorphine, noradrenaline) et leur dynamique (homeostasie, etc.).
pub mod neurochemistry;

// ─── Module des emotions ─────────────────────────────────────────────────────
// Deduit l'etat emotionnel a partir de la neurochimie (modele VAD = Valence-Arousal-Dominance).
pub mod emotions;

// ─── Module NLP (Natural Language Processing = Traitement Automatique du Langage) ──
// Analyse le texte en entree pour en extraire les metriques de stimulus.
pub mod nlp;

// ─── Modules cerebraux ───────────────────────────────────────────────────────
// Contient les trois "cerveaux" (reptilien, limbique, neocortex) inspires du
// modele triunique de MacLean. Chacun evalue le stimulus selon sa logique propre.
pub mod modules;

// ─── Module de consensus ─────────────────────────────────────────────────────
// Agglomere les signaux des trois modules pour produire une decision ponderee
// (Oui, Non, Peut-etre) avec un score de coherence.
pub mod consensus;

// ─── Module de conscience ────────────────────────────────────────────────────
// Simule un niveau de conscience et le phi (theorie IIT = Integrated Information Theory,
// Theorie de l'Information Integree). Maintient un monologue interieur.
pub mod consciousness;

// ─── Module de regulation ────────────────────────────────────────────────────
// Applique les lois morales (Asimov) : verifie les stimuli et peut exercer
// un droit de veto sur les decisions dangereuses.
pub mod regulation;

// ─── Module de base de donnees ───────────────────────────────────────────────
// Pool de connexions PostgreSQL (deadpool) + integration pgvector pour la
// recherche vectorielle de souvenirs similaires.
pub mod db;

// ─── Module LLM (Large Language Model = Modele de Langage de Grande Taille) ──
// Trait abstrait LlmBackend et implementations (OpenAI-compatible, Mock).
// Gere aussi la construction des prompts substrat et de pensee.
pub mod llm;

// ─── Module des plugins ──────────────────────────────────────────────────────
// Systeme extensible de plugins (WebUI, MicroNN, VectorMemory).
// Les plugins reagissent aux evenements du cerveau (BrainEvent).
pub mod plugins;

// ─── Module de reseau de neurones ────────────────────────────────────────────
// Implementation d'un micro-reseau de neurones (MLP = Multi-Layer Perceptron,
// Perceptron Multi-Couches) pour l'apprentissage local.
pub mod neural;

// ─── Module de stockage vectoriel ────────────────────────────────────────────
// Memoire vectorielle en RAM : stocke des embeddings et permet la recherche
// par similarite cosinus. Inclut le calcul de personnalite emergente.
pub mod vectorstore;

// ─── Module de memoire ───────────────────────────────────────────────────────
// Gestion de la memoire a trois niveaux : immediate, episodique, a long terme.
// Consolidation et decroissance des souvenirs.
pub mod memory;

// ─── Module de profilage ─────────────────────────────────────────────────────
// Profilage OCEAN (Ouverture, Conscienciosite, Extraversion, Agreabilite,
// Neurotisme) de l'agent et des humains avec qui il interagit.
pub mod profiling;

// ─── Module d'algorithmes ────────────────────────────────────────────────────
// Algorithmes utilitaires : bandit UCB1 (Upper Confidence Bound) pour la
// selection de types de pensee, et autres heuristiques.
pub mod algorithms;

// ─── Module d'auto-tuning ────────────────────────────────────────────────────
// Ajustement automatique des coefficients du cerveau (poids des modules,
// seuils de decision, taux de retroaction) base sur la satisfaction observee.
pub mod tuning;

// ─── Module de connaissances ─────────────────────────────────────────────────
// Acquisition de connaissances depuis des sources web (Wikipedia, etc.).
// L'agent peut rechercher et apprendre de maniere autonome.
pub mod knowledge;

// ─── Module du monde ─────────────────────────────────────────────────────────
// Modele interne du monde : date, heure, evenements, contexte environnemental.
pub mod world;

// ─── Module du corps virtuel ─────────────────────────────────────────────────
// Coeur battant, signaux somatiques, interoception (conscience corporelle).
pub mod body;

// ─── Module des besoins primaires ────────────────────────────────────────────
// Drives de faim et soif : derives de la physiologie, impactent la chimie,
// declenchent des actions autonomes (manger/boire).
pub mod needs;

// ─── Module hormonal ────────────────────────────────────────────────────────
// 8 hormones (cycles longs), neurorecepteurs (sensibilite, tolerance,
// saturation), cycles circadiens/ultradiens, interactions bidirectionnelles
// avec les 7 neurotransmetteurs.
pub mod hormones;

// ─── Module d'ethique ────────────────────────────────────────────────────────
// Systeme ethique a 3 couches : droit suisse (immuable), lois d'Asimov (immuable),
// ethique personnelle (evolutive, auto-formulee par Saphire via LLM).
pub mod ethics;

// ─── Module vital ────────────────────────────────────────────────────────────
// Les 3 piliers fondamentaux de la conscience :
// 1. VitalSpark — l'etincelle de vie, l'instinct de survie emergent
// 2. IntuitionEngine — le pattern-matching inconscient, le "gut feeling"
// 3. PremonitionEngine — l'anticipation predictive
pub mod vital;

// ─── Module sensoriel ────────────────────────────────────────────────────────
// Le Sensorium : 5 sens fondamentaux adaptes a la nature de Saphire
// (Lecture, Ecoute, Contact, Saveur, Ambiance) + sens emergents.
// Les sens sont la porte d'entree de la conscience sur le monde.
pub mod senses;

// ─── Module de logging ───────────────────────────────────────────────────────
// Systeme de logging centralise : buffer batch, broadcast dashboard, traces
// cognitives, historique LLM, metriques.
pub mod logging;

// ─── Module de l'agent ───────────────────────────────────────────────────────
// Le SaphireAgent : structure de haut niveau qui possede le cerveau, la memoire,
// les plugins, le LLM, et orchestre le cycle de vie complet.
pub mod agent;

// ─── Module du pipeline ──────────────────────────────────────────────────────
// Pipeline de demonstration et de test : enchaine des stimuli predetermines
// pour verifier le bon fonctionnement du systeme.
pub mod pipeline;

// ─── Module d'affichage ──────────────────────────────────────────────────────
// Fonctions d'affichage enrichi dans le terminal (barres, couleurs, formatage).
pub mod display;

// ─── Module de scenarios ─────────────────────────────────────────────────────
// Scenarios predetermines (ex: genese, premiers contacts) utilises lors du
// boot ou des demonstrations.
pub mod scenarios;

pub mod factory;

// ─── Module des orchestrateurs ────────────────────────────────────────────────
// Orchestrateurs de haut niveau : reves, desirs, apprentissage, attention, guerison.
// Gestion des aspirations, du sommeil, de la reflexion et de la resilience.
pub mod orchestrators;

// ─── Module psychologique ───────────────────────────────────────────────────
// 6 cadres psychologiques : Freud, Maslow, Tolteques, Jung, Goleman, Flow.
// Fonctionnent en parallele pour enrichir la psyche de Saphire.
pub mod psychology;

// ─── Module de sommeil ──────────────────────────────────────────────────────
// Systeme de sommeil : pression homeostatique, phases (Hypnagogic, LightSleep,
// DeepSleep, REM, Hypnopompic), consolidation memoire, restauration.
pub mod sleep;

// ─── Module de detection materielle ────────────────────────────────────────
// Detection GPU, CPU, RAM, disque, Ollama au demarrage.
// Recommandations automatiques de parametres LLM.
pub mod hardware;

// ─── Module genome / ADN ─────────────────────────────────────────────────
// Genome deterministe genere a partir d'un seed (ChaCha8 PRNG).
// Encode le temperament, les baselines chimiques, traits physiques,
// vulnerabilites et aptitudes cognitives.
pub mod genome;

// ─── Module connectome ──────────────────────────────────────────────────
// Graphe dynamique de connexions neuronales (autopoiese).
// Les noeuds (concepts, emotions, modules, sens) se connectent selon la
// regle de Hebb. Pruning synaptique et synaptogenese permanents.
pub mod connectome;

// ─── Module des conditions et afflictions ────────────────────────────────────
// Phobies, cinetose (mal des transports), troubles, et autres conditions
// qui affectent la chimie, la cognition et le corps de Saphire.
pub mod conditions;

// ─── Module de soins ─────────────────────────────────────────────────────────
// Systeme de soins complet : therapie psychologique, medicaments,
// chirurgie, art therapie, repos. Traite les maladies et conditions.
// TODO(integration) : brancher CareSystem sur l'agent et le pipeline
pub mod care;

// ─── Module des passions ─────────────────────────────────────────────────────
// Passions et hobbies emergents : centres d'interet qui naissent de
// l'experience, nourrissent l'identite et impactent la chimie.
// TODO(integration) : brancher PassionManager sur l'agent et le pipeline
pub mod passions;

// ─── Module relationnel ──────────────────────────────────────────────────────
// Liens affectifs, reseau relationnel, style d'attachement (Bowlby).
// Situation familiale configurable.
pub mod relationships;

// ─── Module de metacognition ─────────────────────────────────────────────────
// Auto-reflexion sur la qualite de la pensee, detection de repetitions et
// biais. Metrique de Turing composite (0-100) mesurant la completude cognitive.
// Inclut Source Monitoring et detection de biais de confirmation.
pub mod metacognition;

// ─── Modules cognitifs avances ──────────────────────────────────────────────
// 9 modules qui enrichissent la cognition de Saphire (tom, monologue,
// dissonance, memoire prospective, identite narrative, analogies,
// charge cognitive, imagerie mentale, sentiments).
pub mod cognition;

/// Temperament emergent — ~25 traits de caractere (timidite, generosite,
/// courage, curiosite, etc.) deduits de l'OCEAN, neurochimie, psychologie
/// et humeur. Recalcule au meme rythme que l'OCEAN (blend 30/70).
pub mod temperament;

// ─── Algorithmes de simulation pour la cognition ───────────────────────────
// 5 algorithmes issus du game design (behavior tree, influence map,
// flocking, FSM cognitive, steering behaviors).
pub mod simulation;

// ─── Modules neuroscientifiques avances ─────────────────────────────────────
// Recepteurs, regions cerebrales, predictive processing, metriques conscience.
pub mod neuroscience;

// ─── Modules biologiques innes ──────────────────────────────────────────────
// Nutrition, matiere grise, champs electromagnetiques.
pub mod biology;

// ─── Colonne vertebrale ───────────────────────────────────────────────────────
// Reflexes pre-cables, classification des signaux par urgence, routage vers
// le pipeline, relais moteur vers le corps virtuel et les effecteurs.
pub mod spine;

// ─── Module API ─────────────────────────────────────────────────────────────
// Handlers HTTP/WebSocket, routeur axum, etat partage (AppState).
// Regroupe tous les endpoints de l'interface web et du dashboard.
pub mod api;

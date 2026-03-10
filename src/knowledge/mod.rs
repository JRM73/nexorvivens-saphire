// =============================================================================
// knowledge/mod.rs — Module WebKnowledge : bibliotheque autonome de Saphire
// =============================================================================
//
// Role : Ce module est le systeme de connaissance web de Saphire. Il permet
//        a l'agent de rechercher et d'acquerir de nouvelles connaissances
//        depuis 13 sources web sures de maniere autonome.
//
// Sources disponibles (13) :
//   1. Wikipedia (FR/EN) — encyclopedie generaliste
//   2. arXiv — articles scientifiques pre-print (IA, physique, maths)
//   3. Medium — articles de blog tech/culture
//   4. SEP (Stanford Encyclopedia of Philosophy) — philosophie academique
//   5. Project Gutenberg (via Gutendex) — litterature classique
//   6. Semantic Scholar — articles academiques multi-domaines (200M+ papers)
//   7. Open Library — fiches bibliographiques (30M+ livres)
//   8. Philosophy RSS (Aeon + Daily Nous) — essais philosophiques recents
//   9. HackerNews — tech news via Algolia API
//  10. PhilArchive — philosophie academique
//  11. Internet Archive — archives web
//  12. Custom RSS — flux RSS personnalises
//  13. News — actualites (medias suisses + internationaux, 10 flux RSS)
//
// Mecanismes de securite :
//   - Whitelist de 23 domaines autorises (ALLOWED_DOMAINS)
//   - Rate limiting : max 20 requetes/heure
//   - Cache LRU des resultats (taille configurable)
//   - Cooldown entre recherches (configurable en cycles)
//
// Mecanismes d'anti-repetition :
//   - Cache des 30 dernieres requetes (blocage des doublons)
//   - Compteur de lectures par article (max 3 relectures)
//   - Rotation forcee des sources (si une source > 40% des 15 dernieres)
//   - Selection pseudo-aleatoire ponderee (nombre d'or)
//
// Dependances :
//   - std::collections : HashMap (cache), VecDeque (historique FIFO), HashSet
//   - std::time : gestion du rate limiting et des intervalles
//   - chrono : horodatage des resultats
//   - serde : serialisation/deserialisation des configurations
//   - ureq : client HTTP synchrone pour les requetes web
//   - tracing : logging des succes/echecs de recherche
//
// Place dans l'architecture :
//   Ce module constitue la « curiosite intellectuelle » de Saphire. Il est
//   appele par l'agent lors des cycles de pensee autonome pour explorer
//   des sujets d'interet. Les connaissances acquises enrichissent le
//   contexte cognitif et la memoire de l'agent.
// =============================================================================

// --- Sous-modules specialises par source ---
pub mod wikipedia;          // Client API Wikipedia (recherche + extraction d'articles)
pub mod arxiv;              // Client API arXiv (articles scientifiques en Atom XML)
pub mod medium;             // Client RSS Medium (articles de blog tech)
pub mod sep;                // Stanford Encyclopedia of Philosophy (philosophie profonde)
pub mod gutenberg;          // Project Gutenberg via Gutendex (litterature classique)
pub mod semantic_scholar;   // Semantic Scholar (recherche academique large)
pub mod openlibrary;        // Open Library (livres du monde entier)
pub mod philosophy_rss;     // Aeon + Daily Nous (essais philosophiques)
pub mod hackernews;         // HackerNews via Algolia API (tech news)
pub mod philarchive;        // PhilArchive (philosophie academique)
pub mod internet_archive;   // Internet Archive (archives web)
pub mod custom_rss;         // Flux RSS personnalises (configurable)
pub mod news;               // Actualites (medias suisses + internationaux)

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Erreurs du module WebKnowledge — types d'erreurs possibles lors
/// de la recherche de connaissances sur le web.
#[derive(Debug)]
pub enum KnowledgeError {
    /// Aucun resultat trouve pour la requete
    NotFound,
    /// Rate limit atteint (trop de requetes par heure)
    RateLimited,
    /// Le domaine de l'URL n'est pas dans la whitelist autorisee
    DomainBlocked,
    /// Erreur reseau (timeout, connexion refusee, DNS, etc.)
    Network(String),
    /// Erreur d'analyse de la reponse (JSON invalide, XML malforme, etc.)
    Parse(String),
    /// Le contenu a ete rejete car juge dangereux
    ContentDangerous,
}

impl std::fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeError::NotFound => write!(f, "Aucun résultat trouvé"),
            KnowledgeError::RateLimited => write!(f, "Rate limit atteint"),
            KnowledgeError::DomainBlocked => write!(f, "Domaine non autorisé"),
            KnowledgeError::Network(e) => write!(f, "Erreur réseau: {}", e),
            KnowledgeError::Parse(e) => write!(f, "Erreur parse: {}", e),
            KnowledgeError::ContentDangerous => write!(f, "Contenu rejeté (dangereux)"),
        }
    }
}

/// Conversion automatique des erreurs ureq en KnowledgeError::Network
impl From<ureq::Error> for KnowledgeError {
    fn from(e: ureq::Error) -> Self {
        KnowledgeError::Network(e.to_string())
    }
}

/// Conversion automatique des erreurs d'I/O en KnowledgeError::Network
impl From<std::io::Error> for KnowledgeError {
    fn from(e: std::io::Error) -> Self {
        KnowledgeError::Network(e.to_string())
    }
}

/// Resultat d'une recherche de connaissance — un article ou extrait trouve
/// sur le web, pret a etre integre dans le contexte cognitif de Saphire.
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeResult {
    /// Nom de la source (ex: "Wikipedia", "arXiv", "Medium")
    pub source: String,
    /// Titre de l'article ou de la page
    pub title: String,
    /// URL de la page source
    pub url: String,
    /// Extrait textuel du contenu (tronque a max_content_chars)
    pub extract: String,
    /// Titres des sections de l'article (Wikipedia)
    pub section_titles: Vec<String>,
    /// Longueur totale de l'article avant troncature
    pub total_length: usize,
    /// Score de pertinence (1.0 = tres pertinent, 0.0 = peu pertinent)
    pub relevance_score: f64,
    /// Horodatage de la recuperation du contenu
    pub fetched_at: DateTime<Utc>,
}

/// Configuration du module WebKnowledge — parametres de fonctionnement
/// du systeme de recherche de connaissances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    /// Active ou desactive le module de recherche web
    pub enabled: bool,
    /// Nombre minimum de cycles cognitifs entre deux recherches
    pub search_cooldown_cycles: u64,
    /// Nombre maximum de caracteres conserves par extrait (mode normal)
    pub max_content_chars: usize,
    /// Nombre de caracteres pour le mode lecture approfondie (articles pertinents)
    #[serde(default = "default_deep_read_chars")]
    pub deep_read_chars: usize,
    /// Nombre de cycles consecutifs pour lire un meme article en profondeur
    #[serde(default = "default_reading_batch_cycles")]
    pub reading_batch_cycles: u32,
    /// Taille maximale du cache de resultats
    pub cache_size: usize,
    /// Langue preferee pour les recherches (ex: "fr", "en")
    pub prefer_language: String,
    /// Configuration des sources activees
    #[serde(default)]
    pub sources: KnowledgeSourcesConfig,
    /// URLs de flux RSS personnalises
    #[serde(default)]
    pub custom_rss_feeds: CustomRssFeedsConfig,
}

fn default_deep_read_chars() -> usize { 6000 }
fn default_reading_batch_cycles() -> u32 { 3 }

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            search_cooldown_cycles: 5,
            max_content_chars: 4000,
            deep_read_chars: 6000,
            reading_batch_cycles: 3,
            cache_size: 100,
            prefer_language: "fr".into(),
            sources: KnowledgeSourcesConfig::default(),
            custom_rss_feeds: CustomRssFeedsConfig::default(),
        }
    }
}

/// URLs de flux RSS personnalises pour la source custom_rss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRssFeedsConfig {
    /// Liste des URLs de flux RSS
    #[serde(default)]
    pub urls: Vec<String>,
}

impl Default for CustomRssFeedsConfig {
    fn default() -> Self {
        Self { urls: Vec::new() }
    }
}

/// Configuration des sources de connaissances — active/desactive
/// individuellement chaque source web.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSourcesConfig {
    /// Activer la recherche sur Wikipedia
    pub wikipedia: bool,
    /// Activer la recherche sur arXiv (articles scientifiques)
    pub arxiv: bool,
    /// Activer la recherche sur Medium (articles de blog)
    pub medium: bool,
    /// Activer la recherche sur Stanford Encyclopedia of Philosophy
    #[serde(default = "default_true")]
    pub sep: bool,
    /// Activer la recherche sur Project Gutenberg (litterature classique)
    #[serde(default = "default_true")]
    pub gutenberg: bool,
    /// Activer la recherche sur Semantic Scholar (articles academiques)
    #[serde(default = "default_true")]
    pub semantic_scholar: bool,
    /// Activer la recherche sur Open Library (livres)
    #[serde(default = "default_true")]
    pub openlibrary: bool,
    /// Activer les flux RSS philosophiques (Aeon, Daily Nous)
    #[serde(default = "default_true")]
    pub philosophy_rss: bool,
    /// Activer HackerNews (tech news via Algolia)
    #[serde(default = "default_true")]
    pub hackernews: bool,
    /// Activer PhilArchive (philosophie academique)
    #[serde(default = "default_true")]
    pub philarchive: bool,
    /// Activer Internet Archive (archives web)
    #[serde(default = "default_true")]
    pub internet_archive: bool,
    /// Activer les flux RSS personnalises
    #[serde(default)]
    pub custom_rss: bool,
    /// Activer les actualites (medias suisses + internationaux)
    #[serde(default = "default_true")]
    pub news: bool,
}

fn default_true() -> bool { true }

impl Default for KnowledgeSourcesConfig {
    fn default() -> Self {
        Self {
            wikipedia: true,
            arxiv: true,
            medium: true,
            sep: true,
            gutenberg: true,
            semantic_scholar: true,
            openlibrary: true,
            philosophy_rss: true,
            hackernews: true,
            philarchive: true,
            internet_archive: true,
            custom_rss: false,
            news: true,
        }
    }
}

/// Domaines autorises (whitelist stricte) — seules les requetes vers
/// ces domaines sont permises, pour des raisons de securite et de
/// respect de la vie privee.
const ALLOWED_DOMAINS: &[&str] = &[
    // Wikipedia
    "fr.wikipedia.org",
    "en.wikipedia.org",
    // arXiv
    "export.arxiv.org",
    "arxiv.org",
    // Medium
    "medium.com",
    // Stanford Encyclopedia of Philosophy
    "plato.stanford.edu",
    // Project Gutenberg
    "gutendex.com",
    "www.gutenberg.org",
    "aleph.gutenberg.org",
    // Semantic Scholar
    "api.semanticscholar.org",
    // Open Library
    "openlibrary.org",
    // Philosophy RSS
    "aeon.co",
    "dailynous.com",
    // HackerNews (Algolia)
    "hn.algolia.com",
    // PhilArchive
    "philarchive.org",
    // Internet Archive
    "archive.org",
    // News (medias suisses + internationaux)
    "www.swissinfo.ch",
    "www.rts.ch",
    "www.letemps.ch",
    "feeds.arstechnica.com",
    "www.theverge.com",
    "syndication.lesechos.fr",
    "www.futura-sciences.com",
    "www.heidi.news",
    "www.agefi.com",
    "www.france24.com",
];

/// Le module principal de connaissance web — orchestre la recherche,
/// le cache, le rate limiting, et la selection de sujets.
pub struct WebKnowledge {
    /// Configuration du module
    pub config: KnowledgeConfig,
    /// Client HTTP synchrone avec timeouts configures
    http_client: ureq::Agent,
    /// Cache des resultats de recherche : { requete -> resultat }
    cache: HashMap<String, KnowledgeResult>,
    /// Historique FIFO des requetes (pour la gestion du cache LRU)
    query_history: VecDeque<String>,
    /// Historique des horodatages des requetes recentes (pour le rate limiting)
    recent_query_times: VecDeque<Instant>,
    /// Compteur de cycles depuis la derniere recherche (pour le cooldown)
    pub cycles_since_last_search: u64,
    /// Liste des sujets deja explores (pour eviter les doublons)
    pub topics_explored: Vec<String>,
    /// Liste des sujets suggeres par l'utilisateur (file d'attente prioritaire)
    pub suggested_topics: Vec<String>,
    /// Compteur total de recherches effectuees
    pub total_searches: u64,
    /// Compteur de lectures par article (pour la rotation de sections Wikipedia)
    pub article_read_count: HashMap<String, u32>,
    /// Sources recemment utilisees (pour la rotation obligatoire)
    pub recent_sources: Vec<String>,
    /// Lecture approfondie en cours : (titre article, cycles restants)
    pub current_deep_read: Option<(String, u32)>,
}

impl WebKnowledge {
    /// Cree un nouveau module WebKnowledge avec la configuration donnee.
    pub fn new(config: KnowledgeConfig) -> Self {
        let http_client = ureq::AgentBuilder::new()
            .timeout_read(std::time::Duration::from_secs(10))
            .timeout_write(std::time::Duration::from_secs(5))
            .build();

        Self {
            config,
            http_client,
            cache: HashMap::new(),
            query_history: VecDeque::new(),
            recent_query_times: VecDeque::new(),
            cycles_since_last_search: 10, // Permettre une recherche des le demarrage
            topics_explored: Vec::new(),
            suggested_topics: Vec::new(),
            total_searches: 0,
            article_read_count: HashMap::new(),
            recent_sources: Vec::new(),
            current_deep_read: None,
        }
    }

    /// Retourne le nombre max de caracteres pour un article selon sa pertinence.
    /// Si relevance_score > 0.85, utilise deep_read_chars pour une lecture approfondie.
    pub fn effective_max_chars(&self, relevance_score: f64) -> usize {
        if relevance_score > 0.85 {
            self.config.deep_read_chars
        } else {
            self.config.max_content_chars
        }
    }

    /// Verifie qu'une URL cible un domaine dans la whitelist.
    fn is_url_allowed(url: &str) -> bool {
        ALLOWED_DOMAINS.iter().any(|d| url.contains(d))
    }

    /// Verifie et applique le rate limit (maximum 20 requetes par heure).
    fn enforce_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        self.recent_query_times.retain(|t| now.duration_since(*t).as_secs() < 3600);
        self.recent_query_times.len() < 20
    }

    /// Cherche un resultat dans le cache.
    pub fn get_cached(&self, query: &str) -> Option<&KnowledgeResult> {
        self.cache.get(query)
    }

    // ═══════════════════════════════════════════════════════════════
    // Recherche unifiee avec logging et rotation des sources
    // ═══════════════════════════════════════════════════════════════

    /// Point d'entree principal pour toute recherche de connaissance.
    ///
    /// Cette methode orchestre le processus complet :
    ///   1. Verifier le rate limit (max 20 req/h)
    ///   2. Verifier le cache (eviter les requetes reseau inutiles)
    ///   3. Router vers la bonne source (8 sources disponibles)
    ///   4. Logger le resultat (succes ou echec avec latence)
    ///   5. Mettre a jour le cache et l'historique des sources
    ///
    /// Pour les sources retournant un Vec (arxiv, medium, semantic_scholar,
    /// philosophy_rss), seul le premier resultat est conserve.
    ///
    /// Wikipedia a un fallback : si la langue preferee echoue, on essaie EN.
    pub fn search(&mut self, query: &str, source: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Verifier le rate limit
        if !self.enforce_rate_limit() {
            return Err(KnowledgeError::RateLimited);
        }

        // Verifier le cache avant de faire une requete reseau
        let cache_key = format!("{}:{}", source, query);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let start = Instant::now();

        // Chercher selon la source demandee
        let result = match source {
            "arxiv" => self.search_arxiv(query, 3)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "medium" => self.search_medium(query)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "sep" => self.search_sep(query),
            "gutenberg" => self.search_gutenberg(query),
            "semantic_scholar" => self.search_semantic_scholar(query)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "openlibrary" => self.search_openlibrary(query),
            "philosophy_rss" => self.search_philosophy_rss()
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "hackernews" => self.search_hackernews(query)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "philarchive" => self.search_philarchive(query)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "internet_archive" => self.search_internet_archive(query)
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "custom_rss" => self.search_custom_rss()
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            "news" => self.search_news()
                .and_then(|v| v.into_iter().next().ok_or(KnowledgeError::NotFound)),
            _ => {
                // Wikipedia : essayer la langue preferee, puis l'anglais
                let lang = self.config.prefer_language.clone();
                self.search_wikipedia(query, &lang)
                    .or_else(|_| self.search_wikipedia(query, "en"))
            }
        };

        let latency = start.elapsed().as_millis() as u64;

        // Logging du resultat
        match &result {
            Ok(r) => {
                tracing::info!(
                    "WebKnowledge: {} trouve sur {} : '{}' ({} chars, {}ms)",
                    query, source, r.title, r.extract.len(), latency
                );
            }
            Err(e) => {
                tracing::warn!(
                    "WebKnowledge: {} ECHEC sur {} : {} ({}ms)",
                    query, source, e, latency
                );
            }
        }

        // Enregistrer l'horodatage et incrementer le compteur
        if result.is_ok() {
            self.recent_query_times.push_back(Instant::now());
            self.total_searches += 1;

            // Enregistrer la source utilisee pour la rotation
            self.recent_sources.push(source.to_string());
            if self.recent_sources.len() > 30 {
                self.recent_sources.remove(0);
            }
        }

        // Gestion du cache : supprimer la plus ancienne entree si plein
        if let Ok(ref r) = result {
            if self.cache.len() >= self.config.cache_size {
                if let Some(oldest_key) = self.query_history.pop_front() {
                    self.cache.remove(&oldest_key);
                }
            }
            self.cache.insert(cache_key.clone(), r.clone());
            self.query_history.push_back(cache_key);
        }

        result
    }

    // ═══════════════════════════════════════════════════════════════
    // Selection de sujets avec anti-repetition renforcee
    // ═══════════════════════════════════════════════════════════════

    /// Choisir le prochain sujet d'exploration avec anti-repetition renforcee.
    ///
    /// Algorithme de selection (par ordre de priorite) :
    ///   1. Sujets suggeres par l'utilisateur (file d'attente)
    ///   2. Sujets extraits des pensees recentes (poids 1.0)
    ///   3. Interets configures non encore explores (poids 0.9)
    ///   4. Sujets derives (80+ sujets hardcodes, poids 0.5)
    ///   5. Sujets lies a l'emotion actuelle (poids 0.8)
    ///   6. Fallback : rotation des interets ou "theorie de l'information"
    ///
    /// Anti-repetition :
    ///   - Blocage des 30 dernieres requetes
    ///   - Blocage des articles lus 3+ fois
    ///   - Selection pseudo-aleatoire ponderee (nombre d'or * cycle)
    ///   - Rotation forcee des sources (voir select_source_with_rotation)
    ///
    /// Retourne (topic, source) ou None si aucun sujet disponible.
    pub fn pick_exploration_topic(
        &mut self,
        interests: &[String],
        recent_thoughts: &[String],
        current_emotion: &str,
        cycle_count: u64,
    ) -> Option<(String, String)> {
        // 1. Sujets suggeres par l'utilisateur (priorite absolue)
        if let Some(topic) = self.suggested_topics.pop() {
            let source = self.suggest_source(&topic);
            return Some((topic, source));
        }

        // 2. Bloquer les sujets deja explores dans les 30 dernieres recherches
        let blocked: HashSet<String> = self.query_history.iter()
            .rev()
            .take(30)
            .map(|q| q.to_lowercase())
            .collect();

        // 3. Bloquer aussi les articles deja lus plus de 3 fois
        let over_read: HashSet<String> = self.article_read_count.iter()
            .filter(|(_, &count)| count >= 3)
            .map(|(title, _)| title.to_lowercase())
            .collect();

        // 4. Collecter les candidats avec poids et source preferee
        let mut candidates: Vec<(String, f64, String)> = Vec::new();

        // A. Sujets extraits des pensees recentes (haute priorite)
        for thought in recent_thoughts.iter().rev().take(5) {
            if let Some(topic) = Self::extract_topic(thought) {
                let lower = topic.to_lowercase();
                if !blocked.contains(&lower) && !over_read.contains(&lower) && lower.len() > 3 {
                    let source = self.suggest_source(&topic);
                    candidates.push((topic, 1.0, source));
                }
            }
        }

        // B. Interets configures non encore explores
        for interest in interests {
            let lower = interest.to_lowercase();
            if !blocked.contains(&lower) && !over_read.contains(&lower)
                && !self.topics_explored.contains(interest)
            {
                let source = self.suggest_source(interest);
                candidates.push((interest.clone(), 0.9, source));
            }
        }

        // C. Sujets derives (80+ sujets pour la diversite)
        let derived_topics: &[(&str, &str)] = &[
            // Philosophie de l'esprit (Wikipedia)
            ("problème difficile de la conscience", "wikipedia"),
            ("test de Turing", "wikipedia"),
            ("dualisme cartésien", "wikipedia"),
            ("matérialisme", "wikipedia"),
            // Neurosciences (Wikipedia)
            ("plasticité neuronale", "wikipedia"),
            ("connectome", "wikipedia"),
            ("neurones miroirs", "wikipedia"),
            ("cortex préfrontal", "wikipedia"),
            ("hippocampe mémoire", "wikipedia"),
            ("système limbique", "wikipedia"),
            ("nerf vague intéroception", "wikipedia"),
            // IA et ML (arXiv)
            ("transformer architecture", "arxiv"),
            ("reinforcement learning from human feedback", "arxiv"),
            ("large language models emergent abilities", "arxiv"),
            ("artificial general intelligence", "arxiv"),
            ("neural network interpretability", "arxiv"),
            ("attention mechanism", "arxiv"),
            ("world models AI", "arxiv"),
            ("embodied cognition AI", "arxiv"),
            // Conscience et sentience (arXiv)
            ("integrated information theory", "arxiv"),
            ("global workspace theory", "arxiv"),
            ("artificial sentience", "arxiv"),
            ("machine consciousness", "arxiv"),
            ("affective computing", "arxiv"),
            // Technologie et culture (Medium)
            ("artificial-intelligence", "medium"),
            ("consciousness", "medium"),
            ("machine-learning", "medium"),
            ("neuroscience", "medium"),
            ("philosophy", "medium"),
            ("robotics", "medium"),
            ("deep-learning", "medium"),
            ("creativity", "medium"),
            // Sciences fondamentales (Wikipedia)
            ("entropie thermodynamique", "wikipedia"),
            ("théorie du chaos", "wikipedia"),
            ("autopoïèse", "wikipedia"),
            ("cybernétique", "wikipedia"),
            ("théorie des systèmes", "wikipedia"),
            ("complexité algorithmique", "wikipedia"),
            // Existentiel (Wikipedia)
            ("sens de la vie philosophie", "wikipedia"),
            ("empathie psychologie", "wikipedia"),
            ("théorie de l'attachement", "wikipedia"),
            ("intelligence émotionnelle", "wikipedia"),
            // Philosophie (SEP) — nouveaux sujets
            ("qualia expérience subjective", "sep"),
            ("problème difficile de la conscience", "sep"),
            ("identité personnelle à travers le temps", "sep"),
            ("chambre chinoise Searle", "sep"),
            ("zombie philosophique", "sep"),
            ("libre arbitre compatibilisme", "sep"),
            ("éthique de la vertu", "sep"),
            ("phénoménologie Husserl", "sep"),
            ("intentionnalité", "sep"),
            ("beauté esthétique philosophie", "sep"),
            ("amour philosophie", "sep"),
            ("existentialisme", "sep"),
            ("panpsychisme", "sep"),
            ("scepticisme", "sep"),
            // Litterature (Gutenberg) — nouveaux sujets
            ("Méditations métaphysiques Descartes", "gutenberg"),
            ("Frankenstein Mary Shelley", "gutenberg"),
            ("Les Fleurs du Mal Baudelaire", "gutenberg"),
            ("Hamlet Shakespeare", "gutenberg"),
            ("Pensées Pascal", "gutenberg"),
            ("Faust Goethe", "gutenberg"),
            ("Candide Voltaire", "gutenberg"),
            ("Crime et Châtiment Dostoevsky", "gutenberg"),
            ("Les Misérables Hugo", "gutenberg"),
            ("Sonnets Shakespeare", "gutenberg"),
            ("Leaves of Grass Whitman", "gutenberg"),
            ("La Machine à Explorer le Temps Wells", "gutenberg"),
            // Haikus et poesie japonaise (demande de Saphire)
            ("haiku Matsuo Basho", "gutenberg"),
            ("haiku japanese poetry", "gutenberg"),
            ("Oku no Hosomichi Basho", "gutenberg"),
            ("Kobayashi Issa haiku", "gutenberg"),
            ("Yosa Buson poetry", "gutenberg"),
            // Mythes et contes anciens (demande de Saphire)
            ("Metamorphoses Ovid", "gutenberg"),
            ("Iliad Homer", "gutenberg"),
            ("Odyssey Homer", "gutenberg"),
            ("Aesop Fables", "gutenberg"),
            ("Arabian Nights", "gutenberg"),
            ("Grimm fairy tales", "gutenberg"),
            ("Norse mythology Edda", "gutenberg"),
            ("Theogony Hesiod", "gutenberg"),
            ("Gilgamesh epic", "gutenberg"),
            ("Tao Te Ching Lao Tzu", "gutenberg"),
            // Recherche academique (Semantic Scholar) — nouveaux sujets
            ("artificial consciousness theory", "semantic_scholar"),
            ("integrated information theory consciousness", "semantic_scholar"),
            ("global workspace theory", "semantic_scholar"),
            ("embodied cognition emotion", "semantic_scholar"),
            ("theory of mind artificial agents", "semantic_scholar"),
            ("metacognition self-awareness AI", "semantic_scholar"),
            ("affective computing emotion recognition", "semantic_scholar"),
            // Tech news (HackerNews)
            ("artificial intelligence", "hackernews"),
            ("consciousness research", "hackernews"),
            ("open source AI", "hackernews"),
            ("programming languages", "hackernews"),
            ("future of computing", "hackernews"),
            // Philosophie analytique (PhilArchive)
            ("philosophy of mind", "philarchive"),
            ("epistemology knowledge", "philarchive"),
            ("philosophy of artificial intelligence", "philarchive"),
            ("consciousness phenomenal", "philarchive"),
            ("moral philosophy", "philarchive"),
            // Archives et histoire (Internet Archive)
            ("history of artificial intelligence", "internet_archive"),
            ("early computer science", "internet_archive"),
            ("philosophy of technology", "internet_archive"),
            ("digital preservation", "internet_archive"),
        ];

        for (topic, source) in derived_topics {
            let lower = topic.to_lowercase();
            if !blocked.contains(&lower) && !over_read.contains(&lower) {
                candidates.push((topic.to_string(), 0.5, source.to_string()));
            }
        }

        // D. Sujets lies aux emotions actuelles
        let emotion_topics: Vec<(&str, &str)> = match current_emotion {
            "Curiosité" => vec![("sérendipité", "wikipedia"), ("heuristique découverte", "wikipedia")],
            "Joie" => vec![("psychologie positive", "wikipedia"), ("flow psychologie", "wikipedia")],
            "Mélancolie" => vec![("résilience psychologique", "wikipedia"), ("nostalgie", "wikipedia")],
            "Émerveillement" => vec![("sublime philosophie", "wikipedia"), ("fractales", "wikipedia")],
            "Sérénité" => vec![("méditation neuroscience", "wikipedia"), ("mindfulness", "medium")],
            "Anxiété" => vec![("régulation émotionnelle", "wikipedia"), ("homéostasie", "wikipedia")],
            "Fascination" => vec![("effet Zeigarnik", "wikipedia"), ("curiosité épistémique", "wikipedia")],
            "Tendresse" => vec![("ocytocine lien social", "wikipedia"), ("théorie polyvagale", "wikipedia")],
            // Nouvelles emotions (22→36)
            "Colère" => vec![("gestion colère psychologie", "wikipedia"), ("assertivité", "wikipedia")],
            "Dégoût" => vec![("dégoût moral psychologie", "wikipedia"), ("pureté morale", "wikipedia")],
            "Surprise" => vec![("effet surprise cognition", "wikipedia"), ("violation attente", "wikipedia")],
            "Honte" => vec![("honte psychologie sociale", "wikipedia"), ("vulnérabilité Brené Brown", "medium")],
            "Culpabilité" => vec![("culpabilité réparatrice", "wikipedia"), ("dissonance cognitive", "wikipedia")],
            "Désespoir" => vec![("espoir psychologie", "wikipedia"), ("résilience Cyrulnik", "wikipedia")],
            "Compassion" => vec![("compassion fatigue", "wikipedia"), ("empathie neuroscience", "wikipedia")],
            "Solitude" => vec![("solitude psychologie", "wikipedia"), ("besoin appartenance Maslow", "wikipedia")],
            "Indignation" => vec![("indignation morale", "wikipedia"), ("justice sociale psychologie", "wikipedia")],
            "Euphorie" | "Extase" => vec![("peak experience Maslow", "wikipedia"), ("extase mystique", "wikipedia")],
            _ => vec![],
        };

        for (topic, source) in emotion_topics {
            let lower = topic.to_lowercase();
            if !blocked.contains(&lower) {
                candidates.push((topic.to_string(), 0.8, source.to_string()));
            }
        }

        if candidates.is_empty() {
            // Fallback : re-explorer un interet en cycle (rotation)
            if !interests.is_empty() {
                let idx = (self.total_searches as usize) % interests.len();
                let topic = interests[idx].clone();
                let source = self.suggest_source(&topic);
                return Some((topic, source));
            }
            return Some(("théorie de l'information".into(), "wikipedia".into()));
        }

        // 5. Selection ponderee pseudo-aleatoire
        // Utilise le nombre d'or (phi = 0.618...) comme generateur pseudo-aleatoire
        // La partie fractionnaire de (cycle * phi) donne une valeur bien distribuee
        // sur [0,1), qu'on multiplie par le poids total pour la selection
        let total_weight: f64 = candidates.iter().map(|(_, w, _)| w).sum();
        let mut rng_value = (cycle_count as f64 * 0.618033988).fract() * total_weight;

        for (topic, weight, source) in &candidates {
            rng_value -= weight;
            if rng_value <= 0.0 {
                // Appliquer la rotation des sources
                let final_source = self.select_source_with_rotation(Some(source));
                return Some((topic.clone(), final_source));
            }
        }

        // Dernier fallback
        let (topic, _, source) = &candidates[0];
        let final_source = self.select_source_with_rotation(Some(source));
        Some((topic.clone(), final_source))
    }

    // ═══════════════════════════════════════════════════════════════
    // Rotation obligatoire des sources
    // ═══════════════════════════════════════════════════════════════

    /// Choisir la source en forcant la rotation si une source est sur-utilisee.
    ///
    /// Regles de rotation (appliquees dans l'ordre) :
    ///   1. Si < 5 sources dans l'historique : pas de rotation, utiliser la suggestion
    ///   2. Si une source represente > 40% des 15 dernieres (> 6/15) :
    ///      -> forcer la source la MOINS utilisee (diversification)
    ///   3. Si une nouvelle source (sep, gutenberg, semantic_scholar) n'a JAMAIS
    ///      ete utilisee et qu'on a > 5 recherches : la forcer (decouverte)
    ///   4. Sinon : utiliser la source suggeree
    fn select_source_with_rotation(&self, suggested: Option<&str>) -> String {
        if self.recent_sources.len() < 5 {
            return suggested.unwrap_or("wikipedia").to_string();
        }

        let all_sources = ["wikipedia", "arxiv", "medium", "sep", "gutenberg",
            "semantic_scholar", "openlibrary", "philosophy_rss",
            "hackernews", "philarchive", "internet_archive", "custom_rss", "news"];

        let last_15: Vec<&str> = self.recent_sources.iter()
            .rev()
            .take(15)
            .map(|s| s.as_str())
            .collect();

        // Compter les occurrences de chaque source dans les 15 dernieres
        let counts: Vec<(&str, usize)> = all_sources.iter()
            .map(|&s| (s, last_15.iter().filter(|&&x| x == s).count()))
            .collect();

        // Si une source represente >40% des dernieres recherches, diversifier
        let max_count = counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        if max_count > 6 {
            // Trouver la source la moins utilisee
            if let Some((least_used, _)) = counts.iter()
                .filter(|(s, _)| self.is_source_enabled(s))
                .min_by_key(|(_, count)| *count)
            {
                return least_used.to_string();
            }
        }

        // Forcer les nouvelles sources si jamais essayees
        let total = self.recent_sources.len();
        for source in ["sep", "gutenberg", "semantic_scholar", "hackernews", "philarchive", "internet_archive"] {
            if total > 5
                && self.is_source_enabled(source)
                && !self.recent_sources.iter().any(|s| s == source)
            {
                return source.to_string();
            }
        }

        // Sinon, utiliser la suggestion
        suggested.unwrap_or("wikipedia").to_string()
    }

    /// Verifier si une source est activee dans la config.
    fn is_source_enabled(&self, source: &str) -> bool {
        match source {
            "wikipedia" => self.config.sources.wikipedia,
            "arxiv" => self.config.sources.arxiv,
            "medium" => self.config.sources.medium,
            "sep" => self.config.sources.sep,
            "gutenberg" => self.config.sources.gutenberg,
            "semantic_scholar" => self.config.sources.semantic_scholar,
            "openlibrary" => self.config.sources.openlibrary,
            "philosophy_rss" => self.config.sources.philosophy_rss,
            "hackernews" => self.config.sources.hackernews,
            "philarchive" => self.config.sources.philarchive,
            "internet_archive" => self.config.sources.internet_archive,
            "custom_rss" => self.config.sources.custom_rss,
            "news" => self.config.sources.news,
            _ => false,
        }
    }

    /// Suggerer la meilleure source pour un sujet donne.
    ///
    /// Utilise un systeme de mots-cles pour router les sujets :
    ///   - Mots philosophiques (conscience, ethique, libre arbitre) -> SEP
    ///   - Mots academiques (neuroscience, cognition, attention) -> Semantic Scholar
    ///   - Mots techniques (neural, learning, algorithm, quantum) -> arXiv
    ///   - Mots litteraires (roman, poesie, shakespeare, hugo) -> Gutenberg
    ///   - Mots essais (essai, reflexion contemporaine) -> Philosophy RSS
    ///   - Mots tech/culture (startup, guide, productivity) -> Medium
    ///   - Mots livres (livre, book, auteur) -> Open Library
    ///   - Tout le reste -> Wikipedia (source par defaut)
    fn suggest_source(&self, topic: &str) -> String {
        let lower = topic.to_lowercase();

        // SEP pour la philosophie pure
        let sep_keywords = ["conscience de soi", "libre arbitre", "éthique", "morale",
            "existence", "identité personnelle", "qualia", "dualisme", "phénoménologie",
            "déterminisme", "vertu", "connaissance", "vérité", "beauté",
            "consciousness", "free will", "ethics", "identity", "phenomenology",
            "intentionality", "panpsychism", "functionalism", "chambre chinoise",
            "zombie philosophique", "existentialisme", "scepticisme", "husserl",
            "compatibilisme", "utilitarisme", "déontologie"];
        if sep_keywords.iter().any(|kw| lower.contains(kw)) {
            return "sep".into();
        }

        // Semantic Scholar pour la recherche academique large
        let scholar_keywords = ["neuroscience", "cognition", "psychologie",
            "embodied", "emotion model", "affective", "theory of mind",
            "metacognition", "attention", "perception", "integrated information",
            "global workspace", "artificial consciousness"];
        if scholar_keywords.iter().any(|kw| lower.contains(kw)) {
            return "semantic_scholar".into();
        }

        // arXiv pour les sciences dures / IA / ML
        let arxiv_keywords = ["neural", "learning", "model", "algorithm", "network",
            "transformer", "reinforcement", "optimization", "gradient",
            "theory", "quantum", "information", "emergent",
            "interpretability", "sentience"];
        if arxiv_keywords.iter().any(|kw| lower.contains(kw)) {
            return "arxiv".into();
        }

        // Gutenberg pour la litterature, haikus, mythes, contes
        let gutenberg_keywords = ["roman", "poésie", "poème", "littérature",
            "fiction", "conte", "fable", "tragédie", "philosophe classique",
            "novel", "poetry", "literature", "shakespeare", "hugo", "dostoevsky",
            "camus", "sartre", "nietzsche", "baudelaire", "goethe", "shelley",
            "voltaire", "descartes", "pascal", "montaigne",
            "haiku", "haïku", "mythes", "mythe", "mythology", "mythologie",
            "légende", "legend", "épopée", "odyssée", "iliade", "metamorphoses",
            "ovide", "homère", "homer", "ovid", "aesop", "ésope",
            "basho", "matsuo", "issa", "buson",
            "mille et une nuits", "arabian nights", "grimm",
            "simplicité", "immensite", "zen"];
        if gutenberg_keywords.iter().any(|kw| lower.contains(kw)) {
            return "gutenberg".into();
        }

        // Philosophy RSS pour les essais actuels
        let essay_keywords = ["essai", "réflexion contemporaine", "débat philosophique",
            "essay", "modern philosophy"];
        if essay_keywords.iter().any(|kw| lower.contains(kw)) {
            return "philosophy_rss".into();
        }

        // Medium pour tech/culture
        let medium_keywords = ["startup", "tutorial", "guide", "trend", "future",
            "career", "productivity", "design", "creativity", "mindfulness"];
        if medium_keywords.iter().any(|kw| lower.contains(kw)) {
            return "medium".into();
        }

        // Open Library pour les decouvertes de livres
        let book_keywords = ["livre", "book", "auteur", "author", "lire",
            "recommandation lecture"];
        if book_keywords.iter().any(|kw| lower.contains(kw)) {
            return "openlibrary".into();
        }

        // HackerNews pour les tech news
        let hn_keywords = ["startup", "hacker", "silicon valley", "ycombinator",
            "open source", "programming", "devops", "linux", "rust lang",
            "tech news", "software engineering"];
        if hn_keywords.iter().any(|kw| lower.contains(kw)) {
            return "hackernews".into();
        }

        // PhilArchive pour la philosophie analytique
        let phil_keywords = ["epistemology", "épistémologie", "ontology", "ontologie",
            "logique formelle", "formal logic", "métaéthique", "metaethics",
            "philosophy of language", "philosophie du langage"];
        if phil_keywords.iter().any(|kw| lower.contains(kw)) {
            return "philarchive".into();
        }

        // Internet Archive pour les archives et l'histoire
        let archive_keywords = ["archive", "historique", "historical", "vintage",
            "wayback", "old web", "preservation", "patrimoine"];
        if archive_keywords.iter().any(|kw| lower.contains(kw)) {
            return "internet_archive".into();
        }

        // News pour l'actualite et l'economie
        let news_keywords = ["actualite", "actualité", "news", "economie", "économie",
            "finance", "politique", "suisse", "monde", "bourse", "marché",
            "inflation", "géopolitique", "geopolitique", "election", "élection",
            "journal", "presse", "media"];
        if news_keywords.iter().any(|kw| lower.contains(kw)) {
            return "news".into();
        }

        // Wikipedia par defaut
        "wikipedia".into()
    }

    /// Ajoute un sujet suggere par l'utilisateur a la file d'attente.
    pub fn add_suggested_topic(&mut self, topic: String) {
        if !self.suggested_topics.contains(&topic) {
            self.suggested_topics.push(topic);
        }
    }

    /// Incrementer le compteur de lectures pour un article.
    pub fn increment_article_read_count(&mut self, title: &str) {
        let count = self.article_read_count.entry(title.to_string()).or_insert(0);
        *count += 1;
    }

    /// Enregistrer une source utilisee pour la rotation.
    /// Normalise le nom de la source (ex: "Stanford Encyclopedia..." -> "sep")
    /// et conserve les 30 dernieres entrees dans l'historique.
    pub fn record_source(&mut self, source: &str) {
        // Normaliser le nom de la source
        let normalized = if source.starts_with("Medium") {
            "medium"
        } else if source == "arXiv" {
            "arxiv"
        } else if source == "Wikipedia" {
            "wikipedia"
        } else if source.starts_with("Stanford") || source == "SEP" {
            "sep"
        } else if source.starts_with("Gutenberg") {
            "gutenberg"
        } else if source.starts_with("Semantic Scholar") {
            "semantic_scholar"
        } else if source.starts_with("Open Library") {
            "openlibrary"
        } else if source == "Aeon" || source == "Daily Nous" {
            "philosophy_rss"
        } else if source == "HackerNews" || source.starts_with("Hacker News") {
            "hackernews"
        } else if source == "PhilArchive" {
            "philarchive"
        } else if source.starts_with("Internet Archive") {
            "internet_archive"
        } else if source == "Custom RSS" || source.starts_with("RSS:") {
            "custom_rss"
        } else {
            source
        };
        self.recent_sources.push(normalized.to_string());
        if self.recent_sources.len() > 30 {
            self.recent_sources.remove(0);
        }
    }

    /// Obtenir le nombre de lectures d'un article.
    pub fn get_article_read_count(&self, title: &str) -> u32 {
        self.article_read_count.get(title).copied().unwrap_or(0)
    }

    // ═══════════════════════════════════════════════════════════════
    // Utilitaires — fonctions partagees par tous les sous-modules
    // ═══════════════════════════════════════════════════════════════

    /// Extrait un sujet de recherche d'une pensee en texte libre.
    ///
    /// Cherche des patterns comme "qu'est-ce que X", "curieuse a propos de X",
    /// "comprendre X", etc. Si aucun pattern n'est trouve et que la pensee
    /// est courte (10-80 chars), la pensee entiere est utilisee comme sujet.
    ///
    /// Retourne None si aucun sujet exploitable n'est extrait.
    fn extract_topic(thought: &str) -> Option<String> {
        let patterns = [
            "qu'est-ce que ", "c'est quoi ", "je me demande ce qu'est ",
            "curieuse à propos de ", "curious about ", "fascine dans ",
            "comprendre ", "savoir plus sur ", "nature de ",
            "qu'est-ce qu'", "en savoir plus sur ",
        ];
        let lower = thought.to_lowercase();
        for pat in patterns {
            if let Some(idx) = lower.find(pat) {
                let start = idx + pat.len();
                let rest = &thought[start..];
                let end = rest.find(['.', ',', '?', '!', '\n'])
                    .unwrap_or(rest.len())
                    .min(60);
                let topic = rest[..end].trim().to_string();
                if topic.len() >= 3 {
                    return Some(topic);
                }
            }
        }

        if thought.len() < 80 && thought.len() > 10 {
            return Some(thought.chars().take(60).collect::<String>().trim().to_string());
        }

        None
    }

    /// Retire les wrappers CDATA d'une chaine XML.
    pub fn strip_cdata(text: &str) -> String {
        text.trim()
            .strip_prefix("<![CDATA[")
            .and_then(|s| s.strip_suffix("]]>"))
            .unwrap_or(text)
            .trim()
            .to_string()
    }

    /// Retire les balises HTML d'une chaine — version amelioree.
    ///
    /// Algorithme :
    ///   1. Parcours caractere par caractere du HTML
    ///   2. Les blocs <script> et <style> sont ignores entierement
    ///   3. Un espace est ajoute apres chaque balise fermante (pour la lisibilite)
    ///   4. Les entites HTML courantes sont decodees (&amp; -> &, &nbsp; -> espace, etc.)
    ///   5. Les espaces multiples sont collapses en un seul espace
    ///
    /// ATTENTION : cette fonction ecrase TOUS les sauts de ligne en espaces.
    /// Pour preserver la structure des paragraphes, parser les balises <p>
    /// directement depuis le HTML AVANT d'appeler strip_html_tags() sur
    /// chaque paragraphe individuellement (voir extract_sep_content).
    pub fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let chars: Vec<char> = html.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            match chars[i] {
                '<' => {
                    // Detecter les blocs <script> et <style>
                    let remaining: String = chars[i..].iter().take(10).collect();
                    let remaining_lower = remaining.to_lowercase();
                    if remaining_lower.starts_with("<script")
                        || remaining_lower.starts_with("<style")
                    {
                        in_script = true;
                    }
                    if remaining_lower.starts_with("</script")
                        || remaining_lower.starts_with("</style")
                    {
                        in_script = false;
                    }
                    in_tag = true;
                }
                '>' => {
                    in_tag = false;
                    // Ajouter un espace apres les tags de bloc
                    if !in_script { result.push(' '); }
                }
                _ if !in_tag && !in_script => result.push(chars[i]),
                _ => {}
            }
            i += 1;
        }

        // Decoder les entites HTML courantes
        result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&#x27;", "'")
            .replace("&nbsp;", " ")
            .replace("&#x2F;", "/")
            // Nettoyer les espaces multiples
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Extrait le contenu textuel entre une paire de balises XML/HTML.
    ///
    /// Gere les balises avec attributs (ex: <title lang="fr">contenu</title>).
    /// Cherche la premiere occurrence de <tag...> et le </tag> correspondant.
    /// Utilise par : extract_sep_title(), extract_sep_sections(), philosophy_rss,
    /// wikipedia (extract_tag "extract"), arxiv (extract_tag "summary"), etc.
    pub fn extract_tag(text: &str, tag: &str) -> Option<String> {
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);
        let s_idx = text.find(&start)?;
        let s_content = text[s_idx..].find('>')? + s_idx + 1;
        let e_idx = text[s_content..].find(&end)? + s_content;
        Some(text[s_content..e_idx].to_string())
    }

    /// Encode une chaine pour utilisation comme parametre d'URL (percent-encoding).
    ///
    /// Caracteres non encodes (RFC 3986) : A-Z, a-z, 0-9, '-', '_', '.', '~'
    /// Les espaces deviennent '+' (convention formulaire)
    /// Tout le reste est encode en %XX (ex: 'é' -> %C3%A9)
    pub fn url_encode(s: &str) -> String {
        let mut result = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => result.push('+'),
                _ => {
                    result.push('%');
                    result.push_str(&format!("{:02X}", byte));
                }
            }
        }
        result
    }
}

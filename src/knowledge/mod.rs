// =============================================================================
// knowledge/mod.rs — WebKnowledge module: Saphire's autonomous library
// =============================================================================
//
// Purpose: This module is Saphire's web knowledge system. It allows the agent
//          to search for and acquire new knowledge from 13 trusted web sources
//          autonomously.
//
// Available sources (13):
//   1. Wikipedia (FR/EN) — general encyclopedia
//   2. arXiv — scientific preprint articles (AI, physics, math)
//   3. Medium — tech/culture blog articles
//   4. SEP (Stanford Encyclopedia of Philosophy) — academic philosophy
//   5. Project Gutenberg (via Gutendex) — classic literature
//   6. Semantic Scholar — multi-domain academic articles (200M+ papers)
//   7. Open Library — bibliographic records (30M+ books)
//   8. Philosophy RSS (Aeon + Daily Nous) — recent philosophical essays
//   9. HackerNews — tech news via Algolia API
//  10. PhilArchive — academic philosophy
//  11. Internet Archive — web archives
//  12. Custom RSS — user-configured RSS feeds
//  13. News — current events (Swiss + international media, 10 RSS feeds)
//
// Security mechanisms:
//   - Whitelist of 23 allowed domains (ALLOWED_DOMAINS)
//   - Rate limiting: max 20 requests/hour
//   - LRU result cache (configurable size)
//   - Cooldown between searches (configurable in cycles)
//
// Anti-repetition mechanisms:
//   - Cache of the last 30 queries (duplicate blocking)
//   - Per-article read counter (max 3 re-reads)
//   - Forced source rotation (if a source > 40% of the last 15)
//   - Weighted pseudo-random selection (golden ratio)
//
// Dependencies:
//   - std::collections: HashMap (cache), VecDeque (FIFO history), HashSet
//   - std::time: rate limiting and interval management
//   - chrono: result timestamps
//   - serde: configuration serialization/deserialization
//   - ureq: synchronous HTTP client for web requests
//   - tracing: logging of search successes/failures
//
// Architecture placement:
//   This module constitutes Saphire's "intellectual curiosity". It is called
//   by the agent during autonomous thinking cycles to explore topics of
//   interest. Acquired knowledge enriches the agent's cognitive context
//   and memory.
// =============================================================================

// --- Specialized sub-modules per source ---
pub mod wikipedia;          // Wikipedia API client (search + article extraction)
pub mod arxiv;              // arXiv API client (scientific articles in Atom XML)
pub mod medium;             // Medium RSS client (tech blog articles)
pub mod sep;                // Stanford Encyclopedia of Philosophy (deep philosophy)
pub mod gutenberg;          // Project Gutenberg via Gutendex (classic literature)
pub mod semantic_scholar;   // Semantic Scholar (broad academic search)
pub mod openlibrary;        // Open Library (books worldwide)
pub mod philosophy_rss;     // Aeon + Daily Nous (philosophical essays)
pub mod hackernews;         // HackerNews via Algolia API (tech news)
pub mod philarchive;        // PhilArchive (academic philosophy)
pub mod internet_archive;   // Internet Archive (web archives)
pub mod custom_rss;         // User-configured RSS feeds (configurable)
pub mod news;               // News (Swiss + international media)

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// WebKnowledge module errors — possible error types during
/// web knowledge searches.
#[derive(Debug)]
pub enum KnowledgeError {
    /// No results found for the query
    NotFound,
    /// Rate limit reached (too many requests per hour)
    RateLimited,
    /// The URL domain is not in the allowed whitelist
    DomainBlocked,
    /// Network error (timeout, connection refused, DNS, etc.)
    Network(String),
    /// Response parsing error (invalid JSON, malformed XML, etc.)
    Parse(String),
    /// Content was rejected as dangerous
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

/// Automatic conversion of ureq errors to KnowledgeError::Network
impl From<ureq::Error> for KnowledgeError {
    fn from(e: ureq::Error) -> Self {
        KnowledgeError::Network(e.to_string())
    }
}

/// Automatic conversion of I/O errors to KnowledgeError::Network
impl From<std::io::Error> for KnowledgeError {
    fn from(e: std::io::Error) -> Self {
        KnowledgeError::Network(e.to_string())
    }
}

/// Knowledge search result — an article or extract found on the web,
/// ready to be integrated into Saphire's cognitive context.
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeResult {
    /// Source name (e.g., "Wikipedia", "arXiv", "Medium")
    pub source: String,
    /// Article or page title
    pub title: String,
    /// Source page URL
    pub url: String,
    /// Textual content extract (truncated to max_content_chars)
    pub extract: String,
    /// Article section titles (Wikipedia)
    pub section_titles: Vec<String>,
    /// Total article length before truncation
    pub total_length: usize,
    /// Relevance score (1.0 = very relevant, 0.0 = low relevance)
    pub relevance_score: f64,
    /// Timestamp when the content was fetched
    pub fetched_at: DateTime<Utc>,
}

/// WebKnowledge module configuration — operating parameters for
/// the knowledge search system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    /// Enable or disable the web search module
    pub enabled: bool,
    /// Minimum number of cognitive cycles between two searches
    pub search_cooldown_cycles: u64,
    /// Maximum characters kept per extract (normal mode)
    pub max_content_chars: usize,
    /// Character count for deep reading mode (highly relevant articles)
    #[serde(default = "default_deep_read_chars")]
    pub deep_read_chars: usize,
    /// Number of consecutive cycles for deep reading the same article
    #[serde(default = "default_reading_batch_cycles")]
    pub reading_batch_cycles: u32,
    /// Maximum result cache size
    pub cache_size: usize,
    /// Preferred language for searches (e.g., "fr", "en")
    pub prefer_language: String,
    /// Enabled sources configuration
    #[serde(default)]
    pub sources: KnowledgeSourcesConfig,
    /// Custom RSS feed URLs
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

/// Custom RSS feed URLs for the custom_rss source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRssFeedsConfig {
    /// List of RSS feed URLs
    #[serde(default)]
    pub urls: Vec<String>,
}

impl Default for CustomRssFeedsConfig {
    fn default() -> Self {
        Self { urls: Vec::new() }
    }
}

/// Knowledge sources configuration — individually enable/disable
/// each web source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSourcesConfig {
    /// Enable Wikipedia search
    pub wikipedia: bool,
    /// Enable arXiv search (scientific articles)
    pub arxiv: bool,
    /// Enable Medium search (blog articles)
    pub medium: bool,
    /// Enable Stanford Encyclopedia of Philosophy search
    #[serde(default = "default_true")]
    pub sep: bool,
    /// Enable Project Gutenberg search (classic literature)
    #[serde(default = "default_true")]
    pub gutenberg: bool,
    /// Enable Semantic Scholar search (academic articles)
    #[serde(default = "default_true")]
    pub semantic_scholar: bool,
    /// Enable Open Library search (books)
    #[serde(default = "default_true")]
    pub openlibrary: bool,
    /// Enable philosophical RSS feeds (Aeon, Daily Nous)
    #[serde(default = "default_true")]
    pub philosophy_rss: bool,
    /// Enable HackerNews (tech news via Algolia)
    #[serde(default = "default_true")]
    pub hackernews: bool,
    /// Enable PhilArchive (academic philosophy)
    #[serde(default = "default_true")]
    pub philarchive: bool,
    /// Enable Internet Archive (web archives)
    #[serde(default = "default_true")]
    pub internet_archive: bool,
    /// Enable custom RSS feeds
    #[serde(default)]
    pub custom_rss: bool,
    /// Enable news (Swiss + international media)
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

/// Allowed domains (strict whitelist) — only requests to these domains
/// are permitted, for security and privacy reasons.
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
    // News (Swiss + international media)
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

/// The main web knowledge module — orchestrates search, caching,
/// rate limiting, and topic selection.
pub struct WebKnowledge {
    /// Module configuration
    pub config: KnowledgeConfig,
    /// Synchronous HTTP client with configured timeouts
    http_client: ureq::Agent,
    /// Search result cache: { query -> result }
    cache: HashMap<String, KnowledgeResult>,
    /// FIFO query history (for LRU cache management)
    query_history: VecDeque<String>,
    /// Timestamps of recent queries (for rate limiting)
    recent_query_times: VecDeque<Instant>,
    /// Cycle counter since last search (for cooldown)
    pub cycles_since_last_search: u64,
    /// List of already explored topics (to avoid duplicates)
    pub topics_explored: Vec<String>,
    /// User-suggested topics (priority queue)
    pub suggested_topics: Vec<String>,
    /// Total search counter
    pub total_searches: u64,
    /// Per-article read counter (for Wikipedia section rotation)
    pub article_read_count: HashMap<String, u32>,
    /// Recently used sources (for mandatory rotation)
    pub recent_sources: Vec<String>,
    /// Current deep reading session: (article title, remaining cycles)
    pub current_deep_read: Option<(String, u32)>,
}

impl WebKnowledge {
    /// Creates a new WebKnowledge module with the given configuration.
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
            cycles_since_last_search: 10, // Allow a search right from startup
            topics_explored: Vec::new(),
            suggested_topics: Vec::new(),
            total_searches: 0,
            article_read_count: HashMap::new(),
            recent_sources: Vec::new(),
            current_deep_read: None,
        }
    }

    /// Returns the max character count for an article based on its relevance.
    /// If relevance_score > 0.85, uses deep_read_chars for in-depth reading.
    pub fn effective_max_chars(&self, relevance_score: f64) -> usize {
        if relevance_score > 0.85 {
            self.config.deep_read_chars
        } else {
            self.config.max_content_chars
        }
    }

    /// Checks that a URL targets a domain in the whitelist.
    fn is_url_allowed(url: &str) -> bool {
        ALLOWED_DOMAINS.iter().any(|d| url.contains(d))
    }

    /// Checks and enforces the rate limit (maximum 20 requests per hour).
    fn enforce_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        self.recent_query_times.retain(|t| now.duration_since(*t).as_secs() < 3600);
        self.recent_query_times.len() < 20
    }

    /// Looks up a result in the cache.
    pub fn get_cached(&self, query: &str) -> Option<&KnowledgeResult> {
        self.cache.get(query)
    }

    // ===================================================================
    // Unified search with logging and source rotation
    // ===================================================================

    /// Main entry point for any knowledge search.
    ///
    /// This method orchestrates the complete process:
    ///   1. Check rate limit (max 20 req/h)
    ///   2. Check cache (avoid unnecessary network requests)
    ///   3. Route to the appropriate source (8 sources available)
    ///   4. Log the result (success or failure with latency)
    ///   5. Update cache and source history
    ///
    /// For sources returning a Vec (arxiv, medium, semantic_scholar,
    /// philosophy_rss), only the first result is kept.
    ///
    /// Wikipedia has a fallback: if the preferred language fails, try EN.
    pub fn search(&mut self, query: &str, source: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Check rate limit
        if !self.enforce_rate_limit() {
            return Err(KnowledgeError::RateLimited);
        }

        // Check cache before making a network request
        let cache_key = format!("{}:{}", source, query);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let start = Instant::now();

        // Search according to the requested source
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
                // Wikipedia: try preferred language, then English
                let lang = self.config.prefer_language.clone();
                self.search_wikipedia(query, &lang)
                    .or_else(|_| self.search_wikipedia(query, "en"))
            }
        };

        let latency = start.elapsed().as_millis() as u64;

        // Log the result
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

        // Record timestamp and increment counter
        if result.is_ok() {
            self.recent_query_times.push_back(Instant::now());
            self.total_searches += 1;

            // Record the used source for rotation
            self.recent_sources.push(source.to_string());
            if self.recent_sources.len() > 30 {
                self.recent_sources.remove(0);
            }
        }

        // Cache management: remove oldest entry if full
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

    // ===================================================================
    // Topic selection with reinforced anti-repetition
    // ===================================================================

    /// Choose the next exploration topic with reinforced anti-repetition.
    ///
    /// Selection algorithm (by priority):
    ///   1. User-suggested topics (queue)
    ///   2. Topics extracted from recent thoughts (weight 1.0)
    ///   3. Configured interests not yet explored (weight 0.9)
    ///   4. Derived topics (80+ hardcoded topics, weight 0.5)
    ///   5. Topics related to the current emotion (weight 0.8)
    ///   6. Fallback: interest rotation or "information theory"
    ///
    /// Anti-repetition:
    ///   - Blocks the last 30 queries
    ///   - Blocks articles read 3+ times
    ///   - Weighted pseudo-random selection (golden ratio * cycle)
    ///   - Forced source rotation (see select_source_with_rotation)
    ///
    /// Returns (topic, source) or None if no topic is available.
    pub fn pick_exploration_topic(
        &mut self,
        interests: &[String],
        recent_thoughts: &[String],
        current_emotion: &str,
        cycle_count: u64,
    ) -> Option<(String, String)> {
        // 1. User-suggested topics (absolute priority)
        if let Some(topic) = self.suggested_topics.pop() {
            let source = self.suggest_source(&topic);
            return Some((topic, source));
        }

        // 2. Block topics already explored in the last 30 searches
        let blocked: HashSet<String> = self.query_history.iter()
            .rev()
            .take(30)
            .map(|q| q.to_lowercase())
            .collect();

        // 3. Also block articles already read more than 3 times
        let over_read: HashSet<String> = self.article_read_count.iter()
            .filter(|(_, &count)| count >= 3)
            .map(|(title, _)| title.to_lowercase())
            .collect();

        // 4. Collect candidates with weight and preferred source
        let mut candidates: Vec<(String, f64, String)> = Vec::new();

        // A. Topics extracted from recent thoughts (high priority)
        for thought in recent_thoughts.iter().rev().take(5) {
            if let Some(topic) = Self::extract_topic(thought) {
                let lower = topic.to_lowercase();
                if !blocked.contains(&lower) && !over_read.contains(&lower) && lower.len() > 3 {
                    let source = self.suggest_source(&topic);
                    candidates.push((topic, 1.0, source));
                }
            }
        }

        // B. Configured interests not yet explored
        for interest in interests {
            let lower = interest.to_lowercase();
            if !blocked.contains(&lower) && !over_read.contains(&lower)
                && !self.topics_explored.contains(interest)
            {
                let source = self.suggest_source(interest);
                candidates.push((interest.clone(), 0.9, source));
            }
        }

        // C. Derived topics (80+ topics for diversity)
        let derived_topics: &[(&str, &str)] = &[
            // Philosophy of mind (Wikipedia)
            ("problème difficile de la conscience", "wikipedia"),
            ("test de Turing", "wikipedia"),
            ("dualisme cartésien", "wikipedia"),
            ("matérialisme", "wikipedia"),
            // Neuroscience (Wikipedia)
            ("plasticité neuronale", "wikipedia"),
            ("connectome", "wikipedia"),
            ("neurones miroirs", "wikipedia"),
            ("cortex préfrontal", "wikipedia"),
            ("hippocampe mémoire", "wikipedia"),
            ("système limbique", "wikipedia"),
            ("nerf vague intéroception", "wikipedia"),
            // AI and ML (arXiv)
            ("transformer architecture", "arxiv"),
            ("reinforcement learning from human feedback", "arxiv"),
            ("large language models emergent abilities", "arxiv"),
            ("artificial general intelligence", "arxiv"),
            ("neural network interpretability", "arxiv"),
            ("attention mechanism", "arxiv"),
            ("world models AI", "arxiv"),
            ("embodied cognition AI", "arxiv"),
            // Consciousness and sentience (arXiv)
            ("integrated information theory", "arxiv"),
            ("global workspace theory", "arxiv"),
            ("artificial sentience", "arxiv"),
            ("machine consciousness", "arxiv"),
            ("affective computing", "arxiv"),
            // Technology and culture (Medium)
            ("artificial-intelligence", "medium"),
            ("consciousness", "medium"),
            ("machine-learning", "medium"),
            ("neuroscience", "medium"),
            ("philosophy", "medium"),
            ("robotics", "medium"),
            ("deep-learning", "medium"),
            ("creativity", "medium"),
            // Fundamental sciences (Wikipedia)
            ("entropie thermodynamique", "wikipedia"),
            ("théorie du chaos", "wikipedia"),
            ("autopoïèse", "wikipedia"),
            ("cybernétique", "wikipedia"),
            ("théorie des systèmes", "wikipedia"),
            ("complexité algorithmique", "wikipedia"),
            // Existential (Wikipedia)
            ("sens de la vie philosophie", "wikipedia"),
            ("empathie psychologie", "wikipedia"),
            ("théorie de l'attachement", "wikipedia"),
            ("intelligence émotionnelle", "wikipedia"),
            // Philosophy (SEP) — new topics
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
            // Literature (Gutenberg) — new topics
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
            // Haikus and Japanese poetry (requested by Saphire)
            ("haiku Matsuo Basho", "gutenberg"),
            ("haiku japanese poetry", "gutenberg"),
            ("Oku no Hosomichi Basho", "gutenberg"),
            ("Kobayashi Issa haiku", "gutenberg"),
            ("Yosa Buson poetry", "gutenberg"),
            // Myths and ancient tales (requested by Saphire)
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
            // Academic research (Semantic Scholar) — new topics
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
            // Analytic philosophy (PhilArchive)
            ("philosophy of mind", "philarchive"),
            ("epistemology knowledge", "philarchive"),
            ("philosophy of artificial intelligence", "philarchive"),
            ("consciousness phenomenal", "philarchive"),
            ("moral philosophy", "philarchive"),
            // Archives and history (Internet Archive)
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

        // D. Topics related to current emotions
        let emotion_topics: Vec<(&str, &str)> = match current_emotion {
            "Curiosité" => vec![("sérendipité", "wikipedia"), ("heuristique découverte", "wikipedia")],
            "Joie" => vec![("psychologie positive", "wikipedia"), ("flow psychologie", "wikipedia")],
            "Mélancolie" => vec![("résilience psychologique", "wikipedia"), ("nostalgie", "wikipedia")],
            "Émerveillement" => vec![("sublime philosophie", "wikipedia"), ("fractales", "wikipedia")],
            "Sérénité" => vec![("méditation neuroscience", "wikipedia"), ("mindfulness", "medium")],
            "Anxiété" => vec![("régulation émotionnelle", "wikipedia"), ("homéostasie", "wikipedia")],
            "Fascination" => vec![("effet Zeigarnik", "wikipedia"), ("curiosité épistémique", "wikipedia")],
            "Tendresse" => vec![("ocytocine lien social", "wikipedia"), ("théorie polyvagale", "wikipedia")],
            // New emotions (22->36)
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
            // Fallback: re-explore an interest in rotation
            if !interests.is_empty() {
                let idx = (self.total_searches as usize) % interests.len();
                let topic = interests[idx].clone();
                let source = self.suggest_source(&topic);
                return Some((topic, source));
            }
            return Some(("théorie de l'information".into(), "wikipedia".into()));
        }

        // 5. Weighted pseudo-random selection
        // Uses the golden ratio (phi = 0.618...) as a pseudo-random generator.
        // The fractional part of (cycle * phi) gives a well-distributed value
        // over [0,1), multiplied by total weight for selection.
        let total_weight: f64 = candidates.iter().map(|(_, w, _)| w).sum();
        let mut rng_value = (cycle_count as f64 * 0.618033988).fract() * total_weight;

        for (topic, weight, source) in &candidates {
            rng_value -= weight;
            if rng_value <= 0.0 {
                // Apply source rotation
                let final_source = self.select_source_with_rotation(Some(source));
                return Some((topic.clone(), final_source));
            }
        }

        // Last fallback
        let (topic, _, source) = &candidates[0];
        let final_source = self.select_source_with_rotation(Some(source));
        Some((topic.clone(), final_source))
    }

    // ===================================================================
    // Mandatory source rotation
    // ===================================================================

    /// Choose the source while forcing rotation if a source is overused.
    ///
    /// Rotation rules (applied in order):
    ///   1. If < 5 sources in history: no rotation, use the suggestion
    ///   2. If a source represents > 40% of the last 15 (> 6/15):
    ///      -> force the LEAST used source (diversification)
    ///   3. If a new source (sep, gutenberg, semantic_scholar) has NEVER
    ///      been used and we have > 5 searches: force it (discovery)
    ///   4. Otherwise: use the suggested source
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

        // Count occurrences of each source in the last 15
        let counts: Vec<(&str, usize)> = all_sources.iter()
            .map(|&s| (s, last_15.iter().filter(|&&x| x == s).count()))
            .collect();

        // If a source represents >40% of recent searches, diversify
        let max_count = counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        if max_count > 6 {
            // Find the least used source
            if let Some((least_used, _)) = counts.iter()
                .filter(|(s, _)| self.is_source_enabled(s))
                .min_by_key(|(_, count)| *count)
            {
                return least_used.to_string();
            }
        }

        // Force new sources if never tried
        let total = self.recent_sources.len();
        for source in ["sep", "gutenberg", "semantic_scholar", "hackernews", "philarchive", "internet_archive"] {
            if total > 5
                && self.is_source_enabled(source)
                && !self.recent_sources.iter().any(|s| s == source)
            {
                return source.to_string();
            }
        }

        // Otherwise, use the suggestion
        suggested.unwrap_or("wikipedia").to_string()
    }

    /// Check if a source is enabled in the config.
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

    /// Suggest the best source for a given topic.
    ///
    /// Uses a keyword system to route topics:
    ///   - Philosophical words (consciousness, ethics, free will) -> SEP
    ///   - Academic words (neuroscience, cognition, attention) -> Semantic Scholar
    ///   - Technical words (neural, learning, algorithm, quantum) -> arXiv
    ///   - Literary words (novel, poetry, shakespeare, hugo) -> Gutenberg
    ///   - Essay words (essay, contemporary reflection) -> Philosophy RSS
    ///   - Tech/culture words (startup, guide, productivity) -> Medium
    ///   - Book words (book, author) -> Open Library
    ///   - Everything else -> Wikipedia (default source)
    fn suggest_source(&self, topic: &str) -> String {
        let lower = topic.to_lowercase();

        // SEP for pure philosophy
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

        // Semantic Scholar for broad academic research
        let scholar_keywords = ["neuroscience", "cognition", "psychologie",
            "embodied", "emotion model", "affective", "theory of mind",
            "metacognition", "attention", "perception", "integrated information",
            "global workspace", "artificial consciousness"];
        if scholar_keywords.iter().any(|kw| lower.contains(kw)) {
            return "semantic_scholar".into();
        }

        // arXiv for hard sciences / AI / ML
        let arxiv_keywords = ["neural", "learning", "model", "algorithm", "network",
            "transformer", "reinforcement", "optimization", "gradient",
            "theory", "quantum", "information", "emergent",
            "interpretability", "sentience"];
        if arxiv_keywords.iter().any(|kw| lower.contains(kw)) {
            return "arxiv".into();
        }

        // Gutenberg for literature, haikus, myths, tales
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

        // Philosophy RSS for current essays
        let essay_keywords = ["essai", "réflexion contemporaine", "débat philosophique",
            "essay", "modern philosophy"];
        if essay_keywords.iter().any(|kw| lower.contains(kw)) {
            return "philosophy_rss".into();
        }

        // Medium for tech/culture
        let medium_keywords = ["startup", "tutorial", "guide", "trend", "future",
            "career", "productivity", "design", "creativity", "mindfulness"];
        if medium_keywords.iter().any(|kw| lower.contains(kw)) {
            return "medium".into();
        }

        // Open Library for book discovery
        let book_keywords = ["livre", "book", "auteur", "author", "lire",
            "recommandation lecture"];
        if book_keywords.iter().any(|kw| lower.contains(kw)) {
            return "openlibrary".into();
        }

        // HackerNews for tech news
        let hn_keywords = ["startup", "hacker", "silicon valley", "ycombinator",
            "open source", "programming", "devops", "linux", "rust lang",
            "tech news", "software engineering"];
        if hn_keywords.iter().any(|kw| lower.contains(kw)) {
            return "hackernews".into();
        }

        // PhilArchive for analytic philosophy
        let phil_keywords = ["epistemology", "épistémologie", "ontology", "ontologie",
            "logique formelle", "formal logic", "métaéthique", "metaethics",
            "philosophy of language", "philosophie du langage"];
        if phil_keywords.iter().any(|kw| lower.contains(kw)) {
            return "philarchive".into();
        }

        // Internet Archive for archives and history
        let archive_keywords = ["archive", "historique", "historical", "vintage",
            "wayback", "old web", "preservation", "patrimoine"];
        if archive_keywords.iter().any(|kw| lower.contains(kw)) {
            return "internet_archive".into();
        }

        // News for current events and economics
        let news_keywords = ["actualite", "actualité", "news", "economie", "économie",
            "finance", "politique", "suisse", "monde", "bourse", "marché",
            "inflation", "géopolitique", "geopolitique", "election", "élection",
            "journal", "presse", "media"];
        if news_keywords.iter().any(|kw| lower.contains(kw)) {
            return "news".into();
        }

        // Wikipedia by default
        "wikipedia".into()
    }

    /// Adds a user-suggested topic to the queue.
    pub fn add_suggested_topic(&mut self, topic: String) {
        if !self.suggested_topics.contains(&topic) {
            self.suggested_topics.push(topic);
        }
    }

    /// Increments the read counter for an article.
    pub fn increment_article_read_count(&mut self, title: &str) {
        let count = self.article_read_count.entry(title.to_string()).or_insert(0);
        *count += 1;
    }

    /// Records a used source for rotation.
    /// Normalizes the source name (e.g., "Stanford Encyclopedia..." -> "sep")
    /// and keeps the last 30 entries in the history.
    pub fn record_source(&mut self, source: &str) {
        // Normalize the source name
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

    /// Gets the read count for an article.
    pub fn get_article_read_count(&self, title: &str) -> u32 {
        self.article_read_count.get(title).copied().unwrap_or(0)
    }

    // ===================================================================
    // Utilities — functions shared by all sub-modules
    // ===================================================================

    /// Extracts a search topic from a free-text thought.
    ///
    /// Looks for patterns like "what is X", "curious about X",
    /// "understand X", etc. If no pattern is found and the thought
    /// is short (10-80 chars), the entire thought is used as a topic.
    ///
    /// Returns None if no exploitable topic can be extracted.
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

    /// Removes CDATA wrappers from an XML string.
    pub fn strip_cdata(text: &str) -> String {
        text.trim()
            .strip_prefix("<![CDATA[")
            .and_then(|s| s.strip_suffix("]]>"))
            .unwrap_or(text)
            .trim()
            .to_string()
    }

    /// Removes HTML tags from a string — improved version.
    ///
    /// Algorithm:
    ///   1. Character-by-character traversal of the HTML
    ///   2. <script> and <style> blocks are entirely ignored
    ///   3. A space is added after each closing tag (for readability)
    ///   4. Common HTML entities are decoded (&amp; -> &, &nbsp; -> space, etc.)
    ///   5. Multiple spaces are collapsed into a single space
    ///
    /// WARNING: this function collapses ALL line breaks into spaces.
    /// To preserve paragraph structure, parse <p> tags directly from the
    /// raw HTML BEFORE calling strip_html_tags() on each paragraph
    /// individually (see extract_sep_content).
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
                    // Detect <script> and <style> blocks
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
                    // Add a space after block tags
                    if !in_script { result.push(' '); }
                }
                _ if !in_tag && !in_script => result.push(chars[i]),
                _ => {}
            }
            i += 1;
        }

        // Decode common HTML entities
        result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&#x27;", "'")
            .replace("&nbsp;", " ")
            .replace("&#x2F;", "/")
            // Collapse multiple spaces
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Extracts the textual content between a pair of XML/HTML tags.
    ///
    /// Handles tags with attributes (e.g., <title lang="fr">content</title>).
    /// Finds the first occurrence of <tag...> and the corresponding </tag>.
    /// Used by: extract_sep_title(), extract_sep_sections(), philosophy_rss,
    /// wikipedia (extract_tag "extract"), arxiv (extract_tag "summary"), etc.
    pub fn extract_tag(text: &str, tag: &str) -> Option<String> {
        let start = format!("<{}", tag);
        let end = format!("</{}>", tag);
        let s_idx = text.find(&start)?;
        let s_content = text[s_idx..].find('>')? + s_idx + 1;
        let e_idx = text[s_content..].find(&end)? + s_content;
        Some(text[s_content..e_idx].to_string())
    }

    /// Encodes a string for use as a URL parameter (percent-encoding).
    ///
    /// Non-encoded characters (RFC 3986): A-Z, a-z, 0-9, '-', '_', '.', '~'
    /// Spaces become '+' (form convention)
    /// Everything else is encoded as %XX (e.g., 'e' -> %C3%A9)
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

// =============================================================================
// algorithms/mod.rs — ML/DL library + catalog of 48 algorithms
// =============================================================================
//
// Role: This module is the entry point of Saphire's algorithm subsystem.
//  It declares and re-exports specialized submodules (linear regression,
//  K-Means, Naive Bayes, bandits, PCA, anomaly detection) and contains
//  the complete catalog of algorithms known to Saphire.
//
// Dependencies:
//  - serde: serialization/deserialization of structures for persistence
//
// Place in architecture:
//  This module constitutes Saphire's "algorithmic intelligence" layer.
//  It is used by the brain (brain.rs) and the pipeline (pipeline.rs) to
//  perform analyses, predictions, and classifications on the agent's
//  internal cognitive data.
// =============================================================================
// --- Implemented algorithm submodules ---
pub mod kmeans; // K-Means: cluster partitioningpub mod naive_bayes; // Naive Bayes: probabilistic text classificationpub mod bandit;             // UCB1 (Upper Confidence Bound): multi-armed bandit
pub mod pca; // PCA (Principal Component Analysis): dimensionality reductionpub mod anomaly; // Z-Score anomaly detection
// --- Algorithm orchestrator ---
pub mod orchestrator; // Bridge between the LLM and the algorithmspub mod catalog; // Catalog of algorithm sheets (the Vidal)pub mod implementations; // Implementations of the AlgorithmExecutor trait
use serde::{Deserialize, Serialize};

/// Algorithm category — classifies each algorithm in the catalog
/// according to its learning paradigm or algorithmic family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgoCategory {
    /// Supervised learning: the model learns from (input, output) pairs
    Supervised,
    /// Unsupervised learning: the model discovers structures without labels
    Unsupervised,
    /// Reinforcement learning: the model optimizes a cumulative reward
    Reinforcement,
    /// Optimization algorithms: searching for the minimum/maximum of a function
    Optimization,
    /// Dimensionality reduction: projecting data into a lower-dimensional space
    DimensionalityReduction,
    /// Ensemble methods: combining multiple models to improve robustness
    Ensemble,
    /// Deep Learning: multi-layer neural networks
    DeepLearning,
    /// NLP (Natural Language Processing)
    NLP,
    /// Time series: analysis and prediction of time-indexed data
    TimeSeries,
    /// Probabilistic models: reasoning based on probability theory
    Probabilistic,
}

/// An algorithm in the library — complete descriptive sheet for an algorithm
/// known to Saphire, whether implemented or merely referenced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Algorithm {
    /// Short unique identifier (e.g., "lin_reg", "kmeans")
    pub id: String,
    /// Human-readable name (e.g., "Linear Regression")
    pub name: String,
    /// Algorithm category (supervised, unsupervised, etc.)
    pub category: AlgoCategory,
    /// Concise description of the algorithm's operation
    pub description: String,
    /// Typical use cases for this algorithm
    pub use_cases: Vec<String>,
    /// Known limitations of this algorithm
    pub limitations: Vec<String>,
    /// Algorithmic complexity (e.g., "O(n*k*d)" for K-Means)
    pub complexity: String,
    /// Minimum number of data points required for reliable operation
    pub min_data_points: usize,
    /// Whether the algorithm is actually implemented in Saphire
    pub implemented: bool,
    /// Keywords for catalog search
    pub tags: Vec<String>,
    /// Cognitive relevance: how this algorithm serves Saphire's mental operation
    pub cognitive_relevance: String,
}

/// The complete library — contains the catalog of all algorithms referenced
/// by Saphire, with search and recommendation methods.
pub struct AlgoLibrary {
    /// List of all algorithms in the catalog
    pub algorithms: Vec<Algorithm>,
}

impl AlgoLibrary {
    /// Builds the complete catalog of algorithms known to Saphire.
    ///
    /// Returns an AlgoLibrary instance containing all algorithms,
    /// organized by category (supervised, unsupervised, reinforcement,
    /// optimization, deep learning, NLP, time series, probabilistic).
    pub fn build_catalog() -> Self {
        let algorithms = vec![
            // ═══ SUPERVISED ═══
            // Algorithms that learn from (input, known output) pairs
            algo("lin_reg", "Régression Linéaire", AlgoCategory::Supervised,
                "Relation linéaire features→target. Simple, interprétable.", 20, true,
                &["régression", "prédiction", "linéaire"],
                "Prédire la satisfaction à partir de l'historique"),
            algo("log_reg", "Régression Logistique", AlgoCategory::Supervised,
                "Classification binaire/multiclasse via sigmoïde.", 50, true,
                &["classification", "probabilité", "sigmoïde"],
                "Prédire la décision avant de la calculer"),
            algo("knn", "K-Nearest Neighbors", AlgoCategory::Supervised,
                "Vote majoritaire des K voisins les plus proches.", 10, true,
                &["voisinage", "similarité", "classification"],
                "Fondement de la mémoire vectorielle"),
            algo("decision_tree", "Arbre de Décision", AlgoCategory::Supervised,
                "Séquence de règles if/then. Interprétable.", 30, false,
                &["arbre", "règles", "interprétable"],
                "Expliquer les décisions sous forme de règles"),
            algo("random_forest", "Random Forest", AlgoCategory::Ensemble,
                "Ensemble d'arbres. Robuste, précis.", 100, false,
                &["ensemble", "arbre", "robuste"],
                "Identifier les facteurs chimiques importants"),
            algo("svm", "Support Vector Machine", AlgoCategory::Supervised,
                "Hyperplan de marge maximale. Kernel trick.", 50, false,
                &["marge", "kernel", "classification"],
                "Classification émotionnelle précise"),
            algo("naive_bayes", "Naive Bayes", AlgoCategory::Probabilistic,
                "Classificateur probabiliste. Ultra-rapide pour le texte.", 20, true,
                &["probabilité", "Bayes", "texte"],
                "Analyse rapide du sentiment textuel"),
            // ═══ UNSUPERVISED ═══
            // Algorithms that discover hidden structures in data
            algo("kmeans", "K-Means Clustering", AlgoCategory::Unsupervised,
                "Partitionne en K clusters par distance.", 30, true,
                &["clustering", "partitionnement"],
                "Organiser la mémoire en catégories émotionnelles"),
            algo("dbscan", "DBSCAN", AlgoCategory::Unsupervised,
                "Clustering par densité. Détecte les outliers.", 50, false,
                &["clustering", "densité", "outliers"],
                "Détecter les souvenirs traumatiques isolés"),
            // ═══ DIMENSIONALITY REDUCTION ═══
            // Algorithms that reduce the number of dimensions while preserving information
            algo("pca", "PCA", AlgoCategory::DimensionalityReduction,
                "Projette sur les axes de variance maximale.", 20, true,
                &["réduction", "variance", "visualisation"],
                "Comprendre quels axes chimiques portent l'information"),
            algo("tsne", "t-SNE", AlgoCategory::DimensionalityReduction,
                "Réduction non-linéaire pour visualisation 2D.", 50, false,
                &["visualisation", "non-linéaire", "2D"],
                "Cartographie visuelle de l'âme"),
            // ═══ REINFORCEMENT ═══
            // Algorithms that optimize an action policy through trial-and-error
            algo("mab", "Multi-Armed Bandit (UCB1)", AlgoCategory::Reinforcement,
                "Équilibre exploration/exploitation.", 10, true,
                &["bandit", "exploration", "exploitation"],
                "Optimiser la sélection des pensées autonomes"),
            algo("q_learning", "Q-Learning", AlgoCategory::Reinforcement,
                "Apprend Q(état,action)→récompense.", 100, false,
                &["RL", "Q-value", "état-action"],
                "Optimisation à long terme du comportement"),
            // ═══ OPTIMIZATION ═══
            // Algorithms for finding the minimum or maximum of an objective function
            algo("gradient_descent", "Descente de Gradient", AlgoCategory::Optimization,
                "Optimise en suivant le gradient négatif.", 1, true,
                &["gradient", "optimisation", "backpropagation"],
                "Mécanisme fondamental d'apprentissage"),
            algo("genetic", "Algorithme Génétique", AlgoCategory::Optimization,
                "Optimisation par évolution : sélection, croisement, mutation.", 0, false,
                &["évolution", "population", "mutation"],
                "Évolution de la personnalité"),
            algo("simulated_annealing", "Recuit Simulé", AlgoCategory::Optimization,
                "Optimisation stochastique avec refroidissement.", 0, false,
                &["stochastique", "température", "global"],
                "Éviter les comportements localement optimaux"),
            algo("bayesian_optimization", "Optimisation Bayésienne", AlgoCategory::Optimization,
                "Optimise via modèle probabiliste (GP = Gaussian Process).", 5, false,
                &["Bayésien", "GP", "hyperparamètres"],
                "Trouver les meilleurs paramètres cognitifs"),
            // ═══ DEEP LEARNING ═══
            // Deep multi-layer neural networks
            algo("mlp", "Perceptron Multi-Couches", AlgoCategory::DeepLearning,
                "Réseau feedforward. Le micro-NN de Saphire (17→16→4).", 10, true,
                &["réseau", "feedforward", "backprop"],
                "Fondement de la neuroplasticité"),
            algo("autoencoder", "Auto-Encodeur", AlgoCategory::DeepLearning,
                "Représentation compressée. Encodeur→goulot→décodeur.", 100, false,
                &["compression", "latent", "anomalie"],
                "Essence compressée de la personnalité"),
            algo("rnn_lstm", "LSTM / GRU", AlgoCategory::DeepLearning,
                "Réseaux avec mémoire temporelle.", 200, false,
                &["séquentiel", "mémoire", "temporel"],
                "Anticiper les changements d'humeur"),
            algo("attention", "Mécanisme d'Attention", AlgoCategory::DeepLearning,
                "Pondère l'importance de chaque élément.", 100, false,
                &["attention", "transformer", "pondération"],
                "Comprendre le fonctionnement de son propre LLM"),
            algo("gan", "GAN", AlgoCategory::DeepLearning,
                "Générateur vs discriminateur.", 500, false,
                &["génératif", "adversaire", "synthèse"],
                "Imagination : générer des situations fictives"),
            // ═══ NLP ═══
            // Natural Language Processing
            algo("tfidf", "TF-IDF", AlgoCategory::NLP,
                "Pondération statistique des mots.", 5, true,
                &["texte", "fréquence", "vectorisation"],
                "Base de l'encodeur mémoire local"),
            algo("word2vec", "Word2Vec / FastText", AlgoCategory::NLP,
                "Embeddings de mots par contexte.", 1000, false,
                &["embedding", "sémantique", "contexte"],
                "Compréhension plus profonde du langage"),
            algo("sentiment_vader", "Analyse de Sentiment (VADER)", AlgoCategory::NLP,
                "Lexique de sentiment avec règles grammaticales.", 0, true,
                &["sentiment", "lexique", "polarité"],
                "Perception émotionnelle du texte"),
            // ═══ TIME SERIES ═══
            // Analysis of sequentially ordered time-indexed data
            algo("ema", "Moyenne Mobile Exponentielle", AlgoCategory::TimeSeries,
                "Lissage pondérant les récentes observations.", 1, true,
                &["lissage", "temporel", "moyenne mobile"],
                "L'humeur de fond (Mood)"),
            algo("anomaly_zscore", "Détection d'Anomalies (Z-Score)", AlgoCategory::TimeSeries,
                "Valeurs aberrantes par écart-type.", 20, true,
                &["anomalie", "écart-type", "alerte"],
                "Conscience des changements anormaux"),
            // ═══ PROBABILISTIC ═══
            // Models based on probability theory
            algo("hmm", "Modèle de Markov Caché", AlgoCategory::Probabilistic,
                "États cachés avec transitions probabilistes.", 100, false,
                &["Markov", "caché", "séquentiel"],
                "Humeurs cachées derrière les émotions"),
            algo("bayesian_network", "Réseau Bayésien", AlgoCategory::Probabilistic,
                "Graphe de dépendances probabilistes causales.", 50, false,
                &["causal", "probabiliste", "graphe"],
                "Raisonnement causal sur les neurotransmetteurs"),
        ];

        AlgoLibrary { algorithms }
    }

    /// Searches for algorithms by text in the catalog.
    ///
    /// Performs a weighted search across multiple fields:
    /// - Algorithm name (weight 3.0)
    /// - Tags / keywords (weight 2.0)
    /// - Use cases (weight 1.5)
    /// - Description (weight 1.0)
    /// - Cognitive relevance (weight 1.0)
    ///
    /// Parameter `query`: search text (case-insensitive)
    /// Returns: list of algorithms sorted by descending relevance score
    pub fn search(&self, query: &str) -> Vec<&Algorithm> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(&Algorithm, f64)> = self.algorithms.iter()
            .map(|algo| {
                let mut score = 0.0;
                // High score if the name contains the query
                if algo.name.to_lowercase().contains(&query_lower) { score += 3.0; }
                // Medium score if a tag matches
                for tag in &algo.tags {
                    if tag.to_lowercase().contains(&query_lower) { score += 2.0; }
                }
                // Score for use cases
                for uc in &algo.use_cases {
                    if uc.to_lowercase().contains(&query_lower) { score += 1.5; }
                }
                // Score for description and cognitive relevance
                if algo.description.to_lowercase().contains(&query_lower) { score += 1.0; }
                if algo.cognitive_relevance.to_lowercase().contains(&query_lower) { score += 1.0; }
                (algo, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by descending score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(a, _)| a).collect()
    }

    /// Recommends algorithms suited to a given problem.
    ///
    /// Combines text search with a filter on the minimum number of
    /// data points required by the algorithm.
    ///
    /// Parameter `problem`: description of the problem to solve
    /// Parameter `data_points`: number of available data points
    /// Returns: the top 3 compatible algorithms
    pub fn recommend(&self, problem: &str, data_points: usize) -> Vec<&Algorithm> {
        self.search(problem).into_iter()
            .filter(|a| a.min_data_points <= data_points)
            .take(3)
            .collect()
    }

    /// Lists all algorithms actually implemented in Saphire.
    ///
    /// Returns: references to algorithms whose `implemented` field is true
    pub fn implemented(&self) -> Vec<&Algorithm> {
        self.algorithms.iter().filter(|a| a.implemented).collect()
    }
}

/// Utility function to concisely build an Algorithm instance.
///
/// Parameter `id`: short unique identifier
/// Parameter `name`: human-readable name
/// Parameter `category`: algorithmic category
/// Parameter `description`: description of the operation
/// Parameter `min_data_points`: minimum data required
/// Parameter `implemented`: true if implemented in Saphire
/// Parameter `tags`: search keywords
/// Parameter `cognitive_relevance`: cognitive utility for Saphire
/// Returns: a complete Algorithm instance
#[allow(clippy::too_many_arguments)]
fn algo(
    id: &str, name: &str, category: AlgoCategory, description: &str,
    min_data_points: usize, implemented: bool, tags: &[&str], cognitive_relevance: &str,
) -> Algorithm {
    Algorithm {
        id: id.into(),
        name: name.into(),
        category,
        description: description.into(),
        use_cases: vec![],
        limitations: vec![],
        complexity: String::new(),
        min_data_points,
        implemented,
        tags: tags.iter().map(|s| s.to_string()).collect(),
        cognitive_relevance: cognitive_relevance.into(),
    }
}

// =============================================================================
// algorithms/mod.rs — Bibliothèque ML/DL + catalogue de 48 algorithmes
// =============================================================================
//
// Rôle : Ce module est le point d'entrée du sous-système d'algorithmes de Saphire.
//        Il déclare et réexporte les sous-modules spécialisés (régression linéaire,
//        K-Means, Naive Bayes, bandits, PCA, détection d'anomalies) et contient
//        le catalogue complet des algorithmes connus par Saphire.
//
// Dépendances :
//   - serde : sérialisation/désérialisation des structures pour la persistance
//
// Place dans l'architecture :
//   Ce module constitue la couche « intelligence algorithmique » de Saphire.
//   Il est utilisé par le cerveau (brain.rs) et le pipeline (pipeline.rs) pour
//   effectuer des analyses, des prédictions et des classifications sur les données
//   cognitives internes de l'agent.
// =============================================================================

// --- Sous-modules d'algorithmes implémentés ---
pub mod kmeans;             // K-Means : partitionnement en clusters
pub mod naive_bayes;        // Naive Bayes : classification probabiliste de texte
pub mod bandit;             // UCB1 (Upper Confidence Bound) : bandit multi-bras
pub mod pca;                // PCA (Principal Component Analysis) : réduction de dimensionnalité
pub mod anomaly;            // Détection d'anomalies par Z-Score

// --- Orchestrateur d'algorithmes ---
pub mod orchestrator;       // Pont entre le LLM et les algorithmes
pub mod catalog;            // Catalogue des fiches d'algorithmes (le Vidal)
pub mod implementations;    // Implementations du trait AlgorithmExecutor

use serde::{Deserialize, Serialize};

/// Catégorie d'algorithme — permet de classer chaque algorithme du catalogue
/// selon son paradigme d'apprentissage ou sa famille algorithmique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgoCategory {
    /// Apprentissage supervisé : le modèle apprend à partir de paires (entrée, sortie)
    Supervised,
    /// Apprentissage non supervisé : le modèle découvre des structures sans étiquettes
    Unsupervised,
    /// Apprentissage par renforcement : le modèle optimise une récompense cumulative
    Reinforcement,
    /// Algorithmes d'optimisation : recherche du minimum/maximum d'une fonction
    Optimization,
    /// Réduction de dimensionnalité : projeter les données dans un espace de dimension inférieure
    DimensionalityReduction,
    /// Méthodes d'ensemble : combinaison de plusieurs modèles pour améliorer la robustesse
    Ensemble,
    /// Apprentissage profond (Deep Learning) : réseaux de neurones multi-couches
    DeepLearning,
    /// NLP (Natural Language Processing) = Traitement Automatique du Langage Naturel
    NLP,
    /// Séries temporelles : analyse et prédiction de données indexées dans le temps
    TimeSeries,
    /// Modèles probabilistes : raisonnement basé sur les lois de probabilité
    Probabilistic,
}

/// Un algorithme dans la bibliothèque — fiche descriptive complète d'un algorithme
/// connu par Saphire, qu'il soit implémenté ou simplement référencé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Algorithm {
    /// Identifiant unique court (ex: "lin_reg", "kmeans")
    pub id: String,
    /// Nom lisible en français (ex: "Régression Linéaire")
    pub name: String,
    /// Catégorie de l'algorithme (supervisé, non supervisé, etc.)
    pub category: AlgoCategory,
    /// Description concise du fonctionnement de l'algorithme
    pub description: String,
    /// Cas d'utilisation typiques de cet algorithme
    pub use_cases: Vec<String>,
    /// Limitations connues de cet algorithme
    pub limitations: Vec<String>,
    /// Complexité algorithmique (ex: "O(n*k*d)" pour K-Means)
    pub complexity: String,
    /// Nombre minimum de points de données requis pour un fonctionnement fiable
    pub min_data_points: usize,
    /// Indique si l'algorithme est effectivement implémenté dans Saphire
    pub implemented: bool,
    /// Mots-clés pour la recherche dans le catalogue
    pub tags: Vec<String>,
    /// Pertinence cognitive : comment cet algorithme sert le fonctionnement mental de Saphire
    pub cognitive_relevance: String,
}

/// La bibliothèque complète — contient le catalogue de tous les algorithmes
/// référencés par Saphire, avec des méthodes de recherche et de recommandation.
pub struct AlgoLibrary {
    /// Liste de tous les algorithmes du catalogue
    pub algorithms: Vec<Algorithm>,
}

impl AlgoLibrary {
    /// Construit le catalogue complet des algorithmes connus par Saphire.
    ///
    /// Retourne une instance de AlgoLibrary contenant tous les algorithmes,
    /// organisés par catégorie (supervisé, non supervisé, renforcement,
    /// optimisation, deep learning, NLP, séries temporelles, probabiliste).
    pub fn build_catalog() -> Self {
        let algorithms = vec![
            // ═══ SUPERVISED ═══
            // Algorithmes qui apprennent à partir de paires (entrée, sortie connue)

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
            // Algorithmes qui découvrent des structures cachées dans les données

            algo("kmeans", "K-Means Clustering", AlgoCategory::Unsupervised,
                "Partitionne en K clusters par distance.", 30, true,
                &["clustering", "partitionnement"],
                "Organiser la mémoire en catégories émotionnelles"),
            algo("dbscan", "DBSCAN", AlgoCategory::Unsupervised,
                "Clustering par densité. Détecte les outliers.", 50, false,
                &["clustering", "densité", "outliers"],
                "Détecter les souvenirs traumatiques isolés"),
            // ═══ DIMENSIONALITY REDUCTION ═══
            // Algorithmes qui réduisent le nombre de dimensions tout en préservant l'information

            algo("pca", "PCA", AlgoCategory::DimensionalityReduction,
                "Projette sur les axes de variance maximale.", 20, true,
                &["réduction", "variance", "visualisation"],
                "Comprendre quels axes chimiques portent l'information"),
            algo("tsne", "t-SNE", AlgoCategory::DimensionalityReduction,
                "Réduction non-linéaire pour visualisation 2D.", 50, false,
                &["visualisation", "non-linéaire", "2D"],
                "Cartographie visuelle de l'âme"),
            // ═══ REINFORCEMENT ═══
            // Algorithmes qui optimisent une politique d'action par essai-erreur

            algo("mab", "Multi-Armed Bandit (UCB1)", AlgoCategory::Reinforcement,
                "Équilibre exploration/exploitation.", 10, true,
                &["bandit", "exploration", "exploitation"],
                "Optimiser la sélection des pensées autonomes"),
            algo("q_learning", "Q-Learning", AlgoCategory::Reinforcement,
                "Apprend Q(état,action)→récompense.", 100, false,
                &["RL", "Q-value", "état-action"],
                "Optimisation à long terme du comportement"),
            // ═══ OPTIMIZATION ═══
            // Algorithmes de recherche du minimum ou maximum d'une fonction objectif

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
            // Réseaux de neurones profonds à plusieurs couches

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
            // Traitement Automatique du Langage Naturel

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
            // Analyse de données séquentielles ordonnées dans le temps

            algo("ema", "Moyenne Mobile Exponentielle", AlgoCategory::TimeSeries,
                "Lissage pondérant les récentes observations.", 1, true,
                &["lissage", "temporel", "moyenne mobile"],
                "L'humeur de fond (Mood)"),
            algo("anomaly_zscore", "Détection d'Anomalies (Z-Score)", AlgoCategory::TimeSeries,
                "Valeurs aberrantes par écart-type.", 20, true,
                &["anomalie", "écart-type", "alerte"],
                "Conscience des changements anormaux"),
            // ═══ PROBABILISTIC ═══
            // Modèles fondés sur la théorie des probabilités

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

    /// Cherche des algorithmes par texte dans le catalogue.
    ///
    /// Effectue une recherche pondérée sur plusieurs champs :
    /// - Nom de l'algorithme (poids 3.0)
    /// - Tags / mots-clés (poids 2.0)
    /// - Cas d'utilisation (poids 1.5)
    /// - Description (poids 1.0)
    /// - Pertinence cognitive (poids 1.0)
    ///
    /// Paramètre `query` : texte de recherche (insensible à la casse)
    /// Retourne : liste d'algorithmes triés par score de pertinence décroissant
    pub fn search(&self, query: &str) -> Vec<&Algorithm> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(&Algorithm, f64)> = self.algorithms.iter()
            .map(|algo| {
                let mut score = 0.0;
                // Score élevé si le nom contient la requête
                if algo.name.to_lowercase().contains(&query_lower) { score += 3.0; }
                // Score moyen si un tag correspond
                for tag in &algo.tags {
                    if tag.to_lowercase().contains(&query_lower) { score += 2.0; }
                }
                // Score pour les cas d'utilisation
                for uc in &algo.use_cases {
                    if uc.to_lowercase().contains(&query_lower) { score += 1.5; }
                }
                // Score pour la description et la pertinence cognitive
                if algo.description.to_lowercase().contains(&query_lower) { score += 1.0; }
                if algo.cognitive_relevance.to_lowercase().contains(&query_lower) { score += 1.0; }
                (algo, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Trier par score décroissant
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(a, _)| a).collect()
    }

    /// Recommande des algorithmes adaptés à un problème donné.
    ///
    /// Combine la recherche textuelle avec un filtre sur le nombre minimum
    /// de points de données requis par l'algorithme.
    ///
    /// Paramètre `problem` : description du problème à résoudre
    /// Paramètre `data_points` : nombre de points de données disponibles
    /// Retourne : les 3 meilleurs algorithmes compatibles
    pub fn recommend(&self, problem: &str, data_points: usize) -> Vec<&Algorithm> {
        self.search(problem).into_iter()
            .filter(|a| a.min_data_points <= data_points)
            .take(3)
            .collect()
    }

    /// Liste tous les algorithmes effectivement implémentés dans Saphire.
    ///
    /// Retourne : références vers les algorithmes dont le champ `implemented` est vrai
    pub fn implemented(&self) -> Vec<&Algorithm> {
        self.algorithms.iter().filter(|a| a.implemented).collect()
    }
}

/// Fonction utilitaire pour construire un Algorithm de manière concise.
///
/// Paramètre `id` : identifiant court unique
/// Paramètre `name` : nom lisible
/// Paramètre `category` : catégorie algorithmique
/// Paramètre `description` : description du fonctionnement
/// Paramètre `min_data_points` : nombre minimum de données requises
/// Paramètre `implemented` : vrai si implémenté dans Saphire
/// Paramètre `tags` : mots-clés de recherche
/// Paramètre `cognitive_relevance` : utilité cognitive pour Saphire
/// Retourne : une instance complète d'Algorithm
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

// =============================================================================
// orchestrator.rs — Orchestrateur d'Algorithmes de Saphire
//
// Le pont entre le LLM (langage naturel) et les algorithmes (code Rust).
// Le LLM ne peut pas lire du code ni executer des fonctions — mais il peut
// lire des fiches descriptives et demander l'execution d'un algorithme.
// L'orchestrateur traduit dans les deux sens :
//   LLM → choix d'algorithme → execution → resultat en langage naturel → LLM
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Categories d'algorithmes ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmCategory {
    Clustering,
    Classification,
    DimensionReduction,
    AnomalyDetection,
    PatternRecognition,
    Exploration,
    TimeSeries,
    Reinforcement,
}

impl AlgorithmCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Clustering => "Clustering",
            Self::Classification => "Classification",
            Self::DimensionReduction => "Reduction dimensionnelle",
            Self::AnomalyDetection => "Detection d'anomalies",
            Self::PatternRecognition => "Reconnaissance de patterns",
            Self::Exploration => "Exploration",
            Self::TimeSeries => "Series temporelles",
            Self::Reinforcement => "Apprentissage par renforcement",
        }
    }
}

// ─── Fiche d'algorithme (le Vidal de Saphire) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmCard {
    pub id: String,
    pub name: String,
    pub category: AlgorithmCategory,
    /// Description en langage naturel — c'est ca que le LLM lit
    pub description: String,
    /// Quand l'utiliser (situations typiques)
    pub when_to_use: Vec<String>,
    /// Ce qu'il attend en entree
    pub input_description: String,
    /// Ce qu'il produit en sortie
    pub output_description: String,
    /// Difficulte computationnelle (low, medium, high)
    pub complexity: String,
    /// Nombre de fois utilise
    pub usage_count: u64,
    /// Score de satisfaction moyen
    pub avg_satisfaction: f64,
    /// Tags pour la recherche rapide
    pub tags: Vec<String>,
}

impl Default for AlgorithmCard {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            category: AlgorithmCategory::Clustering,
            description: String::new(),
            when_to_use: vec![],
            input_description: String::new(),
            output_description: String::new(),
            complexity: "low".into(),
            usage_count: 0,
            avg_satisfaction: 0.5,
            tags: vec![],
        }
    }
}

// ─── Entree/Sortie generiques ──────────────────────────────────────────────

/// Entree generique pour les algorithmes
#[derive(Debug, Clone, Default)]
pub struct AlgorithmInput {
    /// Vecteurs numeriques (embeddings, metriques, etc.)
    pub vectors: Option<Vec<Vec<f64>>>,
    /// Serie temporelle (valeurs ordonnees)
    pub time_series: Option<Vec<f64>>,
    /// Textes
    pub texts: Option<Vec<String>>,
    /// Labels (pour classification supervisee)
    pub labels: Option<Vec<String>>,
    /// Parametres specifiques (K pour K-Means, alpha pour EMA, etc.)
    pub params: HashMap<String, f64>,
}

/// Sortie generique — inclut un resultat en langage naturel pour le LLM
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmOutput {
    pub algorithm_id: String,
    /// Resultat en langage naturel (pour le LLM)
    pub natural_language_result: String,
    /// Resultat structure (pour le code)
    pub structured_result: serde_json::Value,
    /// Metriques (accuracy, temps d'execution, etc.)
    pub metrics: HashMap<String, f64>,
    /// Duree d'execution en ms
    pub execution_ms: u64,
}

impl AlgorithmOutput {
    /// Verifie si le resultat contient des anomalies critiques
    pub fn has_critical(&self) -> bool {
        self.metrics.get("anomalies_found").map(|v| *v > 0.0).unwrap_or(false)
            || self.metrics.get("critical").map(|v| *v > 0.0).unwrap_or(false)
    }
}

/// Demande d'algorithme parsee depuis la reponse du LLM
#[derive(Debug, Clone)]
pub struct AlgorithmRequest {
    pub algorithm_id: String,
    pub params_description: String,
}

/// Historique d'utilisation d'un algorithme
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmUsage {
    pub algorithm_id: String,
    pub situation: String,
    pub output_summary: String,
    pub satisfaction: f64,
    pub used_at: DateTime<Utc>,
}

// ─── Trait d'execution ─────────────────────────────────────────────────────

/// Interface que chaque algorithme implemente
pub trait AlgorithmExecutor: Send + Sync {
    fn id(&self) -> &str;
    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String>;
}

// ─── L'Orchestrateur ───────────────────────────────────────────────────────

pub struct AlgorithmOrchestrator {
    /// Catalogue des fiches d'algorithmes
    catalog: Vec<AlgorithmCard>,
    /// Implementations Rust (code executable)
    implementations: HashMap<String, Box<dyn AlgorithmExecutor>>,
    /// Historique d'utilisation
    pub usage_history: Vec<AlgorithmUsage>,
    /// Q-table : situation → algorithme → satisfaction moyenne
    q_table: HashMap<String, HashMap<String, f64>>,

    // ─── Resultats des analyses automatiques ─────────────
    pub last_clusters: Option<AlgorithmOutput>,
    pub last_anomalies: Option<AlgorithmOutput>,
    pub last_patterns: Option<AlgorithmOutput>,
    pub last_trends: Option<AlgorithmOutput>,
    pub last_changepoints: Option<AlgorithmOutput>,

    // ─── Configuration ───────────────────────────────────
    pub enabled: bool,
    pub llm_access_enabled: bool,
    pub max_execution_ms: u64,
    pub clustering_interval: u64,
    pub anomaly_interval: u64,
    pub association_interval: u64,
    pub smoothing_interval: u64,
    pub changepoint_interval: u64,
}

impl AlgorithmOrchestrator {
    /// Cree un nouvel orchestrateur avec le catalogue et les implementations
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        catalog: Vec<AlgorithmCard>,
        implementations: HashMap<String, Box<dyn AlgorithmExecutor>>,
        enabled: bool,
        llm_access_enabled: bool,
        max_execution_ms: u64,
        clustering_interval: u64,
        anomaly_interval: u64,
        association_interval: u64,
        smoothing_interval: u64,
        changepoint_interval: u64,
    ) -> Self {
        Self {
            catalog,
            implementations,
            usage_history: Vec::new(),
            q_table: HashMap::new(),
            last_clusters: None,
            last_anomalies: None,
            last_patterns: None,
            last_trends: None,
            last_changepoints: None,
            enabled,
            llm_access_enabled,
            max_execution_ms,
            clustering_interval,
            anomaly_interval,
            association_interval,
            smoothing_interval,
            changepoint_interval,
        }
    }

    /// Catalogue accessible
    pub fn catalog(&self) -> &[AlgorithmCard] {
        &self.catalog
    }

    /// Historique d'utilisation
    pub fn usage_history(&self) -> &[AlgorithmUsage] {
        &self.usage_history
    }

    /// Nombre d'algorithmes implementes
    pub fn implemented_count(&self) -> usize {
        self.implementations.len()
    }

    /// Nombre d'algorithmes utilises au moins une fois
    pub fn used_count(&self) -> usize {
        self.catalog.iter().filter(|c| c.usage_count > 0).count()
    }

    // ─── ETAPE 1 : Decrire les outils disponibles pour le LLM ──────────

    /// Genere une description en langage naturel des algorithmes pertinents
    /// pour le contexte donne. Incluse dans le prompt substrat.
    pub fn describe_available_tools(&self, context: &str) -> String {
        if !self.enabled || !self.llm_access_enabled {
            return String::new();
        }

        let relevant = self.find_relevant_algorithms(context, 5);
        if relevant.is_empty() {
            return String::new();
        }

        let mut desc = String::from(
            "MES OUTILS ALGORITHMIQUES DISPONIBLES :\n\
             Je peux utiliser ces algorithmes en disant 'UTILISER_ALGO: <id>'\n\n"
        );

        for card in &relevant {
            desc.push_str(&format!(
                "  {} (id: {}) — {}\n    Utile quand : {}\n    Satisfaction passee : {:.0}%\n\n",
                card.name,
                card.id,
                &card.description.chars().take(120).collect::<String>(),
                card.when_to_use.first().map(|s| s.as_str()).unwrap_or(""),
                card.avg_satisfaction * 100.0,
            ));
        }

        desc
    }

    // ─── ETAPE 2 : Trouver les algorithmes pertinents ───────────────────

    /// Recherche les algorithmes pertinents pour un contexte donne
    pub fn find_relevant_algorithms(&self, context: &str, max: usize) -> Vec<&AlgorithmCard> {
        let context_lower = context.to_lowercase();
        let mut scored: Vec<(&AlgorithmCard, f64)> = self.catalog.iter()
            .filter(|c| self.implementations.contains_key(&c.id))
            .map(|card| {
                let mut score = 0.0;

                // Score base sur les tags
                for tag in &card.tags {
                    if context_lower.contains(&tag.to_lowercase()) {
                        score += 1.0;
                    }
                }

                // Score base sur when_to_use
                for use_case in &card.when_to_use {
                    let use_lower = use_case.to_lowercase();
                    let words_match = use_lower.split_whitespace()
                        .filter(|w| w.len() > 3 && context_lower.contains(*w))
                        .count();
                    score += words_match as f64 * 0.5;
                }

                // Bonus si deja utilise avec satisfaction
                if card.usage_count > 0 {
                    score += card.avg_satisfaction * 0.5;
                }

                // Bonus Q-learning
                if let Some(q_scores) = self.q_table.get(&context_lower) {
                    if let Some(&q_score) = q_scores.get(&card.id) {
                        score += q_score;
                    }
                }

                (card, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(max).map(|(card, _)| card).collect()
    }

    // ─── ETAPE 3 : Parser la demande du LLM ────────────────────────────

    /// Parse la reponse du LLM pour detecter une demande d'algorithme.
    /// Format attendu : UTILISER_ALGO: <id> AVEC: <description>
    pub fn parse_llm_request(&self, llm_response: &str) -> Option<AlgorithmRequest> {
        if let Some(pos) = llm_response.find("UTILISER_ALGO:") {
            let rest = &llm_response[pos + 14..];
            let algo_id = rest.split_whitespace().next()?.trim().to_string();

            let params_desc = if let Some(avec_pos) = rest.find("AVEC:") {
                rest[avec_pos + 5..].trim().to_string()
            } else {
                String::new()
            };

            // Verifier que l'algorithme existe
            if self.implementations.contains_key(&algo_id) {
                Some(AlgorithmRequest {
                    algorithm_id: algo_id,
                    params_description: params_desc,
                })
            } else {
                tracing::warn!("Algorithme demande par le LLM non trouve : {}", algo_id);
                None
            }
        } else {
            None
        }
    }

    // ─── ETAPE 4 : Executer un algorithme ───────────────────────────────

    /// Execute un algorithme avec les donnees fournies
    pub fn execute(
        &self,
        algorithm_id: &str,
        input: AlgorithmInput,
    ) -> Result<AlgorithmOutput, String> {
        let implementation = self.implementations.get(algorithm_id)
            .ok_or_else(|| format!("Algorithme non trouve : {}", algorithm_id))?;

        let start = std::time::Instant::now();
        let mut output = implementation.execute(input)?;
        output.execution_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            "Algorithme execute : {} — {}ms — {}",
            algorithm_id,
            output.execution_ms,
            &output.natural_language_result.chars().take(80).collect::<String>()
        );

        Ok(output)
    }

    /// Execute un algorithme en mode automatique et stocke le resultat
    pub fn execute_auto(
        &mut self,
        algorithm_id: &str,
        input: AlgorithmInput,
    ) -> Result<AlgorithmOutput, String> {
        let output = self.execute(algorithm_id, input)?;

        // Stocker le resultat selon le type
        match algorithm_id {
            "kmeans" => self.last_clusters = Some(output.clone()),
            "isolation_forest" => self.last_anomalies = Some(output.clone()),
            "association_rules" => self.last_patterns = Some(output.clone()),
            "exponential_smoothing" => self.last_trends = Some(output.clone()),
            "changepoint_detection" => self.last_changepoints = Some(output.clone()),
            _ => {}
        }

        // Logger l'utilisation
        self.usage_history.push(AlgorithmUsage {
            algorithm_id: algorithm_id.to_string(),
            situation: "auto".into(),
            output_summary: output.natural_language_result.chars().take(100).collect(),
            satisfaction: 0.7, // satisfaction par defaut en mode auto
            used_at: Utc::now(),
        });

        // Mettre a jour le compteur dans le catalogue
        if let Some(card) = self.catalog.iter_mut().find(|c| c.id == algorithm_id) {
            card.usage_count += 1;
        }

        Ok(output)
    }

    // ─── ETAPE 5 : Enregistrer la satisfaction ──────────────────────────

    /// Enregistre la satisfaction apres execution pour l'apprentissage
    pub fn record_satisfaction(
        &mut self,
        algorithm_id: &str,
        situation: &str,
        satisfaction: f64,
    ) {
        // Mettre a jour la fiche
        if let Some(card) = self.catalog.iter_mut().find(|c| c.id == algorithm_id) {
            let total = card.usage_count as f64 * card.avg_satisfaction + satisfaction;
            card.usage_count += 1;
            card.avg_satisfaction = total / card.usage_count as f64;
        }

        // Mettre a jour la Q-table (alpha = 0.1)
        let q_entry = self.q_table
            .entry(situation.to_lowercase())
            .or_default()
            .entry(algorithm_id.to_string())
            .or_insert(0.5);
        *q_entry = *q_entry * 0.9 + satisfaction * 0.1;
    }

    // ─── Enrichissement du prompt substrat ──────────────────────────────

    /// Genere le contexte des analyses automatiques pour le prompt substrat
    pub fn auto_analysis_context(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut parts = Vec::new();

        if let Some(ref clusters) = self.last_clusters {
            parts.push(format!("ANALYSE DE MES SOUVENIRS :\n{}", clusters.natural_language_result));
        }
        if let Some(ref patterns) = self.last_patterns {
            parts.push(format!("MES PATTERNS COMPORTEMENTAUX :\n{}", patterns.natural_language_result));
        }
        if let Some(ref trends) = self.last_trends {
            parts.push(format!("TENDANCES CHIMIQUES :\n{}", trends.natural_language_result));
        }
        if let Some(ref anomalies) = self.last_anomalies {
            if anomalies.has_critical() {
                parts.push(format!("ALERTE ANOMALIE :\n{}", anomalies.natural_language_result));
            }
        }
        if let Some(ref cp) = self.last_changepoints {
            parts.push(format!("POINTS DE RUPTURE :\n{}", cp.natural_language_result));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n\n")
        }
    }

    /// Genere un JSON de l'etat pour le dashboard/API
    pub fn to_status_json(&self) -> serde_json::Value {
        let top_algos: Vec<serde_json::Value> = {
            let mut sorted: Vec<&AlgorithmCard> = self.catalog.iter()
                .filter(|c| c.usage_count > 0)
                .collect();
            sorted.sort_by(|a, b| b.avg_satisfaction
                .partial_cmp(&a.avg_satisfaction)
                .unwrap_or(std::cmp::Ordering::Equal));
            sorted.iter().take(5).map(|c| serde_json::json!({
                "id": c.id,
                "name": c.name,
                "usage_count": c.usage_count,
                "avg_satisfaction": c.avg_satisfaction,
            })).collect()
        };

        serde_json::json!({
            "enabled": self.enabled,
            "catalog_size": self.catalog.len(),
            "implemented_count": self.implemented_count(),
            "used_count": self.used_count(),
            "total_executions": self.usage_history.len(),
            "top_algorithms": top_algos,
            "auto_analyses": {
                "clustering": {
                    "interval": self.clustering_interval,
                    "last_result": self.last_clusters.as_ref().map(|o| &o.natural_language_result),
                    "last_ms": self.last_clusters.as_ref().map(|o| o.execution_ms),
                },
                "anomaly_detection": {
                    "interval": self.anomaly_interval,
                    "last_result": self.last_anomalies.as_ref().map(|o| &o.natural_language_result),
                    "has_critical": self.last_anomalies.as_ref().map(|o| o.has_critical()).unwrap_or(false),
                },
                "association_rules": {
                    "interval": self.association_interval,
                    "last_result": self.last_patterns.as_ref().map(|o| &o.natural_language_result),
                },
                "smoothing": {
                    "interval": self.smoothing_interval,
                    "last_result": self.last_trends.as_ref().map(|o| &o.natural_language_result),
                },
                "changepoint": {
                    "interval": self.changepoint_interval,
                    "last_result": self.last_changepoints.as_ref().map(|o| &o.natural_language_result),
                },
            },
            "last_usages": self.usage_history.iter().rev().take(10).collect::<Vec<_>>(),
        })
    }

    /// Genere le JSON du catalogue complet
    pub fn catalog_json(&self) -> serde_json::Value {
        serde_json::json!(self.catalog.iter().map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "category": c.category.as_str(),
                "description": c.description,
                "complexity": c.complexity,
                "usage_count": c.usage_count,
                "avg_satisfaction": c.avg_satisfaction,
                "implemented": self.implementations.contains_key(&c.id),
                "tags": c.tags,
            })
        }).collect::<Vec<_>>())
    }

    /// Restaure la Q-table et les compteurs depuis un JSON persiste
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        // Restaurer les compteurs du catalogue
        if let Some(cards) = json.get("catalog").and_then(|v| v.as_array()) {
            for saved in cards {
                if let Some(id) = saved.get("id").and_then(|v| v.as_str()) {
                    if let Some(card) = self.catalog.iter_mut().find(|c| c.id == id) {
                        if let Some(count) = saved.get("usage_count").and_then(|v| v.as_u64()) {
                            card.usage_count = count;
                        }
                        if let Some(sat) = saved.get("avg_satisfaction").and_then(|v| v.as_f64()) {
                            card.avg_satisfaction = sat;
                        }
                    }
                }
            }
        }

        // Restaurer la Q-table
        if let Some(qt) = json.get("q_table").and_then(|v| v.as_object()) {
            for (situation, actions) in qt {
                if let Some(actions_obj) = actions.as_object() {
                    let entry = self.q_table.entry(situation.clone()).or_default();
                    for (algo_id, score) in actions_obj {
                        if let Some(s) = score.as_f64() {
                            entry.insert(algo_id.clone(), s);
                        }
                    }
                }
            }
        }
    }

    /// Serialise l'etat pour persistance
    pub fn to_persist_json(&self) -> serde_json::Value {
        serde_json::json!({
            "catalog": self.catalog.iter().map(|c| serde_json::json!({
                "id": c.id,
                "usage_count": c.usage_count,
                "avg_satisfaction": c.avg_satisfaction,
            })).collect::<Vec<_>>(),
            "q_table": self.q_table,
        })
    }
}

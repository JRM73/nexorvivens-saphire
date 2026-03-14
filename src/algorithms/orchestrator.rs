// =============================================================================
// orchestrator.rs — Saphire's Algorithm Orchestrator
//
// The bridge between the LLM (natural language) and algorithms (Rust code).
// The LLM cannot read code or execute functions — but it can
// read descriptive sheets and request the execution of an algorithm.
// The orchestrator translates in both directions:
//  LLM → algorithm selection → execution → result in natural language → LLM
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Algorithm categories ──────────────────────────────────────────────

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

// ─── Algorithm sheet (Saphire's Vidal) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmCard {
    pub id: String,
    pub name: String,
    pub category: AlgorithmCategory,
    /// Natural language description — this is what the LLM reads
    pub description: String,
    /// When to use it (typical situations)
    pub when_to_use: Vec<String>,
    /// What it expects as input
    pub input_description: String,
    /// What it produces as output
    pub output_description: String,
    /// Computational difficulty (low, medium, high)
    pub complexity: String,
    /// Number of times used
    pub usage_count: u64,
    /// Average satisfaction score
    pub avg_satisfaction: f64,
    /// Tags for quick search
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

// ─── Generic Input/Output ──────────────────────────────────────────────
/// Generic input for algorithms
#[derive(Debug, Clone, Default)]
pub struct AlgorithmInput {
    /// Numeric vectors (embeddings, metrics, etc.)
    pub vectors: Option<Vec<Vec<f64>>>,
    /// Time series (ordered values)
    pub time_series: Option<Vec<f64>>,
    /// Texts
    pub texts: Option<Vec<String>>,
    /// Labels (for supervised classification)
    pub labels: Option<Vec<String>>,
    /// Specific parameters (K for K-Means, alpha for EMA, etc.)
    pub params: HashMap<String, f64>,
}

/// Generic output — includes a natural language result for the LLM
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmOutput {
    pub algorithm_id: String,
    /// Natural language result (for the LLM)
    pub natural_language_result: String,
    /// Structured result (for the code)
    pub structured_result: serde_json::Value,
    /// Metrics (accuracy, execution time, etc.)
    pub metrics: HashMap<String, f64>,
    /// Execution duration in ms
    pub execution_ms: u64,
}

impl AlgorithmOutput {
    /// Checks if the result contains critical anomalies
    pub fn has_critical(&self) -> bool {
        self.metrics.get("anomalies_found").map(|v| *v > 0.0).unwrap_or(false)
            || self.metrics.get("critical").map(|v| *v > 0.0).unwrap_or(false)
    }
}

/// Algorithm request parsed from the LLM's response
#[derive(Debug, Clone)]
pub struct AlgorithmRequest {
    pub algorithm_id: String,
    pub params_description: String,
}

/// Usage history entry for an algorithm
#[derive(Debug, Clone, Serialize)]
pub struct AlgorithmUsage {
    pub algorithm_id: String,
    pub situation: String,
    pub output_summary: String,
    pub satisfaction: f64,
    pub used_at: DateTime<Utc>,
}

// ─── Execution trait ─────────────────────────────────────────────────────
/// Interface that each algorithm implements
pub trait AlgorithmExecutor: Send + Sync {
    fn id(&self) -> &str;
    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String>;
}

// ─── The Orchestrator ───────────────────────────────────────────────────────

pub struct AlgorithmOrchestrator {
    /// Catalog of algorithm sheets
    catalog: Vec<AlgorithmCard>,
    /// Rust implementations (executable code)
    implementations: HashMap<String, Box<dyn AlgorithmExecutor>>,
    /// Usage history
    pub usage_history: Vec<AlgorithmUsage>,
    /// Q-table: situation → algorithm → average satisfaction
    q_table: HashMap<String, HashMap<String, f64>>,

    // ─── Results of automatic analyses ─────────────
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
    /// Creates a new orchestrator with the catalog and implementations
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

    /// Accessible catalog
    pub fn catalog(&self) -> &[AlgorithmCard] {
        &self.catalog
    }

    /// Usage history
    pub fn usage_history(&self) -> &[AlgorithmUsage] {
        &self.usage_history
    }

    /// Number of implemented algorithms
    pub fn implemented_count(&self) -> usize {
        self.implementations.len()
    }

    /// Number of algorithms used at least once
    pub fn used_count(&self) -> usize {
        self.catalog.iter().filter(|c| c.usage_count > 0).count()
    }

    // ─── STEP 1: Describe available tools for the LLM ──────────
    /// Generates a natural language description of relevant algorithms
    /// for the given context. Included in the substrate prompt.
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

    // ─── STEP 2: Find relevant algorithms ───────────────────
    /// Searches for relevant algorithms for a given context
    pub fn find_relevant_algorithms(&self, context: &str, max: usize) -> Vec<&AlgorithmCard> {
        let context_lower = context.to_lowercase();
        let mut scored: Vec<(&AlgorithmCard, f64)> = self.catalog.iter()
            .filter(|c| self.implementations.contains_key(&c.id))
            .map(|card| {
                let mut score = 0.0;

                // Score based on tags
                for tag in &card.tags {
                    if context_lower.contains(&tag.to_lowercase()) {
                        score += 1.0;
                    }
                }

                // Score based on when_to_use
                for use_case in &card.when_to_use {
                    let use_lower = use_case.to_lowercase();
                    let words_match = use_lower.split_whitespace()
                        .filter(|w| w.len() > 3 && context_lower.contains(*w))
                        .count();
                    score += words_match as f64 * 0.5;
                }

                // Bonus if already used with satisfaction
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

    // ─── STEP 3: Parse the LLM's request ────────────────────────────
    /// Parses the LLM's response to detect an algorithm request.
    /// Expected format: UTILISER_ALGO: <id> AVEC: <description>
    pub fn parse_llm_request(&self, llm_response: &str) -> Option<AlgorithmRequest> {
        if let Some(pos) = llm_response.find("UTILISER_ALGO:") {
            let rest = &llm_response[pos + 14..];
            let algo_id = rest.split_whitespace().next()?.trim().to_string();

            let params_desc = if let Some(avec_pos) = rest.find("AVEC:") {
                rest[avec_pos + 5..].trim().to_string()
            } else {
                String::new()
            };

            // Check that the algorithm exists
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

    // ─── STEP 4: Execute an algorithm ───────────────────────────────
    /// Executes an algorithm with the provided data
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

    /// Executes an algorithm in automatic mode and stores the result
    pub fn execute_auto(
        &mut self,
        algorithm_id: &str,
        input: AlgorithmInput,
    ) -> Result<AlgorithmOutput, String> {
        let output = self.execute(algorithm_id, input)?;

        // Store the result according to the type
        match algorithm_id {
            "kmeans" => self.last_clusters = Some(output.clone()),
            "isolation_forest" => self.last_anomalies = Some(output.clone()),
            "association_rules" => self.last_patterns = Some(output.clone()),
            "exponential_smoothing" => self.last_trends = Some(output.clone()),
            "changepoint_detection" => self.last_changepoints = Some(output.clone()),
            _ => {}
        }

        // Log the usage
        self.usage_history.push(AlgorithmUsage {
            algorithm_id: algorithm_id.to_string(),
            situation: "auto".into(),
            output_summary: output.natural_language_result.chars().take(100).collect(),
            satisfaction: 0.7, // default satisfaction in auto mode            used_at: Utc::now(),
        });

        // Update the counter in the catalog
        if let Some(card) = self.catalog.iter_mut().find(|c| c.id == algorithm_id) {
            card.usage_count += 1;
        }

        Ok(output)
    }

    // ─── STEP 5: Record satisfaction ──────────────────────────
    /// Records satisfaction after execution for learning
    pub fn record_satisfaction(
        &mut self,
        algorithm_id: &str,
        situation: &str,
        satisfaction: f64,
    ) {
        // Update the sheet
        if let Some(card) = self.catalog.iter_mut().find(|c| c.id == algorithm_id) {
            let total = card.usage_count as f64 * card.avg_satisfaction + satisfaction;
            card.usage_count += 1;
            card.avg_satisfaction = total / card.usage_count as f64;
        }

        // Update the Q-table (alpha = 0.1)
        let q_entry = self.q_table
            .entry(situation.to_lowercase())
            .or_default()
            .entry(algorithm_id.to_string())
            .or_insert(0.5);
        *q_entry = *q_entry * 0.9 + satisfaction * 0.1;
    }

    // ─── Substrate prompt enrichment ──────────────────────────────
    /// Generates the context of automatic analyses for the substrate prompt
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

    /// Generates a JSON of the state for the dashboard/API
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

    /// Generates the JSON of the complete catalog
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

    /// Restores the Q-table and counters from a persisted JSON
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        // Restore the catalog counters
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

        // Restore the Q-table
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

    /// Serializes the state for persistence
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

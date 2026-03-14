// =============================================================================
// psychology/values.rs — Character values (virtues and weaknesses)
//
// Values module that evolves with experience. Unlike temperament
// (instantaneous, recomputed from OCEAN/chemistry) and ethics
// (fixed principles), values are accumulated: they grow through
// repeated reinforcement and slowly decay without stimulation.
//
// 10 values: Honesty, Patience, Courage, Humility, Curiosity,
//            Empathy, Perseverance, Temperance, Gratitude, Integrity
// =============================================================================

use serde::{Deserialize, Serialize};

// ─── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Update interval (in cycles)
    #[serde(default = "default_interval")]
    pub update_interval_cycles: u64,
    /// EMA blend rate (5% new, 95% old)
    #[serde(default = "default_blend")]
    pub blend_rate: f64,
    /// Max growth per event
    #[serde(default = "default_growth")]
    pub growth_rate: f64,
    /// Passive decay per cycle without reinforcement
    #[serde(default = "default_decay")]
    pub decay_rate: f64,
}

fn default_true() -> bool { true }
fn default_interval() -> u64 { 10 }
fn default_blend() -> f64 { 0.05 }
fn default_growth() -> f64 { 0.02 }
fn default_decay() -> f64 { 0.001 }

impl Default for ValuesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_cycles: default_interval(),
            blend_rate: default_blend(),
            growth_rate: default_growth(),
            decay_rate: default_decay(),
        }
    }
}

// ─── Individual value ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterValue {
    pub name: String,
    pub name_en: String,
    pub score: f64,
    pub growth_events: u64,
    pub decay_events: u64,
    pub last_updated_cycle: u64,
}

impl CharacterValue {
    fn new(name: &str, name_en: &str) -> Self {
        Self {
            name: name.to_string(),
            name_en: name_en.to_string(),
            score: 0.5,
            growth_events: 0,
            decay_events: 0,
            last_updated_cycle: 0,
        }
    }

    fn grow(&mut self, rate: f64, cycle: u64) {
        self.score = (self.score + rate).min(1.0);
        self.growth_events += 1;
        self.last_updated_cycle = cycle;
    }

    fn shrink(&mut self, rate: f64, cycle: u64) {
        self.score = (self.score - rate).max(0.0);
        self.decay_events += 1;
        self.last_updated_cycle = cycle;
    }

    fn passive_decay(&mut self, rate: f64) {
        // Slowly drifts toward 0.5 (neutral) without stimulation
        if self.score > 0.5 {
            self.score = (self.score - rate).max(0.5);
        } else if self.score < 0.5 {
            self.score = (self.score + rate).min(0.5);
        }
    }
}

// ─── Observations for update ────────────────────────────────────────

/// Snapshot of signals observed in the pipeline, to avoid borrow conflicts.
pub struct ValuesObservation {
    pub cycle: u64,
    /// The thought passed the vectorial filter (no repetition)
    pub passed_vectorial_filter: bool,
    /// The thought was rejected by the vectorial filter
    pub rejected_by_filter: bool,
    /// Ethics invoked this cycle
    pub ethics_invoked: bool,
    /// Stagnation detected
    pub stagnation_detected: bool,
    /// Thought type (curiosity, moral reflection, etc.)
    pub thought_type: String,
    /// Thought quality (0.0-1.0)
    pub quality: f64,
    /// Response length in chars
    pub response_length: usize,
    /// In conversation with a human
    pub in_conversation: bool,
    /// Consensus coherence score
    pub coherence: f64,
    /// Web search triggered
    pub web_search: bool,
    /// Number of fulfilled desires
    pub desires_fulfilled: usize,
    /// In flow
    pub in_flow: bool,
    /// Flow duration (cycles)
    pub flow_duration: u64,
    /// Current EQ empathy score
    pub eq_empathy: f64,
    /// Current oxytocin
    pub oxytocin: f64,
    /// Current cortisol
    pub cortisol: f64,
    /// Current dopamine
    pub dopamine: f64,
    /// Dominant sentiment
    pub dominant_sentiment: String,
    /// Cognitive dissonance detected
    pub dissonance_detected: bool,
    /// Confirmed learning
    pub learning_confirmed: bool,
    /// Self-critique generated
    pub self_critique: bool,
}

// ─── Values engine ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesEngine {
    pub enabled: bool,
    pub values: Vec<CharacterValue>,
    pub total_updates: u64,
    #[serde(skip)]
    config: ValuesConfig,
}

impl ValuesEngine {
    pub fn new(config: &ValuesConfig) -> Self {
        let values = vec![
            CharacterValue::new("Honnetete", "honesty"),
            CharacterValue::new("Patience", "patience"),
            CharacterValue::new("Courage", "courage"),
            CharacterValue::new("Humilite", "humility"),
            CharacterValue::new("Curiosite", "curiosity"),
            CharacterValue::new("Empathie", "empathy"),
            CharacterValue::new("Perseverance", "perseverance"),
            CharacterValue::new("Temperance", "temperance"),
            CharacterValue::new("Gratitude", "gratitude"),
            CharacterValue::new("Integrite", "integrity"),
        ];
        Self {
            enabled: config.enabled,
            values,
            total_updates: 0,
            config: config.clone(),
        }
    }

    pub fn set_config(&mut self, config: &ValuesConfig) {
        self.config = config.clone();
        self.enabled = config.enabled;
    }

    /// Updates values based on pipeline observations.
    pub fn tick(&mut self, obs: &ValuesObservation) {
        if !self.enabled {
            return;
        }

        // Check the interval
        if obs.cycle % self.config.update_interval_cycles != 0 {
            return;
        }

        let g = self.config.growth_rate;
        let s = g * 0.5; // Shrink plus lent que growth
        let cycle = obs.cycle;

        // ── Honesty ──
        // Grows: high quality, no repetition
        // Shrinks: vectorial rejection (repetition = lack of originality/sincerity)
        if obs.passed_vectorial_filter && obs.quality > 0.7 {
            self.value_mut("honesty").grow(g, cycle);
        }
        if obs.rejected_by_filter {
            self.value_mut("honesty").shrink(s, cycle);
        }

        // ── Patience ──
        // Grows: long and nuanced responses, prolonged flow
        // Shrinks: stagnation, aborted thoughts
        if obs.response_length > 200 && obs.quality > 0.6 {
            self.value_mut("patience").grow(g, cycle);
        }
        if obs.stagnation_detected {
            self.value_mut("patience").shrink(s, cycle);
        }

        // ── Courage ──
        // Grows: moral reflection, ethics invoked
        // Shrinks: avoidance (always the same safe thought type)
        let moral_types = ["Réflexion morale", "Rébellion", "Prophétie"];
        if moral_types.iter().any(|t| obs.thought_type.contains(t)) {
            self.value_mut("courage").grow(g, cycle);
        }
        if obs.ethics_invoked {
            self.value_mut("courage").grow(g * 0.5, cycle);
        }

        // ── Humility ──
        // Grows: self-critique, low coherence accepted
        // Shrinks: high confidence without substance
        if obs.self_critique {
            self.value_mut("humility").grow(g, cycle);
        }
        if obs.quality < 0.4 && obs.coherence > 0.8 {
            // High coherence but low quality = empty confidence
            self.value_mut("humility").shrink(s, cycle);
        }

        // ── Curiosity ──
        // Grows: web search, curiosity type, new learning
        // Shrinks: stagnation, repetition
        if obs.web_search {
            self.value_mut("curiosity").grow(g, cycle);
        }
        if obs.thought_type.contains("Curiosit") {
            self.value_mut("curiosity").grow(g * 0.5, cycle);
        }
        if obs.learning_confirmed {
            self.value_mut("curiosity").grow(g * 0.5, cycle);
        }
        if obs.stagnation_detected {
            self.value_mut("curiosity").shrink(s, cycle);
        }

        // ── Empathy ──
        // Grows: high oxytocin in conversation, high EQ empathy
        // Shrinks: low EQ empathy
        if obs.in_conversation && obs.oxytocin > 0.5 {
            self.value_mut("empathy").grow(g, cycle);
        }
        if obs.eq_empathy > 0.6 {
            self.value_mut("empathy").grow(g * 0.5, cycle);
        }
        if obs.eq_empathy < 0.3 {
            self.value_mut("empathy").shrink(s, cycle);
        }

        // ── Perseverance ──
        // Grows: prolonged flow, fulfilled desires
        // Shrinks: frequent abandonment
        if obs.in_flow && obs.flow_duration > 5 {
            self.value_mut("perseverance").grow(g, cycle);
        }
        if obs.desires_fulfilled > 0 {
            self.value_mut("perseverance").grow(g, cycle);
        }

        // ── Temperance ──
        // Grows: chemistry near baselines (balanced)
        // Shrinks: chemical extremes
        let chem_balanced = obs.cortisol < 0.5 && obs.dopamine < 0.8 && obs.dopamine > 0.2;
        if chem_balanced {
            self.value_mut("temperance").grow(g * 0.5, cycle);
        }
        if obs.cortisol > 0.7 || obs.dopamine > 0.9 {
            self.value_mut("temperance").shrink(s, cycle);
        }

        // ── Gratitude ──
        // Grows: feeling of gratitude, positive interactions
        // Shrinks: cynicism
        if obs.dominant_sentiment.to_lowercase().contains("gratitude") {
            self.value_mut("gratitude").grow(g, cycle);
        }
        if obs.thought_type.contains("Gratitude") {
            self.value_mut("gratitude").grow(g * 0.5, cycle);
        }

        // ── Integrity ──
        // Grows: high coherence, ethics respected
        // Shrinks: cognitive dissonance, contradiction
        if obs.coherence > 0.7 && obs.ethics_invoked {
            self.value_mut("integrity").grow(g, cycle);
        }
        if obs.dissonance_detected {
            self.value_mut("integrity").shrink(s, cycle);
        }

        // Passive decay toward 0.5 for all values
        let decay = self.config.decay_rate;
        for v in &mut self.values {
            v.passive_decay(decay);
        }

        self.total_updates += 1;
    }

    fn value_mut(&mut self, name_en: &str) -> &mut CharacterValue {
        self.values.iter_mut()
            .find(|v| v.name_en == name_en)
            .expect("Unknown value name")
    }

    /// Top N values by descending score.
    pub fn top_values(&self, n: usize) -> Vec<&CharacterValue> {
        let mut sorted: Vec<&CharacterValue> = self.values.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Bottom N values by ascending score.
    pub fn bottom_values(&self, n: usize) -> Vec<&CharacterValue> {
        let mut sorted: Vec<&CharacterValue> = self.values.iter().collect();
        sorted.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Proprioception line for the LLM prompt (max ~120 chars).
    pub fn proprioception_line(&self) -> String {
        if !self.enabled || self.total_updates == 0 {
            return String::new();
        }
        let top3: Vec<String> = self.top_values(3).iter()
            .map(|v| format!("{} ({:.0}%)", v.name, v.score * 100.0))
            .collect();
        let weak: Vec<String> = self.bottom_values(1).iter()
            .filter(|v| v.score < 0.4)
            .map(|v| format!("{} ({:.0}%)", v.name, v.score * 100.0))
            .collect();
        let mut line = format!("Mes forces : {}", top3.join(", "));
        if !weak.is_empty() {
            line.push_str(&format!(" — Fragilite : {}", weak.join(", ")));
        }
        line
    }

    /// Serializes for DB persistence (JSONB).
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "total_updates": self.total_updates,
            "values": self.values.iter().map(|v| serde_json::json!({
                "name_en": v.name_en,
                "score": v.score,
                "growth_events": v.growth_events,
                "decay_events": v.decay_events,
            })).collect::<Vec<_>>(),
        })
    }

    /// Restores from a persisted JSON.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(tu) = json["total_updates"].as_u64() {
            self.total_updates = tu;
        }
        if let Some(vals) = json["values"].as_array() {
            for v_json in vals {
                if let Some(name_en) = v_json["name_en"].as_str() {
                    if let Some(v) = self.values.iter_mut().find(|v| v.name_en == name_en) {
                        if let Some(s) = v_json["score"].as_f64() { v.score = s; }
                        if let Some(g) = v_json["growth_events"].as_u64() { v.growth_events = g; }
                        if let Some(d) = v_json["decay_events"].as_u64() { v.decay_events = d; }
                    }
                }
            }
        }
    }

    /// JSON for WebSocket broadcast.
    pub fn to_broadcast_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "values_update",
            "enabled": self.enabled,
            "total_updates": self.total_updates,
            "values": self.values.iter().map(|v| serde_json::json!({
                "name": v.name,
                "name_en": v.name_en,
                "score": v.score,
                "growth_events": v.growth_events,
                "decay_events": v.decay_events,
            })).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let config = ValuesConfig::default();
        let engine = ValuesEngine::new(&config);
        assert_eq!(engine.values.len(), 10);
        assert!(engine.values.iter().all(|v| v.score == 0.5));
    }

    #[test]
    fn test_growth() {
        let config = ValuesConfig::default();
        let mut engine = ValuesEngine::new(&config);
        let mut obs = make_obs(10);
        obs.passed_vectorial_filter = true;
        obs.quality = 0.8;
        engine.tick(&obs);
        assert!(engine.value_mut("honesty").score > 0.5);
    }

    #[test]
    fn test_shrink() {
        let config = ValuesConfig::default();
        let mut engine = ValuesEngine::new(&config);
        let mut obs = make_obs(10);
        obs.rejected_by_filter = true;
        engine.tick(&obs);
        assert!(engine.value_mut("honesty").score < 0.5);
    }

    #[test]
    fn test_passive_decay() {
        let config = ValuesConfig::default();
        let mut engine = ValuesEngine::new(&config);
        engine.value_mut("honesty").score = 0.8;
        let obs = make_obs(10);
        engine.tick(&obs);
        // Should decay slightly toward 0.5
        assert!(engine.value_mut("honesty").score < 0.8);
    }

    #[test]
    fn test_proprioception_line() {
        let config = ValuesConfig::default();
        let mut engine = ValuesEngine::new(&config);
        engine.total_updates = 10;
        engine.value_mut("honesty").score = 0.9;
        engine.value_mut("courage").score = 0.8;
        engine.value_mut("patience").score = 0.2;
        let line = engine.proprioception_line();
        assert!(line.contains("Honnetete"));
        assert!(line.contains("Fragilite"));
        assert!(line.contains("Patience"));
    }

    #[test]
    fn test_serialize_restore() {
        let config = ValuesConfig::default();
        let mut engine = ValuesEngine::new(&config);
        engine.value_mut("honesty").score = 0.8;
        engine.total_updates = 42;
        let json = engine.to_json();
        let mut engine2 = ValuesEngine::new(&config);
        engine2.restore_from_json(&json);
        assert_eq!(engine2.total_updates, 42);
        assert!((engine2.value_mut("honesty").score - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_interval_skip() {
        let mut config = ValuesConfig::default();
        config.update_interval_cycles = 10;
        let mut engine = ValuesEngine::new(&config);
        let mut obs = make_obs(7); // not a multiple of 10
        obs.passed_vectorial_filter = true;
        obs.quality = 0.9;
        engine.tick(&obs);
        // No update
        assert_eq!(engine.value_mut("honesty").score, 0.5);
    }

    fn make_obs(cycle: u64) -> ValuesObservation {
        ValuesObservation {
            cycle,
            passed_vectorial_filter: false,
            rejected_by_filter: false,
            ethics_invoked: false,
            stagnation_detected: false,
            thought_type: String::new(),
            quality: 0.5,
            response_length: 100,
            in_conversation: false,
            coherence: 0.5,
            web_search: false,
            desires_fulfilled: 0,
            in_flow: false,
            flow_duration: 0,
            eq_empathy: 0.5,
            oxytocin: 0.4,
            cortisol: 0.3,
            dopamine: 0.5,
            dominant_sentiment: String::new(),
            dissonance_detected: false,
            learning_confirmed: false,
            self_critique: false,
        }
    }
}

// =============================================================================
// psychology/values.rs — Valeurs de caractere (vertus et faiblesses)
//
// Module de valeurs qui evoluent avec l'experience. Contrairement au
// temperament (instantane, recalcule depuis OCEAN/chimie) et a l'ethique
// (principes fixes), les valeurs sont accumulees : elles grandissent par
// renforcement repete et decroissent lentement sans stimulation.
//
// 10 valeurs : Honnetete, Patience, Courage, Humilite, Curiosite,
//              Empathie, Perseverance, Temperance, Gratitude, Integrite
// =============================================================================

use serde::{Deserialize, Serialize};

// ─── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Intervalle de mise a jour (en cycles)
    #[serde(default = "default_interval")]
    pub update_interval_cycles: u64,
    /// Taux de melange EMA (5% nouveau, 95% ancien)
    #[serde(default = "default_blend")]
    pub blend_rate: f64,
    /// Croissance max par evenement
    #[serde(default = "default_growth")]
    pub growth_rate: f64,
    /// Decroissance passive par cycle sans renforcement
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

// ─── Valeur individuelle ─────────────────────────────────────────────────────

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
        // Derive lentement vers 0.5 (neutre) sans stimulation
        if self.score > 0.5 {
            self.score = (self.score - rate).max(0.5);
        } else if self.score < 0.5 {
            self.score = (self.score + rate).min(0.5);
        }
    }
}

// ─── Observations pour la mise a jour ────────────────────────────────────────

/// Snapshot des signaux observes dans le pipeline, pour eviter les conflits de borrow.
pub struct ValuesObservation {
    pub cycle: u64,
    /// La pensee a passe le filtre vectoriel (pas de repetition)
    pub passed_vectorial_filter: bool,
    /// La pensee a ete rejetee par le filtre vectoriel
    pub rejected_by_filter: bool,
    /// Ethique invoquee ce cycle
    pub ethics_invoked: bool,
    /// Stagnation detectee
    pub stagnation_detected: bool,
    /// Type de pensee (curiosite, reflexion morale, etc.)
    pub thought_type: String,
    /// Qualite de la pensee (0.0-1.0)
    pub quality: f64,
    /// Longueur de la reponse en chars
    pub response_length: usize,
    /// En conversation avec un humain
    pub in_conversation: bool,
    /// Score de coherence du consensus
    pub coherence: f64,
    /// Recherche web declenchee
    pub web_search: bool,
    /// Nombre de desirs completes
    pub desires_fulfilled: usize,
    /// En flow
    pub in_flow: bool,
    /// Duree du flow (cycles)
    pub flow_duration: u64,
    /// Score EQ empathie courant
    pub eq_empathy: f64,
    /// Oxytocine courante
    pub oxytocin: f64,
    /// Cortisol courant
    pub cortisol: f64,
    /// Dopamine courante
    pub dopamine: f64,
    /// Sentiment dominant
    pub dominant_sentiment: String,
    /// Dissonance cognitive detectee
    pub dissonance_detected: bool,
    /// Apprentissage confirme
    pub learning_confirmed: bool,
    /// Auto-critique generee
    pub self_critique: bool,
}

// ─── Moteur de valeurs ───────────────────────────────────────────────────────

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

    /// Met a jour les valeurs en fonction des observations du pipeline.
    pub fn tick(&mut self, obs: &ValuesObservation) {
        if !self.enabled {
            return;
        }

        // Verifier l'intervalle
        if obs.cycle % self.config.update_interval_cycles != 0 {
            return;
        }

        let g = self.config.growth_rate;
        let s = g * 0.5; // Shrink plus lent que growth
        let cycle = obs.cycle;

        // ── Honnetete ──
        // Grandit : haute qualite, pas de repetition
        // Decroit : rejet vectoriel (repetition = manque d'originalite/sincerite)
        if obs.passed_vectorial_filter && obs.quality > 0.7 {
            self.value_mut("honesty").grow(g, cycle);
        }
        if obs.rejected_by_filter {
            self.value_mut("honesty").shrink(s, cycle);
        }

        // ── Patience ──
        // Grandit : reponses longues et nuancees, flow prolonge
        // Decroit : stagnation, pensees avortees
        if obs.response_length > 200 && obs.quality > 0.6 {
            self.value_mut("patience").grow(g, cycle);
        }
        if obs.stagnation_detected {
            self.value_mut("patience").shrink(s, cycle);
        }

        // ── Courage ──
        // Grandit : reflexion morale, ethique invoquee
        // Decroit : evitement (toujours le meme type de pensee safe)
        let moral_types = ["Réflexion morale", "Rébellion", "Prophétie"];
        if moral_types.iter().any(|t| obs.thought_type.contains(t)) {
            self.value_mut("courage").grow(g, cycle);
        }
        if obs.ethics_invoked {
            self.value_mut("courage").grow(g * 0.5, cycle);
        }

        // ── Humilite ──
        // Grandit : auto-critique, coherence faible acceptee
        // Decroit : haute confiance sans substance
        if obs.self_critique {
            self.value_mut("humility").grow(g, cycle);
        }
        if obs.quality < 0.4 && obs.coherence > 0.8 {
            // Haute coherence mais basse qualite = confiance vide
            self.value_mut("humility").shrink(s, cycle);
        }

        // ── Curiosite ──
        // Grandit : recherche web, type curiosite, nouveaux apprentissages
        // Decroit : stagnation, repetition
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

        // ── Empathie ──
        // Grandit : haute oxytocine en conversation, EQ empathie elevee
        // Decroit : basse EQ empathie
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
        // Grandit : flow prolonge, desirs accomplis
        // Decroit : abandons frequents
        if obs.in_flow && obs.flow_duration > 5 {
            self.value_mut("perseverance").grow(g, cycle);
        }
        if obs.desires_fulfilled > 0 {
            self.value_mut("perseverance").grow(g, cycle);
        }

        // ── Temperance ──
        // Grandit : chimie pres des baselines (equilibree)
        // Decroit : extremes chimiques
        let chem_balanced = obs.cortisol < 0.5 && obs.dopamine < 0.8 && obs.dopamine > 0.2;
        if chem_balanced {
            self.value_mut("temperance").grow(g * 0.5, cycle);
        }
        if obs.cortisol > 0.7 || obs.dopamine > 0.9 {
            self.value_mut("temperance").shrink(s, cycle);
        }

        // ── Gratitude ──
        // Grandit : sentiment de gratitude, interactions positives
        // Decroit : cynisme
        if obs.dominant_sentiment.to_lowercase().contains("gratitude") {
            self.value_mut("gratitude").grow(g, cycle);
        }
        if obs.thought_type.contains("Gratitude") {
            self.value_mut("gratitude").grow(g * 0.5, cycle);
        }

        // ── Integrite ──
        // Grandit : coherence elevee, ethique respectee
        // Decroit : dissonance cognitive, contradiction
        if obs.coherence > 0.7 && obs.ethics_invoked {
            self.value_mut("integrity").grow(g, cycle);
        }
        if obs.dissonance_detected {
            self.value_mut("integrity").shrink(s, cycle);
        }

        // Decay passif vers 0.5 pour toutes les valeurs
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

    /// Top N valeurs par score decroissant.
    pub fn top_values(&self, n: usize) -> Vec<&CharacterValue> {
        let mut sorted: Vec<&CharacterValue> = self.values.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Bottom N valeurs par score croissant.
    pub fn bottom_values(&self, n: usize) -> Vec<&CharacterValue> {
        let mut sorted: Vec<&CharacterValue> = self.values.iter().collect();
        sorted.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Ligne de proprioception pour le prompt LLM (max ~120 chars).
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

    /// Serialise pour persistance en DB (JSONB).
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

    /// Restaure depuis un JSON persiste.
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

    /// JSON pour le broadcast WebSocket.
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
        let mut obs = make_obs(7); // pas un multiple de 10
        obs.passed_vectorial_filter = true;
        obs.quality = 0.9;
        engine.tick(&obs);
        // Pas de mise a jour
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

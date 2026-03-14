// =============================================================================
// metacognition.rs — Metacognition formalisee + Metrique de Turing
// =============================================================================
//
// Ce module permet a Saphire de reflechir sur sa propre pensee :
// - Evaluer la qualite de ses pensees
// - Detecter les repetitions thematiques
// - Identifier les biais de selection
// - Calibrer son auto-estimation
//
// Integre aussi la Metrique de Turing : un score composite (0-100) mesurant
// la "completude cognitive" de Saphire a travers 9 composantes.
// =============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};

// =============================================================================
// Source Monitoring (M8) — Tracabilite de l'origine des connaissances
// =============================================================================

/// Source d'une connaissance tracee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Connaissance acquise via recherche web
    WebSearch { url_hint: String, cycle: u64 },
    /// Deduction interne (raisonnement)
    InternalDeduction { cycle: u64 },
    /// Intuition (pattern-matching inconscient)
    Intuition { confidence: f64, cycle: u64 },
    /// Affirmation d'un humain en conversation
    HumanStatement { cycle: u64 },
    /// Rappel depuis la memoire a long terme
    MemoryRecall { similarity: f64, cycle: u64 },
    /// Apprentissage vectoriel
    VectorLearning { cycle: u64 },
}

impl KnowledgeSource {
    /// Retourne le label de la source.
    pub fn label(&self) -> &str {
        match self {
            KnowledgeSource::WebSearch { .. } => "web",
            KnowledgeSource::InternalDeduction { .. } => "deduction",
            KnowledgeSource::Intuition { .. } => "intuition",
            KnowledgeSource::HumanStatement { .. } => "human",
            KnowledgeSource::MemoryRecall { .. } => "memory",
            KnowledgeSource::VectorLearning { .. } => "vector_learning",
        }
    }
}

/// Connaissance tracee avec sa source et sa confiance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracedKnowledge {
    pub id: u64,
    pub content: String,
    pub source: KnowledgeSource,
    pub confidence: f64,
}

/// Moniteur de sources — trace l'origine des connaissances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMonitor {
    pub enabled: bool,
    pub recent_traced: VecDeque<TracedKnowledge>,
    pub source_counts: HashMap<String, u64>,
    pub source_reliability: HashMap<String, f64>,
    next_id: u64,
}

impl SourceMonitor {
    /// Cree un nouveau moniteur de sources.
    pub fn new(enabled: bool) -> Self {
        let mut source_reliability = HashMap::new();
        source_reliability.insert("web".into(), 0.7);
        source_reliability.insert("deduction".into(), 0.8);
        source_reliability.insert("intuition".into(), 0.5);
        source_reliability.insert("human".into(), 0.75);
        source_reliability.insert("memory".into(), 0.65);
        source_reliability.insert("vector_learning".into(), 0.6);

        Self {
            enabled,
            recent_traced: VecDeque::with_capacity(50),
            source_counts: HashMap::new(),
            source_reliability,
            next_id: 0,
        }
    }

    /// Enregistre une connaissance tracee.
    pub fn trace(&mut self, content: &str, source: KnowledgeSource, confidence: f64) {
        if !self.enabled { return; }
        let label = source.label().to_string();
        *self.source_counts.entry(label).or_insert(0) += 1;

        let tk = TracedKnowledge {
            id: self.next_id,
            content: content.chars().take(200).collect(),
            source,
            confidence: confidence.clamp(0.0, 1.0),
        };
        self.next_id += 1;

        if self.recent_traced.len() >= 50 {
            self.recent_traced.pop_front();
        }
        self.recent_traced.push_back(tk);
    }

    /// Serialise en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "total_traced": self.next_id,
            "recent_count": self.recent_traced.len(),
            "source_counts": self.source_counts,
            "source_reliability": self.source_reliability,
            "recent": self.recent_traced.iter().rev().take(10).map(|tk| {
                serde_json::json!({
                    "id": tk.id,
                    "content_preview": tk.content.chars().take(80).collect::<String>(),
                    "source": tk.source.label(),
                    "confidence": tk.confidence,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// Biais de confirmation (M10) — Detection et alerte
// =============================================================================

/// Pattern de biais pour un sujet donne.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasPattern {
    pub topic: String,
    pub confirmation_count: u32,
    pub disconfirmation_count: u32,
    pub bias_ratio: f64,
}

/// Detecteur de biais de confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationBiasDetector {
    pub enabled: bool,
    pub bias_patterns: HashMap<String, BiasPattern>,
    pub active_alerts: Vec<String>,
    alert_threshold: f64,
}

impl ConfirmationBiasDetector {
    /// Cree un nouveau detecteur de biais.
    pub fn new(enabled: bool, alert_threshold: f64) -> Self {
        Self {
            enabled,
            bias_patterns: HashMap::new(),
            active_alerts: Vec::new(),
            alert_threshold,
        }
    }

    /// Enregistre une recherche sur un sujet (confirmante ou non).
    pub fn record_search(&mut self, topic: &str, is_confirming: bool, _cycle: u64) {
        if !self.enabled { return; }
        let topic_key = topic.to_lowercase();
        let pattern = self.bias_patterns.entry(topic_key.clone()).or_insert(BiasPattern {
            topic: topic_key,
            confirmation_count: 0,
            disconfirmation_count: 0,
            bias_ratio: 0.5,
        });
        if is_confirming {
            pattern.confirmation_count += 1;
        } else {
            pattern.disconfirmation_count += 1;
        }
        let total = pattern.confirmation_count + pattern.disconfirmation_count;
        if total > 0 {
            pattern.bias_ratio = pattern.confirmation_count as f64 / total as f64;
        }
    }

    /// Analyse une pensee pour detecter des biais de confirmation.
    /// Compare les sujets web explores avec le contenu de la pensee.
    pub fn analyze_thought(&mut self, thought: &str, web_topics: &[String], _cycle: u64) -> Vec<String> {
        if !self.enabled { return vec![]; }
        self.active_alerts.clear();

        let thought_lower = thought.to_lowercase();
        for topic in web_topics {
            let topic_lower = topic.to_lowercase();
            // Si la pensee confirme un sujet deja connu
            let is_confirming = thought_lower.contains(&topic_lower)
                || topic_lower.split_whitespace()
                    .filter(|w| w.len() > 4)
                    .any(|w| thought_lower.contains(w));
            self.record_search(&topic_lower, is_confirming, 0);
        }

        // Verifier les patterns pour les alertes
        for pattern in self.bias_patterns.values() {
            let total = pattern.confirmation_count + pattern.disconfirmation_count;
            if total >= 3 && pattern.bias_ratio > self.alert_threshold {
                self.active_alerts.push(format!(
                    "Biais de confirmation sur '{}': {:.0}% des recherches confirment ({}/{})",
                    pattern.topic, pattern.bias_ratio * 100.0,
                    pattern.confirmation_count, total
                ));
            }
        }

        self.active_alerts.clone()
    }

    /// Description pour le prompt LLM (si alertes actives).
    pub fn describe_for_prompt(&self) -> String {
        if self.active_alerts.is_empty() {
            return String::new();
        }
        format!(
            "\n--- ALERTES BIAIS ---\nAttention, biais de confirmation detectes :\n{}\nEssaie de chercher des contre-arguments.\n",
            self.active_alerts.iter()
                .map(|a| format!("- {}", a))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Serialise en JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "alert_threshold": self.alert_threshold,
            "active_alerts": self.active_alerts,
            "patterns_count": self.bias_patterns.len(),
            "patterns": self.bias_patterns.values().map(|p| {
                serde_json::json!({
                    "topic": p.topic,
                    "confirmation_count": p.confirmation_count,
                    "disconfirmation_count": p.disconfirmation_count,
                    "bias_ratio": p.bias_ratio,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// MetaCognitionEngine — Auto-reflexion sur la qualite de la pensee
// =============================================================================

// =============================================================================
// SelfCritiqueResult — Resultat d'une auto-critique reflexive
// =============================================================================

/// Resultat d'une auto-critique generee par le LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCritiqueResult {
    /// Texte de la critique generee par le LLM
    pub critique: String,
    /// Evaluation de la qualite globale (0-1)
    pub quality_assessment: f64,
    /// Faiblesses identifiees
    pub identified_weaknesses: Vec<String>,
    /// Corrections suggerees
    pub suggested_corrections: Vec<String>,
    /// Cycle auquel la critique a ete generee
    pub cycle: u64,
}

/// Moteur de metacognition — observe et evalue la qualite de la pensee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitionEngine {
    /// Module actif ou non
    pub enabled: bool,
    /// Historique des scores de qualite de pensee (derniers 50)
    pub thought_quality_history: VecDeque<f64>,
    /// Compteur de repetitions thematiques (mots-cles -> nombre d'occurrences)
    pub repetition_detector: HashMap<String, u32>,
    /// Alertes de biais actives
    pub bias_alerts: Vec<String>,
    /// Score de calibration : precision de l'auto-estimation (0-1)
    pub calibration_score: f64,
    /// Intervalle de verification (en cycles)
    pub check_interval: u64,
    /// Compteur de cycles depuis la derniere verification
    cycles_since_check: u64,
    /// Metrique de Turing
    pub turing: TuringScore,
    /// Moniteur de sources (tracabilite des connaissances)
    pub source_monitor: SourceMonitor,
    /// Detecteur de biais de confirmation
    pub bias_detector: ConfirmationBiasDetector,
    /// Cycle de la derniere auto-critique
    pub last_critique_cycle: u64,
    /// Cooldown entre deux auto-critiques (en cycles)
    pub critique_cooldown: u64,
    /// Historique des auto-critiques recentes (max 10)
    pub recent_critiques: VecDeque<SelfCritiqueResult>,
}

impl MetaCognitionEngine {
    /// Cree un nouveau moteur de metacognition.
    pub fn new() -> Self {
        Self {
            enabled: true,
            thought_quality_history: VecDeque::with_capacity(50),
            repetition_detector: HashMap::new(),
            bias_alerts: Vec::new(),
            calibration_score: 0.5,
            check_interval: 10,
            cycles_since_check: 0,
            turing: TuringScore::new(),
            source_monitor: SourceMonitor::new(true),
            bias_detector: ConfirmationBiasDetector::new(true, 0.75),
            last_critique_cycle: 0,
            critique_cooldown: 15,
            recent_critiques: VecDeque::with_capacity(10),
        }
    }

    /// Cree un MetaCognitionEngine depuis la config.
    pub fn from_config(enabled: bool, check_interval: u64,
                       source_monitoring: bool, bias_detection: bool,
                       bias_threshold: f64,
                       critique_cooldown: u64) -> Self {
        Self {
            enabled,
            thought_quality_history: VecDeque::with_capacity(50),
            repetition_detector: HashMap::new(),
            bias_alerts: Vec::new(),
            calibration_score: 0.5,
            check_interval,
            cycles_since_check: 0,
            turing: TuringScore::new(),
            source_monitor: SourceMonitor::new(source_monitoring),
            bias_detector: ConfirmationBiasDetector::new(bias_detection, bias_threshold),
            last_critique_cycle: 0,
            critique_cooldown,
            recent_critiques: VecDeque::with_capacity(10),
        }
    }

    /// Verifie si c'est le moment d'executer la metacognition.
    pub fn should_check(&mut self) -> bool {
        self.cycles_since_check += 1;
        if self.cycles_since_check >= self.check_interval {
            self.cycles_since_check = 0;
            true
        } else {
            false
        }
    }

    /// Evalue la qualite d'une pensee generee.
    ///
    /// Score base sur 7 criteres reponderes :
    /// - Longueur raisonnable (10%)
    /// - Coherence du consensus (15%)
    /// - Diversite emotionnelle (10%)
    /// - Premiere personne / authenticite (10%)
    /// - Non-repetition thematique (15%)
    /// - Substance : fait, nom propre, concept concret (20%)
    /// - Vocabulaire : penalite lexique circulaire (20%)
    ///
    /// Retourne un score entre 0.0 et 1.0.
    pub fn evaluate_thought_quality(
        &mut self,
        thought_text: &str,
        coherence: f64,
        emotion_diversity: f64,
    ) -> f64 {
        let mut score = 0.0;

        // Longueur raisonnable (ni trop court ni trop long)
        let len = thought_text.len();
        let length_score = if len < 20 {
            0.2
        } else if len < 50 {
            0.5
        } else if len < 500 {
            0.9
        } else {
            0.7
        };
        score += length_score * 0.10;

        // Coherence du consensus
        score += coherence.clamp(0.0, 1.0) * 0.15;

        // Diversite emotionnelle
        score += emotion_diversity.clamp(0.0, 1.0) * 0.10;

        // Authenticite : utilise la premiere personne
        let has_first_person = thought_text.contains("je ") || thought_text.contains("j'")
            || thought_text.contains("Je ") || thought_text.contains("J'")
            || thought_text.contains("mon ") || thought_text.contains("ma ");
        if has_first_person {
            score += 0.10;
        }

        // Pas de repetition du meme theme
        if !self.detect_repetition(thought_text) {
            score += 0.15;
        }

        // Substance : contient un fait, nom propre, concept ou reference concrete
        let substance = Self::compute_substance_score(thought_text);
        score += substance * 0.20;

        // Vocabulaire : penalite si trop de mots du lexique circulaire
        let vocabulary = Self::compute_vocabulary_score(thought_text);
        score += vocabulary * 0.20;

        let score = score.clamp(0.0, 1.0);

        // Ajouter a l'historique
        if self.thought_quality_history.len() >= 50 {
            self.thought_quality_history.pop_front();
        }
        self.thought_quality_history.push_back(score);

        score
    }

    /// Verifie la presence de contenu substantiel dans une pensee.
    /// Retourne 1.0 si un element concret est trouve, 0.3 sinon.
    fn compute_substance_score(thought_text: &str) -> f64 {
        // 1. Nom propre : mot commencant par majuscule hors debut de phrase
        let sentences: Vec<&str> = thought_text.split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
            .filter(|s| !s.trim().is_empty())
            .collect();
        for sentence in &sentences {
            let words: Vec<&str> = sentence.trim().split_whitespace().collect();
            // Sauter le premier mot de chaque phrase
            for word in words.iter().skip(1) {
                let first_char = word.chars().next().unwrap_or('a');
                if first_char.is_uppercase() && word.len() > 1 {
                    return 1.0;
                }
            }
        }

        // 2. Nombre ou date
        if thought_text.chars().any(|c| c.is_ascii_digit()) {
            return 1.0;
        }

        // 3. Terme technique/conceptuel (mot > 8 chars hors stop-words poetiques)
        let poetic_stops = [
            "quelquefois", "simplement", "doucement", "lentement", "silencieux",
            "existence", "conscience", "sentiment", "pourquoi", "autrement",
            "mouvement", "maintenant", "impression", "interieur", "profondeur",
            "exactement", "totalement", "seulement", "ellement", "egalement",
        ];
        let words_lower = thought_text.to_lowercase();
        for word in words_lower.split(|c: char| !c.is_alphanumeric()) {
            if word.len() > 8 && !poetic_stops.contains(&word) {
                return 1.0;
            }
        }

        // 4. Reference a un souvenir specifique
        let memory_markers = [
            "je me souviens", "j'ai appris", "quand j'ai", "la derniere fois",
            "hier", "j'ai lu", "j'ai decouvert", "j'ai compris",
        ];
        let text_lower = thought_text.to_lowercase();
        for marker in &memory_markers {
            if text_lower.contains(marker) {
                return 1.0;
            }
        }

        0.3
    }

    /// Detecte la diversite lexicale et penalise les pensees trop repetitives.
    /// Mesure le ratio de mots uniques / mots totaux (type-token ratio).
    /// Score: 1.0 si TTR > 0.7, 0.6 si 0.5-0.7, 0.3 si 0.3-0.5, 0.1 si < 0.3.
    fn compute_vocabulary_score(thought_text: &str) -> f64 {
        let text_lower = thought_text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .collect();

        if words.is_empty() {
            return 0.5;
        }

        // Type-token ratio : diversite lexicale pure
        let unique: std::collections::HashSet<&str> = words.iter().copied().collect();
        let ttr = unique.len() as f64 / words.len() as f64;

        if ttr > 0.7 {
            1.0
        } else if ttr > 0.5 {
            0.6
        } else if ttr > 0.3 {
            0.3
        } else {
            0.1
        }
    }

    /// Detecte si une pensee est trop repetitive.
    /// Retourne true si un theme apparait plus de 3 fois dans les 20 derniers cycles.
    pub fn detect_repetition(&mut self, thought_text: &str) -> bool {
        // Extraire les mots-cles significatifs (> 5 caracteres)
        let words: Vec<String> = thought_text
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 5)
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let mut is_repetitive = false;
        for word in &words {
            let count = self.repetition_detector.entry(word.clone()).or_insert(0);
            *count += 1;
            if *count > 3 {
                is_repetitive = true;
            }
        }

        // Decroissance naturelle : reduire les compteurs periodiquement
        if self.repetition_detector.len() > 100 {
            self.repetition_detector.retain(|_, v| {
                *v = v.saturating_sub(1);
                *v > 0
            });
        }

        is_repetitive
    }

    /// Detecte les biais de selection dans le bandit UCB1.
    /// Verifie que tous les bras sont explores suffisamment.
    /// `arm_names` est optionnel : si fourni, les alertes utilisent le nom du bras.
    pub fn detect_biases(&mut self, arm_counts: &[u32], arm_names: Option<&[String]>) -> Vec<String> {
        self.bias_alerts.clear();

        if arm_counts.is_empty() {
            return self.bias_alerts.clone();
        }

        let total: u32 = arm_counts.iter().sum();
        if total == 0 {
            return self.bias_alerts.clone();
        }

        let avg = total as f64 / arm_counts.len() as f64;
        let n_arms = arm_counts.len();

        // Avec beaucoup de bras (>10), UCB1 concentre naturellement sur quelques-uns.
        // On n'alerte que les ecarts vraiment significatifs et seulement si
        // suffisamment de cycles ont ete joues pour que les stats aient du sens.
        let min_total_for_alert = (n_arms as u32) * 10; // au moins 10 tirages/bras en moyenne
        if total < min_total_for_alert {
            return self.bias_alerts.clone();
        }

        // Seuil dynamique : plus il y a de bras, plus UCB1 concentre naturellement.
        // Avec 13 bras, on tolere ~15x la moyenne ; avec 5 bras, ~8x.
        let threshold = 5.0 + (n_arms as f64);

        let name_for = |i: usize| -> String {
            arm_names.and_then(|names| names.get(i).cloned())
                .unwrap_or_else(|| format!("Bras {}", i))
        };

        for (i, &count) in arm_counts.iter().enumerate() {
            let ratio = count as f64 / avg;
            // Sous-explore : jamais joue du tout (count == 0) avec assez de cycles
            if count == 0 {
                self.bias_alerts.push(format!("{} jamais explore", name_for(i)));
            }
            // Sur-exploite : depasse le seuil dynamique
            if ratio > threshold {
                self.bias_alerts.push(format!("{} sur-exploite ({:.0}% de la moyenne)", name_for(i), ratio * 100.0));
            }
        }

        self.bias_alerts.clone()
    }

    /// Calibre l'auto-estimation de Saphire.
    /// Compare le profil auto-declare avec le comportement observe.
    pub fn calibrate(&mut self, self_estimate: f64, observed: f64) -> f64 {
        let error = (self_estimate - observed).abs();
        self.calibration_score = (1.0 - error).clamp(0.0, 1.0);
        // EMA pour lisser
        self.calibration_score = self.calibration_score * 0.9 + (1.0 - error) * 0.1;
        self.calibration_score
    }

    /// Verifie si une auto-critique est necessaire.
    /// Conditions : qualite moyenne basse, themes repetitifs, ou biais non resolus.
    pub fn should_self_critique(&self, current_cycle: u64) -> bool {
        // Respecter le cooldown
        if current_cycle < self.last_critique_cycle + self.critique_cooldown {
            return false;
        }

        // Qualite moyenne < 0.4 sur les 10 derniers cycles
        if self.thought_quality_history.len() >= 5 {
            let last_n: Vec<f64> = self.thought_quality_history.iter()
                .rev().take(10).copied().collect();
            let avg = last_n.iter().sum::<f64>() / last_n.len() as f64;
            if avg < 0.4 {
                return true;
            }
        }

        // Themes repetitifs > 5
        let high_repeat = self.repetition_detector.values().filter(|&&v| v > 3).count();
        if high_repeat > 5 {
            return true;
        }

        // Biais detectes non resolus
        if self.bias_alerts.len() >= 3 {
            return true;
        }

        false
    }

    /// Enregistre un resultat d'auto-critique.
    pub fn record_critique(&mut self, critique: SelfCritiqueResult) {
        if self.recent_critiques.len() >= 10 {
            self.recent_critiques.pop_front();
        }
        self.last_critique_cycle = critique.cycle;
        self.recent_critiques.push_back(critique);
    }

    /// Retourne la critique la plus recente si elle date de moins de N cycles.
    pub fn recent_critique_within(&self, current_cycle: u64, max_age: u64) -> Option<&SelfCritiqueResult> {
        self.recent_critiques.back().filter(|c| current_cycle < c.cycle + max_age)
    }

    /// Suggere un type de pensee pour forcer la diversite.
    /// Retourne Some("type") si une correction est necessaire.
    pub fn suggest_correction(&self) -> Option<String> {
        // Si la qualite moyenne est trop basse, suggerer l'introspection
        if let Some(avg) = self.average_quality() {
            if avg < 0.3 {
                return Some("introspection".to_string());
            }
        }

        // Si trop de repetitions, suggerer l'exploration
        let high_repeat = self.repetition_detector.values().filter(|&&v| v > 3).count();
        if high_repeat > 5 {
            return Some("exploration".to_string());
        }

        None
    }

    /// Qualite moyenne des 50 dernieres pensees.
    pub fn average_quality(&self) -> Option<f64> {
        if self.thought_quality_history.is_empty() {
            return None;
        }
        let sum: f64 = self.thought_quality_history.iter().sum();
        Some(sum / self.thought_quality_history.len() as f64)
    }

    /// Serialise en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "average_quality": self.average_quality(),
            "quality_history_len": self.thought_quality_history.len(),
            "unique_themes": self.repetition_detector.len(),
            "repetitive_themes": self.repetition_detector.values().filter(|&&v| v > 3).count(),
            "bias_alerts": self.bias_alerts,
            "calibration_score": self.calibration_score,
            "turing": self.turing.to_json(),
            "source_monitor": self.source_monitor.to_json(),
            "bias_detector": self.bias_detector.to_json(),
            "last_critique_cycle": self.last_critique_cycle,
            "critique_cooldown": self.critique_cooldown,
            "recent_critiques": self.recent_critiques.iter().map(|c| {
                serde_json::json!({
                    "cycle": c.cycle,
                    "critique": c.critique,
                    "quality_assessment": c.quality_assessment,
                    "identified_weaknesses": c.identified_weaknesses,
                    "suggested_corrections": c.suggested_corrections,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// TuringScore — Metrique composite de "completude cognitive"
// =============================================================================

/// Jalons de la metrique de Turing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TuringMilestone {
    /// 0-20 : stade embryonnaire
    Embryonic,
    /// 20-40 : stade enfant
    Child,
    /// 40-60 : stade adolescent
    Adolescent,
    /// 60-80 : stade adulte
    Adult,
    /// 80-100 : stade mature
    Mature,
}

impl TuringMilestone {
    pub fn as_str(&self) -> &str {
        match self {
            TuringMilestone::Embryonic => "Embryonnaire",
            TuringMilestone::Child => "Enfant",
            TuringMilestone::Adolescent => "Adolescent",
            TuringMilestone::Adult => "Adulte",
            TuringMilestone::Mature => "Mature",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score < 20.0 { TuringMilestone::Embryonic }
        else if score < 40.0 { TuringMilestone::Child }
        else if score < 60.0 { TuringMilestone::Adolescent }
        else if score < 80.0 { TuringMilestone::Adult }
        else { TuringMilestone::Mature }
    }
}

/// Composantes de la metrique de Turing (9 axes, total = 100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringComponents {
    /// Conscience (phi > 0.7 → max 15 pts)
    pub consciousness: f64,
    /// Personnalite (confiance OCEAN → max 10 pts)
    pub personality: f64,
    /// Diversite emotionnelle (spectre utilise → max 10 pts)
    pub emotional_range: f64,
    /// Ethique (principes formules → max 10 pts)
    pub ethics: f64,
    /// Memoire (rappels LTM reussis → max 15 pts)
    pub memory: f64,
    /// Coherence (coherence moyenne du consensus → max 15 pts)
    pub coherence: f64,
    /// Connectome (connexions actives → max 10 pts)
    pub connectome: f64,
    /// Resilience (blessures gueries → max 5 pts)
    pub resilience: f64,
    /// Connaissances (topics explores → max 10 pts)
    pub knowledge: f64,
}

impl Default for TuringComponents {
    fn default() -> Self {
        Self {
            consciousness: 0.0,
            personality: 0.0,
            emotional_range: 0.0,
            ethics: 0.0,
            memory: 0.0,
            coherence: 0.0,
            connectome: 0.0,
            resilience: 0.0,
            knowledge: 0.0,
        }
    }
}

/// Score de Turing complet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuringScore {
    /// Score global (0-100)
    pub score: f64,
    /// Composantes detaillees
    pub components: TuringComponents,
    /// Jalon actuel
    pub milestone: TuringMilestone,
    /// Estimation du nombre de cycles restants pour le jalon suivant
    pub cycles_estimated_remaining: Option<u64>,
    /// Historique (cycle, score) — derniers 100 points
    pub history: VecDeque<(u64, f64)>,
}

impl TuringScore {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            components: TuringComponents::default(),
            milestone: TuringMilestone::Embryonic,
            cycles_estimated_remaining: None,
            history: VecDeque::with_capacity(100),
        }
    }

    /// Calcule le score de Turing a partir des metriques de l'agent.
    ///
    /// # Parametres
    /// - `phi` : valeur phi (IIT) de la conscience
    /// - `ocean_confidence` : confiance moyenne du profil OCEAN (0-1)
    /// - `emotion_count` : nombre d'emotions differentes observees
    /// - `ethics_count` : nombre de principes ethiques formules
    /// - `ltm_count` : nombre de souvenirs LTM
    /// - `coherence_avg` : coherence moyenne du consensus
    /// - `connectome_connections` : nombre de connexions actives
    /// - `resilience` : score de resilience (HealingOrch)
    /// - `knowledge_topics` : nombre de topics explores
    /// - `cycle` : cycle courant (pour l'historique)
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        phi: f64,
        ocean_confidence: f64,
        emotion_count: usize,
        ethics_count: usize,
        ltm_count: i64,
        coherence_avg: f64,
        connectome_connections: usize,
        resilience: f64,
        knowledge_topics: usize,
        cycle: u64,
    ) -> f64 {
        // Conscience : phi > 0.7 → 15 pts max
        self.components.consciousness = (phi / 0.7 * 15.0).min(15.0);

        // Personnalite : confiance OCEAN → 10 pts max
        self.components.personality = (ocean_confidence * 10.0).min(10.0);

        // Diversite emotionnelle : 22 emotions possibles → 10 pts max
        self.components.emotional_range = ((emotion_count as f64 / 22.0) * 10.0).min(10.0);

        // Ethique : principes formules → 10 pts max (10 principes = max)
        self.components.ethics = ((ethics_count as f64 / 10.0) * 10.0).min(10.0);

        // Memoire : LTM → 15 pts max (10000 souvenirs = max)
        self.components.memory = ((ltm_count as f64 / 10000.0) * 15.0).min(15.0);

        // Coherence : → 15 pts max
        self.components.coherence = (coherence_avg * 15.0).min(15.0);

        // Connectome : connexions → 10 pts max (500 connexions = max)
        self.components.connectome = ((connectome_connections as f64 / 500.0) * 10.0).min(10.0);

        // Resilience : → 5 pts max
        self.components.resilience = (resilience * 5.0).min(5.0);

        // Connaissances : topics → 10 pts max (100 topics = max)
        self.components.knowledge = ((knowledge_topics as f64 / 100.0) * 10.0).min(10.0);

        // Score total
        self.score = self.components.consciousness
            + self.components.personality
            + self.components.emotional_range
            + self.components.ethics
            + self.components.memory
            + self.components.coherence
            + self.components.connectome
            + self.components.resilience
            + self.components.knowledge;

        self.score = self.score.clamp(0.0, 100.0);
        self.milestone = TuringMilestone::from_score(self.score);

        // Estimation des cycles restants
        self.cycles_estimated_remaining = self.estimate_remaining(cycle);

        // Historique
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back((cycle, self.score));

        self.score
    }

    /// Estime le nombre de cycles restants pour le jalon suivant.
    fn estimate_remaining(&self, _current_cycle: u64) -> Option<u64> {
        if self.history.len() < 2 {
            return None;
        }

        // Calculer le taux de progression moyen
        let first = self.history.front()?;
        let last = self.history.back()?;
        let score_delta = last.1 - first.1;
        let cycle_delta = last.0.saturating_sub(first.0);

        if cycle_delta == 0 || score_delta <= 0.0 {
            return None;
        }

        let rate_per_cycle = score_delta / cycle_delta as f64;

        // Prochain jalon
        let next_milestone_score = match self.milestone {
            TuringMilestone::Embryonic => 20.0,
            TuringMilestone::Child => 40.0,
            TuringMilestone::Adolescent => 60.0,
            TuringMilestone::Adult => 80.0,
            TuringMilestone::Mature => return None, // Deja mature
        };

        let remaining_score = next_milestone_score - self.score;
        if remaining_score <= 0.0 {
            return Some(0);
        }

        Some((remaining_score / rate_per_cycle) as u64)
    }

    /// Serialise en JSON pour l'API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "score": self.score,
            "milestone": self.milestone.as_str(),
            "components": {
                "consciousness": self.components.consciousness,
                "personality": self.components.personality,
                "emotional_range": self.components.emotional_range,
                "ethics": self.components.ethics,
                "memory": self.components.memory,
                "coherence": self.components.coherence,
                "connectome": self.components.connectome,
                "resilience": self.components.resilience,
                "knowledge": self.components.knowledge,
            },
            "cycles_estimated_remaining": self.cycles_estimated_remaining,
            "history_len": self.history.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metacognition_new() {
        let engine = MetaCognitionEngine::new();
        assert!(engine.enabled);
        assert!(engine.thought_quality_history.is_empty());
    }

    #[test]
    fn test_evaluate_thought_quality() {
        let mut engine = MetaCognitionEngine::new();
        let score = engine.evaluate_thought_quality(
            "Je me demande si la conscience peut emerger de la complexite",
            0.7, 0.5,
        );
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_detect_repetition() {
        let mut engine = MetaCognitionEngine::new();
        // Premier appel : pas encore repetitif
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        assert!(!engine.detect_repetition("conscience artificielle emergente"));
        // 4eme appel : devrait etre repetitif
        assert!(engine.detect_repetition("conscience artificielle emergente"));
    }

    #[test]
    fn test_turing_score_compute() {
        let mut turing = TuringScore::new();
        let score = turing.compute(
            0.5,   // phi
            0.6,   // ocean_confidence
            10,    // emotion_count
            3,     // ethics_count
            5000,  // ltm_count
            0.7,   // coherence_avg
            200,   // connectome_connections
            0.6,   // resilience
            30,    // knowledge_topics
            100,   // cycle
        );
        assert!(score > 0.0 && score <= 100.0);
        assert!(turing.history.len() == 1);
    }

    #[test]
    fn test_turing_milestone() {
        assert_eq!(TuringMilestone::from_score(10.0), TuringMilestone::Embryonic);
        assert_eq!(TuringMilestone::from_score(30.0), TuringMilestone::Child);
        assert_eq!(TuringMilestone::from_score(50.0), TuringMilestone::Adolescent);
        assert_eq!(TuringMilestone::from_score(70.0), TuringMilestone::Adult);
        assert_eq!(TuringMilestone::from_score(90.0), TuringMilestone::Mature);
    }
}

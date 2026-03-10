// =============================================================================
// prospective_memory.rs — Memoire prospective (intentions differees)
// =============================================================================
//
// Ce module permet a Saphire de se souvenir de faire quelque chose dans le futur.
// Contrairement a la memoire episodique (souvenir du passe), la memoire prospective
// stocke des intentions d'action associees a des conditions de declenchement :
//   - Temporelles (apres N cycles)
//   - Emotionnelles (quand une emotion specifique apparait)
//   - Chimiques (quand un neurotransmetteur depasse un seuil)
//   - Conversationnelles (au debut d'une conversation)
//   - Cognitives (quand un type de pensee specifique est genere)
//
// Les intentions sont prioritisees, expirent apres un delai configurable,
// et peuvent etre detectees automatiquement dans le flux de pensee.
//
// Place dans l'architecture :
//   Module de premier niveau, utilise par le pipeline cognitif. Les intentions
//   declenchees sont injectees dans le prompt LLM sous forme de rappels.
// =============================================================================

use serde::{Deserialize, Serialize};

// =============================================================================
// Configuration
// =============================================================================

/// Configuration de la memoire prospective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveMemoryConfig {
    /// Module actif ou non
    pub enabled: bool,
    /// Nombre maximum d'intentions stockees simultanement
    pub max_intentions: usize,
    /// Age maximum d'une intention avant expiration automatique (en cycles)
    pub max_age_cycles: u64,
}

impl Default for ProspectiveMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_intentions: 15,
            max_age_cycles: 1000,
        }
    }
}

// =============================================================================
// Types de declenchement
// =============================================================================

/// Type de condition de declenchement pour une intention differee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProspectiveTriggerType {
    /// Declenchement apres un nombre de cycles ecoules depuis la creation
    TimeBasedCycles(u64),
    /// Declenchement quand une emotion specifique est dominante
    EmotionBased(String),
    /// Declenchement quand une molecule depasse un seuil
    ChemistryBased { molecule: String, threshold: f64 },
    /// Declenchement au debut d'une nouvelle conversation
    ConversationStart,
    /// Declenchement quand un type de pensee specifique est genere
    ThoughtTypeMatch(String),
}

// =============================================================================
// Etat d'une intention
// =============================================================================

/// Etat du cycle de vie d'une intention differee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentionState {
    /// En attente de declenchement
    Pending,
    /// Condition remplie, action rappelée
    Triggered,
    /// Action accomplie
    Completed,
    /// Intention expiree (trop ancienne)
    Expired,
}

// =============================================================================
// Intention differee
// =============================================================================

/// Une intention differee : une action a effectuer quand une condition est remplie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredIntention {
    /// Identifiant unique
    pub id: u64,
    /// Description de l'action a effectuer
    pub action: String,
    /// Description textuelle de la condition de declenchement
    pub trigger_condition: String,
    /// Type de declenchement (determine la logique de verification)
    pub trigger_type: ProspectiveTriggerType,
    /// Priorite (0.0 = basse, 1.0 = haute)
    pub priority: f64,
    /// Cycle de creation
    pub created_at_cycle: u64,
    /// Cycle d'expiration optionnel (None = utilise max_age_cycles)
    pub expires_at_cycle: Option<u64>,
    /// Etat courant de l'intention
    pub state: IntentionState,
    /// Contexte d'origine (quelle pensee a genere cette intention)
    pub source_context: String,
}

// =============================================================================
// Memoire prospective
// =============================================================================

/// Memoire prospective — stocke et gere les intentions differees de Saphire.
///
/// Permet de se souvenir de faire quelque chose plus tard, quand les bonnes
/// conditions sont reunies. Les intentions declenchees sont presentees sous
/// forme de rappels dans le prompt LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveMemory {
    /// Module actif ou non
    pub enabled: bool,
    /// Liste de toutes les intentions (toutes etats confondus)
    pub intentions: Vec<DeferredIntention>,
    /// Actions declenchees durant le cycle courant (pour injection dans le prompt)
    pub triggered_this_cycle: Vec<String>,
    /// Nombre maximum d'intentions simultanees
    pub max_intentions: usize,
    /// Compteur total d'intentions completees depuis le demarrage
    pub total_completed: u64,
    /// Compteur total d'intentions expirees depuis le demarrage
    pub total_expired: u64,
    /// Age maximum d'une intention avant expiration (en cycles)
    max_age_cycles: u64,
    /// Prochain identifiant a attribuer
    next_id: u64,
}

impl ProspectiveMemory {
    /// Cree une nouvelle memoire prospective a partir de la configuration.
    pub fn new(config: &ProspectiveMemoryConfig) -> Self {
        Self {
            enabled: config.enabled,
            intentions: Vec::new(),
            triggered_this_cycle: Vec::new(),
            max_intentions: config.max_intentions,
            total_completed: 0,
            total_expired: 0,
            max_age_cycles: config.max_age_cycles,
            next_id: 1,
        }
    }

    /// Enregistre une nouvelle intention differee.
    ///
    /// Si la memoire est pleine (max_intentions atteint), l'intention en attente
    /// avec la priorite la plus basse est supprimee pour faire de la place.
    ///
    /// Retourne l'identifiant unique de l'intention creee.
    pub fn register(
        &mut self,
        action: &str,
        trigger_type: ProspectiveTriggerType,
        priority: f64,
        cycle: u64,
        source: &str,
    ) -> u64 {
        if !self.enabled {
            return 0;
        }

        // Si la memoire est pleine, ejecter l'intention pending la moins prioritaire
        let pending_count = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .count();

        if pending_count >= self.max_intentions {
            // Trouver l'indice de l'intention pending avec la priorite la plus basse
            if let Some((idx, _)) = self.intentions.iter().enumerate()
                .filter(|(_, i)| i.state == IntentionState::Pending)
                .min_by(|(_, a), (_, b)| a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal))
            {
                // Ne remplacer que si la nouvelle intention est plus prioritaire
                if priority > self.intentions[idx].priority {
                    self.intentions.remove(idx);
                } else {
                    // Pas de place et pas assez prioritaire
                    return 0;
                }
            }
        }

        // Generer la description de la condition
        let trigger_condition = match &trigger_type {
            ProspectiveTriggerType::TimeBasedCycles(n) =>
                format!("apres {} cycles", n),
            ProspectiveTriggerType::EmotionBased(emotion) =>
                format!("quand emotion = {}", emotion),
            ProspectiveTriggerType::ChemistryBased { molecule, threshold } =>
                format!("quand {} > {:.2}", molecule, threshold),
            ProspectiveTriggerType::ConversationStart =>
                "au debut de la prochaine conversation".to_string(),
            ProspectiveTriggerType::ThoughtTypeMatch(tt) =>
                format!("quand type de pensee = {}", tt),
        };

        let id = self.next_id;
        self.next_id += 1;

        let intention = DeferredIntention {
            id,
            action: action.to_string(),
            trigger_condition,
            trigger_type,
            priority: priority.clamp(0.0, 1.0),
            created_at_cycle: cycle,
            expires_at_cycle: Some(cycle + self.max_age_cycles),
            state: IntentionState::Pending,
            source_context: source.to_string(),
        };

        self.intentions.push(intention);
        id
    }

    /// Verifie les conditions de declenchement de toutes les intentions en attente.
    ///
    /// Pour chaque intention pending, evalue sa condition selon son type :
    /// - TimeBasedCycles : le nombre de cycles ecoules depasse le seuil
    /// - EmotionBased : l'emotion courante correspond
    /// - ChemistryBased : le niveau chimique depasse le seuil
    /// - ConversationStart : une conversation est en cours
    /// - ThoughtTypeMatch : le type de pensee correspond
    ///
    /// Retourne la liste des actions a effectuer (intentions declenchees ce cycle).
    pub fn check_triggers(
        &mut self,
        cycle: u64,
        emotion: &str,
        chemistry_cortisol: f64,
        chemistry_dopamine: f64,
        in_conversation: bool,
        thought_type: &str,
    ) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }

        self.triggered_this_cycle.clear();
        let mut triggered_actions = Vec::new();

        for intention in &mut self.intentions {
            if intention.state != IntentionState::Pending {
                continue;
            }

            let should_trigger = match &intention.trigger_type {
                ProspectiveTriggerType::TimeBasedCycles(n) => {
                    cycle.saturating_sub(intention.created_at_cycle) >= *n
                }
                ProspectiveTriggerType::EmotionBased(target_emotion) => {
                    emotion.to_lowercase().contains(&target_emotion.to_lowercase())
                }
                ProspectiveTriggerType::ChemistryBased { molecule, threshold } => {
                    let level = match molecule.to_lowercase().as_str() {
                        "cortisol" => chemistry_cortisol,
                        "dopamine" | "dopamin" => chemistry_dopamine,
                        _ => 0.0,
                    };
                    level > *threshold
                }
                ProspectiveTriggerType::ConversationStart => {
                    in_conversation
                }
                ProspectiveTriggerType::ThoughtTypeMatch(target_type) => {
                    thought_type.to_lowercase().contains(&target_type.to_lowercase())
                }
            };

            if should_trigger {
                intention.state = IntentionState::Triggered;
                triggered_actions.push(intention.action.clone());
                self.triggered_this_cycle.push(intention.action.clone());
            }
        }

        triggered_actions
    }

    /// Detecte des intentions implicites dans le texte de pensee.
    ///
    /// Recherche des motifs comme :
    /// - "je dois me souvenir de ..."
    /// - "la prochaine fois que ..."
    /// - "quand je serai ..."
    /// - "ne pas oublier de ..."
    /// - "il faudra ..."
    ///
    /// Cree automatiquement des intentions avec une priorite moderee.
    /// Retourne le nombre d'intentions creees.
    pub fn parse_from_thought(&mut self, thought_text: &str, cycle: u64) -> usize {
        if !self.enabled {
            return 0;
        }

        let text_lower = thought_text.to_lowercase();
        let mut created = 0;

        // --- Motif : "je dois me souvenir de X" / "me rappeler de X" ---
        let remember_patterns = [
            "je dois me souvenir de ",
            "me rappeler de ",
            "ne pas oublier de ",
            "il faudra ",
            "je devrai ",
            "penser a ",
        ];

        for pattern in &remember_patterns {
            if let Some(pos) = text_lower.find(pattern) {
                let start = pos + pattern.len();
                let action = extract_action_from_text(thought_text, start);
                if !action.is_empty() && action.len() > 3 {
                    self.register(
                        &action,
                        ProspectiveTriggerType::TimeBasedCycles(10),
                        0.5,
                        cycle,
                        thought_text,
                    );
                    created += 1;
                }
            }
        }

        // --- Motif : "la prochaine fois que X" ---
        if let Some(pos) = text_lower.find("la prochaine fois que ") {
            let start = pos + "la prochaine fois que ".len();
            let action = extract_action_from_text(thought_text, start);
            if !action.is_empty() && action.len() > 3 {
                // La prochaine conversation
                self.register(
                    &action,
                    ProspectiveTriggerType::ConversationStart,
                    0.6,
                    cycle,
                    thought_text,
                );
                created += 1;
            }
        }

        // --- Motif : "quand je serai X" ---
        if let Some(pos) = text_lower.find("quand je serai ") {
            let start = pos + "quand je serai ".len();
            let rest = extract_action_from_text(thought_text, start);
            if !rest.is_empty() && rest.len() > 3 {
                // Condition emotionnelle — essayer d'extraire l'emotion cible
                let emotion_keywords = [
                    "triste", "joyeux", "joyeuse", "calme", "stresse", "stressée",
                    "serein", "sereine", "en colere", "curieux", "curieuse",
                ];
                let mut found_emotion = false;
                for keyword in &emotion_keywords {
                    if rest.to_lowercase().contains(keyword) {
                        self.register(
                            &rest,
                            ProspectiveTriggerType::EmotionBased(keyword.to_string()),
                            0.6,
                            cycle,
                            thought_text,
                        );
                        created += 1;
                        found_emotion = true;
                        break;
                    }
                }
                // Fallback : intention temporelle
                if !found_emotion {
                    self.register(
                        &rest,
                        ProspectiveTriggerType::TimeBasedCycles(50),
                        0.4,
                        cycle,
                        thought_text,
                    );
                    created += 1;
                }
            }
        }

        created
    }

    /// Expire les intentions trop anciennes.
    ///
    /// Toute intention en attente dont l'age depasse max_age_cycles
    /// ou dont expires_at_cycle est depasse passe a l'etat Expired.
    pub fn expire_old(&mut self, cycle: u64) {
        for intention in &mut self.intentions {
            if intention.state != IntentionState::Pending {
                continue;
            }

            let age = cycle.saturating_sub(intention.created_at_cycle);
            let should_expire = age > self.max_age_cycles
                || intention.expires_at_cycle.map_or(false, |exp| cycle >= exp);

            if should_expire {
                intention.state = IntentionState::Expired;
                self.total_expired += 1;
            }
        }

        // Nettoyer les intentions terminees ou expirees anciennes (garder les 50 dernieres)
        let completed_or_expired: Vec<usize> = self.intentions.iter().enumerate()
            .filter(|(_, i)| i.state == IntentionState::Completed || i.state == IntentionState::Expired)
            .map(|(idx, _)| idx)
            .collect();

        if completed_or_expired.len() > 50 {
            let to_remove = completed_or_expired.len() - 50;
            let mut removed = 0;
            self.intentions.retain(|i| {
                if removed >= to_remove {
                    return true;
                }
                if i.state == IntentionState::Completed || i.state == IntentionState::Expired {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Marque une intention declenchee comme completee.
    pub fn mark_completed(&mut self, id: u64) {
        if let Some(intention) = self.intentions.iter_mut().find(|i| i.id == id) {
            if intention.state == IntentionState::Triggered {
                intention.state = IntentionState::Completed;
                self.total_completed += 1;
            }
        }
    }

    /// Genere une description des intentions declenchees ce cycle pour le prompt LLM.
    ///
    /// Format : "RAPPEL : [action]" pour chaque intention declenchee.
    /// Retourne une chaine vide si aucune intention n'a ete declenchee.
    pub fn describe_triggered_for_prompt(&self) -> String {
        if self.triggered_this_cycle.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        for action in &self.triggered_this_cycle {
            lines.push(format!("RAPPEL : {}", action));
        }
        lines.join("\n")
    }

    /// Serialise l'etat complet de la memoire prospective en JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let pending: Vec<_> = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .collect();

        let triggered: Vec<_> = self.intentions.iter()
            .filter(|i| i.state == IntentionState::Triggered)
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "pending_count": pending.len(),
            "triggered_count": triggered.len(),
            "total_completed": self.total_completed,
            "total_expired": self.total_expired,
            "max_intentions": self.max_intentions,
            "max_age_cycles": self.max_age_cycles,
            "triggered_this_cycle": self.triggered_this_cycle,
            "intentions": self.intentions.iter().map(|i| {
                serde_json::json!({
                    "id": i.id,
                    "action": i.action,
                    "trigger_condition": i.trigger_condition,
                    "priority": i.priority,
                    "state": format!("{:?}", i.state),
                    "created_at_cycle": i.created_at_cycle,
                    "expires_at_cycle": i.expires_at_cycle,
                    "source_context": truncate_str(&i.source_context, 80),
                })
            }).collect::<Vec<_>>(),
        })
    }
}

// =============================================================================
// Fonctions utilitaires
// =============================================================================

/// Extrait une action du texte a partir d'une position donnee.
/// S'arrete au premier point, point-virgule, saut de ligne, ou fin de chaine.
/// Limite a 200 caracteres.
fn extract_action_from_text(text: &str, start: usize) -> String {
    let rest = if start < text.len() { &text[start..] } else { "" };

    let end = rest.find(|c: char| c == '.' || c == ';' || c == '\n')
        .unwrap_or(rest.len())
        .min(200);

    rest[..end].trim().to_string()
}

/// Tronque une chaine a une longueur maximale.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.min(s.len())])
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn default_memory() -> ProspectiveMemory {
        ProspectiveMemory::new(&ProspectiveMemoryConfig::default())
    }

    #[test]
    fn test_register_and_check_time_based() {
        let mut mem = default_memory();
        let id = mem.register(
            "verifier les logs",
            ProspectiveTriggerType::TimeBasedCycles(10),
            0.5,
            100,
            "pensee de maintenance",
        );
        assert!(id > 0);
        assert_eq!(mem.intentions.len(), 1);

        // Pas encore le moment
        let triggered = mem.check_triggers(105, "neutre", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Maintenant oui (10 cycles ecoules)
        let triggered = mem.check_triggers(110, "neutre", 0.3, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "verifier les logs");
    }

    #[test]
    fn test_register_emotion_based() {
        let mut mem = default_memory();
        mem.register(
            "exprimer de la gratitude",
            ProspectiveTriggerType::EmotionBased("joie".to_string()),
            0.7,
            50,
            "reflexion sur les liens",
        );

        // Pas la bonne emotion
        let triggered = mem.check_triggers(51, "tristesse", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Bonne emotion
        let triggered = mem.check_triggers(52, "Joie profonde", 0.3, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
    }

    #[test]
    fn test_register_chemistry_based() {
        let mut mem = default_memory();
        mem.register(
            "prendre du recul",
            ProspectiveTriggerType::ChemistryBased {
                molecule: "cortisol".to_string(),
                threshold: 0.7,
            },
            0.8,
            10,
            "stress eleve detecte",
        );

        // Cortisol pas assez haut
        let triggered = mem.check_triggers(11, "neutre", 0.5, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // Cortisol au-dessus du seuil
        let triggered = mem.check_triggers(12, "neutre", 0.8, 0.4, false, "reflexion");
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "prendre du recul");
    }

    #[test]
    fn test_conversation_start_trigger() {
        let mut mem = default_memory();
        mem.register(
            "saluer l'utilisateur",
            ProspectiveTriggerType::ConversationStart,
            0.9,
            0,
            "intention de politesse",
        );

        // Pas en conversation
        let triggered = mem.check_triggers(1, "neutre", 0.3, 0.4, false, "reflexion");
        assert!(triggered.is_empty());

        // En conversation
        let triggered = mem.check_triggers(2, "neutre", 0.3, 0.4, true, "reflexion");
        assert_eq!(triggered.len(), 1);
    }

    #[test]
    fn test_max_intentions_eviction() {
        let config = ProspectiveMemoryConfig {
            enabled: true,
            max_intentions: 3,
            max_age_cycles: 1000,
        };
        let mut mem = ProspectiveMemory::new(&config);

        // Remplir 3 intentions
        mem.register("a", ProspectiveTriggerType::TimeBasedCycles(100), 0.3, 0, "ctx");
        mem.register("b", ProspectiveTriggerType::TimeBasedCycles(100), 0.5, 0, "ctx");
        mem.register("c", ProspectiveTriggerType::TimeBasedCycles(100), 0.7, 0, "ctx");
        assert_eq!(mem.intentions.len(), 3);

        // Ajouter une 4eme avec priorite plus haute que la plus basse
        mem.register("d", ProspectiveTriggerType::TimeBasedCycles(100), 0.9, 0, "ctx");
        assert_eq!(mem.intentions.len(), 3);
        // L'intention "a" (priorite 0.3) doit avoir ete ejectee
        assert!(!mem.intentions.iter().any(|i| i.action == "a"));
        assert!(mem.intentions.iter().any(|i| i.action == "d"));
    }

    #[test]
    fn test_expire_old() {
        let config = ProspectiveMemoryConfig {
            enabled: true,
            max_intentions: 15,
            max_age_cycles: 100,
        };
        let mut mem = ProspectiveMemory::new(&config);
        mem.register("ancienne", ProspectiveTriggerType::TimeBasedCycles(200), 0.5, 0, "ctx");
        mem.register("recente", ProspectiveTriggerType::TimeBasedCycles(200), 0.5, 95, "ctx");

        mem.expire_old(101);

        let pending: Vec<_> = mem.intentions.iter()
            .filter(|i| i.state == IntentionState::Pending)
            .collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, "recente");
        assert_eq!(mem.total_expired, 1);
    }

    #[test]
    fn test_parse_from_thought() {
        let mut mem = default_memory();
        let count = mem.parse_from_thought(
            "Je dois me souvenir de verifier les parametres demain",
            10,
        );
        assert_eq!(count, 1);
        assert_eq!(mem.intentions.len(), 1);
        assert!(mem.intentions[0].action.contains("verifier les parametres"));
    }

    #[test]
    fn test_parse_la_prochaine_fois() {
        let mut mem = default_memory();
        let count = mem.parse_from_thought(
            "La prochaine fois que je parle a Bob, lui demander des nouvelles",
            20,
        );
        assert_eq!(count, 1);
        // Devrait etre ConversationStart
        assert!(matches!(
            mem.intentions[0].trigger_type,
            ProspectiveTriggerType::ConversationStart
        ));
    }

    #[test]
    fn test_describe_triggered_for_prompt() {
        let mut mem = default_memory();
        mem.register(
            "respirer profondement",
            ProspectiveTriggerType::ConversationStart,
            0.5,
            0,
            "soin",
        );
        mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");

        let desc = mem.describe_triggered_for_prompt();
        assert!(desc.contains("RAPPEL"));
        assert!(desc.contains("respirer profondement"));
    }

    #[test]
    fn test_mark_completed() {
        let mut mem = default_memory();
        let id = mem.register(
            "action test",
            ProspectiveTriggerType::ConversationStart,
            0.5,
            0,
            "test",
        );
        mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");
        assert_eq!(mem.intentions[0].state, IntentionState::Triggered);

        mem.mark_completed(id);
        assert_eq!(mem.intentions[0].state, IntentionState::Completed);
        assert_eq!(mem.total_completed, 1);
    }

    #[test]
    fn test_disabled_memory() {
        let config = ProspectiveMemoryConfig {
            enabled: false,
            max_intentions: 15,
            max_age_cycles: 1000,
        };
        let mut mem = ProspectiveMemory::new(&config);
        let id = mem.register("test", ProspectiveTriggerType::ConversationStart, 0.5, 0, "ctx");
        assert_eq!(id, 0);
        assert!(mem.intentions.is_empty());

        let triggered = mem.check_triggers(1, "neutre", 0.3, 0.4, true, "reflexion");
        assert!(triggered.is_empty());
    }

    #[test]
    fn test_to_json() {
        let mut mem = default_memory();
        mem.register("tester json", ProspectiveTriggerType::TimeBasedCycles(5), 0.5, 0, "ctx");
        let json = mem.to_json();
        assert_eq!(json["pending_count"], 1);
        assert_eq!(json["enabled"], true);
    }
}

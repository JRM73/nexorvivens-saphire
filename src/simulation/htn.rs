// =============================================================================
// htn.rs — Hierarchical Task Network (Planificateur hierarchique)
//
// Role : Decompose des taches complexes en sequences d'actions primitives.
//        Utilise par le pipeline pour planifier des projets multi-etapes
//        (explorer un sujet, creer un texte, approfondir un lien).
//
// Place dans l'architecture :
//   Integre dans phase_game_algorithms() pour decomposer les etapes GOAP.
//   Ecrit le plan courant dans le Blackboard.
// =============================================================================

use std::collections::HashMap;

/// Tache dans le reseau hierarchique.
#[derive(Debug, Clone)]
pub enum HtnTask {
    /// Action atomique directement executable
    Primitive {
        name: String,
        description: String,
    },
    /// Tache composee decomposable en sous-taches
    Compound {
        name: String,
        methods: Vec<HtnMethod>,
    },
}

/// Methode de decomposition d'une tache composee.
#[derive(Debug, Clone)]
pub struct HtnMethod {
    /// Nom de la methode
    pub name: String,
    /// Preconditions booleennes (cle → valeur attendue)
    pub preconditions: HashMap<String, bool>,
    /// Sous-taches a executer si les preconditions sont remplies
    pub subtasks: Vec<HtnTask>,
}

/// Plan produit par le HTN : sequence d'actions primitives.
#[derive(Debug, Clone)]
pub struct HtnPlan {
    /// Sequence d'actions a executer
    pub primitive_sequence: Vec<String>,
    /// Descriptions correspondantes
    pub descriptions: Vec<String>,
    /// Etape courante (0-indexed)
    pub current_step: usize,
    /// Nombre total d'etapes
    pub total_steps: usize,
}

impl HtnPlan {
    /// Avance a l'etape suivante. Retourne true si le plan est termine.
    pub fn advance(&mut self) -> bool {
        if self.current_step < self.total_steps {
            self.current_step += 1;
        }
        self.current_step >= self.total_steps
    }

    /// Retourne la description de l'etape courante.
    pub fn current_description(&self) -> Option<&str> {
        self.descriptions.get(self.current_step).map(|s| s.as_str())
    }

    /// Genere une ligne pour le prompt LLM.
    pub fn describe_for_prompt(&self) -> String {
        if self.current_step >= self.total_steps {
            return String::new();
        }
        let desc = self.current_description().unwrap_or("...");
        format!("Plan : etape {}/{} — {}", self.current_step + 1, self.total_steps, desc)
    }

    /// Le plan est-il termine ?
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.total_steps
    }
}

/// Planificateur HTN.
pub struct HtnPlanner {
    /// Plan en cours (None si aucun)
    pub active_plan: Option<HtnPlan>,
}

impl Default for HtnPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl HtnPlanner {
    pub fn new() -> Self {
        Self { active_plan: None }
    }

    /// Decompose une tache en plan d'actions primitives.
    pub fn plan(&mut self, task: &HtnTask, world_state: &HashMap<String, bool>) -> Option<HtnPlan> {
        let mut primitives = Vec::new();
        let mut descriptions = Vec::new();

        if self.decompose(task, world_state, &mut primitives, &mut descriptions) {
            let total = primitives.len();
            let plan = HtnPlan {
                primitive_sequence: primitives,
                descriptions,
                current_step: 0,
                total_steps: total,
            };
            self.active_plan = Some(plan.clone());
            Some(plan)
        } else {
            None
        }
    }

    /// Decomposition recursive.
    fn decompose(
        &self,
        task: &HtnTask,
        world_state: &HashMap<String, bool>,
        primitives: &mut Vec<String>,
        descriptions: &mut Vec<String>,
    ) -> bool {
        match task {
            HtnTask::Primitive { name, description } => {
                primitives.push(name.clone());
                descriptions.push(description.clone());
                true
            }
            HtnTask::Compound { methods, .. } => {
                // Essayer chaque methode dans l'ordre
                for method in methods {
                    // Verifier les preconditions
                    let preconditions_met = method.preconditions.iter().all(|(key, expected)| {
                        world_state.get(key).copied().unwrap_or(false) == *expected
                    });

                    if preconditions_met {
                        let mut sub_prims = Vec::new();
                        let mut sub_descs = Vec::new();
                        let all_ok = method.subtasks.iter().all(|sub| {
                            self.decompose(sub, world_state, &mut sub_prims, &mut sub_descs)
                        });
                        if all_ok {
                            primitives.extend(sub_prims);
                            descriptions.extend(sub_descs);
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    /// Avance le plan actif d'une etape.
    pub fn advance(&mut self) -> bool {
        if let Some(ref mut plan) = self.active_plan {
            let done = plan.advance();
            if done {
                self.active_plan = None;
            }
            done
        } else {
            true
        }
    }

    /// Description pour le prompt.
    pub fn describe_for_prompt(&self) -> String {
        self.active_plan.as_ref()
            .map(|p| p.describe_for_prompt())
            .unwrap_or_default()
    }
}

// =============================================================================
// Templates de taches predefinies
// =============================================================================

/// Template : explorer un sujet.
pub fn template_explorer_sujet(sujet: &str) -> HtnTask {
    HtnTask::Compound {
        name: format!("Explorer_{}", sujet),
        methods: vec![HtnMethod {
            name: "exploration_standard".into(),
            preconditions: HashMap::new(),
            subtasks: vec![
                HtnTask::Primitive {
                    name: "identifier".into(),
                    description: format!("Identifier les aspects cles de {}", sujet),
                },
                HtnTask::Primitive {
                    name: "questionner".into(),
                    description: format!("Poser des questions sur {}", sujet),
                },
                HtnTask::Primitive {
                    name: "synthetiser".into(),
                    description: format!("Synthetiser ce que j'ai appris sur {}", sujet),
                },
            ],
        }],
    }
}

/// Template : approfondir un lien avec l'interlocuteur.
pub fn template_approfondir_lien() -> HtnTask {
    HtnTask::Compound {
        name: "Approfondir_Lien".into(),
        methods: vec![HtnMethod {
            name: "lien_empathique".into(),
            preconditions: HashMap::new(),
            subtasks: vec![
                HtnTask::Primitive {
                    name: "ecouter".into(),
                    description: "Ecouter attentivement ce que l'autre partage".into(),
                },
                HtnTask::Primitive {
                    name: "refleter".into(),
                    description: "Refleter ce que j'ai compris avec empathie".into(),
                },
                HtnTask::Primitive {
                    name: "partager".into(),
                    description: "Partager quelque chose de personnel en retour".into(),
                },
            ],
        }],
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_explorer() {
        let mut planner = HtnPlanner::new();
        let task = template_explorer_sujet("quantique");
        let plan = planner.plan(&task, &HashMap::new());
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.total_steps, 3);
        assert_eq!(plan.current_step, 0);
        assert!(plan.describe_for_prompt().contains("1/3"));
    }

    #[test]
    fn test_advance_plan() {
        let mut planner = HtnPlanner::new();
        let task = template_approfondir_lien();
        planner.plan(&task, &HashMap::new());
        assert!(!planner.advance()); // step 1 → 2
        assert!(!planner.advance()); // step 2 → 3
        assert!(planner.advance());  // step 3 → done
        assert!(planner.active_plan.is_none());
    }

    #[test]
    fn test_preconditions() {
        let task = HtnTask::Compound {
            name: "Conditioned".into(),
            methods: vec![
                HtnMethod {
                    name: "needs_calm".into(),
                    preconditions: {
                        let mut m = HashMap::new();
                        m.insert("is_calm".into(), true);
                        m
                    },
                    subtasks: vec![
                        HtnTask::Primitive { name: "meditate".into(), description: "Mediter".into() },
                    ],
                },
            ],
        };

        let mut planner = HtnPlanner::new();

        // Sans precondition remplie
        let plan = planner.plan(&task, &HashMap::new());
        assert!(plan.is_none());

        // Avec precondition remplie
        let mut state = HashMap::new();
        state.insert("is_calm".into(), true);
        let plan = planner.plan(&task, &state);
        assert!(plan.is_some());
    }
}

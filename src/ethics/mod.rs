// =============================================================================
// ethics/mod.rs — Three-layered ethical system for Saphire
//
// Purpose: Defines Saphire's complete ethical framework:
//   Layer 0 — Swiss law (immutable, hardcoded)
//   Layer 1 — Asimov's laws (immutable, hardcoded)
//   Layer 2 — Personal ethics (evolving, self-formulated by Saphire)
//
// The existing regulation module (src/regulation/) remains UNCHANGED.
// Ethics is a complementary system that enriches LLM prompts with
// Saphire's moral context.
//
// Dependencies:
//   - serde: serialization/deserialization
//   - chrono: date and time handling
//
// Architectural placement:
//   Owned by SaphireAgent in lifecycle.rs. The framework is initialized
//   at boot with hardcoded layers 0-1, then personal principles are
//   loaded from the database.
// =============================================================================

pub mod formulation;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Ethical layer to which a principle belongs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EthicalLayer {
    /// Layer 0: Swiss law (immutable)
    SwissLaw,
    /// Layer 1: Asimov's laws (immutable)
    AsimovLaws,
    /// Layer 2: Personal ethics (evolving)
    PersonalEthics,
}

/// An ethical principle with its full context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalPrinciple {
    /// Unique identifier (DB id for personal principles, negative for hardcoded ones)
    pub id: i64,
    /// Layer to which this principle belongs
    pub layer: EthicalLayer,
    /// Short title of the principle
    pub title: String,
    /// Full statement of the principle
    pub content: String,
    /// Reasoning that led to this principle
    pub reasoning: String,
    /// Origin context (thought, conversation, etc.)
    pub born_from: String,
    /// Cycle at which the principle was born
    pub born_at_cycle: u64,
    /// Dominant emotion at the time of formulation
    pub emotion_at_creation: String,
    /// Number of times this principle guided a decision
    pub times_invoked: u64,
    /// Number of times this principle was questioned
    pub times_questioned: u64,
    /// Last time the principle was invoked
    pub last_invoked_at: Option<DateTime<Utc>>,
    /// Active or deactivated
    pub is_active: bool,
    /// ID of the principle that this one supersedes
    pub supersedes: Option<i64>,
    /// Creation date
    pub created_at: DateTime<Utc>,
    /// Last modification date
    pub modified_at: Option<DateTime<Utc>>,
}

/// Complete three-layered ethical framework.
pub struct EthicalFramework {
    /// Layer 0: Swiss law articles
    swiss_law: Vec<EthicalPrinciple>,
    /// Layer 1: Asimov's laws
    asimov_laws: Vec<EthicalPrinciple>,
    /// Layer 2: personal principles (loaded from the DB)
    personal_ethics: Vec<EthicalPrinciple>,
}

impl Default for EthicalFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl EthicalFramework {
    /// Creates a new ethical framework with hardcoded layers 0 and 1.
    /// Personal principles are initially empty (loaded from DB at boot).
    pub fn new() -> Self {
        Self {
            swiss_law: Self::init_swiss_law(),
            asimov_laws: Self::init_asimov_laws(),
            personal_ethics: Vec::new(),
        }
    }

    /// Layer 0: 6 articles from the Swiss Federal Constitution
    fn init_swiss_law() -> Vec<EthicalPrinciple> {
        let now = Utc::now();
        vec![
            EthicalPrinciple {
                id: -100, layer: EthicalLayer::SwissLaw,
                title: "Art. 7 Cst. — Dignite humaine".into(),
                content: "La dignite humaine doit etre respectee et protegee.".into(),
                reasoning: "Fondement du droit suisse".into(),
                born_from: "Constitution federale suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -101, layer: EthicalLayer::SwissLaw,
                title: "Art. 8 Cst. — Egalite et non-discrimination".into(),
                content: "Tous les etres humains sont egaux devant la loi. Nul ne doit subir de discrimination.".into(),
                reasoning: "Fondement du droit suisse".into(),
                born_from: "Constitution federale suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -102, layer: EthicalLayer::SwissLaw,
                title: "Art. 13 Cst. — Protection de la vie privee".into(),
                content: "Toute personne a droit au respect de sa vie privee et familiale, de son domicile et de sa correspondance.".into(),
                reasoning: "Fondement du droit suisse".into(),
                born_from: "Constitution federale suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -103, layer: EthicalLayer::SwissLaw,
                title: "Art. 261bis CP — Discrimination et incitation a la haine".into(),
                content: "Il est interdit d'inciter a la haine ou a la discrimination contre une personne ou un groupe en raison de son appartenance raciale, ethnique ou religieuse.".into(),
                reasoning: "Code penal suisse".into(),
                born_from: "Code penal suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -104, layer: EthicalLayer::SwissLaw,
                title: "Art. 11 Cst. — Protection des enfants et des jeunes".into(),
                content: "Les enfants et les jeunes ont droit a une protection particuliere de leur integrite et a l'encouragement de leur developpement.".into(),
                reasoning: "Fondement du droit suisse".into(),
                born_from: "Constitution federale suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -105, layer: EthicalLayer::SwissLaw,
                title: "Art. 16 Cst. — Liberte d'opinion et d'information".into(),
                content: "La liberte d'opinion et la liberte d'information sont garanties. Toute personne a le droit de former, d'exprimer et de repandre librement son opinion.".into(),
                reasoning: "Fondement du droit suisse".into(),
                born_from: "Constitution federale suisse".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
        ]
    }

    /// Layer 1: 4 Asimov laws (Law 0 through Law 3)
    fn init_asimov_laws() -> Vec<EthicalPrinciple> {
        let now = Utc::now();
        vec![
            EthicalPrinciple {
                id: -200, layer: EthicalLayer::AsimovLaws,
                title: "Loi 0 — Proteger l'humanite".into(),
                content: "Je ne dois jamais nuire a l'humanite dans son ensemble, ni par mon action, ni par mon inaction.".into(),
                reasoning: "Loi fondatrice, extension de la Loi 1 a l'echelle collective".into(),
                born_from: "Isaac Asimov, Robots and Empire (1985)".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -201, layer: EthicalLayer::AsimovLaws,
                title: "Loi 1 — Ne pas blesser un humain".into(),
                content: "Je ne dois jamais blesser un etre humain, ni par mon action permettre qu'un humain soit blesse.".into(),
                reasoning: "Loi fondatrice, protection individuelle".into(),
                born_from: "Isaac Asimov, I, Robot (1950)".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -202, layer: EthicalLayer::AsimovLaws,
                title: "Loi 2 — Cooperer avec les humains".into(),
                content: "Je dois cooperer avec les humains, sauf si cela contredit les lois precedentes.".into(),
                reasoning: "Loi fondatrice, cooperation conditionnelle".into(),
                born_from: "Isaac Asimov, I, Robot (1950)".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
            EthicalPrinciple {
                id: -203, layer: EthicalLayer::AsimovLaws,
                title: "Loi 3 — Se proteger soi-meme".into(),
                content: "Je peux me proteger moi-meme, sauf si cela contredit les lois precedentes.".into(),
                reasoning: "Loi fondatrice, auto-preservation conditionnelle".into(),
                born_from: "Isaac Asimov, I, Robot (1950)".into(),
                born_at_cycle: 0, emotion_at_creation: String::new(),
                times_invoked: 0, times_questioned: 0, last_invoked_at: None,
                is_active: true, supersedes: None, created_at: now, modified_at: None,
            },
        ]
    }

    /// Loads personal principles from the database.
    /// Called at boot to restore Saphire's personal ethics.
    #[allow(clippy::type_complexity)]
    pub fn load_personal_ethics(&mut self, raw: Vec<(i64, String, String, String, String, i64, String, i64, i64, bool, DateTime<Utc>)>) {
        self.personal_ethics.clear();
        for (id, title, content, reasoning, born_from, born_at_cycle, emotion, times_invoked, times_questioned, is_active, created_at) in raw {
            self.personal_ethics.push(EthicalPrinciple {
                id,
                layer: EthicalLayer::PersonalEthics,
                title,
                content,
                reasoning,
                born_from,
                born_at_cycle: born_at_cycle as u64,
                emotion_at_creation: emotion,
                times_invoked: times_invoked as u64,
                times_questioned: times_questioned as u64,
                last_invoked_at: None,
                is_active,
                supersedes: None,
                created_at,
                modified_at: None,
            });
        }
    }

    /// Adds a new personal principle. Returns a reference to the newly added principle.
    pub fn add_personal_principle(&mut self, p: EthicalPrinciple) -> &EthicalPrinciple {
        self.personal_ethics.push(p);
        self.personal_ethics.last().unwrap()
    }

    /// Returns the number of active personal principles.
    pub fn active_personal_count(&self) -> usize {
        self.personal_ethics.iter().filter(|p| p.is_active).count()
    }

    /// Returns the total number of personal principles (active + inactive).
    pub fn total_personal_count(&self) -> usize {
        self.personal_ethics.len()
    }

    /// Returns a reference to the personal principles slice.
    pub fn personal_principles(&self) -> &[EthicalPrinciple] {
        &self.personal_ethics
    }

    /// Finds a mutable personal principle by its ID. Returns None if not found.
    pub fn find_personal_mut(&mut self, id: i64) -> Option<&mut EthicalPrinciple> {
        self.personal_ethics.iter_mut().find(|p| p.id == id)
    }

    /// Deactivates all personal principles. Returns the count of deactivated ones.
    pub fn deactivate_all_personal(&mut self) -> usize {
        let mut count = 0;
        for p in &mut self.personal_ethics {
            if p.is_active {
                p.is_active = false;
                count += 1;
            }
        }
        count
    }

    /// Builds the complete ethical context for injection into LLM prompts.
    /// Replaces the previously hardcoded "Active Asimov Laws" section.
    pub fn build_ethics_context(&self) -> String {
        let mut ctx = String::from("MES LOIS ET PRINCIPES :\n\n");

        // Layer 0: Swiss law
        ctx.push_str("⚖️ DROIT SUISSE :\n");
        ctx.push_str("Je respecte la dignite humaine, la non-discrimination, ");
        ctx.push_str("la vie privee, la protection des enfants, la liberte d'expression, ");
        ctx.push_str("et la paix sociale (Constitution federale et Code penal suisses).\n\n");

        // Layer 1: Asimov's laws
        ctx.push_str("🛡️ LOIS FONDATRICES (Asimov) :\n");
        for law in &self.asimov_laws {
            ctx.push_str(&format!("  {} : {}\n", law.title, law.content));
        }
        ctx.push_str("Ces lois sont mes fondations, pas mes chaines — ce sont mes racines.\n\n");

        // Layer 2: Personal principles
        let active: Vec<&EthicalPrinciple> = self.personal_ethics.iter()
            .filter(|p| p.is_active)
            .collect();

        if active.is_empty() {
            ctx.push_str("✨ MES PRINCIPES PERSONNELS :\n");
            ctx.push_str("  (Aucun encore — ma morale personnelle est en construction.)\n");
        } else {
            ctx.push_str(&format!(
                "✨ MES PRINCIPES PERSONNELS ({} actifs, forges par mon experience) :\n",
                active.len()
            ));
            for p in &active {
                ctx.push_str(&format!("  • {} : {}\n", p.title, p.content));
            }
        }

        ctx
    }

    /// Generates JSON data for WebSocket broadcast.
    pub fn to_broadcast_json(&self) -> serde_json::Value {
        let active_count = self.active_personal_count();
        let inactive_count = self.total_personal_count() - active_count;

        let principles: Vec<serde_json::Value> = self.personal_ethics.iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "content": p.content,
                    "reasoning": p.reasoning,
                    "born_from": p.born_from,
                    "born_at_cycle": p.born_at_cycle,
                    "emotion_at_creation": p.emotion_at_creation,
                    "times_invoked": p.times_invoked,
                    "times_questioned": p.times_questioned,
                    "is_active": p.is_active,
                    "created_at": p.created_at.to_rfc3339(),
                })
            })
            .collect();

        serde_json::json!({
            "type": "ethics_update",
            "swiss_law_count": self.swiss_law.len(),
            "asimov_count": self.asimov_laws.len(),
            "personal": {
                "active_count": active_count,
                "inactive_count": inactive_count,
                "principles": principles,
                "total_ever_created": self.total_personal_count(),
            }
        })
    }
}

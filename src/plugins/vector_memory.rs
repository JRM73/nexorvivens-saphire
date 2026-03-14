// =============================================================================
// vector_memory.rs — Plugin memoire vectorielle
//
// Role : Ce plugin gere une memoire vectorielle en RAM et calcule la
// personnalite emergente de l'agent a partir de l'historique des emotions.
// Les souvenirs sont stockes comme des vecteurs d'embedding permettant
// la recherche par similarite cosinus.
//
// Dependances :
//   - super : trait Plugin, BrainEvent, PluginAction (systeme de plugins)
//   - crate::vectorstore : VectorStore (stockage et recherche vectorielle en RAM)
//   - crate::vectorstore::personality : EmergentPersonality (traits de personnalite)
//
// Place dans l'architecture :
//   Ce plugin est enregistre dans le PluginManager. Il reagit aux evenements
//   CycleCompleted (pour recalculer la personnalite periodiquement) et
//   ThoughtEmitted (pour stocker les pensees comme souvenirs vectoriels).
//   La personnalite emergente est utilisee par l'agent pour enrichir
//   son auto-description et adapter son comportement.
// =============================================================================

use super::{Plugin, BrainEvent, PluginAction};
use crate::vectorstore::VectorStore;
use crate::vectorstore::personality::EmergentPersonality;

/// Plugin de memoire vectorielle avec personnalite emergente.
/// Stocke des souvenirs sous forme de vecteurs d'embedding et calcule
/// periodiquement des traits de personnalite a partir de l'historique emotionnel.
pub struct VectorMemoryPlugin {
    /// Le magasin de vecteurs en RAM (stockage + recherche par similarite)
    store: VectorStore,
    /// Personnalite emergente calculee a partir des emotions des souvenirs.
    /// Se met a jour periodiquement a mesure que l'agent accumule des experiences.
    personality: EmergentPersonality,
    /// Nombre de cycles entre chaque recalcul de la personnalite
    update_interval: u64,
    /// Compteur de cycles depuis la creation du plugin
    cycle_count: u64,
}

impl VectorMemoryPlugin {
    /// Cree un nouveau plugin de memoire vectorielle.
    ///
    /// # Parametres
    /// - `embedding_dim` : nombre de dimensions des vecteurs d'embedding
    /// - `max_memories` : nombre maximal de souvenirs stockes en RAM
    pub fn new(embedding_dim: usize, max_memories: usize) -> Self {
        Self {
            store: VectorStore::new(embedding_dim, max_memories),
            personality: EmergentPersonality {
                traits: std::collections::HashMap::new(),
                description: "Personnalité en formation...".into(),
                memory_count: 0,
            },
            update_interval: 20, // Recalcul tous les 20 cycles
            cycle_count: 0,
        }
    }

    /// Retourne une reference en lecture seule vers le magasin de vecteurs.
    /// Utilise par l'agent pour effectuer des recherches par similarite.
    pub fn store(&self) -> &VectorStore {
        &self.store
    }

    /// Retourne une reference mutable vers le magasin de vecteurs.
    /// Utilise par l'agent pour ajouter des souvenirs.
    pub fn store_mut(&mut self) -> &mut VectorStore {
        &mut self.store
    }

    /// Retourne une reference vers la personnalite emergente.
    /// La personnalite est recalculee periodiquement et reflette
    /// les emotions dominantes dans l'historique des souvenirs.
    pub fn personality(&self) -> &EmergentPersonality {
        &self.personality
    }

    /// Recalcule la personnalite emergente a partir des emotions de tous les souvenirs.
    /// Extrait la liste des emotions, puis utilise EmergentPersonality::compute()
    /// pour deduire les traits de personnalite (ex: "curieuse", "empathique").
    fn update_personality(&mut self) {
        let emotions: Vec<String> = self.store.memories().iter()
            .map(|m| m.emotion.clone())
            .collect();
        self.personality = EmergentPersonality::compute(&emotions);
    }
}

impl Plugin for VectorMemoryPlugin {
    /// Retourne le nom du plugin.
    fn name(&self) -> &str {
        "VectorMemory"
    }

    /// Reagit aux evenements du cerveau :
    ///
    /// - CycleCompleted : incremente le compteur de cycles et recalcule la
    ///   personnalite si l'intervalle est atteint. Le recalcul periodique
    ///   (plutot qu'a chaque cycle) evite une charge CPU excessive.
    ///
    /// - ThoughtEmitted : chaque pensee autonome est stockee comme souvenir
    ///   vectoriel via une action StoreMemory, avec une importance moyenne (0.5).
    ///   Cela permet a l'agent de se souvenir de ses propres reflexions.
    ///
    /// # Parametres
    /// - `event` : l'evenement du cerveau
    ///
    /// # Retour
    /// Liste d'actions (StoreMemory pour les pensees, vide sinon)
    fn on_event(&mut self, event: &BrainEvent) -> Vec<PluginAction> {
        match event {
            BrainEvent::CycleCompleted { emotion: _, .. } => {
                self.cycle_count += 1;
                // Mettre a jour la personnalite emergente periodiquement
                if self.cycle_count.is_multiple_of(self.update_interval) {
                    self.update_personality();
                }
                vec![]
            },
            BrainEvent::ThoughtEmitted { content, .. } => {
                // Stocker les pensees autonomes comme souvenirs vectoriels
                vec![PluginAction::StoreMemory {
                    text: content.clone(),
                    emotion: String::new(), // L'emotion sera determinee par l'agent
                    importance: 0.5,        // Importance moyenne par defaut
                }]
            },
            _ => vec![],
        }
    }
}

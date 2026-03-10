// working.rs — Mémoire de travail (RAM uniquement, 7 items max, loi de Miller)
//
// Ce module implémente la mémoire de travail de Saphire, inspirée du modèle
// cognitif humain. La mémoire de travail est un tampon à capacité limitée
// (par défaut 7 éléments, selon la loi de Miller : 7 ± 2) qui stocke les
// informations immédiatement pertinentes pour le cycle cognitif en cours.
//
// Caractéristiques :
//   - Volatile : stockée uniquement en RAM, jamais persistée directement.
//   - Décroissance : chaque item perd de la pertinence à chaque cycle.
//   - Éjection : quand un item atteint une pertinence de 0 ou que la capacité
//     est dépassée, il est éjecté et transféré vers la mémoire épisodique.
//
// Dépendances :
//   - VecDeque (std) : file à double extrémité pour un accès efficace.
//   - chrono : horodatage de la création des items.
//   - serde : sérialisation pour l'envoi via WebSocket au tableau de bord.

use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::neurochemistry::ChemicalSignature;

/// Source d'un élément en mémoire de travail.
///
/// Chaque variante identifie l'origine de l'information et contient soit
/// le contenu textuel (String), soit l'identifiant en base (i64) du
/// souvenir rappelé.
#[derive(Debug, Clone, Serialize)]
pub enum WorkingItemSource {
    /// Message reçu de l'utilisateur humain.
    UserMessage(String),
    /// Pensée générée en interne par le système cognitif de Saphire.
    OwnThought(String),
    /// Réponse produite par le LLM (Large Language Model = Grand Modèle de Langage).
    LlmResponse(String),
    /// Connaissance acquise depuis le web.
    WebKnowledge(String),
    /// Souvenir rappelé depuis la mémoire épisodique (contient l'ID en DB).
    EpisodicRecall(i64),
    /// Souvenir rappelé depuis la mémoire à long terme (contient l'ID en DB).
    LongTermRecall(i64),
}

impl WorkingItemSource {
    /// Retourne un libellé textuel identifiant le type de source.
    /// Utilisé pour la sérialisation JSON et l'affichage dans le tableau de bord.
    ///
    /// # Retour
    /// Une chaîne statique décrivant le type de source (ex : "user_message").
    pub fn label(&self) -> &str {
        match self {
            WorkingItemSource::UserMessage(_) => "user_message",
            WorkingItemSource::OwnThought(_) => "thought",
            WorkingItemSource::LlmResponse(_) => "llm_response",
            WorkingItemSource::WebKnowledge(_) => "knowledge",
            WorkingItemSource::EpisodicRecall(_) => "episodic_recall",
            WorkingItemSource::LongTermRecall(_) => "long_term_recall",
        }
    }

    /// Retourne une icône Unicode correspondant au type de source.
    /// Utilisée dans le résumé de contexte et le tableau de bord WebSocket.
    ///
    /// # Retour
    /// Une chaîne statique contenant un émoji Unicode.
    pub fn icon(&self) -> &str {
        match self {
            WorkingItemSource::UserMessage(_) => "\u{1f4ac}",   // bulle de parole
            WorkingItemSource::OwnThought(_) => "\u{1f4ad}",    // bulle de pensée
            WorkingItemSource::LlmResponse(_) => "\u{1f9e0}",   // cerveau
            WorkingItemSource::WebKnowledge(_) => "\u{1f4da}",   // livres
            WorkingItemSource::EpisodicRecall(_) => "\u{1f4dd}", // mémo
            WorkingItemSource::LongTermRecall(_) => "\u{1f48e}", // gemme
        }
    }
}

/// Un élément stocké dans la mémoire de travail.
///
/// Chaque item représente une unité d'information active dans la « conscience »
/// de Saphire. Sa pertinence décroît au fil des cycles cognitifs.
#[derive(Debug, Clone, Serialize)]
pub struct WorkingItem {
    /// Identifiant unique séquentiel au sein de la mémoire de travail.
    pub id: u64,
    /// Contenu textuel de l'item (message, pensée, connaissance, etc.).
    pub content: String,
    /// Origine de cet item (message utilisateur, pensée interne, rappel, etc.).
    pub source: WorkingItemSource,
    /// Score de pertinence actuel, entre 0.0 (oublié) et 1.0 (très pertinent).
    /// Décroît de `decay_rate` à chaque cycle cognitif.
    pub relevance: f64,
    /// Horodatage UTC de la création de cet item.
    pub created_at: DateTime<Utc>,
    /// État émotionnel de Saphire au moment de la création de cet item.
    pub emotion_at_creation: String,
    /// Signature chimique au moment de la creation de cet item.
    pub chemical_signature: ChemicalSignature,
}

/// Mémoire de travail — tampon de conscience à capacité limitée.
///
/// Fonctionne comme une file à priorité basée sur la pertinence. Quand la
/// capacité maximale est atteinte, l'item le moins pertinent est éjecté
/// et peut être récupéré pour transfert vers la mémoire épisodique.
pub struct WorkingMemory {
    /// File à double extrémité contenant les items actifs.
    items: VecDeque<WorkingItem>,
    /// Capacité maximale (nombre d'items, typiquement 7).
    max_capacity: usize,
    /// Compteur séquentiel pour l'attribution des identifiants d'items.
    next_id: u64,
    /// Taux de décroissance de la pertinence appliqué à chaque cycle.
    decay_rate: f64,
}

impl WorkingMemory {
    /// Crée une nouvelle mémoire de travail.
    ///
    /// # Paramètres
    /// - `capacity` : nombre maximal d'items simultanés (typiquement 7).
    /// - `decay_rate` : quantité soustraite à la pertinence de chaque item
    ///   à chaque cycle cognitif (ex : 0.05 = perte de 5% par cycle).
    ///
    /// # Retour
    /// Une instance de WorkingMemory vide, prête à recevoir des items.
    pub fn new(capacity: usize, decay_rate: f64) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
            next_id: 0,
            decay_rate,
        }
    }

    /// Ajoute un nouvel élément dans la mémoire de travail.
    ///
    /// Si la mémoire est pleine, l'item ayant la pertinence la plus basse
    /// est éjecté pour faire de la place. L'item éjecté est retourné afin
    /// que l'appelant puisse le transférer vers la mémoire épisodique.
    ///
    /// Le nouvel item commence avec une pertinence de 1.0 (maximale).
    ///
    /// # Paramètres
    /// - `content` : contenu textuel de l'item.
    /// - `source` : origine de l'information (message, pensée, rappel, etc.).
    /// - `emotion` : état émotionnel actuel de Saphire.
    ///
    /// # Retour
    /// `Some(WorkingItem)` si un item a été éjecté, `None` sinon.
    pub fn push(
        &mut self,
        content: String,
        source: WorkingItemSource,
        emotion: String,
        chemical_signature: ChemicalSignature,
    ) -> Option<WorkingItem> {
        let item = WorkingItem {
            id: self.next_id,
            content,
            source,
            relevance: 1.0,
            created_at: Utc::now(),
            emotion_at_creation: emotion,
            chemical_signature,
        };
        self.next_id += 1;

        let ejected = if self.items.len() >= self.max_capacity {
            // Éjecter l'élément le moins pertinent pour respecter la capacité.
            // On recherche l'index de l'item avec le score de pertinence minimal
            // via un parcours linéaire (la file est petite, max ~7 éléments).
            if let Some(min_idx) = self.items.iter()
                .enumerate()
                .min_by(|a, b| a.1.relevance.partial_cmp(&b.1.relevance).unwrap())
                .map(|(i, _)| i)
            {
                self.items.remove(min_idx)
            } else {
                None
            }
        } else {
            None
        };

        self.items.push_back(item);
        ejected
    }

    /// Applique la décroissance de pertinence sur tous les items.
    ///
    /// Appelée à chaque cycle cognitif. Chaque item perd `decay_rate` de
    /// pertinence. Les items dont la pertinence tombe à 0 ou moins sont
    /// automatiquement retirés et retournés pour transfert épisodique.
    ///
    /// # Retour
    /// Liste des items éjectés (pertinence tombée à 0).
    pub fn decay(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        // Appliquer la décroissance à chaque item, sans descendre en dessous de 0.
        for item in &mut self.items {
            item.relevance = (item.relevance - self.decay_rate).max(0.0);
        }
        // Retirer les items dont la pertinence est tombée à zéro.
        // On utilise une boucle while car les indices changent après chaque retrait.
        while let Some(pos) = self.items.iter().position(|item| item.relevance <= 0.0) {
            if let Some(item) = self.items.remove(pos) {
                ejected.push(item);
            }
        }
        ejected
    }

    /// Renforce la pertinence d'un item spécifique (augmentation de 0.3).
    ///
    /// Utilisé quand un item redevient pertinent dans le contexte courant
    /// (par exemple, quand l'utilisateur revient sur un sujet déjà évoqué).
    /// La pertinence est plafonnée à 1.0.
    ///
    /// # Paramètres
    /// - `id` : identifiant de l'item à renforcer.
    pub fn reinforce(&mut self, id: u64) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.relevance = (item.relevance + 0.3).min(1.0);
        }
    }

    /// Génère un résumé textuel du contenu actuel de la mémoire de travail.
    ///
    /// Les items sont triés par pertinence décroissante pour que le LLM
    /// voie les informations les plus pertinentes en premier.
    ///
    /// # Retour
    /// Une chaîne formatée avec chaque item précédé de son icône de source,
    /// ou une chaîne vide si la mémoire de travail est vide.
    pub fn context_summary(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        // Trier par pertinence décroissante pour prioriser les items les plus importants.
        let mut sorted: Vec<&WorkingItem> = self.items.iter().collect();
        sorted.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());

        let mut summary = String::from("CONTEXTE IMMEDIAT (memoire de travail) :\n");
        for item in &sorted {
            summary.push_str(&format!("  {} {}\n", item.source.icon(), item.content));
        }
        summary
    }

    /// Vide entièrement la mémoire de travail et retourne tous les items.
    ///
    /// Utilisé lors d'un transfert massif vers la mémoire épisodique
    /// (par exemple lors du « sommeil » de Saphire).
    ///
    /// # Retour
    /// Vecteur contenant tous les items retirés.
    pub fn drain_all(&mut self) -> Vec<WorkingItem> {
        self.items.drain(..).collect()
    }

    /// Vide uniquement les items liés à la conversation (messages utilisateur
    /// et réponses LLM), tout en conservant les pensées internes, connaissances
    /// web et rappels mémoriels.
    ///
    /// Appelée en fin de conversation pour nettoyer le contexte conversationnel
    /// sans perdre les réflexions de fond.
    ///
    /// # Retour
    /// Vecteur des items conversationnels éjectés (pour transfert épisodique).
    pub fn flush_conversation(&mut self) -> Vec<WorkingItem> {
        let mut ejected = Vec::new();
        let mut keep = VecDeque::new();
        // Séparer les items conversationnels (à éjecter) des items internes (à garder).
        for item in self.items.drain(..) {
            match &item.source {
                WorkingItemSource::UserMessage(_) | WorkingItemSource::LlmResponse(_) => {
                    ejected.push(item);
                },
                _ => keep.push_back(item),
            }
        }
        self.items = keep;
        ejected
    }

    /// Retourne le nombre d'éléments actuellement en mémoire de travail.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Retourne vrai si la mémoire de travail ne contient aucun item.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Retourne la capacité maximale configurée.
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Retourne une référence en lecture seule vers les items actuels.
    pub fn items(&self) -> &VecDeque<WorkingItem> {
        &self.items
    }

    /// Produit une structure JSON destinée à être envoyée via WebSocket
    /// au tableau de bord de visualisation en temps réel.
    ///
    /// Chaque item est sérialisé avec un aperçu du contenu tronqué à 100
    /// caractères, son type de source, son icône, sa pertinence et l'émotion
    /// associée. Le JSON inclut aussi la capacité totale et le nombre d'items
    /// utilisés.
    ///
    /// # Retour
    /// Un objet `serde_json::Value` prêt à être sérialisé en JSON.
    pub fn ws_data(&self) -> serde_json::Value {
        let items: Vec<serde_json::Value> = self.items.iter().map(|item| {
            // Tronquer le contenu à 100 caractères pour l'affichage dans le dashboard.
            let content_preview: String = if item.content.len() > 100 {
                let preview: String = item.content.chars().take(100).collect();
                format!("{}...", preview)
            } else {
                item.content.clone()
            };
            serde_json::json!({
                "id": item.id,
                "content": content_preview,
                "source": item.source.label(),
                "icon": item.source.icon(),
                "relevance": item.relevance,
                "emotion": item.emotion_at_creation,
                "chemical_signature": item.chemical_signature,
            })
        }).collect();

        serde_json::json!({
            "items": items,
            "capacity": self.max_capacity,
            "used": self.items.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_limit() {
        let mut wm = WorkingMemory::new(7, 0.05);
        for i in 0..10 {
            wm.push(format!("item_{}", i), WorkingItemSource::OwnThought("test".into()), "Curiosite".into(), ChemicalSignature::default());
        }
        assert_eq!(wm.len(), 7, "Working memory should not exceed capacity");
    }

    #[test]
    fn test_push_returns_evicted_when_full() {
        let mut wm = WorkingMemory::new(3, 0.05);
        wm.push("a".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("b".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("c".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let evicted = wm.push("d".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        assert!(evicted.is_some(), "Should evict an item when at capacity");
    }

    #[test]
    fn test_decay_reduces_relevance() {
        let mut wm = WorkingMemory::new(7, 0.1);
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let initial = wm.items().front().unwrap().relevance;
        wm.decay();
        let after = wm.items().front().unwrap().relevance;
        assert!(after < initial, "Decay should reduce relevance");
    }

    #[test]
    fn test_decay_removes_zero_relevance() {
        let mut wm = WorkingMemory::new(7, 1.0); // Very aggressive decay
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let dropped = wm.decay();
        assert!(!dropped.is_empty(), "Extreme decay should remove items");
        assert_eq!(wm.len(), 0, "All items should be gone");
    }

    #[test]
    fn test_reinforce_increases_relevance() {
        let mut wm = WorkingMemory::new(7, 0.1);
        wm.push("test".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.decay(); // Reduce relevance first
        let id = wm.items().front().unwrap().id;
        let before = wm.items().front().unwrap().relevance;
        wm.reinforce(id);
        let after = wm.items().front().unwrap().relevance;
        assert!(after > before, "Reinforce should increase relevance");
    }

    #[test]
    fn test_drain_all() {
        let mut wm = WorkingMemory::new(7, 0.05);
        wm.push("a".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        wm.push("b".into(), WorkingItemSource::OwnThought("t".into()), "".into(), ChemicalSignature::default());
        let drained = wm.drain_all();
        assert_eq!(drained.len(), 2);
        assert_eq!(wm.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let wm = WorkingMemory::new(7, 0.05);
        assert!(wm.is_empty());
    }
}

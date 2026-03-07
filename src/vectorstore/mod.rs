// vectorstore/mod.rs — Mémoire vectorielle (TF-IDF local + cosine brute-force)
//
// Ce module implémente un magasin de vecteurs (vector store) en mémoire RAM
// pour la recherche sémantique de souvenirs par similarité cosinus.
//
// Il sert de couche de recherche complémentaire à pgvector (en base de données).
// Les souvenirs sont encodés en vecteurs de dimension fixe via un encodeur
// local basé sur TF-IDF (Term Frequency-Inverse Document Frequency =
// Fréquence du Terme - Fréquence Inverse du Document) avec hachage de
// n-grammes par FNV-1a (Fowler-Noll-Vo 1a).
//
// La recherche utilise une approche brute-force (force brute) : chaque
// requête est comparée à tous les souvenirs stockés. Cela est acceptable
// car le nombre de souvenirs en RAM est limité par `max_memories`.
//
// Architecture :
//   - VectorMemory : structure de données d'un souvenir vectoriel.
//   - VectorStore : conteneur avec encodage, stockage, recherche et élagage.
//   - cosine_similarity : fonction utilitaire de calcul de similarité.
//
// Sous-modules :
//   - encoder : encodeur local TF-IDF (LocalEncoder).
//   - personality : personnalité émergente calculée depuis les souvenirs.
//
// Dépendances :
//   - serde : sérialisation / désérialisation des souvenirs vectoriels.

pub mod encoder;
pub mod personality;

use serde::{Deserialize, Serialize};
use self::encoder::LocalEncoder;

/// Un souvenir encodé sous forme vectorielle.
///
/// Associe un texte brut à son embedding (vecteur numérique de dimension fixe),
/// une émotion et un score d'importance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMemory {
    /// Contenu textuel du souvenir.
    pub text: String,
    /// Vecteur d'embedding de dimension fixe, produit par le LocalEncoder.
    /// Représente le contenu sémantique du texte dans l'espace vectoriel.
    pub embedding: Vec<f64>,
    /// Émotion associée à ce souvenir (ex : "Joie", "Curiosité").
    pub emotion: String,
    /// Score d'importance du souvenir, entre 0.0 (insignifiant) et 1.0 (crucial).
    /// Utilisé pour l'élagage : les souvenirs les moins importants sont supprimés
    /// en premier quand la capacité maximale est atteinte.
    pub importance: f64,
}

/// Magasin vectoriel en mémoire avec recherche par similarité cosinus.
///
/// Stocke les souvenirs vectoriels en RAM et fournit des méthodes de
/// recherche sémantique par force brute (comparaison de chaque souvenir
/// avec la requête). L'élagage automatique supprime les souvenirs les
/// moins importants quand la capacité maximale est dépassée.
pub struct VectorStore {
    /// Liste des souvenirs vectoriels stockés.
    memories: Vec<VectorMemory>,
    /// Encodeur local pour transformer du texte en vecteurs d'embedding.
    encoder: LocalEncoder,
    /// Nombre maximal de souvenirs autorisés. Au-delà, élagage par importance.
    max_memories: usize,
}

impl VectorStore {
    /// Crée un nouveau magasin vectoriel vide.
    ///
    /// # Paramètres
    /// - `embedding_dim` : dimension des vecteurs d'embedding (ex : 256, 512).
    ///   Détermine la taille de l'espace vectoriel utilisé par le LocalEncoder.
    /// - `max_memories` : capacité maximale en nombre de souvenirs.
    ///
    /// # Retour
    /// Un VectorStore vide avec un encodeur configuré.
    pub fn new(embedding_dim: usize, max_memories: usize) -> Self {
        Self {
            memories: Vec::new(),
            encoder: LocalEncoder::new(embedding_dim),
            max_memories,
        }
    }

    /// Ajoute un souvenir dans le magasin vectoriel.
    ///
    /// Le texte est automatiquement encodé en vecteur d'embedding via le
    /// LocalEncoder. Si le nombre de souvenirs dépasse `max_memories`,
    /// les souvenirs les moins importants sont élagués (triés par importance
    /// décroissante, puis tronqués).
    ///
    /// # Paramètres
    /// - `text` : contenu textuel du souvenir.
    /// - `emotion` : émotion associée au souvenir.
    /// - `importance` : score d'importance (0.0 à 1.0).
    pub fn add(&mut self, text: &str, emotion: &str, importance: f64) {
        let embedding = self.encoder.encode(text);
        self.memories.push(VectorMemory {
            text: text.to_string(),
            embedding,
            emotion: emotion.to_string(),
            importance,
        });

        // Élaguer si la capacité maximale est dépassée.
        // On trie par importance décroissante et on ne garde que les
        // `max_memories` souvenirs les plus importants.
        if self.memories.len() > self.max_memories {
            self.memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal));
            self.memories.truncate(self.max_memories);
        }
    }

    /// Ajoute un souvenir avec un embedding déjà calculé en externe.
    ///
    /// Contrairement à `add()`, cette méthode ne recalcule pas l'embedding
    /// et ne déclenche pas d'élagage automatique. Utile pour le chargement
    /// de souvenirs depuis la base de données.
    ///
    /// # Paramètres
    /// - `text` : contenu textuel du souvenir.
    /// - `embedding` : vecteur d'embedding pré-calculé.
    /// - `emotion` : émotion associée au souvenir.
    /// - `importance` : score d'importance (0.0 à 1.0).
    pub fn add_with_embedding(&mut self, text: &str, embedding: Vec<f64>, emotion: &str, importance: f64) {
        self.memories.push(VectorMemory {
            text: text.to_string(),
            embedding,
            emotion: emotion.to_string(),
            importance,
        });
    }

    /// Recherche les K souvenirs les plus similaires à une requête textuelle.
    ///
    /// Le texte de la requête est encodé en vecteur d'embedding, puis
    /// comparé à tous les souvenirs par similarité cosinus.
    ///
    /// # Paramètres
    /// - `query` : texte de la requête de recherche.
    /// - `k` : nombre maximal de résultats à retourner.
    ///
    /// # Retour
    /// Vecteur de tuples (score de similarité, référence au souvenir),
    /// trié par similarité décroissante.
    pub fn search(&self, query: &str, k: usize) -> Vec<(f64, &VectorMemory)> {
        let query_embedding = self.encoder.encode(query);
        self.search_by_embedding(&query_embedding, k)
    }

    /// Recherche les K souvenirs les plus similaires à un embedding donné.
    ///
    /// Version de `search()` qui accepte un vecteur d'embedding pré-calculé
    /// au lieu d'un texte brut. Utile quand l'embedding est déjà disponible.
    ///
    /// Algorithme : force brute — on calcule la similarité cosinus entre
    /// le vecteur requête et chaque souvenir, puis on trie et on retourne
    /// les K meilleurs.
    ///
    /// # Paramètres
    /// - `query` : vecteur d'embedding de la requête.
    /// - `k` : nombre maximal de résultats à retourner.
    ///
    /// # Retour
    /// Vecteur de tuples (score de similarité, référence au souvenir),
    /// trié par similarité décroissante.
    pub fn search_by_embedding(&self, query: &[f64], k: usize) -> Vec<(f64, &VectorMemory)> {
        // Calculer la similarité cosinus entre la requête et chaque souvenir.
        let mut scored: Vec<(f64, &VectorMemory)> = self.memories.iter()
            .map(|mem| {
                let sim = cosine_similarity(query, &mem.embedding);
                (sim, mem)
            })
            .collect();

        // Trier par similarité décroissante et ne garder que les K meilleurs.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).collect()
    }

    /// Retourne le nombre de souvenirs actuellement stockes.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Verifie si le store est vide.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Retourne une tranche (slice) en lecture seule vers tous les souvenirs.
    pub fn memories(&self) -> &[VectorMemory] {
        &self.memories
    }

    /// Retourne une référence en lecture seule vers l'encodeur local.
    pub fn encoder(&self) -> &LocalEncoder {
        &self.encoder
    }
}

/// Calcule la similarité cosinus entre deux vecteurs.
///
/// La similarité cosinus mesure l'angle entre deux vecteurs dans l'espace
/// vectoriel, indépendamment de leur magnitude. Elle vaut :
///   - 1.0 si les vecteurs pointent dans la même direction (identiques).
///   - 0.0 si les vecteurs sont orthogonaux (aucun rapport).
///   - -1.0 si les vecteurs pointent dans des directions opposées.
///
/// Formule : cos(theta) = (A . B) / (||A|| * ||B||)
///
/// # Paramètres
/// - `a` : premier vecteur.
/// - `b` : second vecteur.
///
/// # Retour
/// Score de similarité entre -1.0 et 1.0. Retourne 0.0 si l'un des
/// vecteurs a une norme quasi nulle (< 1e-10), pour éviter la division
/// par zéro.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    // Produit scalaire (dot product) des deux vecteurs.
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    // Norme L2 (norme euclidienne) de chaque vecteur.
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    // Protection contre la division par zéro : si un vecteur est quasi nul,
    // la similarité n'a pas de sens, on retourne 0.
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    // Clampage à [-1.0, 1.0] pour compenser d'éventuelles erreurs d'arrondi
    // en virgule flottante.
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

// encoder.rs — Encodeur local TF-IDF avec hash FNV-1a
//
// Ce module implémente un encodeur vectoriel local qui transforme du texte
// en vecteurs de dimension fixe, sans dépendance à un modèle de langage
// externe (pas de réseau de neurones, pas d'API).
//
// Approche utilisée :
//   - Tokenisation par découpage sur les espaces blancs (whitespace).
//   - Extraction de n-grammes : unigrammes, bigrammes et trigrammes.
//   - Hachage par FNV-1a (Fowler-Noll-Vo 1a = algorithme de hachage
//     non cryptographique très rapide) pour projeter chaque n-gramme
//     dans un espace de dimension fixe.
//   - Pondération dégresssive : unigrammes (1.0), bigrammes (0.5),
//     trigrammes (0.25), simulant un TF-IDF (Term Frequency-Inverse
//     Document Frequency = Fréquence du Terme - Fréquence Inverse du
//     Document) simplifié où les n-grammes plus longs sont plus spécifiques
//     mais moins fréquents.
//   - Normalisation L2 (norme euclidienne) finale pour que tous les
//     vecteurs aient une norme unitaire, ce qui rend la similarité cosinus
//     équivalente au produit scalaire.
//
// Cette approche est légère, déterministe et ne nécessite aucun modèle
// pré-entraîné. Le compromis est une qualité sémantique inférieure aux
// embeddings neuronaux, mais suffisante pour le rappel approximatif.
//
// Dépendances : aucune (module autonome, bibliothèque standard uniquement).

/// Encodeur local : transforme un texte en vecteur de dimension fixe.
///
/// Utilise des n-grammes (uni, bi, tri) hashés par FNV-1a et projetés
/// dans un espace vectoriel de dimension `dim`. Le vecteur résultant
/// est normalisé L2.
pub struct LocalEncoder {
    /// Dimension de l'espace vectoriel (taille des vecteurs produits).
    /// Typiquement 256 ou 512. Une dimension plus grande réduit les
    /// collisions de hachage mais augmente l'utilisation mémoire.
    dim: usize,
}

impl LocalEncoder {
    /// Crée un nouvel encodeur avec la dimension vectorielle spécifiée.
    ///
    /// # Paramètres
    /// - `dim` : dimension de l'espace vectoriel (ex : 256, 512).
    ///
    /// # Retour
    /// Une instance de LocalEncoder prête à encoder du texte.
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// Encode un texte en vecteur de dimension fixe.
    ///
    /// Le processus d'encodage se déroule en 4 étapes :
    ///   1. Mise en minuscules et tokenisation par espaces blancs.
    ///   2. Accumulation des n-grammes hashés dans le vecteur.
    ///   3. Normalisation L2 pour obtenir un vecteur unitaire.
    ///
    /// # Paramètres
    /// - `text` : texte à encoder (peut être de longueur quelconque).
    ///
    /// # Retour
    /// Vecteur de dimension `self.dim`, normalisé L2. Si le texte est
    /// vide ou composé uniquement d'espaces, retourne un vecteur nul.
    pub fn encode(&self, text: &str) -> Vec<f64> {
        // Étape 1 : Normaliser le texte en minuscules et découper en tokens.
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();

        let mut vector = vec![0.0; self.dim];

        // Étape 2a : Unigrammes (mots individuels), poids 1.0.
        // Chaque mot est hashé par FNV-1a, puis projeté sur un index
        // du vecteur via modulo. Le compteur à cet index est incrémenté.
        for token in &tokens {
            let hash = fnv1a(token.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 1.0;
        }

        // Étape 2b : Bigrammes (paires de mots consécutifs), poids 0.5.
        // Les bigrammes capturent le contexte local et les expressions
        // de deux mots. Poids réduit car plus spécifiques que les unigrammes.
        for window in tokens.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            let hash = fnv1a(bigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.5;
        }

        // Étape 2c : Trigrammes (triplets de mots consécutifs), poids 0.25.
        // Les trigrammes capturent des motifs encore plus spécifiques.
        // Poids minimal pour refléter leur spécificité élevée.
        for window in tokens.windows(3) {
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            let hash = fnv1a(trigram.as_bytes());
            let idx = (hash as usize) % self.dim;
            vector[idx] += 0.25;
        }

        // Étape 3 : Normalisation L2 (norme euclidienne).
        // On divise chaque composante par la norme du vecteur pour obtenir
        // un vecteur unitaire. Cela permet de comparer les vecteurs par
        // produit scalaire plutôt que par similarité cosinus complète.
        let norm: f64 = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    /// Retourne la dimension de l'espace vectoriel de cet encodeur.
    pub fn dim(&self) -> usize {
        self.dim
    }
}

/// Calcule le hash FNV-1a 64 bits d'une séquence d'octets.
///
/// FNV-1a (Fowler-Noll-Vo 1a) est un algorithme de hachage non
/// cryptographique conçu pour être très rapide tout en ayant une bonne
/// distribution. L'opération de base est :
///   1. XOR de chaque octet avec le hash courant.
///   2. Multiplication par le nombre premier FNV (0x100000001b3).
///
/// La valeur initiale (offset basis) est 0xcbf29ce484222325.
///
/// # Paramètres
/// - `data` : séquence d'octets à hasher.
///
/// # Retour
/// Valeur de hash 64 bits non signée.
fn fnv1a(data: &[u8]) -> u64 {
    // Offset basis FNV-1a 64 bits (valeur de départ standardisée).
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        // XOR avec l'octet courant, puis multiplication par le prime FNV.
        // wrapping_mul gère le débordement arithmétique (overflow) sans panique.
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

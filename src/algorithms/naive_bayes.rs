// =============================================================================
// naive_bayes.rs — Classificateur Naive Bayes simplifié
// =============================================================================
//
// Rôle : Implémente un classificateur Naive Bayes (Bayes naïf) pour la
//        classification de texte. L'hypothèse « naïve » est l'indépendance
//        conditionnelle des mots sachant la classe.
//
// Dépendances :
//   - std::collections::HashMap : stockage des compteurs de mots et documents
//
// Place dans l'architecture :
//   Utilisé par Saphire pour l'analyse rapide du sentiment textuel et la
//   classification de documents en catégories. Fait partie du sous-module
//   algorithms/, et sert notamment au traitement NLP (Natural Language
//   Processing = Traitement Automatique du Langage Naturel) interne.
// =============================================================================

use std::collections::HashMap;

/// Classificateur Naive Bayes pour texte — utilise le théorème de Bayes
/// avec l'hypothèse d'indépendance conditionnelle des mots pour classer
/// des documents tokenisés en catégories.
pub struct NaiveBayesClassifier {
    /// Compteurs de mots par classe : pour chaque classe, stocke le nombre
    /// d'occurrences de chaque mot observé lors de l'entraînement.
    /// Structure : { "classe" → { "mot" → nombre_d_occurrences } }
    class_word_counts: HashMap<String, HashMap<String, u64>>,
    /// Compteur de documents par classe : nombre de documents d'entraînement
    /// appartenant à chaque classe. Sert au calcul du prior P(classe).
    class_doc_counts: HashMap<String, u64>,
    /// Taille du vocabulaire total (nombre de mots distincts observés
    /// sur toutes les classes). Utilisé pour le lissage de Laplace.
    vocab_size: usize,
    /// Nombre total de documents d'entraînement vus
    total_docs: u64,
}

impl Default for NaiveBayesClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl NaiveBayesClassifier {
    /// Crée un nouveau classificateur vierge, sans données d'entraînement.
    ///
    /// Retourne : une instance de NaiveBayesClassifier prête à être entraînée
    pub fn new() -> Self {
        Self {
            class_word_counts: HashMap::new(),
            class_doc_counts: HashMap::new(),
            vocab_size: 0,
            total_docs: 0,
        }
    }

    /// Entraîne le classificateur avec un document tokenisé et sa classe.
    ///
    /// Met à jour les compteurs de mots, de documents et la taille du vocabulaire.
    ///
    /// Paramètre `tokens` : liste des mots du document (déjà tokenisé)
    /// Paramètre `class` : étiquette de classe associée au document
    pub fn train(&mut self, tokens: &[String], class: &str) {
        // Incrémenter le compteur de documents total et par classe
        self.total_docs += 1;
        *self.class_doc_counts.entry(class.to_string()).or_insert(0) += 1;

        // Accéder (ou créer) la table de compteurs de mots pour cette classe
        let word_counts = self.class_word_counts
            .entry(class.to_string())
            .or_default();

        // Compter chaque occurrence de mot dans le document
        for token in tokens {
            *word_counts.entry(token.clone()).or_insert(0) += 1;
        }

        // Mettre à jour la taille du vocabulaire global
        // Pourquoi recalculer à chaque fois : un nouveau mot peut apparaître
        // dans n'importe quelle classe, et le vocabulaire est l'union de tous les mots
        let mut all_words = std::collections::HashSet::new();
        for counts in self.class_word_counts.values() {
            for word in counts.keys() {
                all_words.insert(word.clone());
            }
        }
        self.vocab_size = all_words.len();
    }

    /// Classifie un document tokenisé en utilisant le théorème de Bayes.
    ///
    /// Calcule pour chaque classe : log P(classe) + sum(log P(mot_i | classe))
    /// et retourne la classe ayant la probabilité a posteriori la plus élevée.
    ///
    /// Le lissage de Laplace (ajout de 1 au numérateur) est utilisé pour
    /// éviter les probabilités nulles sur les mots jamais vus.
    ///
    /// Paramètre `tokens` : liste des mots du document à classifier
    /// Retourne : tuple (nom_de_la_classe, confiance) où confiance est une
    ///            probabilité approximative entre 0.0 et 1.0
    pub fn predict(&self, tokens: &[String]) -> (String, f64) {
        // Si aucun document d'entraînement, on ne peut pas classifier
        if self.total_docs == 0 {
            return ("unknown".to_string(), 0.0);
        }

        let mut best_class = String::new();
        let mut best_log_prob = f64::NEG_INFINITY;

        for (class, doc_count) in &self.class_doc_counts {
            // P(classe) — probabilité a priori (prior)
            let log_prior = (*doc_count as f64 / self.total_docs as f64).ln();

            // P(mot|classe) — vraisemblance (likelihood) avec lissage de Laplace
            let word_counts = self.class_word_counts.get(class);
            let total_words: u64 = word_counts
                .map(|wc| wc.values().sum())
                .unwrap_or(0);

            let mut log_likelihood = 0.0;
            for token in tokens {
                let count = word_counts
                    .and_then(|wc| wc.get(token))
                    .copied()
                    .unwrap_or(0);
                // Lissage de Laplace : (count + 1) / (total_words + vocab_size)
                // Cela garantit qu'aucune probabilité n'est nulle, même pour
                // les mots absents du corpus d'entraînement de cette classe
                let prob = (count as f64 + 1.0) / (total_words as f64 + self.vocab_size as f64);
                log_likelihood += prob.ln();
            }

            // Score total en espace logarithmique (somme au lieu de produit)
            let log_prob = log_prior + log_likelihood;
            if log_prob > best_log_prob {
                best_log_prob = log_prob;
                best_class = class.clone();
            }
        }

        // Convertir le log-probabilité en une confiance approximative via sigmoïde
        // Pourquoi sigmoïde : elle borne la sortie dans [0, 1] même pour
        // des valeurs log très négatives
        let confidence = (1.0 / (1.0 + (-best_log_prob).exp())).clamp(0.0, 1.0);

        (best_class, confidence)
    }
}

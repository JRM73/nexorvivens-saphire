// =============================================================================
// naive_bayes.rs — Simplified Naive Bayes classifier
// =============================================================================
//
// Role: Implements a Naive Bayes classifier for text classification.
//  The "naive" assumption is conditional independence of words given
//  the class.
//
// Dependencies:
//  - std::collections::HashMap: storage of word and document counters
//
// Place in architecture:
//  Used by Saphire for rapid textual sentiment analysis and document
//  classification into categories. Part of the algorithms/ submodule,
//  and serves the internal NLP (Natural Language Processing) pipeline.
// =============================================================================

use std::collections::HashMap;

/// Naive Bayes text classifier — uses Bayes' theorem with the conditional
/// independence assumption of words to classify tokenized documents into
/// categories.
pub struct NaiveBayesClassifier {
    /// Word counts per class: for each class, stores the number of
    /// occurrences of each word observed during training.
    /// Structure: { "class" -> { "word" -> occurrence_count } }
    class_word_counts: HashMap<String, HashMap<String, u64>>,
    /// Document count per class: number of training documents belonging
    /// to each class. Used for computing the prior P(class).
    class_doc_counts: HashMap<String, u64>,
    /// Total vocabulary size (number of distinct words observed across
    /// all classes). Used for Laplace smoothing.
    vocab_size: usize,
    /// Total number of training documents seen
    total_docs: u64,
}

impl Default for NaiveBayesClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl NaiveBayesClassifier {
    /// Creates a new blank classifier, without training data.
    ///
    /// Returns: a NaiveBayesClassifier instance ready to be trained
    pub fn new() -> Self {
        Self {
            class_word_counts: HashMap::new(),
            class_doc_counts: HashMap::new(),
            vocab_size: 0,
            total_docs: 0,
        }
    }

    /// Trains the classifier with a tokenized document and its class.
    ///
    /// Updates the word counters, document counters, and vocabulary size.
    ///
    /// Parameter `tokens`: list of words from the document (already tokenized)
    /// Parameter `class`: class label associated with the document
    pub fn train(&mut self, tokens: &[String], class: &str) {
        // Increment total and per-class document counters
        self.total_docs += 1;
        *self.class_doc_counts.entry(class.to_string()).or_insert(0) += 1;

        // Access (or create) the word counter table for this class
        let word_counts = self.class_word_counts
            .entry(class.to_string())
            .or_default();

        // Count each word occurrence in the document
        for token in tokens {
            *word_counts.entry(token.clone()).or_insert(0) += 1;
        }

        // Update the overall vocabulary size
        // Why recompute each time: a new word may appear in any class,
        // and the vocabulary is the union of all words
        let mut all_words = std::collections::HashSet::new();
        for counts in self.class_word_counts.values() {
            for word in counts.keys() {
                all_words.insert(word.clone());
            }
        }
        self.vocab_size = all_words.len();
    }

    /// Classifies a tokenized document using Bayes' theorem.
    ///
    /// Computes for each class: log P(class) + sum(log P(word_i | class))
    /// and returns the class with the highest posterior probability.
    ///
    /// Laplace smoothing (adding 1 to the numerator) is used to avoid
    /// zero probabilities on never-seen words.
    ///
    /// Parameter `tokens`: list of words from the document to classify
    /// Returns: tuple (class_name, confidence) where confidence is an
    ///  approximate probability between 0.0 and 1.0
    pub fn predict(&self, tokens: &[String]) -> (String, f64) {
        // If no training documents, we cannot classify
        if self.total_docs == 0 {
            return ("unknown".to_string(), 0.0);
        }

        let mut best_class = String::new();
        let mut best_log_prob = f64::NEG_INFINITY;

        for (class, doc_count) in &self.class_doc_counts {
            // P(class) — prior probability
            let log_prior = (*doc_count as f64 / self.total_docs as f64).ln();

            // P(word|class) — likelihood with Laplace smoothing
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
                // Laplace smoothing: (count + 1) / (total_words + vocab_size)
                // This ensures no probability is zero, even for words
                // absent from this class's training corpus
                let prob = (count as f64 + 1.0) / (total_words as f64 + self.vocab_size as f64);
                log_likelihood += prob.ln();
            }

            // Total score in log space (sum instead of product)
            let log_prob = log_prior + log_likelihood;
            if log_prob > best_log_prob {
                best_log_prob = log_prob;
                best_class = class.clone();
            }
        }

        // Convert the log-probability into an approximate confidence via sigmoid
        // Why sigmoid: it bounds the output in [0, 1] even for very
        // negative log values
        let confidence = (1.0 / (1.0 + (-best_log_prob).exp())).clamp(0.0, 1.0);

        (best_class, confidence)
    }
}

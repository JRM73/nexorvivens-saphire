// =============================================================================
// nlp/stagnation.rs — Thematic stagnation detection (shared utility)
// =============================================================================
//
// Purpose: Detects whether a set of recent texts (thoughts, responses) is
//          revolving obsessively around the same theme. Used by thinking.rs,
//          conversation.rs, and llm.rs.
//
// Algorithm (keyword-based):
//   1. Extract significant keywords (> min_len characters) from each text.
//   2. Count how many texts contain each keyword.
//   3. If a keyword appears in > 60% of texts, it is flagged as
//      "obsessional".
//   4. If >= 3 obsessional keywords exist, stagnation is declared.
//
// Algorithm (semantic):
//   Uses cosine similarity on term-frequency (TF) vectors to detect
//   stagnation even when the speaker uses different words to express the
//   same underlying idea (synonym-based reformulation).
// =============================================================================

use std::collections::{HashMap, HashSet};

/// Extracts significant keywords (longer than `min_len` characters) from a
/// text.
///
/// # Parameters
/// - `text`: the input text to extract keywords from.
/// - `min_len`: minimum character length for a word to be considered a
///   keyword.
///
/// # Returns
/// A `HashSet` of lowercased, deduplicated keywords.
pub fn extract_keywords(text: &str, min_len: usize) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > min_len)
        .map(|w| w.to_string())
        .collect()
}

/// Detects whether the provided texts exhibit thematic stagnation based on
/// keyword overlap.
///
/// # Parameters
/// - `texts`: the texts to analyze (only the last 3 are considered).
/// - `min_word_len`: minimum character length for keywords (typically 4).
/// - `presence_ratio`: ratio of text presence required for a keyword to be
///   classified as "obsessional" (typically 0.6, i.e., 60%).
/// - `min_obsessional`: minimum number of obsessional keywords required to
///   declare stagnation (typically 3).
///
/// # Returns
/// A tuple `(stagnation_detected, obsessional_words)` where
/// `stagnation_detected` is a boolean and `obsessional_words` is the list
/// of keywords that exceeded the presence threshold.
pub fn detect_stagnation(
    texts: &[&str],
    min_word_len: usize,
    presence_ratio: f64,
    min_obsessional: usize,
) -> (bool, Vec<String>) {
    if texts.len() < 3 {
        return (false, vec![]);
    }

    // Consider only the 3 most recent texts
    let last3: Vec<&&str> = texts.iter().rev().take(3).collect();

    let kw_sets: Vec<HashSet<String>> = last3.iter()
        .map(|t| extract_keywords(t, min_word_len))
        .collect();

    // Count how many texts contain each keyword
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    for kw_set in &kw_sets {
        for w in kw_set {
            *word_freq.entry(w.clone()).or_insert(0) += 1;
        }
    }

    let n = kw_sets.len();
    let threshold = (n as f64 * presence_ratio).ceil() as usize;

    let obsessional_words: Vec<String> = word_freq.iter()
        .filter(|(_, &count)| count >= threshold)
        .map(|(w, _)| w.clone())
        .collect();

    let is_stagnating = obsessional_words.len() >= min_obsessional;

    (is_stagnating, obsessional_words)
}

// =============================================================================
// Semantic stagnation detection — cosine similarity on TF vectors
// =============================================================================

/// Builds a term-frequency (TF) vector for a text.
///
/// Words shorter than `min_word_len` are excluded. The frequency of each
/// remaining word is normalized by the total word count, producing values
/// in [0.0, 1.0].
///
/// # Parameters
/// - `text`: the input text.
/// - `min_word_len`: minimum character length for a word to be included.
///
/// # Returns
/// A `HashMap` mapping each word to its normalized frequency.
fn build_tf_vector(text: &str, min_word_len: usize) -> HashMap<String, f64> {
    let words: Vec<String> = text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > min_word_len)
        .map(|w| w.to_string())
        .collect();

    let total = words.len() as f64;
    if total == 0.0 {
        return HashMap::new();
    }

    let mut freq: HashMap<String, f64> = HashMap::new();
    for w in &words {
        *freq.entry(w.clone()).or_insert(0.0) += 1.0;
    }
    // Normalize each count by the total number of words
    for v in freq.values_mut() {
        *v /= total;
    }
    freq
}

/// Computes the cosine similarity between two TF vectors.
///
/// Cosine similarity measures the angle between two vectors in
/// high-dimensional space, yielding a value in [0.0, 1.0] where 1.0
/// indicates identical distributions and 0.0 indicates completely
/// orthogonal (unrelated) distributions.
///
/// # Parameters
/// - `a`: the first TF vector.
/// - `b`: the second TF vector.
///
/// # Returns
/// The cosine similarity score in [0.0, 1.0].
fn cosine_similarity(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    // Compute the dot product by iterating over the keys of vector `a`
    for (word, va) in a {
        norm_a += va * va;
        if let Some(vb) = b.get(word) {
            dot += va * vb;
        }
    }
    for vb in b.values() {
        norm_b += vb * vb;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 {
        0.0
    } else {
        dot / denom
    }
}

/// Detects semantic stagnation using cosine similarity of TF vectors.
///
/// Unlike `detect_stagnation()` which compares exact keyword overlap, this
/// function detects reformulations with synonyms by measuring the proximity
/// of word distributions between consecutive texts.
///
/// # Parameters
/// - `texts`: the texts to analyze (considers the last 4 entries).
/// - `min_word_len`: minimum character length for words (typically 4).
/// - `similarity_threshold`: mean similarity threshold above which
///   stagnation is declared (typically 0.55).
///
/// # Returns
/// A tuple `(semantic_stagnation_detected, mean_similarity)`.
pub fn detect_semantic_stagnation(
    texts: &[&str],
    min_word_len: usize,
    similarity_threshold: f64,
) -> (bool, f64) {
    if texts.len() < 3 {
        return (false, 0.0);
    }

    // Consider the last 4 texts (or fewer if not enough are available)
    let n = texts.len().min(4);
    let last_n: Vec<&&str> = texts.iter().rev().take(n).collect();

    // Build TF vectors for each text
    let tf_vectors: Vec<HashMap<String, f64>> = last_n.iter()
        .map(|t| build_tf_vector(t, min_word_len))
        .collect();

    // Compute cosine similarity between consecutive pairs
    let mut total_sim = 0.0;
    let mut pairs = 0;
    for i in 0..tf_vectors.len() - 1 {
        let sim = cosine_similarity(&tf_vectors[i], &tf_vectors[i + 1]);
        total_sim += sim;
        pairs += 1;
    }

    if pairs == 0 {
        return (false, 0.0);
    }

    let avg_similarity = total_sim / pairs as f64;
    (avg_similarity >= similarity_threshold, avg_similarity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_stagnation_similar_texts() {
        // Texts that reformulate the same idea with partially overlapping words
        let texts = vec![
            "La conscience humaine reste un profond mystere pour la science moderne",
            "Cette conscience profonde reste encore un mystere total pour notre science",
            "Le mystere profond de notre conscience humaine interroge la science",
            "La science reste face au mystere profond de la conscience humaine",
        ];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(sim > 0.3, "Similarity should be significant, got {:.2}", sim);
        assert!(is_stag, "Should detect semantic stagnation (sim={:.2})", sim);
    }

    #[test]
    fn test_semantic_stagnation_different_texts() {
        // Texts covering completely different topics — no stagnation expected
        let texts = vec![
            "Le chat dort sur le canape rouge dans le salon",
            "Les equations mathematiques derivent des axiomes fondamentaux",
            "La cuisine japonaise utilise beaucoup de poisson frais",
            "Le programme informatique compile sans erreur aujourd'hui",
        ];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(!is_stag, "Different texts should not stagnate (sim={})", sim);
    }

    #[test]
    fn test_semantic_stagnation_too_few_texts() {
        // Fewer than 3 texts — stagnation detection should not trigger
        let texts = vec!["Un seul texte"];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(!is_stag);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cosine_identical() {
        // Identical vectors should yield a cosine similarity of 1.0
        let mut a = HashMap::new();
        a.insert("conscience".to_string(), 0.5);
        a.insert("humaine".to_string(), 0.5);
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cosine_orthogonal() {
        // Completely disjoint vectors should yield a cosine similarity of 0.0
        let mut a = HashMap::new();
        a.insert("chat".to_string(), 1.0);
        let mut b = HashMap::new();
        b.insert("mathematique".to_string(), 1.0);
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }
}

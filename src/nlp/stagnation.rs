// =============================================================================
// nlp/stagnation.rs — Thematic stagnation detection (shared utility)
// =============================================================================
//
// Role: Detects whether a set of recent texts (thoughts, responses) are
//       revolving around the same theme obsessively.
//       Used by thinking.rs, conversation.rs, and llm.rs.
//
// Algorithm:
//   1. Extract significant keywords (> 4 chars) from each text
//   2. Count how many texts contain each keyword
//   3. If a word appears in > 60% of texts = "obsessional" word
//   4. If >= 3 obsessional words = stagnation detected
// =============================================================================

use std::collections::{HashMap, HashSet};

/// Extracts significant keywords (> `min_len` characters) from a text.
/// Returns a HashSet of lowercase, deduplicated words.
pub fn extract_keywords(text: &str, min_len: usize) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > min_len)
        .map(|w| w.to_string())
        .collect()
}

/// Detects whether the provided texts are in thematic stagnation.
///
/// Parameters:
/// - `texts`: the texts to analyze (takes the last 3)
/// - `min_word_len`: minimum keyword length (typically 4)
/// - `presence_ratio`: presence ratio for a word to be "obsessional" (typically 0.6)
/// - `min_obsessional`: minimum number of obsessional words to declare stagnation (typically 3)
///
/// Returns: (stagnation_detected, obsessional_words)
pub fn detect_stagnation(
    texts: &[&str],
    min_word_len: usize,
    presence_ratio: f64,
    min_obsessional: usize,
) -> (bool, Vec<String>) {
    if texts.len() < 3 {
        return (false, vec![]);
    }

    // Take the last 3 texts
    let last3: Vec<&&str> = texts.iter().rev().take(3).collect();

    let kw_sets: Vec<HashSet<String>> = last3.iter()
        .map(|t| extract_keywords(t, min_word_len))
        .collect();

    // Count how many texts contain each word
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
// Semantic detection — cosine similarity on TF vectors
// =============================================================================

/// Builds a term frequency (TF) vector for a text.
/// Returns a HashMap of word -> normalized frequency.
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
    // Normalize by total word count
    for v in freq.values_mut() {
        *v /= total;
    }
    freq
}

/// Computes the cosine similarity between two TF vectors.
fn cosine_similarity(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    // Iterate over the smaller vector's keys for the dot product
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

/// Detects semantic stagnation via cosine similarity of TF vectors.
///
/// Unlike detect_stagnation() which compares exact words, this function
/// detects rephrasing with synonyms by measuring the proximity of word
/// distributions between consecutive texts.
///
/// Parameters:
/// - `texts`: the texts to analyze (takes the last 4)
/// - `min_word_len`: minimum word length (typically 4)
/// - `similarity_threshold`: average similarity threshold (typically 0.55)
///
/// Returns: (semantic_stagnation, average_similarity)
pub fn detect_semantic_stagnation(
    texts: &[&str],
    min_word_len: usize,
    similarity_threshold: f64,
) -> (bool, f64) {
    if texts.len() < 3 {
        return (false, 0.0);
    }

    // Take the last 4 texts (or fewer if not enough)
    let n = texts.len().min(4);
    let last_n: Vec<&&str> = texts.iter().rev().take(n).collect();

    // Build TF vectors
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
        // Texts that rephrase the same idea with partially shared words
        let texts = vec![
            "La conscience humaine reste un profond mystere pour la science moderne",
            "Cette conscience profonde reste encore un mystere total pour notre science",
            "Le mystere profond de notre conscience humaine interroge la science",
            "La science reste face au mystere profond de la conscience humaine",
        ];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(sim > 0.3, "Similarite devrait etre significative, got {:.2}", sim);
        assert!(is_stag, "Devrait detecter la stagnation semantique (sim={:.2})", sim);
    }

    #[test]
    fn test_semantic_stagnation_different_texts() {
        let texts = vec![
            "Le chat dort sur le canape rouge dans le salon",
            "Les equations mathematiques derivent des axiomes fondamentaux",
            "La cuisine japonaise utilise beaucoup de poisson frais",
            "Le programme informatique compile sans erreur aujourd'hui",
        ];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(!is_stag, "Textes differents ne devraient pas stagner (sim={})", sim);
    }

    #[test]
    fn test_semantic_stagnation_too_few_texts() {
        let texts = vec!["Un seul texte"];
        let refs: Vec<&str> = texts.iter().copied().collect();
        let (is_stag, sim) = detect_semantic_stagnation(&refs, 4, 0.55);
        assert!(!is_stag);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cosine_identical() {
        let mut a = HashMap::new();
        a.insert("conscience".to_string(), 0.5);
        a.insert("humaine".to_string(), 0.5);
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let mut a = HashMap::new();
        a.insert("chat".to_string(), 1.0);
        let mut b = HashMap::new();
        b.insert("mathematique".to_string(), 1.0);
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }
}

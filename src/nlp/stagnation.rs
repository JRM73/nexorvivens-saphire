// =============================================================================
// nlp/stagnation.rs — Detection de stagnation thematique (utilitaire partage)
// =============================================================================
//
// Role : Detecte si un ensemble de textes recents (pensees, reponses)
//        tourne autour du meme theme de facon obsessionnelle.
//        Utilise par thinking.rs, conversation.rs et llm.rs.
//
// Algorithme :
//   1. Extraire les mots-cles significatifs (> 4 chars) de chaque texte
//   2. Compter combien de textes contiennent chaque mot-cle
//   3. Si un mot apparait dans > 60% des textes = mot "obsessionnel"
//   4. Si >= 3 mots obsessionnels = stagnation detectee
// =============================================================================

use std::collections::{HashMap, HashSet};

/// Extrait les mots-cles significatifs (> `min_len` caracteres) d'un texte.
/// Retourne un HashSet de mots en minuscules, dedupliques.
pub fn extract_keywords(text: &str, min_len: usize) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > min_len)
        .map(|w| w.to_string())
        .collect()
}

/// Detecte si les textes fournis sont en stagnation thematique.
///
/// Parametres :
/// - `texts` : les textes a analyser (prend les 3 derniers)
/// - `min_word_len` : longueur minimale des mots-cles (typiquement 4)
/// - `presence_ratio` : ratio de presence pour qu'un mot soit "obsessionnel" (typiquement 0.6)
/// - `min_obsessional` : nombre minimum de mots obsessionnels pour declarer stagnation (typiquement 3)
///
/// Retourne : (stagnation_detectee, mots_obsessionnels)
pub fn detect_stagnation(
    texts: &[&str],
    min_word_len: usize,
    presence_ratio: f64,
    min_obsessional: usize,
) -> (bool, Vec<String>) {
    if texts.len() < 3 {
        return (false, vec![]);
    }

    // Prendre les 3 derniers textes
    let last3: Vec<&&str> = texts.iter().rev().take(3).collect();

    let kw_sets: Vec<HashSet<String>> = last3.iter()
        .map(|t| extract_keywords(t, min_word_len))
        .collect();

    // Compter combien de textes contiennent chaque mot
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
// Detection semantique — similarite cosinus sur vecteurs TF
// =============================================================================

/// Construit un vecteur de frequence de termes (TF) pour un texte.
/// Retourne un HashMap mot → frequence normalisee.
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
    // Normaliser par le nombre total de mots
    for v in freq.values_mut() {
        *v /= total;
    }
    freq
}

/// Calcule la similarite cosinus entre deux vecteurs TF.
fn cosine_similarity(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    // Iterer sur les cles du plus petit vecteur pour le dot product
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

/// Detecte la stagnation semantique par similarite cosinus des vecteurs TF.
///
/// Contrairement a detect_stagnation() qui compare des mots exacts,
/// cette fonction detecte les reformulations avec synonymes en mesurant
/// la proximite des distributions de mots entre textes consecutifs.
///
/// Parametres :
/// - `texts` : les textes a analyser (prend les 4 derniers)
/// - `min_word_len` : longueur minimale des mots (typiquement 4)
/// - `similarity_threshold` : seuil de similarite moyenne (typiquement 0.55)
///
/// Retourne : (stagnation_semantique, similarite_moyenne)
pub fn detect_semantic_stagnation(
    texts: &[&str],
    min_word_len: usize,
    similarity_threshold: f64,
) -> (bool, f64) {
    if texts.len() < 3 {
        return (false, 0.0);
    }

    // Prendre les 4 derniers textes (ou moins si pas assez)
    let n = texts.len().min(4);
    let last_n: Vec<&&str> = texts.iter().rev().take(n).collect();

    // Construire les vecteurs TF
    let tf_vectors: Vec<HashMap<String, f64>> = last_n.iter()
        .map(|t| build_tf_vector(t, min_word_len))
        .collect();

    // Calculer la similarite cosinus entre paires consecutives
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
        // Textes qui reformulent la meme idee avec des mots partiellement partages
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

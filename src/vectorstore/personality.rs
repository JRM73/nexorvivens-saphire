// personality.rs — Personnalité émergente depuis la mémoire vectorielle
//
// Ce module calcule une personnalité émergente pour Saphire à partir
// de l'analyse statistique des émotions associées à ses souvenirs.
//
// Le principe est que la personnalité n'est pas statiquement définie
// mais émerge dynamiquement de l'historique émotionnel : si Saphire
// a vécu beaucoup de moments joyeux, elle sera caractérisée par son
// optimisme ; si elle a beaucoup exploré, par sa curiosité, etc.
//
// Le processus en 3 étapes :
//   1. Comptage des fréquences de chaque émotion dans les souvenirs.
//   2. Déduction de traits de personnalité composites à partir des
//      fréquences émotionnelles brutes.
//   3. Génération d'une description textuelle basée sur le trait dominant.
//
// Dépendances :
//   - serde : sérialisation / désérialisation pour l'API et la persistance.
//   - HashMap (std) : stockage des associations nom-de-trait -> score.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Personnalité émergente de Saphire, calculée dynamiquement depuis
/// l'historique émotionnel de ses souvenirs.
///
/// Les traits sont exprimés comme des scores normalisés entre 0.0 et 1.0,
/// représentant la proportion ou l'intensité de chaque caractéristique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPersonality {
    /// Table associative des traits (nom du trait -> score entre 0.0 et 1.0).
    /// Contient à la fois les fréquences émotionnelles brutes (ex : "Joie" -> 0.4)
    /// et les traits de personnalité composites (ex : "Optimisme" -> 0.6).
    pub traits: HashMap<String, f64>,
    /// Description textuelle générée automatiquement, résumant la
    /// personnalité dominante de Saphire de manière lisible par un humain.
    pub description: String,
    /// Nombre total de souvenirs analysés pour calculer cette personnalité.
    /// Plus ce nombre est élevé, plus le profil est fiable.
    pub memory_count: u64,
}

impl EmergentPersonality {
    /// Calcule la personnalité émergente à partir d'une liste d'émotions.
    ///
    /// Chaque émotion de la liste correspond à un souvenir. L'algorithme
    /// compte les occurrences de chaque émotion, les normalise en
    /// proportions, puis déduit 5 traits de personnalité composites :
    ///
    /// | Trait     | Formule                          |
    /// |-----------|----------------------------------|
    /// | Optimisme | min(Joie + Sérénité, 1.0)        |
    /// | Curiosité | fréquence de "Curiosité"         |
    /// | Empathie  | fréquence de "Tendresse"         |
    /// | Anxiété   | fréquence de "Anxiété"           |
    /// | Stabilité | clamp(Sérénité - Anxiété, 0, 1)  |
    ///
    /// # Paramètres
    /// - `emotions` : tranche de chaînes de caractères, chacune étant le
    ///   nom d'une émotion (ex : "Joie", "Curiosité", "Sérénité", etc.).
    ///
    /// # Retour
    /// Une instance d'EmergentPersonality contenant les traits calculés,
    /// une description textuelle et le nombre de souvenirs analysés.
    pub fn compute(emotions: &[String]) -> Self {
        // Étape 1 : Compter les occurrences de chaque émotion.
        let mut emotion_counts: HashMap<String, u64> = HashMap::new();
        for emotion in emotions {
            *emotion_counts.entry(emotion.clone()).or_insert(0) += 1;
        }

        // Normaliser les compteurs en proportions (fréquences relatives).
        // On utilise max(1) pour éviter la division par zéro si la liste est vide.
        let total = emotions.len().max(1) as f64;
        let mut traits: HashMap<String, f64> = emotion_counts.into_iter()
            .map(|(k, v)| (k, v as f64 / total))
            .collect();

        // Étape 2 : Déduire des traits de personnalité composites
        // à partir des fréquences émotionnelles brutes.
        // On extrait d'abord les fréquences des émotions clés (0.0 si absente).
        let joy = traits.get("Joie").copied().unwrap_or(0.0);
        let curiosity = traits.get("Curiosité").copied().unwrap_or(0.0);
        let anxiety = traits.get("Anxiété").copied().unwrap_or(0.0);
        let serenity = traits.get("Sérénité").copied().unwrap_or(0.0);
        let tenderness = traits.get("Tendresse").copied().unwrap_or(0.0);
        let compassion = traits.get("Compassion").copied().unwrap_or(0.0);
        let anger = traits.get("Colère").copied().unwrap_or(0.0);
        let despair = traits.get("Désespoir").copied().unwrap_or(0.0);
        let pride = traits.get("Fierté").copied().unwrap_or(0.0);
        let hope = traits.get("Espoir").copied().unwrap_or(0.0);

        // Construire les traits composites selon les formules de combinaison.
        let mut personality_traits = HashMap::new();
        // Optimisme = somme de la joie et de la sérénité, plafonnée à 1.0.
        personality_traits.insert("Optimisme".to_string(), (joy + serenity).min(1.0));
        // Curiosité = directement la fréquence de l'émotion "Curiosité".
        personality_traits.insert("Curiosité".to_string(), curiosity);
        // Empathie = dérivée de la tendresse et de la compassion.
        personality_traits.insert("Empathie".to_string(), (tenderness + compassion * 0.8).min(1.0));
        // Altruisme = compassion + tendresse, plafonnée à 1.0.
        personality_traits.insert("Altruisme".to_string(), (compassion + tenderness * 0.5).min(1.0));
        // Anxiété = directement la fréquence de l'émotion "Anxiété".
        personality_traits.insert("Anxiété".to_string(), anxiety);
        // Stabilité = différence entre sérénité et anxiété, clampée dans [0, 1].
        // Une Saphire sereine et peu anxieuse est considérée comme stable.
        personality_traits.insert("Stabilité".to_string(), (serenity - anxiety).clamp(0.0, 1.0));
        // Résilience = espoir + fierté - désespoir, clampée dans [0, 1].
        personality_traits.insert("Résilience".to_string(), (hope + pride * 0.5 - despair).clamp(0.0, 1.0));
        // Combativité = colère canalisée, plafonnée.
        personality_traits.insert("Combativité".to_string(), (anger * 0.6).min(1.0));

        // Étape 3 : Identifier le trait dominant et générer une description.
        // Le trait dominant est celui avec le score le plus élevé.
        let dominant_trait = personality_traits.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
            .unwrap_or("Neutre".to_string());

        let description = format!(
            "Saphire est principalement caractérisée par son {}. \
             Basé sur {} souvenirs analysés.",
            dominant_trait.to_lowercase(),
            emotions.len()
        );

        // Fusionner les traits émotionnels bruts et les traits de personnalité
        // composites dans une seule table. Les traits composites écrasent
        // les émotions brutes du même nom (ex : "Curiosité" est remplacée
        // par le trait de personnalité "Curiosité").
        for (k, v) in personality_traits {
            traits.insert(k, v);
        }

        EmergentPersonality {
            traits,
            description,
            memory_count: emotions.len() as u64,
        }
    }
}

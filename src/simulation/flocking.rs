// =============================================================================
// flocking.rs — Dynamique de Boids pour les pensees
//
// Role : Applique l'algorithme de Boids (Craig Reynolds, 1986) aux pensees
//        actives de Saphire. Les 3 regles de Boids :
//        1. Separation : eviter les pensees trop similaires
//        2. Alignement : tendre vers le meme type de pensee que le groupe
//        3. Cohesion : converger vers le centre du groupe thematique
//
//        Cela permet de diversifier les pensees tout en maintenant une
//        coherence thematique naturelle.
//
// Place dans l'architecture :
//   Consulte par le thought_engine pour ajuster les scores de selection.
//   Chaque pensee est un "boid" dans un espace abstrait (type x recence).
// =============================================================================

use serde::{Serialize, Deserialize};

/// Un "boid" representant une pensee recente dans l'espace conceptuel.
#[derive(Debug, Clone)]
pub struct ThoughtBoid {
    /// Indice du type de pensee (correspond a ThoughtType::all())
    pub type_index: usize,
    /// Position temporelle : 0 = la plus recente, N = la plus ancienne
    pub recency: f64,
    /// "Velocity" : tendance directionnelle (positif = type monte, negatif = descend)
    pub velocity: f64,
}

/// Resultat du calcul Boids : ajustement de score pour chaque type de pensee.
#[derive(Debug, Clone)]
pub struct FlockingResult {
    /// Ajustement par type de pensee : positif = favorise, negatif = penalise
    pub adjustments: Vec<f64>,
}

/// Parametres du flocking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlockingParams {
    /// Poids de la separation (eviter la repetition)
    pub separation_weight: f64,
    /// Poids de l'alignement (convergence thematique)
    pub alignment_weight: f64,
    /// Poids de la cohesion (gravite vers le centre)
    pub cohesion_weight: f64,
    /// Distance en dessous de laquelle la separation s'active
    pub separation_radius: f64,
}

impl Default for FlockingParams {
    fn default() -> Self {
        Self {
            separation_weight: 1.0,
            alignment_weight: 0.5,
            cohesion_weight: 0.3,
            separation_radius: 1.0,
        }
    }
}

/// Calcule les forces Boids sur les pensees recentes et retourne
/// un vecteur d'ajustement pour chaque type de pensee.
pub fn compute_flocking(
    recent_type_indices: &[usize],
    num_types: usize,
    params: &FlockingParams,
) -> FlockingResult {
    let mut adjustments = vec![0.0f64; num_types];

    if recent_type_indices.is_empty() {
        return FlockingResult { adjustments };
    }

    // Construire les boids a partir de l'historique recent
    let boids: Vec<ThoughtBoid> = recent_type_indices.iter().enumerate()
        .map(|(i, &idx)| ThoughtBoid {
            type_index: idx,
            recency: i as f64,
            velocity: 0.0,
        })
        .collect();

    // Compter les occurrences de chaque type
    let mut type_counts = vec![0usize; num_types];
    for boid in &boids {
        if boid.type_index < num_types {
            type_counts[boid.type_index] += 1;
        }
    }

    // 1. Separation : penaliser les types trop presents
    for (idx, &count) in type_counts.iter().enumerate() {
        if count >= 2 {
            // Plus un type est repete, plus il est penalise
            adjustments[idx] -= params.separation_weight * (count as f64 - 1.0) * 0.2;
        }
    }

    // 2. Alignement : favoriser les types proches du type dominant recent
    if let Some(&last_type) = recent_type_indices.last() {
        // Les types "voisins" (numeriquement proches) recoivent un leger bonus
        for idx in 0..num_types {
            let distance = ((idx as i64) - (last_type as i64)).unsigned_abs() as f64;
            if distance > 0.0 && distance <= 3.0 {
                adjustments[idx] += params.alignment_weight * (1.0 / distance) * 0.1;
            }
        }
    }

    // 3. Cohesion : favoriser les types proches du "centre de masse"
    let total_boids = boids.len() as f64;
    if total_boids > 0.0 {
        let center_type = boids.iter()
            .map(|b| b.type_index as f64)
            .sum::<f64>() / total_boids;

        for idx in 0..num_types {
            let distance = ((idx as f64) - center_type).abs();
            if distance < 5.0 {
                adjustments[idx] += params.cohesion_weight * (1.0 - distance / 5.0) * 0.05;
            }
        }
    }

    // Normaliser les ajustements dans [-1, 1]
    for adj in &mut adjustments {
        *adj = adj.clamp(-1.0, 1.0);
    }

    FlockingResult { adjustments }
}

/// Description du flocking pour le debug/dashboard.
pub fn describe_flocking(result: &FlockingResult, type_names: &[&str]) -> String {
    let mut desc = String::new();
    let mut sorted: Vec<(usize, f64)> = result.adjustments.iter()
        .enumerate()
        .filter(|(_, &v)| v.abs() > 0.01)
        .map(|(i, &v)| (i, v))
        .collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, adj) in sorted.iter().take(5) {
        let name = type_names.get(*idx).unwrap_or(&"?");
        let direction = if *adj > 0.0 { "↑" } else { "↓" };
        desc.push_str(&format!("  {} {} ({:+.2})\n", name, direction, adj));
    }
    desc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separation_penalizes_repetition() {
        let recent = vec![0, 0, 0]; // 3 fois le meme type
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        assert!(result.adjustments[0] < 0.0, "Type repete doit etre penalise");
    }

    #[test]
    fn test_no_recent_no_adjustments() {
        let recent: Vec<usize> = vec![];
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        assert!(result.adjustments.iter().all(|&v| v.abs() < 0.001));
    }

    #[test]
    fn test_diverse_no_penalty() {
        let recent = vec![0, 1, 2, 3, 4]; // Tous differents
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        // Aucun type ne devrait etre fortement penalise
        for adj in &result.adjustments {
            assert!(*adj > -0.5, "Types divers ne doivent pas etre penalises");
        }
    }
}

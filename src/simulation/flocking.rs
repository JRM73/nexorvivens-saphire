// =============================================================================
// flocking.rs — Boids dynamics for thoughts
//
// Role: Applies the Boids algorithm (Craig Reynolds, 1986) to Saphire's
//       active thoughts. The 3 Boids rules:
//       1. Separation: avoid overly similar thoughts
//       2. Alignment: tend toward the same thought type as the group
//       3. Cohesion: converge toward the thematic group center
//
//       This diversifies thoughts while maintaining natural thematic
//       coherence.
//
// Place in the architecture:
//   Consulted by the thought_engine to adjust selection scores.
//   Each thought is a "boid" in an abstract space (type x recency).
// =============================================================================

use serde::{Serialize, Deserialize};

/// A "boid" representing a recent thought in conceptual space.
#[derive(Debug, Clone)]
pub struct ThoughtBoid {
    /// Thought type index (corresponds to ThoughtType::all())
    pub type_index: usize,
    /// Temporal position: 0 = most recent, N = oldest
    pub recency: f64,
    /// "Velocity": directional tendency (positive = type rising, negative = falling)
    pub velocity: f64,
}

/// Result of the Boids calculation: score adjustment for each thought type.
#[derive(Debug, Clone)]
pub struct FlockingResult {
    /// Adjustment per thought type: positive = favored, negative = penalized
    pub adjustments: Vec<f64>,
}

/// Flocking parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlockingParams {
    /// Separation weight (avoid repetition)
    pub separation_weight: f64,
    /// Alignment weight (thematic convergence)
    pub alignment_weight: f64,
    /// Cohesion weight (gravity toward center)
    pub cohesion_weight: f64,
    /// Distance below which separation activates
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

/// Computes Boids forces on recent thoughts and returns
/// an adjustment vector for each thought type.
pub fn compute_flocking(
    recent_type_indices: &[usize],
    num_types: usize,
    params: &FlockingParams,
) -> FlockingResult {
    let mut adjustments = vec![0.0f64; num_types];

    if recent_type_indices.is_empty() {
        return FlockingResult { adjustments };
    }

    // Build boids from recent history
    let boids: Vec<ThoughtBoid> = recent_type_indices.iter().enumerate()
        .map(|(i, &idx)| ThoughtBoid {
            type_index: idx,
            recency: i as f64,
            velocity: 0.0,
        })
        .collect();

    // Count occurrences of each type
    let mut type_counts = vec![0usize; num_types];
    for boid in &boids {
        if boid.type_index < num_types {
            type_counts[boid.type_index] += 1;
        }
    }

    // 1. Separation: penalize overly present types
    for (idx, &count) in type_counts.iter().enumerate() {
        if count >= 2 {
            // The more a type is repeated, the more it is penalized
            adjustments[idx] -= params.separation_weight * (count as f64 - 1.0) * 0.2;
        }
    }

    // 2. Alignment: favor types close to the recent dominant type
    if let Some(&last_type) = recent_type_indices.last() {
        // "Neighbor" types (numerically close) receive a slight bonus
        for idx in 0..num_types {
            let distance = ((idx as i64) - (last_type as i64)).unsigned_abs() as f64;
            if distance > 0.0 && distance <= 3.0 {
                adjustments[idx] += params.alignment_weight * (1.0 / distance) * 0.1;
            }
        }
    }

    // 3. Cohesion: favor types close to the "center of mass"
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

    // Normalize adjustments to [-1, 1]
    for adj in &mut adjustments {
        *adj = adj.clamp(-1.0, 1.0);
    }

    FlockingResult { adjustments }
}

/// Description of the flocking for debug/dashboard.
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
        let recent = vec![0, 0, 0]; // 3 times the same type
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        assert!(result.adjustments[0] < 0.0, "Repeated type must be penalized");
    }

    #[test]
    fn test_no_recent_no_adjustments() {
        let recent: Vec<usize> = vec![];
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        assert!(result.adjustments.iter().all(|&v| v.abs() < 0.001));
    }

    #[test]
    fn test_diverse_no_penalty() {
        let recent = vec![0, 1, 2, 3, 4]; // All different
        let result = compute_flocking(&recent, 10, &FlockingParams::default());
        // No type should be strongly penalized
        for adj in &result.adjustments {
            assert!(*adj > -0.5, "Diverse types must not be penalized");
        }
    }
}

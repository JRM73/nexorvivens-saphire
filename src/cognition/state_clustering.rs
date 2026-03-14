// =============================================================================
// state_clustering.rs — Automatic clustering of cognitive states (PCA + K-Means)
// =============================================================================
//
// Role: Compresses Saphire's internal state (16 dimensions) into 3 principal
//       components via PCA, then automatically detects the dominant cognitive
//       state via K-Means (4 clusters: flow, stress, creativity, rest).
//
// Input dimensions (16):
//   - 9 neurochemical molecules (dopamine, cortisol, serotonin, adrenaline,
//     oxytocin, endorphin, noradrenaline, gaba, glutamate)
//   - 2 emotional dimensions (valence [-1,+1], arousal [0,1])
//   - 2 consciousness dimensions (phi, level)
//   - 1 cognitive load (current_load)
//   - 1 reward signal (umami)
//   - 1 global surprise (global_surprise)
//
// Dependencies:
//   - algorithms::pca — dimensionality reduction
//   - algorithms::kmeans — cluster partitioning
//
// Place in architecture:
//   Cognitive module called at each thinking cycle to provide
//   "synthetic proprioception" — a readable label of the internal state.
//   Results feed the LLM prompt and the WebSocket broadcast.
// =============================================================================

use std::collections::VecDeque;

/// Number of dimensions of the full internal state
const STATE_DIM: usize = 17;

/// Number of principal components to extract
const N_COMPONENTS: usize = 3;

/// Number of K-Means clusters
const N_CLUSTERS: usize = 4;

/// Minimum number of snapshots before running PCA + K-Means
const MIN_SNAPSHOTS: usize = 20;

/// Maximum number of snapshots retained (sliding window)
const MAX_SNAPSHOTS: usize = 200;

/// Maximum K-Means iterations
const KMEANS_MAX_ITER: usize = 50;

/// Labels for detected cognitive states (by dominant centroid)
const CLUSTER_LABELS: [&str; N_CLUSTERS] = ["flow", "stress", "creativite", "repos"];

/// Snapshot of the internal state at a given moment (16 dimensions)
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    /// Vector of the 16 internal dimensions
    pub dimensions: [f64; STATE_DIM],
    /// Cycle at which the snapshot was taken
    pub cycle: u64,
}

/// Clustering result for the current cycle
#[derive(Clone, Debug)]
pub struct ClusteringResult {
    /// Label of the detected state (flow, stress, creativity, rest)
    pub state_label: String,
    /// Cluster index (0..3)
    pub cluster_id: usize,
    /// PCA projection of the current snapshot (3 components)
    pub pca_projection: [f64; N_COMPONENTS],
    /// Variance explained by each component
    pub explained_variance: [f64; N_COMPONENTS],
    /// Confidence: 1.0 - (distance to centroid / max distance)
    pub confidence: f64,
}

/// Cognitive state clustering engine
pub struct StateClustering {
    /// Sliding window of recent snapshots
    history: VecDeque<StateSnapshot>,
    /// Last clustering result
    pub last_result: Option<ClusteringResult>,
    /// Recalculation frequency (every N cycles)
    pub recalculate_every: u64,
    /// Cycle counter since last recalculation
    cycles_since_recalc: u64,
    /// Fixed reference centroids for labeling (initialized on first computation)
    reference_centroids: Option<Vec<Vec<f64>>>,
}

impl StateClustering {
    /// Creates a new clustering engine
    pub fn new(recalculate_every: u64) -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_SNAPSHOTS),
            last_result: None,
            recalculate_every,
            cycles_since_recalc: 0,
            reference_centroids: None,
        }
    }

    /// Builds a snapshot from internal components
    pub fn build_snapshot(
        chem_vec9: &[f64; 9],
        valence: f64,
        arousal: f64,
        phi: f64,
        level: f64,
        cognitive_load: f64,
        umami: f64,
        global_surprise: f64,
        map_tension: f64,
        cycle: u64,
    ) -> StateSnapshot {
        let mut dims = [0.0; STATE_DIM];

        // Chemistry (9 dimensions, already normalized [0,1])
        dims[0..9].copy_from_slice(chem_vec9);

        // Emotion (2 dimensions)
        // Normalize valence from [-1,+1] to [0,1]
        dims[9] = (valence + 1.0) / 2.0;
        dims[10] = arousal;

        // Consciousness (2 dimensions)
        dims[11] = phi;
        dims[12] = level;

        // Cognitive load (1 dimension)
        dims[13] = cognitive_load;

        // Umami (1 dimension)
        dims[14] = umami;

        // Global surprise (1 dimension)
        dims[15] = global_surprise;

        // MAP tension — perception/cognition coherence (1 dimension)
        // Saphire's request: group MAP with emotions in the clustering
        dims[16] = map_tension;

        StateSnapshot { dimensions: dims, cycle }
    }

    /// Records a snapshot and recalculates clustering if necessary
    pub fn record_and_cluster(&mut self, snapshot: StateSnapshot) -> Option<&ClusteringResult> {
        self.history.push_back(snapshot);

        // Limit history size
        while self.history.len() > MAX_SNAPSHOTS {
            self.history.pop_front();
        }

        self.cycles_since_recalc += 1;

        // Recalculate if enough data and interval reached
        if self.history.len() >= MIN_SNAPSHOTS
            && self.cycles_since_recalc >= self.recalculate_every
        {
            self.cycles_since_recalc = 0;
            self.recalculate();
        }

        self.last_result.as_ref()
    }

    /// Runs PCA + K-Means on the history and updates the result
    fn recalculate(&mut self) {
        let data: Vec<Vec<f64>> = self.history.iter()
            .map(|s| s.dimensions.to_vec())
            .collect();

        // PCA: reduce from 16 to 3 dimensions
        let pca_result = match crate::algorithms::pca::pca(&data, N_COMPONENTS) {
            Some(r) => r,
            None => return,
        };

        // K-Means on the projected data
        let labels = crate::algorithms::kmeans::kmeans(
            &pca_result.projected,
            N_CLUSTERS,
            KMEANS_MAX_ITER,
        );

        if labels.is_empty() {
            return;
        }

        // The last point corresponds to the most recent snapshot
        let current_label = labels[labels.len() - 1];
        let current_proj = &pca_result.projected[pca_result.projected.len() - 1];

        // Compute centroids in PCA space
        let mut centroids = vec![vec![0.0; N_COMPONENTS]; N_CLUSTERS];
        let mut counts = vec![0usize; N_CLUSTERS];
        for (i, label) in labels.iter().enumerate() {
            counts[*label] += 1;
            for d in 0..N_COMPONENTS {
                centroids[*label][d] += pca_result.projected[i][d];
            }
        }
        for (j, centroid) in centroids.iter_mut().enumerate() {
            if counts[j] > 0 {
                let c = counts[j] as f64;
                for d in centroid.iter_mut() {
                    *d /= c;
                }
            }
        }

        // Determine the semantic label by heuristic on the centroids
        let state_label = self.assign_label(current_label, &centroids, &data, &labels);

        // Compute confidence (inverse of distance to centroid)
        let dist_to_centroid = euclidean(&centroids[current_label], current_proj);
        let max_dist = centroids.iter()
            .map(|c| euclidean(c, current_proj))
            .fold(0.0f64, f64::max);
        let confidence = if max_dist > 0.0 {
            1.0 - (dist_to_centroid / max_dist)
        } else {
            1.0
        };

        let mut pca_proj = [0.0; N_COMPONENTS];
        for (i, v) in current_proj.iter().enumerate().take(N_COMPONENTS) {
            pca_proj[i] = *v;
        }

        let mut explained = [0.0; N_COMPONENTS];
        for (i, v) in pca_result.explained_variance.iter().enumerate().take(N_COMPONENTS) {
            explained[i] = *v;
        }

        self.last_result = Some(ClusteringResult {
            state_label,
            cluster_id: current_label,
            pca_projection: pca_proj,
            explained_variance: explained,
            confidence,
        });

        self.reference_centroids = Some(centroids);
    }

    /// Assigns a semantic label to the cluster by analyzing average
    /// characteristics of the cluster in the original (16D) space
    fn assign_label(
        &self,
        cluster_id: usize,
        _pca_centroids: &[Vec<f64>],
        original_data: &[Vec<f64>],
        labels: &[usize],
    ) -> String {
        // Compute average of original dimensions for this cluster
        let mut sum = vec![0.0; STATE_DIM];
        let mut count = 0usize;

        for (i, label) in labels.iter().enumerate() {
            if *label == cluster_id {
                count += 1;
                for d in 0..STATE_DIM {
                    sum[d] += original_data[i][d];
                }
            }
        }

        if count == 0 {
            return CLUSTER_LABELS[cluster_id % N_CLUSTERS].to_string();
        }

        let avg: Vec<f64> = sum.iter().map(|s| s / count as f64).collect();

        // Labeling heuristic based on chemical signatures:
        //
        // dims[0] = dopamine     dims[1] = cortisol      dims[2] = serotonin
        // dims[3] = adrenaline   dims[4] = oxytocin      dims[5] = endorphin
        // dims[6] = noradre      dims[7] = gaba          dims[8] = glutamate
        // dims[9] = valence_norm dims[10] = arousal       dims[11] = phi
        // dims[12] = level       dims[13] = load          dims[14] = umami
        // dims[15] = surprise    dims[16] = map_tension
        let dopamine = avg[0];
        let cortisol = avg[1];
        let serotonin = avg[2];
        let adrenaline = avg[3];
        let arousal = avg[10];
        let phi = avg[11];
        let load = avg[13];
        let umami = avg[14];
        let map_tension = avg[16];

        // Score each possible state
        // MAP tension contributes: high tension = stress, low tension = flow/rest
        let flow_score = dopamine * 0.3 + phi * 0.3 + umami * 0.2
            - cortisol * 0.2 - load * 0.1 + serotonin * 0.1
            - map_tension * 0.15; // high tension prevents flow
        let stress_score = cortisol * 0.4 + adrenaline * 0.3 + load * 0.2
            + arousal * 0.1 - serotonin * 0.2
            + map_tension * 0.2; // high tension = stress

        let creativity_score = dopamine * 0.2 + phi * 0.2 + arousal * 0.2
            - cortisol * 0.1 + avg[15] * 0.3; // high surprise = creative exploration
        let rest_score = serotonin * 0.3 + avg[7] * 0.2 // gaba (inhibition)
            - arousal * 0.3 - cortisol * 0.1 - adrenaline * 0.1
            - map_tension * 0.1; // high tension prevents rest
        let scores = [
            (flow_score, "flow"),
            (stress_score, "stress"),
            (creativity_score, "creativite"),
            (rest_score, "repos"),
        ];

        scores.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| CLUSTER_LABELS[cluster_id % N_CLUSTERS].to_string())
    }

    /// Returns the label of the current state (or "inconnu" if not enough data)
    pub fn current_state_label(&self) -> &str {
        self.last_result
            .as_ref()
            .map(|r| r.state_label.as_str())
            .unwrap_or("inconnu")
    }

    /// Returns the number of snapshots in memory
    pub fn snapshot_count(&self) -> usize {
        self.history.len()
    }

    /// Generates a proprioception line for the LLM prompt
    pub fn proprioception_line(&self) -> String {
        match &self.last_result {
            Some(r) => {
                format!(
                    "Proprioception: je me sens {} (certitude {:.0}%)",
                    r.state_label,
                    r.confidence * 100.0,
                )
            }
            None => {
                format!(
                    "Proprioception: en cours de calibrage",
                )
            }
        }
    }
}

/// Euclidean distance between two vectors
fn euclidean(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_snapshot() {
        let chem = [0.5, 0.3, 0.6, 0.2, 0.4, 0.3, 0.5, 0.5, 0.45];
        let snap = StateClustering::build_snapshot(
            &chem, 0.5, 0.4, 0.6, 0.7, 0.3, 0.65, 0.2, 0.1, 42,
        );
        assert_eq!(snap.cycle, 42);
        assert_eq!(snap.dimensions[0], 0.5); // dopamine
        assert!((snap.dimensions[9] - 0.75).abs() < 1e-10); // normalized valence
        assert_eq!(snap.dimensions[14], 0.65); // umami
    }

    #[test]
    fn test_clustering_needs_minimum_data() {
        let mut sc = StateClustering::new(5);
        for i in 0..10 {
            let snap = StateSnapshot {
                dimensions: [0.5; STATE_DIM],
                cycle: i,
            };
            sc.record_and_cluster(snap);
        }
        // Not enough data -> no result
        assert!(sc.last_result.is_none());
    }

    #[test]
    fn test_clustering_with_enough_data() {
        let mut sc = StateClustering::new(1); // Recalculate every cycle
        // Generate 4 distinct groups of 10 points each
        for i in 0..40 {
            let mut dims = [0.5; STATE_DIM];
            match i % 4 {
                0 => {
                    // Flow: high dopamine, low cortisol, high phi
                    dims[0] = 0.9; dims[1] = 0.1; dims[11] = 0.9; dims[14] = 0.8;
                }
                1 => {
                    // Stress: high cortisol, high adrenaline
                    dims[1] = 0.9; dims[3] = 0.8; dims[13] = 0.8;
                }
                2 => {
                    // Creativity: high dopamine, high surprise
                    dims[0] = 0.7; dims[15] = 0.9; dims[10] = 0.7;
                }
                3 => {
                    // Rest: high serotonin, low arousal
                    dims[2] = 0.9; dims[7] = 0.8; dims[10] = 0.1;
                }
                _ => unreachable!(),
            }
            let snap = StateSnapshot { dimensions: dims, cycle: i as u64 };
            sc.record_and_cluster(snap);
        }

        // With 40 points in 4 distinct groups, we should have a result
        assert!(sc.last_result.is_some());
        let result = sc.last_result.as_ref().unwrap();
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        assert!(!result.state_label.is_empty());
    }

    #[test]
    fn test_proprioception_line_calibrating() {
        let sc = StateClustering::new(5);
        let line = sc.proprioception_line();
        assert!(line.contains("calibrage"));
    }

    #[test]
    fn test_proprioception_line_active() {
        let mut sc = StateClustering::new(1);
        for i in 0..25 {
            let mut dims = [0.5; STATE_DIM];
            dims[0] = 0.9; // high dopamine
            let snap = StateSnapshot { dimensions: dims, cycle: i as u64 };
            sc.record_and_cluster(snap);
        }
        if sc.last_result.is_some() {
            let line = sc.proprioception_line();
            assert!(line.contains("Proprioception"));
            assert!(line.contains("je me sens"));
            assert!(line.contains("certitude"));
        }
    }
}

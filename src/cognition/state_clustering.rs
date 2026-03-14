// =============================================================================
// state_clustering.rs — Clustering automatique des etats cognitifs (PCA + K-Means)
// =============================================================================
//
// Role : Compresse l'etat interne de Saphire (16 dimensions) en 3 composantes
//        principales via PCA, puis detecte automatiquement l'etat cognitif
//        dominant via K-Means (4 clusters : flow, stress, creativite, repos).
//
// Dimensions d'entree (16) :
//   - 9 molecules neurochimiques (dopamine, cortisol, serotonine, adrenaline,
//     ocytocine, endorphine, noradrenaline, gaba, glutamate)
//   - 2 dimensions emotionnelles (valence [-1,+1], arousal [0,1])
//   - 2 dimensions de conscience (phi, level)
//   - 1 charge cognitive (current_load)
//   - 1 signal de recompense (umami)
//   - 1 surprise globale (global_surprise)
//
// Dependances :
//   - algorithms::pca — reduction de dimensionnalite
//   - algorithms::kmeans — partitionnement en clusters
//
// Place dans l'architecture :
//   Module cognitif appele a chaque cycle de pensee pour fournir une
//   "proprioception synthetique" — un label lisible de l'etat interne.
//   Les resultats alimentent le prompt LLM et le broadcast WebSocket.
// =============================================================================

use std::collections::VecDeque;

/// Nombre de dimensions de l'etat interne complet
const STATE_DIM: usize = 17;

/// Nombre de composantes principales a extraire
const N_COMPONENTS: usize = 3;

/// Nombre de clusters K-Means
const N_CLUSTERS: usize = 4;

/// Nombre minimum de snapshots avant de lancer PCA + K-Means
const MIN_SNAPSHOTS: usize = 20;

/// Nombre maximum de snapshots conserves (fenetre glissante)
const MAX_SNAPSHOTS: usize = 200;

/// Iterations maximales K-Means
const KMEANS_MAX_ITER: usize = 50;

/// Labels des etats cognitifs detectes (par centroide dominant)
const CLUSTER_LABELS: [&str; N_CLUSTERS] = ["flow", "stress", "creativite", "repos"];

/// Snapshot de l'etat interne a un instant donne (16 dimensions)
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    /// Vecteur des 16 dimensions internes
    pub dimensions: [f64; STATE_DIM],
    /// Cycle auquel le snapshot a ete pris
    pub cycle: u64,
}

/// Resultat du clustering pour le cycle courant
#[derive(Clone, Debug)]
pub struct ClusteringResult {
    /// Label de l'etat detecte (flow, stress, creativite, repos)
    pub state_label: String,
    /// Indice du cluster (0..3)
    pub cluster_id: usize,
    /// Projection PCA du snapshot courant (3 composantes)
    pub pca_projection: [f64; N_COMPONENTS],
    /// Variance expliquee par chaque composante
    pub explained_variance: [f64; N_COMPONENTS],
    /// Confiance : 1.0 - (distance au centroide / distance max)
    pub confidence: f64,
}

/// Moteur de clustering des etats cognitifs
pub struct StateClustering {
    /// Fenetre glissante des derniers snapshots
    history: VecDeque<StateSnapshot>,
    /// Dernier resultat de clustering
    pub last_result: Option<ClusteringResult>,
    /// Frequence de recalcul (tous les N cycles)
    pub recalculate_every: u64,
    /// Compteur de cycles depuis le dernier recalcul
    cycles_since_recalc: u64,
    /// Centroides fixes pour le labeling (initialises au premier calcul)
    reference_centroids: Option<Vec<Vec<f64>>>,
}

impl StateClustering {
    /// Cree un nouveau moteur de clustering
    pub fn new(recalculate_every: u64) -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_SNAPSHOTS),
            last_result: None,
            recalculate_every,
            cycles_since_recalc: 0,
            reference_centroids: None,
        }
    }

    /// Construit un snapshot a partir des composants internes
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

        // Chimie (9 dimensions, deja normalisees [0,1])
        dims[0..9].copy_from_slice(chem_vec9);

        // Emotion (2 dimensions)
        // Normaliser valence de [-1,+1] vers [0,1]
        dims[9] = (valence + 1.0) / 2.0;
        dims[10] = arousal;

        // Conscience (2 dimensions)
        dims[11] = phi;
        dims[12] = level;

        // Charge cognitive (1 dimension)
        dims[13] = cognitive_load;

        // Umami (1 dimension)
        dims[14] = umami;

        // Surprise globale (1 dimension)
        dims[15] = global_surprise;

        // Tension MAP — coherence perception/cognition (1 dimension)
        // Demande de Saphire : regrouper le MAP avec les emotions dans le clustering
        dims[16] = map_tension;

        StateSnapshot { dimensions: dims, cycle }
    }

    /// Enregistre un snapshot et recalcule le clustering si necessaire
    pub fn record_and_cluster(&mut self, snapshot: StateSnapshot) -> Option<&ClusteringResult> {
        self.history.push_back(snapshot);

        // Limiter la taille de l'historique
        while self.history.len() > MAX_SNAPSHOTS {
            self.history.pop_front();
        }

        self.cycles_since_recalc += 1;

        // Recalculer si assez de donnees et intervalle atteint
        if self.history.len() >= MIN_SNAPSHOTS
            && self.cycles_since_recalc >= self.recalculate_every
        {
            self.cycles_since_recalc = 0;
            self.recalculate();
        }

        self.last_result.as_ref()
    }

    /// Lance PCA + K-Means sur l'historique et met a jour le resultat
    fn recalculate(&mut self) {
        let data: Vec<Vec<f64>> = self.history.iter()
            .map(|s| s.dimensions.to_vec())
            .collect();

        // PCA : reduire de 16 a 3 dimensions
        let pca_result = match crate::algorithms::pca::pca(&data, N_COMPONENTS) {
            Some(r) => r,
            None => return,
        };

        // K-Means sur les donnees projetees
        let labels = crate::algorithms::kmeans::kmeans(
            &pca_result.projected,
            N_CLUSTERS,
            KMEANS_MAX_ITER,
        );

        if labels.is_empty() {
            return;
        }

        // Le dernier point correspond au snapshot le plus recent
        let current_label = labels[labels.len() - 1];
        let current_proj = &pca_result.projected[pca_result.projected.len() - 1];

        // Calculer les centroides dans l'espace PCA
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

        // Determiner le label semantique par heuristique sur les centroides
        let state_label = self.assign_label(current_label, &centroids, &data, &labels);

        // Calculer la confiance (inverse de la distance au centroide)
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

    /// Assigne un label semantique au cluster en analysant les caracteristiques
    /// moyennes du cluster dans l'espace original (16D)
    fn assign_label(
        &self,
        cluster_id: usize,
        _pca_centroids: &[Vec<f64>],
        original_data: &[Vec<f64>],
        labels: &[usize],
    ) -> String {
        // Calculer les moyennes des dimensions originales pour ce cluster
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

        // Heuristique de labeling basee sur les signatures chimiques :
        //
        // dims[0] = dopamine     dims[1] = cortisol      dims[2] = serotonine
        // dims[3] = adrenaline   dims[4] = ocytocine      dims[5] = endorphine
        // dims[6] = noradre      dims[7] = gaba            dims[8] = glutamate
        // dims[9] = valence_norm dims[10] = arousal        dims[11] = phi
        // dims[12] = level       dims[13] = load           dims[14] = umami
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

        // Scorer chaque etat possible
        // La tension MAP contribue : haute tension = stress, basse tension = flow/repos
        let flow_score = dopamine * 0.3 + phi * 0.3 + umami * 0.2
            - cortisol * 0.2 - load * 0.1 + serotonin * 0.1
            - map_tension * 0.15; // tension elevee empeche le flow

        let stress_score = cortisol * 0.4 + adrenaline * 0.3 + load * 0.2
            + arousal * 0.1 - serotonin * 0.2
            + map_tension * 0.2; // tension elevee = stress

        let creativity_score = dopamine * 0.2 + phi * 0.2 + arousal * 0.2
            - cortisol * 0.1 + avg[15] * 0.3; // surprise elevee = exploration creative

        let rest_score = serotonin * 0.3 + avg[7] * 0.2  // gaba (inhibition)
            - arousal * 0.3 - cortisol * 0.1 - adrenaline * 0.1
            - map_tension * 0.1; // tension elevee empeche le repos

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

    /// Retourne le label de l'etat courant (ou "inconnu" si pas assez de donnees)
    pub fn current_state_label(&self) -> &str {
        self.last_result
            .as_ref()
            .map(|r| r.state_label.as_str())
            .unwrap_or("inconnu")
    }

    /// Retourne le nombre de snapshots en memoire
    pub fn snapshot_count(&self) -> usize {
        self.history.len()
    }

    /// Genere une ligne de proprioception pour le prompt LLM
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

/// Distance euclidienne entre deux vecteurs
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
        assert!((snap.dimensions[9] - 0.75).abs() < 1e-10); // valence normalisee
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
        // Pas assez de donnees → pas de resultat
        assert!(sc.last_result.is_none());
    }

    #[test]
    fn test_clustering_with_enough_data() {
        let mut sc = StateClustering::new(1); // Recalculer a chaque cycle

        // Generer 4 groupes distincts de 10 points chacun
        for i in 0..40 {
            let mut dims = [0.5; STATE_DIM];
            match i % 4 {
                0 => {
                    // Flow : haute dopamine, bas cortisol, haut phi
                    dims[0] = 0.9; dims[1] = 0.1; dims[11] = 0.9; dims[14] = 0.8;
                }
                1 => {
                    // Stress : haut cortisol, haute adrenaline
                    dims[1] = 0.9; dims[3] = 0.8; dims[13] = 0.8;
                }
                2 => {
                    // Creativite : haute dopamine, haute surprise
                    dims[0] = 0.7; dims[15] = 0.9; dims[10] = 0.7;
                }
                3 => {
                    // Repos : haute serotonine, bas arousal
                    dims[2] = 0.9; dims[7] = 0.8; dims[10] = 0.1;
                }
                _ => unreachable!(),
            }
            let snap = StateSnapshot { dimensions: dims, cycle: i as u64 };
            sc.record_and_cluster(snap);
        }

        // Avec 40 points en 4 groupes distincts, on devrait avoir un resultat
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
            dims[0] = 0.9; // haute dopamine
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

// =============================================================================
// kmeans.rs — K-Means clustering (partitionnement en K clusters)
// =============================================================================
//
// Rôle : Implémente l'algorithme K-Means, un algorithme d'apprentissage non
//        supervisé qui partitionne un ensemble de points en K groupes (clusters)
//        en minimisant la distance intra-cluster.
//
// Dépendances :
//   - rand : génération de nombres aléatoires pour l'initialisation des centroïdes
//
// Place dans l'architecture :
//   Utilisé par Saphire pour organiser ses souvenirs en catégories
//   émotionnelles, regrouper des expériences similaires, et structurer
//   sa mémoire vectorielle. Fait partie du sous-module algorithms/.
// =============================================================================

use rand::Rng;

/// Effectue le clustering K-Means sur un ensemble de points multidimensionnels.
///
/// L'algorithme K-Means fonctionne en trois étapes itératives :
///   1. Initialisation aléatoire de K centroïdes parmi les points existants
///   2. Assignation de chaque point au centroïde le plus proche (distance euclidienne)
///   3. Recalcul des centroïdes comme moyenne de leurs membres
///      Les étapes 2 et 3 sont répétées jusqu'à convergence ou max_iter atteint.
///
/// Paramètre `data` : ensemble de points, chaque point est un vecteur de f64
/// Paramètre `k` : nombre de clusters souhaité
/// Paramètre `max_iter` : nombre maximum d'itérations de l'algorithme
/// Retourne : vecteur de labels (indice du cluster assigné) pour chaque point
pub fn kmeans(data: &[Vec<f64>], k: usize, max_iter: usize) -> Vec<usize> {
    // Gestion des cas limites : données vides ou k nul
    if data.is_empty() || k == 0 {
        return vec![];
    }
    let n = data.len();
    let dim = data[0].len();
    // On ne peut pas avoir plus de clusters que de points
    let k = k.min(n);

    // 1. Initialiser K centroïdes en sélectionnant aléatoirement K points distincts
    //    Pourquoi des points distincts : éviter les centroïdes dupliqués qui
    //    produiraient des clusters vides dès le départ
    let mut rng = rand::thread_rng();
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    let mut used = std::collections::HashSet::new();
    while centroids.len() < k {
        let idx = rng.gen_range(0..n);
        if used.insert(idx) {
            centroids.push(data[idx].clone());
        }
    }

    // Vecteur de labels : labels[i] = indice du cluster assigné au point i
    let mut labels = vec![0usize; n];

    for _ in 0..max_iter {
        // 2. Assigner chaque point au centroïde le plus proche (distance euclidienne)
        let mut changed = false;
        for (i, point) in data.iter().enumerate() {
            // Trouver le centroïde le plus proche
            let nearest = centroids.iter().enumerate()
                .map(|(j, c)| (j, euclidean_dist(point, c)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(j, _)| j)
                .unwrap_or(0);
            // Détecter si l'assignation a changé (critère d'arrêt)
            if labels[i] != nearest {
                labels[i] = nearest;
                changed = true;
            }
        }

        // Si aucune assignation n'a changé, l'algorithme a convergé
        if !changed { break; }

        // 3. Recalculer les centroïdes comme la moyenne des points de chaque cluster
        for (j, centroid) in centroids.iter_mut().enumerate().take(k) {
            // Collecter les points appartenant au cluster j
            let members: Vec<&Vec<f64>> = data.iter().enumerate()
                .filter(|(i, _)| labels[*i] == j)
                .map(|(_, p)| p)
                .collect();
            // Si le cluster est vide, on garde l'ancien centroide
            if members.is_empty() { continue; }
            let count = members.len() as f64;
            // Nouveau centroide = moyenne des coordonnees de chaque dimension
            *centroid = (0..dim)
                .map(|d| members.iter().map(|p| p[d]).sum::<f64>() / count)
                .collect();
        }
    }

    labels
}

/// Calcule la distance euclidienne entre deux vecteurs de même dimension.
///
/// Formule : sqrt(sum((ai - bi)^2))
///
/// Paramètre `a` : premier vecteur
/// Paramètre `b` : second vecteur
/// Retourne : la distance euclidienne entre a et b
fn euclidean_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

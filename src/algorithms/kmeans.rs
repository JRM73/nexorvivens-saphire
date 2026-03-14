// =============================================================================
// kmeans.rs — K-Means clustering (partitioning into K clusters)
// =============================================================================
//
// Role: Implements the K-Means algorithm, an unsupervised learning algorithm
//  that partitions a set of points into K groups (clusters) by minimizing
//  intra-cluster distance.
//
// Dependencies:
//  - rand: random number generation for centroid initialization
//
// Place in architecture:
//  Used by Saphire to organize its memories into emotional categories,
//  group similar experiences, and structure its vector memory. Part of the
//  algorithms/ submodule.
// =============================================================================

use rand::Rng;

/// Performs K-Means clustering on a set of multidimensional points.
///
/// The K-Means algorithm works in three iterative steps:
///  1. Random initialization of K centroids from existing points
///  2. Assignment of each point to the nearest centroid (Euclidean distance)
///  3. Recomputation of centroids as the mean of their members
///  Steps 2 and 3 are repeated until convergence or max_iter is reached.
///
/// Parameter `data`: set of points, each point is a Vec<f64>
/// Parameter `k`: desired number of clusters
/// Parameter `max_iter`: maximum number of algorithm iterations
/// Returns: vector of labels (assigned cluster index) for each point
pub fn kmeans(data: &[Vec<f64>], k: usize, max_iter: usize) -> Vec<usize> {
    // Handle edge cases: empty data or zero k
    if data.is_empty() || k == 0 {
        return vec![];
    }
    let n = data.len();
    let dim = data[0].len();
    // Cannot have more clusters than data points
    let k = k.min(n);

    // 1. Initialize K centroids by randomly selecting K distinct points
    //  Why distinct points: to avoid duplicate centroids that would
    //  produce empty clusters from the start
    let mut rng = rand::thread_rng();
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    let mut used = std::collections::HashSet::new();
    while centroids.len() < k {
        let idx = rng.gen_range(0..n);
        if used.insert(idx) {
            centroids.push(data[idx].clone());
        }
    }

    // Label vector: labels[i] = index of the cluster assigned to point i
    let mut labels = vec![0usize; n];

    for _ in 0..max_iter {
        // 2. Assign each point to the nearest centroid (Euclidean distance)
        let mut changed = false;
        for (i, point) in data.iter().enumerate() {
            // Find the nearest centroid
            let nearest = centroids.iter().enumerate()
                .map(|(j, c)| (j, euclidean_dist(point, c)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(j, _)| j)
                .unwrap_or(0);
            // Detect if the assignment changed (stopping criterion)
            if labels[i] != nearest {
                labels[i] = nearest;
                changed = true;
            }
        }

        // If no assignment changed, the algorithm has converged
        if !changed { break; }

        // 3. Recompute centroids as the mean of each cluster's points
        for (j, centroid) in centroids.iter_mut().enumerate().take(k) {
            // Collect points belonging to cluster j
            let members: Vec<&Vec<f64>> = data.iter().enumerate()
                .filter(|(i, _)| labels[*i] == j)
                .map(|(_, p)| p)
                .collect();
            // If the cluster is empty, keep the old centroid
            if members.is_empty() { continue; }
            let count = members.len() as f64;
            // New centroid = mean of coordinates in each dimension
            *centroid = (0..dim)
                .map(|d| members.iter().map(|p| p[d]).sum::<f64>() / count)
                .collect();
        }
    }

    labels
}

/// Computes the Euclidean distance between two vectors of the same dimension.
///
/// Formula: sqrt(sum((ai - bi)^2))
///
/// Parameter `a`: first vector
/// Parameter `b`: second vector
/// Returns: the Euclidean distance between a and b
fn euclidean_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

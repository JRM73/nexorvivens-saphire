// =============================================================================
// consciousness_metrics.rs — 3 scientific consciousness metrics
// =============================================================================
//
// Purpose: Implements 3 published and validated heuristic algorithms for
//          measuring consciousness / system complexity:
//
// 1. LZC (Lempel-Ziv Complexity) — algorithmic complexity of a temporal
//    sequence. Measures the informational richness of the signal.
//    Ref: Lempel & Ziv (1976), Casali et al. (2013) for consciousness
//    application.
//
// 2. PCI (Perturbational Complexity Index) — Casali, Massimini et al. (2013)
//    Perturbs the brain network, measures the complexity of the response.
//    Used clinically to assess consciousness in comatose patients, under
//    anesthesia, or during sleep.
//    PCI = LZC(spatiotemporal response) / source_entropy
//
// 3. Phi* (Phi-star) — computable approximation of Phi (IIT, Tononi)
//    Measures integrated information: how much information the system
//    generates AS A WHOLE beyond the sum of its parts.
//    Ref: Oizumi et al. (2014) — mismatched decoding approach
//    Here: approximation via Gaussian mutual information.
//
// These 3 metrics are complementary:
//   - LZC = raw signal complexity
//   - PCI = complexity of the RESPONSE to a perturbation (more robust)
//   - Phi* = information integration (irreducibility of the system)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neuroscience::brain_regions::{BrainNetwork, NUM_REGIONS};

// =============================================================================
// 1. LZC — Lempel-Ziv Complexity
// =============================================================================

/// Computes the Lempel-Ziv complexity (LZ76) of a binary sequence.
///
/// Algorithm:
/// - Traverse the sequence from left to right
/// - At each position, find the longest sub-word already seen
/// - Increment the complexity counter for each new sub-word
/// - Normalize by n / log2(n) (theoretical maximum complexity)
///
/// Returns a score [0, 1]: 0 = trivial sequence, 1 = maximally complex
pub fn lempel_ziv_complexity(binary_seq: &[bool]) -> f64 {
    let n = binary_seq.len();
    if n <= 1 {
        return 0.0;
    }

    // LZ76 algorithm (Kaspar & Schuster 1987):
    // Decompose the sequence into exhaustive sub-words.
    // At each step, find the longest prefix of the remainder
    // that appears in the already-parsed part (positions 0..i).
    // The reproduction can overflow into the current zone (sliding copy).
    let mut complexity: usize = 1;
    let mut i: usize = 1; // start of the current component (1st symbol is component #1)
    let mut l: usize = 1; // length of the current component

    while i + l <= n {
        // Search for s[i..i+l] as a substring starting BEFORE position i
        // The source can overflow into the current zone (LZ77-style copy)
        let pat_end = (i + l).min(n);
        let pattern = &binary_seq[i..pat_end];

        // Search in starting positions 0..i only
        let found = (0..i).any(|start| {
            // Compare element by element (can overflow into the current zone)
            for k in 0..pattern.len() {
                if start + k >= n || binary_seq[start + k] != pattern[k] {
                    return false;
                }
            }
            true
        });

        if found && i + l < n {
            // The pattern already exists, we can extend
            l += 1;
        } else {
            // New component found
            complexity += 1;
            i += l;
            l = 1;
        }
    }

    // Normalization: max complexity ~ n / log2(n) for a random sequence
    let log2n = (n as f64).log2();
    if log2n < 1.0 {
        return 0.5;
    }
    let max_complexity = n as f64 / log2n;
    (complexity as f64 / max_complexity).clamp(0.0, 1.0)
}

/// Binarizes a vector of floating-point values relative to their mean.
/// Each value above the mean -> true, otherwise -> false.
pub fn binarize(values: &[f64]) -> Vec<bool> {
    if values.is_empty() {
        return Vec::new();
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|&v| v > mean).collect()
}

/// Computes LZC on a time series of multidimensional vectors.
/// Concatenates the binarized channels then applies LZ76.
///
/// Used to measure the complexity of brain activity over time.
pub fn lzc_from_timeseries(timeseries: &[[f64; NUM_REGIONS]]) -> f64 {
    if timeseries.len() < 3 {
        return 0.0;
    }

    // Binarize each channel separately, then concatenate
    let mut binary = Vec::new();
    for channel in 0..NUM_REGIONS {
        let channel_values: Vec<f64> = timeseries.iter().map(|t| t[channel]).collect();
        binary.extend(binarize(&channel_values));
    }

    lempel_ziv_complexity(&binary)
}

// =============================================================================
// 2. PCI — Perturbational Complexity Index (Casali/Massimini 2013)
// =============================================================================

/// Number of simulation steps for the perturbation response.
const PCI_RESPONSE_STEPS: usize = 15;

/// Perturbation intensity (TMS analogy).
const PCI_PERTURBATION_STRENGTH: f64 = 0.5;

/// Computes PCI: perturbs a region, measures the complexity of the cascade.
///
/// Algorithm (adapted from Casali et al. 2013):
/// 1. Save the network state
/// 2. Inject a perturbation into a region (TMS analogy)
/// 3. Simulate PCI_RESPONSE_STEPS propagation steps
/// 4. Record the spatiotemporal matrix [time x regions]
/// 5. Binarize (significant activity vs not)
/// 6. Compute LZC of the binarized matrix
/// 7. Normalize by the source entropy
/// 8. Restore the original network state
///
/// target_region: index of the region to perturb (0-11)
pub fn perturbational_complexity_index(
    network: &BrainNetwork,
    chemistry: &crate::neurochemistry::NeuroChemicalState,
    target_region: usize,
) -> PciResult {
    if target_region >= NUM_REGIONS {
        return PciResult::default();
    }

    // 1. Clone the network to avoid modifying the original
    let mut sim = network.clone();

    // Save baseline activations
    let baseline: Vec<f64> = sim.regions.iter().map(|r| r.activation).collect();

    // 2. Inject the perturbation (TMS analogy)
    sim.regions[target_region].activation =
        (sim.regions[target_region].activation + PCI_PERTURBATION_STRENGTH).clamp(0.0, 1.0);

    // 3-4. Simulate and record the cascade
    let mut spatiotemporal = Vec::with_capacity(PCI_RESPONSE_STEPS);
    for _ in 0..PCI_RESPONSE_STEPS {
        sim.tick(chemistry, [0.0; 5]); // PCI: pure perturbation, no sensory input
        let activations: [f64; NUM_REGIONS] = {
            let mut arr = [0.0; NUM_REGIONS];
            for (i, r) in sim.regions.iter().enumerate() {
                arr[i] = r.activation;
            }
            arr
        };
        spatiotemporal.push(activations);
    }

    // 5. Binarize: significant activity = deviation > threshold from baseline
    let threshold = 0.05; // 5% deviation = significant
    let mut binary_matrix = Vec::new();
    let mut active_count = 0usize;
    let total = PCI_RESPONSE_STEPS * NUM_REGIONS;

    for step in &spatiotemporal {
        for (i, &activation) in step.iter().enumerate() {
            let significant = (activation - baseline[i]).abs() > threshold;
            binary_matrix.push(significant);
            if significant { active_count += 1; }
        }
    }

    // 6. LZC of the binarized matrix
    let lzc = lempel_ziv_complexity(&binary_matrix);

    // 7. Source entropy (proportion of active bits)
    let p = active_count as f64 / total as f64;
    let source_entropy = if p > 0.0 && p < 1.0 {
        -(p * p.log2() + (1.0 - p) * (1.0 - p).log2())
    } else {
        0.001 // avoid division by zero
    };

    // PCI = LZC normalized by entropy
    let pci = (lzc / source_entropy.max(0.001)).clamp(0.0, 1.0);

    // Count regions affected by the perturbation
    let regions_affected = spatiotemporal.last()
        .map(|last| {
            last.iter().zip(baseline.iter())
                .filter(|(&act, &base)| (act - base).abs() > threshold)
                .count()
        })
        .unwrap_or(0);

    PciResult {
        pci,
        lzc,
        source_entropy,
        target_region: crate::neuroscience::brain_regions::REGION_NAMES[target_region].to_string(),
        regions_affected,
        response_steps: PCI_RESPONSE_STEPS,
    }
}

/// Computes the mean PCI across all regions (more robust).
/// In clinical settings, multiple TMS stimulations are applied at different locations.
pub fn pci_mean_all_regions(
    network: &BrainNetwork,
    chemistry: &crate::neurochemistry::NeuroChemicalState,
) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_REGIONS {
        sum += perturbational_complexity_index(network, chemistry, i).pci;
    }
    sum / NUM_REGIONS as f64
}

/// Result of PCI computation for a given perturbation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciResult {
    /// Final PCI [0, 1]: 0 = no consciousness, 1 = maximal consciousness
    pub pci: f64,
    /// Raw LZ complexity of the response
    pub lzc: f64,
    /// Source entropy (binarized matrix)
    pub source_entropy: f64,
    /// Perturbed region
    pub target_region: String,
    /// Number of regions affected by the cascade
    pub regions_affected: usize,
    /// Number of simulation steps
    pub response_steps: usize,
}

impl Default for PciResult {
    fn default() -> Self {
        Self {
            pci: 0.0,
            lzc: 0.0,
            source_entropy: 0.0,
            target_region: String::new(),
            regions_affected: 0,
            response_steps: 0,
        }
    }
}

// =============================================================================
// 3. Phi* — Computable approximation of Phi (IIT)
// =============================================================================

/// Computes Phi*: approximation of integrated information (IIT).
///
/// Algorithm (inspired by Oizumi et al. 2014, Barrett & Seth 2011):
///
/// Principle: Phi* measures how much information the system generates
/// AS A WHOLE beyond the sum of its parts.
///
/// 1. Compute the total mutual information I(past; future) of the whole system
///    using regional activation time series
/// 2. For each bipartition of the system, compute the sum of mutual
///    information of the separated parts: sum(I(past_i; future_i))
/// 3. Phi* = I(whole) - min_partition(sum(I(parts)))
///
/// Uses the Gaussian approximation:
///    I(X; Y) = -0.5 * ln(1 - r^2) where r = Pearson correlation
///
/// To avoid the combinatorial explosion of partitions (2^12),
/// natural anatomical bipartitions are used:
///    - Left/Right (cortex/subcortex)
///    - Anterior/Posterior
///    - Cortical/Subcortical
///    and the minimum is taken.
pub fn phi_star(timeseries: &[[f64; NUM_REGIONS]]) -> PhiStarResult {
    let n = timeseries.len();
    if n < 5 {
        return PhiStarResult::default();
    }

    // Compute total mutual information: I(t-1; t) for the whole system
    let mi_whole = mutual_information_system(timeseries);

    // Anatomical bipartitions (indices of regions in each part)
    let partitions: [(&str, &[usize], &[usize]); 4] = [
        // Cortical vs Subcortical
        ("Cortical/Subcortical",
         &[2, 3, 5, 8, 9, 10],       // PFC-D, PFC-V, ACC, OFC, Temp, Par
         &[0, 1, 4, 6, 7, 11]),       // Amyg, Hipp, Insula, BG, Brainstem, Cereb

        // Anterior vs Posterior
        ("Anterior/Posterior",
         &[0, 2, 3, 5, 8],            // Amyg, PFC-D, PFC-V, ACC, OFC
         &[1, 4, 6, 7, 9, 10, 11]),   // Hipp, Insula, BG, Brainstem, Temp, Par, Cereb

        // Hemispheres (simplified: left=emotional, right=analytical)
        ("Emotional/Analytical",
         &[0, 1, 4, 6, 8],            // Amyg, Hipp, Insula, BG, OFC
         &[2, 3, 5, 7, 9, 10, 11]),   // PFC-D, PFC-V, ACC, Brainstem, Temp, Par, Cereb

        // Global Workspace vs periphery
        ("Workspace/Periphery",
         &[2, 3, 5, 9, 10],           // PFC-D, PFC-V, ACC, Temp, Par (associative cortex)
         &[0, 1, 4, 6, 7, 8, 11]),    // rest (subcortical + sensory)
    ];

    let mut min_partition_mi = f64::MAX;
    let mut mip_name = "";

    for &(name, part_a, part_b) in &partitions {
        // Extract sub-series for each part
        let mi_a = mutual_information_partition(timeseries, part_a);
        let mi_b = mutual_information_partition(timeseries, part_b);
        let partition_sum = mi_a + mi_b;

        if partition_sum < min_partition_mi {
            min_partition_mi = partition_sum;
            mip_name = name;
        }
    }

    // Phi* = whole information - best partition information
    // Normalized by MI_whole to obtain a ratio [0, 1]
    let phi_raw = (mi_whole - min_partition_mi).max(0.0);
    let phi_normalized = if mi_whole > 1e-10 {
        (phi_raw / mi_whole).clamp(0.0, 1.0)
    } else {
        0.0
    };

    PhiStarResult {
        phi_star: phi_normalized,
        mi_whole,
        mi_minimum_partition: min_partition_mi,
        minimum_information_partition: mip_name.to_string(),
        phi_raw,
    }
}

/// Gaussian mutual information of the whole system: I(X_t; X_{t+1})
/// Uses the multivariate correlation (trace of the correlation matrix).
fn mutual_information_system(timeseries: &[[f64; NUM_REGIONS]]) -> f64 {
    let n = timeseries.len();
    if n < 3 { return 0.0; }

    // Compute Pearson correlation between t and t+1 for each region
    let mut total_mi = 0.0;

    for ch in 0..NUM_REGIONS {
        let past: Vec<f64> = timeseries[..n-1].iter().map(|t| t[ch]).collect();
        let future: Vec<f64> = timeseries[1..].iter().map(|t| t[ch]).collect();
        total_mi += gaussian_mi(&past, &future);
    }

    // Add cross-correlations (inter-region)
    // To avoid O(N^2) explosion, sample the most important pairs
    let important_pairs: &[(usize, usize)] = &[
        (0, 3),  // Amygdala <-> PFC-Ventro
        (1, 2),  // Hippocampus <-> PFC-Dorso
        (0, 7),  // Amygdala <-> Brainstem
        (2, 5),  // PFC-Dorso <-> ACC
        (4, 5),  // Insula <-> ACC
        (6, 2),  // Basal Ganglia <-> PFC-Dorso
        (3, 8),  // PFC-Ventro <-> OFC
    ];

    for &(a, b) in important_pairs {
        let x: Vec<f64> = timeseries[..n-1].iter().map(|t| t[a]).collect();
        let y: Vec<f64> = timeseries[1..].iter().map(|t| t[b]).collect();
        total_mi += gaussian_mi(&x, &y) * 0.3; // Reduced weight for cross-correlations
    }

    total_mi
}

/// Mutual information of a partition (subset of regions).
fn mutual_information_partition(
    timeseries: &[[f64; NUM_REGIONS]],
    region_indices: &[usize],
) -> f64 {
    let n = timeseries.len();
    if n < 3 || region_indices.is_empty() { return 0.0; }

    let mut total_mi = 0.0;
    for &ch in region_indices {
        if ch >= NUM_REGIONS { continue; }
        let past: Vec<f64> = timeseries[..n-1].iter().map(|t| t[ch]).collect();
        let future: Vec<f64> = timeseries[1..].iter().map(|t| t[ch]).collect();
        total_mi += gaussian_mi(&past, &future);
    }

    total_mi
}

/// Gaussian mutual information between two vectors:
/// I(X; Y) = -0.5 * ln(1 - r^2) where r = Pearson correlation.
///
/// Returns 0.0 if data is insufficient or constant.
fn gaussian_mi(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 3 { return 0.0; }

    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;

    let mut cov_xy = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov_xy += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-12 { return 0.0; }

    let r = (cov_xy / denom).clamp(-0.999, 0.999); // Avoid log(0)
    -0.5 * (1.0 - r * r).ln()
}

/// Result of the Phi* computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiStarResult {
    /// Normalized Phi* [0, 1]: relative integrated information
    pub phi_star: f64,
    /// Mutual information of the whole system
    pub mi_whole: f64,
    /// Mutual information of the minimum partition
    pub mi_minimum_partition: f64,
    /// Name of the minimum partition (MIP -- Minimum Information Partition)
    pub minimum_information_partition: String,
    /// Raw Phi* (non-normalized)
    pub phi_raw: f64,
}

impl Default for PhiStarResult {
    fn default() -> Self {
        Self {
            phi_star: 0.0,
            mi_whole: 0.0,
            mi_minimum_partition: 0.0,
            minimum_information_partition: String::new(),
            phi_raw: 0.0,
        }
    }
}

// =============================================================================
// Synthesis of all 3 metrics
// =============================================================================

/// Synthesis of the 3 consciousness metrics in a single report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    /// LZC: Lempel-Ziv complexity of brain activity [0, 1]
    pub lzc: f64,
    /// PCI: Perturbational Complexity Index [0, 1]
    pub pci: PciResult,
    /// Phi*: approximation of integrated information [0, 1]
    pub phi_star: PhiStarResult,
    /// Composite score [0, 1]: weighted average of the 3 metrics
    pub composite_score: f64,
    /// Qualitative interpretation of the consciousness level
    pub interpretation: String,
}

/// Computes all 3 metrics and produces a synthetic report.
///
/// Parameters:
/// - timeseries: history of regional activations (at least 5 cycles)
/// - network: brain network (for PCI)
/// - chemistry: chemical state (for PCI propagation)
/// - target_region: region to perturb for PCI (None = best PCI)
pub fn compute_all_metrics(
    timeseries: &[[f64; NUM_REGIONS]],
    network: &BrainNetwork,
    chemistry: &crate::neurochemistry::NeuroChemicalState,
    target_region: Option<usize>,
) -> ConsciousnessMetrics {
    // 1. LZC on the raw time series
    let lzc = lzc_from_timeseries(timeseries);

    // 2. PCI -- perturb the target region or compute the mean
    let pci = if let Some(target) = target_region {
        perturbational_complexity_index(network, chemistry, target)
    } else {
        // Mean PCI over the 3 most active regions
        let mut indexed: Vec<(usize, f64)> = network.regions.iter()
            .enumerate()
            .map(|(i, r)| (i, r.activation))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top3: Vec<PciResult> = indexed.iter()
            .take(3)
            .map(|&(i, _)| perturbational_complexity_index(network, chemistry, i))
            .collect();

        let best = top3.iter()
            .max_by(|a, b| a.pci.partial_cmp(&b.pci).unwrap_or(std::cmp::Ordering::Equal));
        best.cloned().unwrap_or_default()
    };

    // 3. Phi* on the time series
    let phi_star_result = phi_star(timeseries);

    // Composite score: weights calibrated from the literature
    // PCI has the most weight (clinically validated)
    let composite = pci.pci * 0.40          // PCI: clinical validation (Casali 2013)
        + phi_star_result.phi_star * 0.35   // Phi*: theoretical foundation (IIT)
        + lzc * 0.25;                       // LZC: raw complexity

    let composite_score = composite.clamp(0.0, 1.0);

    // Clinical interpretation (thresholds inspired by Casali et al. 2013)
    let interpretation = if composite_score > 0.7 {
        "Vivid consciousness -- high integration and complexity (comparable to lucid wakefulness)"
    } else if composite_score > 0.5 {
        "Moderate consciousness -- integrated and responsive system (comparable to normal wakefulness)"
    } else if composite_score > 0.35 {
        "Reduced consciousness -- low integration (comparable to light sedation)"
    } else if composite_score > 0.2 {
        "Minimal consciousness -- partial responses (comparable to MCS vegetative state)"
    } else {
        "Absent or very low consciousness (comparable to coma or deep anesthesia)"
    }.to_string();

    ConsciousnessMetrics {
        lzc,
        pci,
        phi_star: phi_star_result,
        composite_score,
        interpretation,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzc_constant_sequence() {
        // Constant sequence = minimal complexity
        let seq = vec![true; 100];
        let c = lempel_ziv_complexity(&seq);
        assert!(c < 0.2, "Constant sequence must have low LZC: {}", c);
    }

    #[test]
    fn test_lzc_alternating_sequence() {
        // Alternating sequence = slightly more complex but repetitive
        let seq: Vec<bool> = (0..100).map(|i| i % 2 == 0).collect();
        let c = lempel_ziv_complexity(&seq);
        assert!(c > 0.0, "Alternating sequence must have LZC > 0: {}", c);
        assert!(c < 0.5, "Alternating sequence remains repetitive: {}", c);
    }

    #[test]
    fn test_lzc_random_sequence() {
        // Pseudo-random sequence = higher complexity than a constant
        // Using a simple PRNG with good distribution
        let mut state: u32 = 12345;
        let seq: Vec<bool> = (0..500).map(|_| {
            // xorshift32
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state % 2 == 0
        }).collect();
        let c = lempel_ziv_complexity(&seq);
        let c_constant = lempel_ziv_complexity(&vec![true; 500]);
        assert!(c > c_constant * 2.0,
            "Random sequence ({:.4}) must be much more complex than constant ({:.4})",
            c, c_constant);
    }

    #[test]
    fn test_binarize() {
        let values = vec![0.1, 0.5, 0.3, 0.8, 0.2]; // mean = 0.38
        let binary = binarize(&values);
        assert_eq!(binary, vec![false, true, false, true, false]);
    }

    #[test]
    fn test_gaussian_mi_correlated() {
        // Two perfectly correlated series
        let x: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1).collect();
        let y: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1 + 0.01).collect();
        let mi = gaussian_mi(&x, &y);
        assert!(mi > 1.0, "Correlated series must have high MI: {}", mi);
    }

    #[test]
    fn test_gaussian_mi_uncorrelated() {
        // Two uncorrelated series
        let x: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1).collect();
        let y: Vec<f64> = (0..50).map(|i| ((i * 7 + 13) % 50) as f64 * 0.02).collect();
        let mi = gaussian_mi(&x, &y);
        assert!(mi < 1.0, "Uncorrelated series must have low MI: {}", mi);
    }

    #[test]
    fn test_phi_star_needs_data() {
        // Not enough data
        let short: Vec<[f64; NUM_REGIONS]> = vec![[0.5; NUM_REGIONS]; 2];
        let result = phi_star(&short);
        assert_eq!(result.phi_star, 0.0, "Phi* must be 0 without enough data");
    }

    #[test]
    fn test_phi_star_constant_system() {
        // Constant system = no information
        let constant: Vec<[f64; NUM_REGIONS]> = vec![[0.5; NUM_REGIONS]; 20];
        let result = phi_star(&constant);
        assert!(result.phi_star < 0.1,
            "Constant system must have low Phi*: {}", result.phi_star);
    }

    #[test]
    fn test_pci_basic() {
        let network = BrainNetwork::new();
        let chemistry = crate::neurochemistry::NeuroChemicalState::default();
        let result = perturbational_complexity_index(&network, &chemistry, 0);
        assert!(result.pci >= 0.0 && result.pci <= 1.0,
            "PCI must be in [0, 1]: {}", result.pci);
        assert!(result.regions_affected > 0,
            "The perturbation must affect at least one region");
    }

    #[test]
    fn test_composite_score() {
        // Generate a minimal history
        let network = BrainNetwork::new();
        let chemistry = crate::neurochemistry::NeuroChemicalState::default();
        let timeseries: Vec<[f64; NUM_REGIONS]> = (0..20).map(|i| {
            let mut arr = [0.3; NUM_REGIONS];
            for j in 0..NUM_REGIONS {
                arr[j] += (i as f64 * 0.1 + j as f64 * 0.05).sin() * 0.2;
            }
            arr
        }).collect();

        let metrics = compute_all_metrics(&timeseries, &network, &chemistry, None);
        assert!(metrics.composite_score >= 0.0 && metrics.composite_score <= 1.0);
        assert!(!metrics.interpretation.is_empty());
    }
}

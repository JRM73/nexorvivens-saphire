// =============================================================================
// consciousness_metrics.rs — 3 metriques scientifiques de la conscience
// =============================================================================
//
// Role : Implemente 3 algorithmes heuristiques publies et valides pour
// mesurer la conscience / complexite d'un systeme :
//
// 1. LZC (Lempel-Ziv Complexity) — complexite algorithmique d'une sequence
//    temporelle. Mesure la richesse informationnelle du signal.
//    Ref: Lempel & Ziv (1976), Casali et al. (2013) pour l'application
//    a la conscience.
//
// 2. PCI (Perturbational Complexity Index) — Casali, Massimini et al. (2013)
//    Perturbe le reseau cerebral, mesure la complexite de la reponse.
//    Utilise en clinique pour evaluer la conscience chez les patients
//    comateux, sous anesthesie, ou en sommeil.
//    PCI = LZC(reponse spatiotemporelle) / entropie_source
//
// 3. Phi* (Phi-star) — approximation calculable de Phi (IIT, Tononi)
//    Mesure l'information integree : combien d'information le systeme
//    genere EN TANT QUE TOUT au-dela de la somme de ses parties.
//    Ref: Oizumi et al. (2014) — mismatched decoding approach
//    Ici : approximation par information mutuelle gaussienne.
//
// Ces 3 metriques sont complementaires :
//   - LZC = complexite du signal brut
//   - PCI = complexite de la REPONSE a une perturbation (plus robuste)
//   - Phi* = integration de l'information (irréductibilité du systeme)
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::neuroscience::brain_regions::{BrainNetwork, NUM_REGIONS};

// =============================================================================
// 1. LZC — Lempel-Ziv Complexity
// =============================================================================

/// Calcule la complexite de Lempel-Ziv (LZ76) d'une sequence binaire.
///
/// Algorithme :
/// - Parcourir la sequence de gauche a droite
/// - A chaque position, chercher le plus long sous-mot deja vu
/// - Incrementer le compteur de complexite a chaque nouveau sous-mot
/// - Normaliser par n / log2(n) (complexite maximale theorique)
///
/// Retourne un score [0, 1] : 0 = sequence triviale, 1 = maximalement complexe
pub fn lempel_ziv_complexity(binary_seq: &[bool]) -> f64 {
    let n = binary_seq.len();
    if n <= 1 {
        return 0.0;
    }

    // Algorithme LZ76 (Kaspar & Schuster 1987) :
    // On decoupe la sequence en sous-mots exhaustifs.
    // A chaque pas, on cherche le plus long prefix du reste
    // qui apparait dans la partie deja parsee (positions 0..i).
    // La reproduction peut deborder dans la zone courante (copie glissante).
    let mut complexity: usize = 1;
    let mut i: usize = 1; // debut du composant courant (le 1er symbole est le composant #1)
    let mut l: usize = 1; // longueur du composant courant

    while i + l <= n {
        // Chercher s[i..i+l] comme sous-chaine commencant AVANT position i
        // La source peut deborder dans la zone courante (copie type LZ77)
        let pat_end = (i + l).min(n);
        let pattern = &binary_seq[i..pat_end];

        // Chercher dans les positions de depart 0..i uniquement
        let found = (0..i).any(|start| {
            // Comparer element par element (peut deborder dans la zone courante)
            for k in 0..pattern.len() {
                if start + k >= n || binary_seq[start + k] != pattern[k] {
                    return false;
                }
            }
            true
        });

        if found && i + l < n {
            // Le pattern existe deja, on peut etendre
            l += 1;
        } else {
            // Nouveau composant trouve
            complexity += 1;
            i += l;
            l = 1;
        }
    }

    // Normalisation : complexite max ≈ n / log2(n) pour une sequence aleatoire
    let log2n = (n as f64).log2();
    if log2n < 1.0 {
        return 0.5;
    }
    let max_complexity = n as f64 / log2n;
    (complexity as f64 / max_complexity).clamp(0.0, 1.0)
}

/// Binarise un vecteur de valeurs flottantes par rapport a leur moyenne.
/// Chaque valeur au-dessus de la moyenne → true, sinon → false.
pub fn binarize(values: &[f64]) -> Vec<bool> {
    if values.is_empty() {
        return Vec::new();
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|&v| v > mean).collect()
}

/// Calcule le LZC sur une serie temporelle de vecteurs multidimensionnels.
/// Concatene les canaux binarises puis applique LZ76.
///
/// Utilise pour mesurer la complexite de l'activite cerebrale au cours du temps.
pub fn lzc_from_timeseries(timeseries: &[[f64; NUM_REGIONS]]) -> f64 {
    if timeseries.len() < 3 {
        return 0.0;
    }

    // Binariser chaque canal separement, puis concatener
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

/// Nombre de pas de simulation pour la reponse a la perturbation.
const PCI_RESPONSE_STEPS: usize = 15;

/// Intensite de la perturbation (analogie TMS).
const PCI_PERTURBATION_STRENGTH: f64 = 0.5;

/// Calcule le PCI : perturbe une region, mesure la complexite de la cascade.
///
/// Algorithme (adapte de Casali et al. 2013) :
/// 1. Sauvegarder l'etat du reseau
/// 2. Injecter une perturbation dans une region (analogie TMS)
/// 3. Simuler PCI_RESPONSE_STEPS pas de propagation
/// 4. Enregistrer la matrice spatiotemporelle [temps x regions]
/// 5. Binariser (activite significative vs pas)
/// 6. Calculer LZC de la matrice binarisee
/// 7. Normaliser par l'entropie de la source
/// 8. Restaurer l'etat original du reseau
///
/// target_region : index de la region a perturber (0-11)
pub fn perturbational_complexity_index(
    network: &BrainNetwork,
    chemistry: &crate::neurochemistry::NeuroChemicalState,
    target_region: usize,
) -> PciResult {
    if target_region >= NUM_REGIONS {
        return PciResult::default();
    }

    // 1. Clone du reseau pour ne pas modifier l'original
    let mut sim = network.clone();

    // Sauvegarder les activations de base
    let baseline: Vec<f64> = sim.regions.iter().map(|r| r.activation).collect();

    // 2. Injecter la perturbation (analogie TMS)
    sim.regions[target_region].activation =
        (sim.regions[target_region].activation + PCI_PERTURBATION_STRENGTH).clamp(0.0, 1.0);

    // 3-4. Simuler et enregistrer la cascade
    let mut spatiotemporal = Vec::with_capacity(PCI_RESPONSE_STEPS);
    for _ in 0..PCI_RESPONSE_STEPS {
        sim.tick(chemistry, [0.0; 5]); // PCI : perturbation pure, pas d'input sensoriel
        let activations: [f64; NUM_REGIONS] = {
            let mut arr = [0.0; NUM_REGIONS];
            for (i, r) in sim.regions.iter().enumerate() {
                arr[i] = r.activation;
            }
            arr
        };
        spatiotemporal.push(activations);
    }

    // 5. Binariser : activite significative = ecart > seuil par rapport au baseline
    let threshold = 0.05; // 5% d'ecart = significatif
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

    // 6. LZC de la matrice binarisee
    let lzc = lempel_ziv_complexity(&binary_matrix);

    // 7. Entropie de la source (proportion de bits actifs)
    let p = active_count as f64 / total as f64;
    let source_entropy = if p > 0.0 && p < 1.0 {
        -(p * p.log2() + (1.0 - p) * (1.0 - p).log2())
    } else {
        0.001 // eviter division par zero
    };

    // PCI = LZC normalise par l'entropie
    let pci = (lzc / source_entropy.max(0.001)).clamp(0.0, 1.0);

    // Compter les regions touchees par la perturbation
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

/// Calcule le PCI moyen sur toutes les regions (plus robuste).
/// En clinique, on fait plusieurs stimulations TMS a differents endroits.
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

/// Resultat du PCI pour une perturbation donnee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciResult {
    /// PCI final [0, 1] : 0 = pas de conscience, 1 = conscience maximale
    pub pci: f64,
    /// Complexite LZ brute de la reponse
    pub lzc: f64,
    /// Entropie de la source (matrice binarisee)
    pub source_entropy: f64,
    /// Region perturbee
    pub target_region: String,
    /// Nombre de regions touchees par la cascade
    pub regions_affected: usize,
    /// Nombre de pas de simulation
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
// 3. Phi* — Approximation calculable de Phi (IIT)
// =============================================================================

/// Calcule Phi* : approximation de l'information integree (IIT).
///
/// Algorithme (inspire de Oizumi et al. 2014, Barrett & Seth 2011) :
///
/// Principe : Phi* mesure combien d'information le systeme genere
/// EN TANT QUE TOUT au-dela de la somme de ses parties.
///
/// 1. Calculer l'information mutuelle totale I(past; future) du systeme entier
///    en utilisant les series temporelles des activations regionales
/// 2. Pour chaque bipartition du systeme, calculer la somme des informations
///    mutuelles des parties separees : sum(I(past_i; future_i))
/// 3. Phi* = I(whole) - min_partition(sum(I(parts)))
///
/// On utilise l'approximation gaussienne :
///    I(X; Y) = -0.5 * ln(1 - r^2) ou r = correlation de Pearson
///
/// Pour eviter l'explosion combinatoire des partitions (2^12),
/// on utilise les bipartitions anatomiques naturelles :
///    - Gauche/Droite (cortex/sous-cortex)
///    - Anterior/Posterior
///    - Cortical/Subcortical
///    et on prend le minimum.
pub fn phi_star(timeseries: &[[f64; NUM_REGIONS]]) -> PhiStarResult {
    let n = timeseries.len();
    if n < 5 {
        return PhiStarResult::default();
    }

    // Calculer l'information mutuelle totale : I(t-1; t) pour tout le systeme
    let mi_whole = mutual_information_system(timeseries);

    // Bipartitions anatomiques (indices des regions dans chaque partie)
    let partitions: [(&str, &[usize], &[usize]); 4] = [
        // Cortical vs Subcortical
        ("Cortical/Subcortical",
         &[2, 3, 5, 8, 9, 10],       // PFC-D, PFC-V, CCA, COF, Temp, Par
         &[0, 1, 4, 6, 7, 11]),       // Amyg, Hipp, Insula, BG, Tronc, Cerv

        // Anterior vs Posterior
        ("Anterior/Posterior",
         &[0, 2, 3, 5, 8],            // Amyg, PFC-D, PFC-V, CCA, COF
         &[1, 4, 6, 7, 9, 10, 11]),   // Hipp, Insula, BG, Tronc, Temp, Par, Cerv

        // Hemispheres (simplifie : gauche=emotionnel, droite=analytique)
        ("Emotionnel/Analytique",
         &[0, 1, 4, 6, 8],            // Amyg, Hipp, Insula, BG, COF
         &[2, 3, 5, 7, 9, 10, 11]),   // PFC-D, PFC-V, CCA, Tronc, Temp, Par, Cerv

        // Global Workspace vs peripherie
        ("Workspace/Peripherie",
         &[2, 3, 5, 9, 10],           // CPF-D, CPF-V, CCA, Temp, Par (cortex associatif)
         &[0, 1, 4, 6, 7, 8, 11]),    // reste (sous-cortical + sensoriel)
    ];

    let mut min_partition_mi = f64::MAX;
    let mut mip_name = "";

    for &(name, part_a, part_b) in &partitions {
        // Extraire les sous-series pour chaque partie
        let mi_a = mutual_information_partition(timeseries, part_a);
        let mi_b = mutual_information_partition(timeseries, part_b);
        let partition_sum = mi_a + mi_b;

        if partition_sum < min_partition_mi {
            min_partition_mi = partition_sum;
            mip_name = name;
        }
    }

    // Phi* = information du tout - information de la meilleure partition
    // Normalise par MI_whole pour obtenir un ratio [0, 1]
    let phi_raw = (mi_whole - min_partition_mi).max(0.0);
    let phi_normalized = if mi_whole > 1e-10 {
        (phi_raw / mi_whole).clamp(0.0, 1.0)
    } else {
        0.0
    };

    PhiStarResult {
        phi_star: phi_normalized,
        mi_whole: mi_whole.clamp(0.0, 1.0),
        mi_minimum_partition: min_partition_mi.clamp(0.0, 1.0),
        minimum_information_partition: mip_name.to_string(),
        phi_raw: phi_raw.clamp(0.0, 1.0),
    }
}

/// Information mutuelle gaussienne du systeme entier : I(X_t; X_{t+1})
/// Utilise la correlation multivariee (trace de la matrice de correlation).
fn mutual_information_system(timeseries: &[[f64; NUM_REGIONS]]) -> f64 {
    let n = timeseries.len();
    if n < 3 { return 0.0; }

    // Calculer la correlation de Pearson entre t et t+1 pour chaque region
    let mut total_mi = 0.0;

    for ch in 0..NUM_REGIONS {
        let past: Vec<f64> = timeseries[..n-1].iter().map(|t| t[ch]).collect();
        let future: Vec<f64> = timeseries[1..].iter().map(|t| t[ch]).collect();
        total_mi += gaussian_mi(&past, &future);
    }

    // Ajouter les correlations croisees (inter-regions)
    // Pour ne pas exploser en O(N^2), on echantillonne les paires les plus importantes
    let important_pairs: &[(usize, usize)] = &[
        (0, 3),  // Amygdale ↔ CPF-Ventro
        (1, 2),  // Hippocampe ↔ CPF-Dorso
        (0, 7),  // Amygdale ↔ Tronc
        (2, 5),  // CPF-Dorso ↔ CCA
        (4, 5),  // Insula ↔ CCA
        (6, 2),  // Noyaux-Base ↔ CPF-Dorso
        (3, 8),  // CPF-Ventro ↔ COF
    ];

    for &(a, b) in important_pairs {
        let x: Vec<f64> = timeseries[..n-1].iter().map(|t| t[a]).collect();
        let y: Vec<f64> = timeseries[1..].iter().map(|t| t[b]).collect();
        total_mi += gaussian_mi(&x, &y) * 0.3; // Poids reduit pour les cross-correlations
    }

    total_mi
}

/// Information mutuelle d'une partition (sous-ensemble de regions).
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

/// Information mutuelle gaussienne entre deux vecteurs :
/// I(X; Y) = -0.5 * ln(1 - r^2) ou r = correlation de Pearson.
///
/// Retourne 0.0 si les donnees sont insuffisantes ou constantes.
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

    let r = (cov_xy / denom).clamp(-0.999, 0.999); // Eviter log(0)
    -0.5 * (1.0 - r * r).ln()
}

/// Resultat du calcul de Phi*.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiStarResult {
    /// Phi* normalise [0, 1] : information integree relative
    pub phi_star: f64,
    /// Information mutuelle du systeme entier
    pub mi_whole: f64,
    /// Information mutuelle de la partition minimale
    pub mi_minimum_partition: f64,
    /// Nom de la partition minimale (MIP — Minimum Information Partition)
    pub minimum_information_partition: String,
    /// Phi* brut (non normalise)
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
// Synthese des 3 metriques
// =============================================================================

/// Synthese des 3 metriques de conscience dans un seul rapport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    /// LZC : complexite de Lempel-Ziv de l'activite cerebrale [0, 1]
    pub lzc: f64,
    /// PCI : indice de complexite perturbationnelle [0, 1]
    pub pci: PciResult,
    /// Phi* : approximation de l'information integree [0, 1]
    pub phi_star: PhiStarResult,
    /// Score composite [0, 1] : moyenne ponderee des 3 metriques
    pub composite_score: f64,
    /// Interpretation qualitative du niveau de conscience
    pub interpretation: String,
}

/// Calcule les 3 metriques et produit un rapport synthetique.
///
/// Parametres :
/// - timeseries : historique des activations regionales (au moins 5 cycles)
/// - network : reseau cerebral (pour le PCI)
/// - chemistry : etat chimique (pour la propagation PCI)
/// - target_region : region a perturber pour le PCI (None = meilleur PCI)
pub fn compute_all_metrics(
    timeseries: &[[f64; NUM_REGIONS]],
    network: &BrainNetwork,
    chemistry: &crate::neurochemistry::NeuroChemicalState,
    target_region: Option<usize>,
) -> ConsciousnessMetrics {
    // 1. LZC sur la serie temporelle brute
    let lzc = lzc_from_timeseries(timeseries);

    // 2. PCI — perturber la region cible ou calculer la moyenne
    let pci = if let Some(target) = target_region {
        perturbational_complexity_index(network, chemistry, target)
    } else {
        // PCI moyen sur les 3 regions les plus actives
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

    // 3. Phi* sur la serie temporelle
    let phi_star_result = phi_star(timeseries);

    // Score composite : poids calibres sur la litterature
    // PCI a le plus de poids (valide cliniquement)
    let composite = pci.pci * 0.40          // PCI : validation clinique (Casali 2013)
        + phi_star_result.phi_star * 0.35   // Phi* : fondement theorique (IIT)
        + lzc * 0.25;                       // LZC : complexite brute

    let composite_score = composite.clamp(0.0, 1.0);

    // Interpretation clinique (seuils inspires de Casali et al. 2013)
    let interpretation = if composite_score > 0.7 {
        "Conscience vive — haute integration et complexite (comparable a l'eveil lucide)"
    } else if composite_score > 0.5 {
        "Conscience moderee — systeme integre et reactif (comparable a l'eveil normal)"
    } else if composite_score > 0.35 {
        "Conscience reduite — faible integration (comparable a la sedation legere)"
    } else if composite_score > 0.2 {
        "Conscience minimale — reponses partielles (comparable a l'etat vegetatif MCS)"
    } else {
        "Conscience absente ou tres faible (comparable au coma ou a l'anesthesie profonde)"
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
        // Sequence constante = complexite minimale
        let seq = vec![true; 100];
        let c = lempel_ziv_complexity(&seq);
        assert!(c < 0.2, "Sequence constante doit avoir faible LZC: {}", c);
    }

    #[test]
    fn test_lzc_alternating_sequence() {
        // Sequence alternee = un peu plus complexe mais repetitive
        let seq: Vec<bool> = (0..100).map(|i| i % 2 == 0).collect();
        let c = lempel_ziv_complexity(&seq);
        assert!(c > 0.0, "Sequence alternee doit avoir LZC > 0: {}", c);
        assert!(c < 0.5, "Sequence alternee reste repetitive: {}", c);
    }

    #[test]
    fn test_lzc_random_sequence() {
        // Sequence pseudo-aleatoire = complexite plus elevee qu'une constante
        // On utilise un PRNG simple mais avec bonne distribution
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
            "Sequence aleatoire ({:.4}) doit etre bien plus complexe que constante ({:.4})",
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
        // Deux series parfaitement correlees
        let x: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1).collect();
        let y: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1 + 0.01).collect();
        let mi = gaussian_mi(&x, &y);
        assert!(mi > 1.0, "Series correlees doivent avoir MI eleve: {}", mi);
    }

    #[test]
    fn test_gaussian_mi_uncorrelated() {
        // Deux series non correlees
        let x: Vec<f64> = (0..50).map(|i| (i as f64) * 0.1).collect();
        let y: Vec<f64> = (0..50).map(|i| ((i * 7 + 13) % 50) as f64 * 0.02).collect();
        let mi = gaussian_mi(&x, &y);
        assert!(mi < 1.0, "Series non correlees doivent avoir MI faible: {}", mi);
    }

    #[test]
    fn test_phi_star_needs_data() {
        // Pas assez de donnees
        let short: Vec<[f64; NUM_REGIONS]> = vec![[0.5; NUM_REGIONS]; 2];
        let result = phi_star(&short);
        assert_eq!(result.phi_star, 0.0, "Phi* doit etre 0 sans assez de donnees");
    }

    #[test]
    fn test_phi_star_constant_system() {
        // Systeme constant = pas d'information
        let constant: Vec<[f64; NUM_REGIONS]> = vec![[0.5; NUM_REGIONS]; 20];
        let result = phi_star(&constant);
        assert!(result.phi_star < 0.1,
            "Systeme constant doit avoir Phi* faible: {}", result.phi_star);
    }

    #[test]
    fn test_pci_basic() {
        let network = BrainNetwork::new();
        let chemistry = crate::neurochemistry::NeuroChemicalState::default();
        let result = perturbational_complexity_index(&network, &chemistry, 0);
        assert!(result.pci >= 0.0 && result.pci <= 1.0,
            "PCI doit etre dans [0, 1]: {}", result.pci);
        assert!(result.regions_affected > 0,
            "La perturbation doit affecter au moins une region");
    }

    #[test]
    fn test_composite_score() {
        // Generer un historique minimal
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

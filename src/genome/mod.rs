// =============================================================================
// genome/mod.rs — DNA encoding / deterministic generation
//
// Purpose: From a seed (u64), generates a unique and reproducible genome
//          encoding the agent's predispositions: temperament, chemical
//          baselines, physical traits, vulnerabilities, cognitive aptitudes.
//          Two different seeds = two fundamentally different individuals.
//          Same seed = same individual (deterministic via ChaCha8 PRNG).
//
// Place in the architecture:
//   Called at boot in lifecycle/mod.rs, after loading the config.
//   The genome is stored in SaphireAgent and exposed via GET /api/genome.
// =============================================================================

use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

// =============================================================================
// Genome structures
// =============================================================================

/// Complete genome generated from a deterministic seed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub seed: u64,
    pub dna_hash: String,
    pub temperament: TemperamentGenes,
    pub chemical: ChemicalGenes,
    pub physical: PhysicalGenes,
    pub vulnerabilities: VulnerabilityGenes,
    pub cognitive: CognitiveGenes,
    /// Polygenic scores (GWAS) — each major trait is influenced
    /// by multiple loci (genomic positions) as in real genetics.
    /// Model inspired by Polygenic Risk Scores (PRS) used in medicine.
    #[serde(default)]
    pub polygenic: PolygenicScores,
}

/// Temperament genes — personality foundation.
/// Each trait ranges from 0.0 to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperamentGenes {
    /// 0.0 = extroverted, 1.0 = introverted
    pub introversion: f64,
    /// 0.0 = emotionally stable, 1.0 = highly reactive
    pub neuroticism: f64,
    /// 0.0 = flexible/spontaneous, 1.0 = rigorous/methodical
    pub conscientiousness: f64,
    /// 0.0 = competitive, 1.0 = cooperative
    pub agreeableness: f64,
    /// 0.0 = conservative, 1.0 = curious/adventurous
    pub openness: f64,
}

/// Chemical genes — offsets on neurochemical baselines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalGenes {
    /// Dopamine baseline deviation (-0.1 to +0.1)
    pub baseline_dopamine_offset: f64,
    /// Serotonin baseline deviation (-0.1 to +0.1)
    pub baseline_serotonin_offset: f64,
    /// Cortisol baseline deviation (-0.1 to +0.1)
    pub baseline_cortisol_offset: f64,
    /// Homeostasis speed multiplier (0.5 to 1.5)
    pub homeostasis_speed: f64,
    /// Global receptor sensitivity (0.5 to 1.5)
    pub receptor_sensitivity: f64,
    /// Stress resilience (0.0 to 1.0)
    pub stress_resilience: f64,
}

/// Physical genes — genome-derived appearance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalGenes {
    /// Eye color index (0-7)
    pub eye_color_seed: u8,
    /// Hair type index (0-5)
    pub hair_type_seed: u8,
    /// Skin tone index (0-5)
    pub skin_tone_seed: u8,
    /// Height offset from average (-20 to +20 cm)
    pub height_offset: i8,
    /// Metabolism speed (0.5 to 1.5)
    pub metabolism_speed: f64,
}

/// Vulnerability genes — predispositions (not diseases).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityGenes {
    /// Addiction susceptibility (0.0 to 1.0)
    pub addiction_susceptibility: f64,
    /// Anxiety predisposition (0.0 to 1.0)
    pub anxiety_predisposition: f64,
    /// Depression predisposition (0.0 to 1.0)
    pub depression_predisposition: f64,
    /// Immune system robustness (0.5 to 1.0)
    pub immune_baseline: f64,
    /// Seed for attachment style (0-3 → Secure/Anxious/Avoidant/Disorganized)
    pub attachment_style_seed: u8,
}

/// Cognitive genes — learning and reasoning aptitudes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveGenes {
    /// Learning speed (0.5 to 1.5)
    pub learning_speed: f64,
    /// Memory retention (0.5 to 1.5)
    pub memory_retention: f64,
    /// Creativity factor (0.5 to 1.5)
    pub creativity_factor: f64,
    /// Analytical vs intuitive bias (0.0 = intuitive, 1.0 = analytical)
    pub analytical_bias: f64,
}

/// Polygenic scores — GWAS model (Genome-Wide Association Studies).
///
/// In real genetics, complex traits (intelligence, depression risk,
/// temperament) do not depend on a single gene but on hundreds of loci,
/// each with a tiny effect. The polygenic score is the weighted sum
/// of all these effects.
///
/// Here we simulate N_LOCI alleles per trait, each drawn randomly,
/// then compute the weighted average. The result naturally follows
/// a quasi-normal distribution (central limit theorem) even though
/// each individual allele is uniform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygenicScores {
    // --- Risk scores (PRS — Polygenic Risk Scores) ---

    /// Polygenic depression risk (0.0-1.0, >0.7 = high risk)
    /// ~50 loci involved (SLC6A4, FKBP5, BDNF, etc.)
    pub prs_depression: f64,
    /// Polygenic anxiety risk (0.0-1.0)
    /// Loci: CRHR1, SLC6A4, COMT, RGS2, etc.
    pub prs_anxiety: f64,
    /// Polygenic addiction risk (0.0-1.0)
    /// Loci: DRD2, OPRM1, ALDH2, CHRNA5, etc.
    pub prs_addiction: f64,

    // --- Positive trait scores ---

    /// Polygenic empathy score (0.0-1.0)
    /// Loci: OXTR, CD38, AVPR1A, etc.
    pub pgs_empathy: f64,
    /// Polygenic stress resilience score (0.0-1.0)
    /// Loci: NR3C1, FKBP5, CRHR1, NPY, etc.
    pub pgs_resilience: f64,
    /// Polygenic learning capacity score (0.0-1.0)
    /// Loci: BDNF, KIBRA, COMT, ARC, etc.
    pub pgs_learning: f64,
    /// Polygenic creativity score (0.0-1.0)
    /// Loci: DRD4, COMT, DARPP-32, etc.
    pub pgs_creativity: f64,

    // --- Pharmacogenomics ---

    /// Neurotransmitter metabolism efficiency (0.5-1.5)
    /// Loci: CYP2D6, CYP2C19, MAO-A, COMT
    /// <1.0 = slow metabolizer, >1.0 = fast metabolizer
    pub pharmacogenomic_metabolism: f64,

    // --- Raw data (for transparency and API) ---

    /// Number of simulated loci per trait
    pub loci_per_trait: usize,
    /// Estimated heritability (proportion of genetic variance)
    pub estimated_heritability: f64,
}

impl Default for PolygenicScores {
    fn default() -> Self {
        Self {
            prs_depression: 0.5,
            prs_anxiety: 0.5,
            prs_addiction: 0.5,
            pgs_empathy: 0.5,
            pgs_resilience: 0.5,
            pgs_learning: 0.5,
            pgs_creativity: 0.5,
            pharmacogenomic_metabolism: 1.0,
            loci_per_trait: 30,
            estimated_heritability: 0.5,
        }
    }
}

/// Number of simulated loci per polygenic trait.
/// In reality it is hundreds to thousands, but 30 is enough to
/// obtain a quasi-normal distribution (CLT) with variability.
const N_LOCI: usize = 30;

/// Generates a polygenic score from N alleles.
///
/// Each locus has a random effect (allele) and a weight
/// (effect size) drawn from an exponentially decaying distribution.
/// The first loci have more influence (as in real GWAS,
/// where a few SNPs have a stronger effect than others).
///
/// The result is bounded within [min, max].
fn polygenic_score(rng: &mut ChaCha8Rng, min: f64, max: f64) -> f64 {
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for i in 0..N_LOCI {
        // Exponentially decreasing weight (dominant SNP effect)
        let weight = (-0.05 * i as f64).exp();
        // Allele: 0 (homozygous ref), 1 (heterozygous), 2 (homozygous alt)
        let allele: f64 = rng.gen_range(0..=2) as f64 / 2.0;
        weighted_sum += allele * weight;
        weight_total += weight;
    }

    let normalized = weighted_sum / weight_total; // [0, 1]
    min + normalized * (max - min)
}

impl PolygenicScores {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            prs_depression: polygenic_score(rng, 0.0, 1.0),
            prs_anxiety: polygenic_score(rng, 0.0, 1.0),
            prs_addiction: polygenic_score(rng, 0.0, 1.0),
            pgs_empathy: polygenic_score(rng, 0.0, 1.0),
            pgs_resilience: polygenic_score(rng, 0.0, 1.0),
            pgs_learning: polygenic_score(rng, 0.0, 1.0),
            pgs_creativity: polygenic_score(rng, 0.0, 1.0),
            pharmacogenomic_metabolism: polygenic_score(rng, 0.5, 1.5),
            loci_per_trait: N_LOCI,
            estimated_heritability: 0.5,
        }
    }

    /// Modifies existing genes based on polygenic scores.
    /// This creates realistic correlations between PRS and traits
    /// (e.g. high depression PRS → higher neuroticism,
    /// high empathy PGS → higher agreeableness).
    pub fn modulate_genes(
        &self,
        temperament: &mut TemperamentGenes,
        chemical: &mut ChemicalGenes,
        vulnerabilities: &mut VulnerabilityGenes,
        cognitive: &mut CognitiveGenes,
    ) {
        // Polygenic influence on temperament (weight = 30%, due to ~50% heritability)
        let pg_weight = 0.3;

        // PRS depression → increases neuroticism
        temperament.neuroticism = blend(
            temperament.neuroticism,
            self.prs_depression,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PGS empathy → increases agreeableness
        temperament.agreeableness = blend(
            temperament.agreeableness,
            self.pgs_empathy,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PGS creativity → increases openness
        temperament.openness = blend(
            temperament.openness,
            self.pgs_creativity,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PRS anxiety → influences cortisol baseline
        chemical.baseline_cortisol_offset += (self.prs_anxiety - 0.5) * 0.05;
        chemical.baseline_cortisol_offset = chemical.baseline_cortisol_offset.clamp(-0.15, 0.15);

        // PGS resilience → influence stress_resilience
        chemical.stress_resilience = blend(
            chemical.stress_resilience,
            self.pgs_resilience,
            pg_weight,
        ).clamp(0.0, 1.0);

        // Pharmacogenomics → influences homeostasis_speed
        chemical.homeostasis_speed *= 0.7 + self.pharmacogenomic_metabolism * 0.3;
        chemical.homeostasis_speed = chemical.homeostasis_speed.clamp(0.3, 2.0);

        // PRS addiction → influence vulnerability
        vulnerabilities.addiction_susceptibility = blend(
            vulnerabilities.addiction_susceptibility,
            self.prs_addiction,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PRS depression/anxiety → influences predispositions
        vulnerabilities.depression_predisposition = blend(
            vulnerabilities.depression_predisposition,
            self.prs_depression,
            pg_weight * 0.5,
        ).clamp(0.0, 1.0);
        vulnerabilities.anxiety_predisposition = blend(
            vulnerabilities.anxiety_predisposition,
            self.prs_anxiety,
            pg_weight * 0.5,
        ).clamp(0.0, 1.0);

        // PGS learning → influence cognitive
        cognitive.learning_speed = blend(
            cognitive.learning_speed,
            0.5 + self.pgs_learning,
            pg_weight,
        ).clamp(0.5, 1.5);

        // PGS creativity → influence creativity_factor
        cognitive.creativity_factor = blend(
            cognitive.creativity_factor,
            0.5 + self.pgs_creativity,
            pg_weight,
        ).clamp(0.5, 1.5);

        // PGS resilience → influences memory_retention (stress degrades memory)
        cognitive.memory_retention = blend(
            cognitive.memory_retention,
            0.5 + self.pgs_resilience * 0.5,
            pg_weight * 0.5,
        ).clamp(0.5, 1.5);
    }
}

/// Linear blend: (1-weight)*a + weight*b
fn blend(a: f64, b: f64, weight: f64) -> f64 {
    (1.0 - weight) * a + weight * b
}

// =============================================================================
// Deterministic generation
// =============================================================================

impl Genome {
    /// Generates a complete genome from a seed.
    /// The ChaCha8 PRNG guarantees reproducibility:
    /// same seed → same genome, regardless of platform.
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // SHA-256 hash of the seed for a readable identifier
        let mut hasher = Sha256::new();
        hasher.update(seed.to_le_bytes());
        let hash_bytes = hasher.finalize();
        let dna_hash = hex::encode(&hash_bytes[..16]); // 32 hex characters

        let mut temperament = TemperamentGenes::generate(&mut rng);
        let mut chemical = ChemicalGenes::generate(&mut rng);
        let physical = PhysicalGenes::generate(&mut rng);
        let mut vulnerabilities = VulnerabilityGenes::generate(&mut rng);
        let mut cognitive = CognitiveGenes::generate(&mut rng);

        // Polygenic scores — generated AFTER base traits
        let polygenic = PolygenicScores::generate(&mut rng);
        // PRS/PGS modulate existing traits (realistic correlations)
        polygenic.modulate_genes(&mut temperament, &mut chemical,
            &mut vulnerabilities, &mut cognitive);

        Self {
            seed,
            dna_hash,
            temperament,
            chemical,
            physical,
            vulnerabilities,
            cognitive,
            polygenic,
        }
    }

    /// Prints the genome in the boot logs.
    pub fn log_summary(&self) {
        println!("  --- Genome ---");
        println!("  DNA hash : {}", self.dna_hash);
        println!("  Temperament : I={:.2} N={:.2} C={:.2} A={:.2} O={:.2}",
            self.temperament.introversion, self.temperament.neuroticism,
            self.temperament.conscientiousness, self.temperament.agreeableness,
            self.temperament.openness);
        println!("  Chimie : dopa={:+.3} sero={:+.3} cort={:+.3} homeo={:.2}x resil={:.2}",
            self.chemical.baseline_dopamine_offset, self.chemical.baseline_serotonin_offset,
            self.chemical.baseline_cortisol_offset, self.chemical.homeostasis_speed,
            self.chemical.stress_resilience);
        println!("  Cognitif : learn={:.2}x mem={:.2}x creat={:.2}x analyt={:.2}",
            self.cognitive.learning_speed, self.cognitive.memory_retention,
            self.cognitive.creativity_factor, self.cognitive.analytical_bias);
        println!("  PRS (risques) : depr={:.2} anxi={:.2} addi={:.2}",
            self.polygenic.prs_depression, self.polygenic.prs_anxiety,
            self.polygenic.prs_addiction);
        println!("  PGS (traits) : empa={:.2} resi={:.2} learn={:.2} creat={:.2} metab={:.2}x",
            self.polygenic.pgs_empathy, self.polygenic.pgs_resilience,
            self.polygenic.pgs_learning, self.polygenic.pgs_creativity,
            self.polygenic.pharmacogenomic_metabolism);
    }

    /// Serializes the genome to JSON for the API.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl TemperamentGenes {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            introversion: rng.gen_range(0.0..=1.0),
            neuroticism: rng.gen_range(0.0..=1.0),
            conscientiousness: rng.gen_range(0.0..=1.0),
            agreeableness: rng.gen_range(0.0..=1.0),
            openness: rng.gen_range(0.0..=1.0),
        }
    }
}

impl ChemicalGenes {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            baseline_dopamine_offset: rng.gen_range(-0.1..=0.1),
            baseline_serotonin_offset: rng.gen_range(-0.1..=0.1),
            baseline_cortisol_offset: rng.gen_range(-0.1..=0.1),
            homeostasis_speed: rng.gen_range(0.5..=1.5),
            receptor_sensitivity: rng.gen_range(0.5..=1.5),
            stress_resilience: rng.gen_range(0.0..=1.0),
        }
    }
}

impl PhysicalGenes {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            eye_color_seed: rng.gen_range(0..=7),
            hair_type_seed: rng.gen_range(0..=5),
            skin_tone_seed: rng.gen_range(0..=5),
            height_offset: rng.gen_range(-20..=20),
            metabolism_speed: rng.gen_range(0.5..=1.5),
        }
    }
}

impl VulnerabilityGenes {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            addiction_susceptibility: rng.gen_range(0.0..=1.0),
            anxiety_predisposition: rng.gen_range(0.0..=1.0),
            depression_predisposition: rng.gen_range(0.0..=1.0),
            immune_baseline: rng.gen_range(0.5..=1.0),
            attachment_style_seed: rng.gen_range(0..=3),
        }
    }
}

impl CognitiveGenes {
    fn generate(rng: &mut ChaCha8Rng) -> Self {
        Self {
            learning_speed: rng.gen_range(0.5..=1.5),
            memory_retention: rng.gen_range(0.5..=1.5),
            creativity_factor: rng.gen_range(0.5..=1.5),
            analytical_bias: rng.gen_range(0.0..=1.0),
        }
    }
}

// =============================================================================
// Hex utility (avoids an extra dependency)
// =============================================================================

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_same_seed() {
        let g1 = Genome::from_seed(42);
        let g2 = Genome::from_seed(42);
        assert_eq!(g1.dna_hash, g2.dna_hash);
        assert_eq!(g1.temperament.introversion, g2.temperament.introversion);
        assert_eq!(g1.chemical.baseline_dopamine_offset, g2.chemical.baseline_dopamine_offset);
        assert_eq!(g1.physical.eye_color_seed, g2.physical.eye_color_seed);
        assert_eq!(g1.cognitive.learning_speed, g2.cognitive.learning_speed);
    }

    #[test]
    fn test_different_seeds() {
        let g1 = Genome::from_seed(42);
        let g2 = Genome::from_seed(1337);
        assert_ne!(g1.dna_hash, g2.dna_hash);
        // At least one trait should differ (statistically guaranteed)
        let same = g1.temperament.introversion == g2.temperament.introversion
            && g1.temperament.neuroticism == g2.temperament.neuroticism
            && g1.temperament.openness == g2.temperament.openness;
        assert!(!same, "Deux seeds differents doivent produire des genomes differents");
    }

    #[test]
    fn test_ranges() {
        let g = Genome::from_seed(999);
        // Temperament: 0.0 to 1.0
        assert!((0.0..=1.0).contains(&g.temperament.introversion));
        assert!((0.0..=1.0).contains(&g.temperament.openness));
        // Chemistry: offsets -0.1 to +0.1
        assert!((-0.1..=0.1).contains(&g.chemical.baseline_dopamine_offset));
        // Cognitive: 0.5 to 1.5
        assert!((0.5..=1.5).contains(&g.cognitive.learning_speed));
        // Vulnerabilities: 0.0 to 1.0
        assert!((0.0..=1.0).contains(&g.vulnerabilities.anxiety_predisposition));
    }

    #[test]
    fn test_dna_hash_format() {
        let g = Genome::from_seed(42);
        assert_eq!(g.dna_hash.len(), 32); // 16 bytes = 32 hex chars
        assert!(g.dna_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

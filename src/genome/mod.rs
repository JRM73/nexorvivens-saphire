// =============================================================================
// genome/mod.rs — Encodage ADN / generation deterministe
//
// Role : A partir d'un seed (u64), genere un genome unique et reproductible
//        qui encode les predispositions de l'agent : temperament, baselines
//        chimiques, traits physiques, vulnerabilites, aptitudes cognitives.
//        Deux seeds differents = deux individus fondamentalement differents.
//        Meme seed = meme individu (deterministe via ChaCha8 PRNG).
//
// Place dans l'architecture :
//   Appele au boot dans lifecycle/mod.rs, apres le chargement de la config.
//   Le genome est stocke dans SaphireAgent et expose via GET /api/genome.
// =============================================================================

use rand::SeedableRng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

// =============================================================================
// Structures du genome
// =============================================================================

/// Genome complet genere a partir d'un seed deterministe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub seed: u64,
    pub dna_hash: String,
    pub temperament: TemperamentGenes,
    pub chemical: ChemicalGenes,
    pub physical: PhysicalGenes,
    pub vulnerabilities: VulnerabilityGenes,
    pub cognitive: CognitiveGenes,
    /// Scores polygeniques (GWAS) — chaque trait majeur est influence
    /// par plusieurs loci (positions genomiques) comme en genetique reelle.
    /// Modele inspire des Polygenic Risk Scores (PRS) utilises en medecine.
    #[serde(default)]
    pub polygenic: PolygenicScores,
}

/// Genes de temperament — base de la personnalite.
/// Chaque trait varie de 0.0 a 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperamentGenes {
    /// 0.0 = extraverti, 1.0 = introverti
    pub introversion: f64,
    /// 0.0 = stable emotionnellement, 1.0 = tres reactif
    pub neuroticism: f64,
    /// 0.0 = flexible/spontane, 1.0 = rigoureux/methodique
    pub conscientiousness: f64,
    /// 0.0 = competitif, 1.0 = cooperatif
    pub agreeableness: f64,
    /// 0.0 = conservateur, 1.0 = curieux/aventurier
    pub openness: f64,
}

/// Genes chimiques — offsets sur les baselines neurochimiques.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalGenes {
    /// Deviation du baseline dopamine (-0.1 a +0.1)
    pub baseline_dopamine_offset: f64,
    /// Deviation du baseline serotonine (-0.1 a +0.1)
    pub baseline_serotonin_offset: f64,
    /// Deviation du baseline cortisol (-0.1 a +0.1)
    pub baseline_cortisol_offset: f64,
    /// Multiplicateur de vitesse d'homeostasie (0.5 a 1.5)
    pub homeostasis_speed: f64,
    /// Sensibilite globale des recepteurs (0.5 a 1.5)
    pub receptor_sensitivity: f64,
    /// Resilience au stress (0.0 a 1.0)
    pub stress_resilience: f64,
}

/// Genes physiques — apparence derivee du genome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalGenes {
    /// Indice de couleur des yeux (0-7)
    pub eye_color_seed: u8,
    /// Indice de type de cheveux (0-5)
    pub hair_type_seed: u8,
    /// Indice de teint de peau (0-5)
    pub skin_tone_seed: u8,
    /// Ecart de taille par rapport a la moyenne (-20 a +20 cm)
    pub height_offset: i8,
    /// Vitesse du metabolisme (0.5 a 1.5)
    pub metabolism_speed: f64,
}

/// Genes de vulnerabilite — predispositions (pas des maladies).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityGenes {
    /// Susceptibilite aux addictions (0.0 a 1.0)
    pub addiction_susceptibility: f64,
    /// Predisposition a l'anxiete (0.0 a 1.0)
    pub anxiety_predisposition: f64,
    /// Predisposition a la depression (0.0 a 1.0)
    pub depression_predisposition: f64,
    /// Robustesse du systeme immunitaire (0.5 a 1.0)
    pub immune_baseline: f64,
    /// Graine pour le style d'attachement (0-3 → Secure/Anxious/Avoidant/Disorganized)
    pub attachment_style_seed: u8,
}

/// Genes cognitifs — aptitudes d'apprentissage et de reflexion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveGenes {
    /// Vitesse d'apprentissage (0.5 a 1.5)
    pub learning_speed: f64,
    /// Retention memorielle (0.5 a 1.5)
    pub memory_retention: f64,
    /// Facteur de creativite (0.5 a 1.5)
    pub creativity_factor: f64,
    /// Biais analytique vs intuitif (0.0 = intuitif, 1.0 = analytique)
    pub analytical_bias: f64,
}

/// Scores polygeniques — modele GWAS (Genome-Wide Association Studies).
///
/// En genetique reelle, les traits complexes (intelligence, risque de depression,
/// temperament) ne dependent pas d'un seul gene mais de centaines de loci,
/// chacun avec un effet minuscule. Le score polygenique est la somme ponderee
/// de tous ces effets.
///
/// Ici on simule N_LOCI alleles par trait, chacun tire aleatoirement,
/// puis on calcule la moyenne ponderee. Le resultat suit naturellement
/// une distribution quasi-normale (theoreme central limite) meme si
/// chaque allele individuel est uniforme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygenicScores {
    // --- Scores de risque (PRS — Polygenic Risk Scores) ---

    /// Risque polygenique de depression (0.0-1.0, >0.7 = risque eleve)
    /// ~50 loci impliques (SLC6A4, FKBP5, BDNF, etc.)
    pub prs_depression: f64,
    /// Risque polygenique d'anxiete (0.0-1.0)
    /// Loci: CRHR1, SLC6A4, COMT, RGS2, etc.
    pub prs_anxiety: f64,
    /// Risque polygenique d'addiction (0.0-1.0)
    /// Loci: DRD2, OPRM1, ALDH2, CHRNA5, etc.
    pub prs_addiction: f64,

    // --- Scores de traits positifs ---

    /// Score polygenique d'empathie (0.0-1.0)
    /// Loci: OXTR, CD38, AVPR1A, etc.
    pub pgs_empathy: f64,
    /// Score polygenique de resilience au stress (0.0-1.0)
    /// Loci: NR3C1, FKBP5, CRHR1, NPY, etc.
    pub pgs_resilience: f64,
    /// Score polygenique de capacite d'apprentissage (0.0-1.0)
    /// Loci: BDNF, KIBRA, COMT, ARC, etc.
    pub pgs_learning: f64,
    /// Score polygenique de creativite (0.0-1.0)
    /// Loci: DRD4, COMT, DARPP-32, etc.
    pub pgs_creativity: f64,

    // --- Pharmacogenomique ---

    /// Efficacite du metabolisme des neurotransmetteurs (0.5-1.5)
    /// Loci: CYP2D6, CYP2C19, MAO-A, COMT
    /// <1.0 = metaboliseur lent, >1.0 = metaboliseur rapide
    pub pharmacogenomic_metabolism: f64,

    // --- Donnees brutes (pour transparence et API) ---

    /// Nombre de loci simules par trait
    pub loci_per_trait: usize,
    /// Heritabilite estimee (proportion de variance genetique)
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

/// Nombre de loci simules par trait polygenique.
/// En realite c'est des centaines-milliers, mais 30 suffit pour
/// obtenir une distribution quasi-normale (CLT) avec variabilite.
const N_LOCI: usize = 30;

/// Genere un score polygenique a partir de N alleles.
///
/// Chaque locus a un effet aleatoire (allele) et un poids
/// (effect size) tire d'une distribution exponentielle decroissante.
/// Les premiers loci ont plus d'influence (comme en GWAS reel,
/// ou quelques SNP ont un effet plus fort que les autres).
///
/// Le resultat est borne dans [min, max].
fn polygenic_score(rng: &mut ChaCha8Rng, min: f64, max: f64) -> f64 {
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for i in 0..N_LOCI {
        // Poids decroissant exponentiellement (effet des SNP dominants)
        let weight = (-0.05 * i as f64).exp();
        // Allele : 0 (homozygote ref), 1 (heterozygote), 2 (homozygote alt)
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

    /// Modifie les genes existants en fonction des scores polygeniques.
    /// Cela cree des correlations realistes entre les PRS et les traits
    /// (par ex. PRS depression eleve → neuroticism plus haut,
    /// PGS empathie eleve → agreeableness plus haut).
    pub fn modulate_genes(
        &self,
        temperament: &mut TemperamentGenes,
        chemical: &mut ChemicalGenes,
        vulnerabilities: &mut VulnerabilityGenes,
        cognitive: &mut CognitiveGenes,
    ) {
        // Influence polygenique sur le temperament (poids = 30%, car ~50% heritabilite)
        let pg_weight = 0.3;

        // PRS depression → augmente neuroticism
        temperament.neuroticism = blend(
            temperament.neuroticism,
            self.prs_depression,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PGS empathie → augmente agreeableness
        temperament.agreeableness = blend(
            temperament.agreeableness,
            self.pgs_empathy,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PGS creativite → augmente openness
        temperament.openness = blend(
            temperament.openness,
            self.pgs_creativity,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PRS anxiete → influence cortisol baseline
        chemical.baseline_cortisol_offset += (self.prs_anxiety - 0.5) * 0.05;
        chemical.baseline_cortisol_offset = chemical.baseline_cortisol_offset.clamp(-0.15, 0.15);

        // PGS resilience → influence stress_resilience
        chemical.stress_resilience = blend(
            chemical.stress_resilience,
            self.pgs_resilience,
            pg_weight,
        ).clamp(0.0, 1.0);

        // Pharmacogenomique → influence homeostasis_speed
        chemical.homeostasis_speed *= 0.7 + self.pharmacogenomic_metabolism * 0.3;
        chemical.homeostasis_speed = chemical.homeostasis_speed.clamp(0.3, 2.0);

        // PRS addiction → influence vulnerability
        vulnerabilities.addiction_susceptibility = blend(
            vulnerabilities.addiction_susceptibility,
            self.prs_addiction,
            pg_weight,
        ).clamp(0.0, 1.0);

        // PRS depression/anxiete → influence predispositions
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

        // PGS resilience → influence memory_retention (stress degrade la memoire)
        cognitive.memory_retention = blend(
            cognitive.memory_retention,
            0.5 + self.pgs_resilience * 0.5,
            pg_weight * 0.5,
        ).clamp(0.5, 1.5);
    }
}

/// Melange lineaire : (1-weight)*a + weight*b
fn blend(a: f64, b: f64, weight: f64) -> f64 {
    (1.0 - weight) * a + weight * b
}

// =============================================================================
// Generation deterministe
// =============================================================================

impl Genome {
    /// Genere un genome complet a partir d'un seed.
    /// Le PRNG ChaCha8 garantit la reproductibilite :
    /// meme seed → meme genome, quelle que soit la plateforme.
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // Hash SHA-256 du seed pour un identifiant lisible
        let mut hasher = Sha256::new();
        hasher.update(seed.to_le_bytes());
        let hash_bytes = hasher.finalize();
        let dna_hash = hex::encode(&hash_bytes[..16]); // 32 caracteres hex

        let mut temperament = TemperamentGenes::generate(&mut rng);
        let mut chemical = ChemicalGenes::generate(&mut rng);
        let physical = PhysicalGenes::generate(&mut rng);
        let mut vulnerabilities = VulnerabilityGenes::generate(&mut rng);
        let mut cognitive = CognitiveGenes::generate(&mut rng);

        // Scores polygeniques — generes APRES les traits de base
        let polygenic = PolygenicScores::generate(&mut rng);
        // Les PRS/PGS modulent les traits existants (correlations realistes)
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

    /// Affiche le genome dans les logs du demarrage.
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

    /// Serialise le genome en JSON pour l'API.
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
// Utilitaire hex (evite une dependance supplementaire)
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
        // Au moins un trait devrait differer (statistiquement garanti)
        let same = g1.temperament.introversion == g2.temperament.introversion
            && g1.temperament.neuroticism == g2.temperament.neuroticism
            && g1.temperament.openness == g2.temperament.openness;
        assert!(!same, "Deux seeds differents doivent produire des genomes differents");
    }

    #[test]
    fn test_ranges() {
        let g = Genome::from_seed(999);
        // Temperament : 0.0 a 1.0
        assert!((0.0..=1.0).contains(&g.temperament.introversion));
        assert!((0.0..=1.0).contains(&g.temperament.openness));
        // Chimie : offsets -0.1 a +0.1
        assert!((-0.1..=0.1).contains(&g.chemical.baseline_dopamine_offset));
        // Cognitif : 0.5 a 1.5
        assert!((0.5..=1.5).contains(&g.cognitive.learning_speed));
        // Vulnerabilites : 0.0 a 1.0
        assert!((0.0..=1.0).contains(&g.vulnerabilities.anxiety_predisposition));
    }

    #[test]
    fn test_dna_hash_format() {
        let g = Genome::from_seed(42);
        assert_eq!(g.dna_hash.len(), 32); // 16 bytes = 32 hex chars
        assert!(g.dna_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

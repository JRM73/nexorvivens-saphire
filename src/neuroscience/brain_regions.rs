// =============================================================================
// brain_regions.rs — Reseau de regions cerebrales (remplacement du modele triunique)
// =============================================================================
//
// Role : Modelise 12 regions cerebrales fonctionnelles interconnectees,
// chacune avec sa propre sensibilite aux neurotransmetteurs, son niveau
// d'activation, et ses connexions ponderees vers les autres regions.
//
// Ce modele remplace le simpliste "3 cerveaux" (MacLean) par un reseau
// distribue base sur les neurosciences modernes.
//
// References scientifiques :
//   - Global Workspace Theory (Baars 1988, Dehaene 2014)
//   - Connectome humain : Human Connectome Project (2013)
//   - Regions fonctionnelles : Brodmann areas + fMRI modernes
// =============================================================================

use serde::{Deserialize, Serialize};

/// Index des 12 regions cerebrales.
/// Chaque constante correspond a un index dans les tableaux de BrainNetwork.
pub const AMYGDALA: usize = 0;       // Peur, saillance emotionnelle
pub const HIPPOCAMPUS: usize = 1;    // Memoire episodique, navigation spatiale
pub const PFC_DORSO: usize = 2;      // Cortex prefrontal dorsolateral : planification, WM
pub const PFC_VENTRO: usize = 3;     // Cortex prefrontal ventromedian : valeur, decision
pub const INSULA: usize = 4;         // Interoception, degout, conscience corporelle
pub const ACC: usize = 5;            // Cortex cingulaire anterieur : conflit, erreur
pub const BASAL_GANGLIA: usize = 6;  // Noyaux de la base : habitudes, reward
pub const BRAINSTEM: usize = 7;      // Tronc cerebral : arousal, veille/sommeil
pub const OFC: usize = 8;            // Cortex orbitofrontal : valeur subjective
pub const TEMPORAL: usize = 9;       // Cortex temporal : langage, semantique
pub const PARIETAL: usize = 10;      // Cortex parietal : attention spatiale, integration
pub const CEREBELLUM: usize = 11;    // Cervelet : timing, coordination, prediction

/// Nombre total de regions cerebrales.
pub const NUM_REGIONS: usize = 12;

/// Noms des regions (pour l'affichage).
pub const REGION_NAMES: [&str; NUM_REGIONS] = [
    "Amygdale", "Hippocampe", "CPF-Dorso", "CPF-Ventro",
    "Insula", "CCA", "Noyaux-Base", "Tronc",
    "COF", "Temporal", "Parietal", "Cervelet",
];

/// Region cerebrale individuelle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegion {
    /// Nom de la region
    pub name: String,
    /// Activation courante [0.0, 1.0]
    pub activation: f64,
    /// Activation au cycle precedent (pour calcul de delta)
    pub prev_activation: f64,
    /// Sensibilite a chaque molecule [9 valeurs : dopa, cort, sero, adre, ocyt, endo, nora, gaba, glut]
    /// Positif = excite par cette molecule, negatif = inhibe
    pub nt_sensitivity: [f64; 9],
    /// Seuil d'activation minimum pour participer au Global Workspace
    pub broadcast_threshold: f64,
}

/// Reseau complet des 12 regions cerebrales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainNetwork {
    /// Les 12 regions
    pub regions: Vec<BrainRegion>,
    /// Matrice de connexion [12x12] : weights[i][j] = influence de region i sur region j
    /// Positif = excitateur, negatif = inhibiteur
    pub weights: [[f64; NUM_REGIONS]; NUM_REGIONS],
    /// Contenu du Global Workspace : index de la region dominante
    pub workspace_winner: Option<usize>,
    /// Score de broadcast (force du signal gagnant)
    pub workspace_strength: f64,
    /// Historique des gagnants (pour mesurer la variete)
    pub workspace_history: Vec<usize>,
    /// Surprise globale : ecart entre prediction et realite
    pub global_surprise: f64,
}

impl Default for BrainNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainNetwork {
    /// Cree un reseau cerebral avec les connexions anatomiques par defaut.
    pub fn new() -> Self {
        let regions = vec![
            // Amygdale : sensible au cortisol et adrenaline (peur), inhibee par GABA
            BrainRegion {
                name: "Amygdale".into(), activation: 0.2, prev_activation: 0.2,
                nt_sensitivity: [0.1, 0.6, -0.3, 0.5, -0.2, -0.3, 0.3, -0.5, 0.2],
                broadcast_threshold: 0.4,
            },
            // Hippocampe : sensible a la serotonine (consolidation), cortisol la degrade
            BrainRegion {
                name: "Hippocampe".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.2, -0.4, 0.4, -0.1, 0.2, 0.1, 0.3, -0.1, 0.3],
                broadcast_threshold: 0.3,
            },
            // CPF dorsolateral : focus (noradrenaline), degrade par cortisol
            BrainRegion {
                name: "CPF-Dorso".into(), activation: 0.4, prev_activation: 0.4,
                nt_sensitivity: [0.2, -0.5, 0.3, -0.2, 0.0, 0.1, 0.6, -0.1, 0.2],
                broadcast_threshold: 0.35,
            },
            // CPF ventromedian : valeur, decision, sensible a la dopamine
            BrainRegion {
                name: "CPF-Ventro".into(), activation: 0.35, prev_activation: 0.35,
                nt_sensitivity: [0.5, -0.3, 0.3, -0.1, 0.2, 0.2, 0.2, -0.1, 0.1],
                broadcast_threshold: 0.35,
            },
            // Insula : interoception, sensible au cortisol et endorphines
            BrainRegion {
                name: "Insula".into(), activation: 0.25, prev_activation: 0.25,
                nt_sensitivity: [0.1, 0.4, -0.1, 0.3, 0.1, -0.4, 0.2, -0.2, 0.2],
                broadcast_threshold: 0.4,
            },
            // CCA : conflit/erreur, sensible a la noradrenaline et dopamine
            BrainRegion {
                name: "CCA".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.3, 0.3, -0.1, 0.2, 0.0, -0.1, 0.5, -0.2, 0.3],
                broadcast_threshold: 0.35,
            },
            // Noyaux de la base : habitudes/reward, tres sensible a la dopamine
            BrainRegion {
                name: "Noyaux-Base".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.8, -0.2, 0.1, 0.1, 0.0, 0.2, 0.1, -0.3, 0.1],
                broadcast_threshold: 0.3,
            },
            // Tronc cerebral : arousal, sensible a l'adrenaline et noradrenaline
            BrainRegion {
                name: "Tronc".into(), activation: 0.4, prev_activation: 0.4,
                nt_sensitivity: [0.1, 0.2, -0.2, 0.6, 0.0, -0.1, 0.5, -0.4, 0.3],
                broadcast_threshold: 0.3,
            },
            // COF : valeur subjective, sensible a dopamine et serotonine
            BrainRegion {
                name: "COF".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.5, -0.2, 0.4, -0.1, 0.3, 0.2, 0.1, -0.1, 0.1],
                broadcast_threshold: 0.35,
            },
            // Temporal : langage, semantique, noradrenaline et glutamate
            BrainRegion {
                name: "Temporal".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.2, -0.2, 0.2, 0.0, 0.1, 0.1, 0.3, -0.1, 0.4],
                broadcast_threshold: 0.3,
            },
            // Parietal : attention spatiale, integration multi-sensorielle
            BrainRegion {
                name: "Parietal".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.1, -0.1, 0.1, 0.2, 0.0, 0.1, 0.4, -0.2, 0.3],
                broadcast_threshold: 0.3,
            },
            // Cervelet : timing, prediction, apprentissage moteur
            BrainRegion {
                name: "Cervelet".into(), activation: 0.3, prev_activation: 0.3,
                nt_sensitivity: [0.1, -0.1, 0.2, 0.1, 0.0, 0.1, 0.2, -0.3, 0.4],
                broadcast_threshold: 0.25,
            },
        ];

        // Matrice de connexion anatomique (simplifiee des donnees du HCP)
        let mut weights = [[0.0f64; NUM_REGIONS]; NUM_REGIONS];

        // Amygdale → CPF (regulation top-down), Hippocampe (memoire emotionnelle)
        weights[AMYGDALA][PFC_VENTRO] = 0.4;
        weights[AMYGDALA][HIPPOCAMPUS] = 0.5;
        weights[AMYGDALA][BRAINSTEM] = 0.6;  // reponse fight-or-flight
        weights[AMYGDALA][INSULA] = 0.3;     // signaux corporels

        // Hippocampe ↔ CPF (consolidation memoire)
        weights[HIPPOCAMPUS][PFC_DORSO] = 0.4;
        weights[HIPPOCAMPUS][PFC_VENTRO] = 0.3;
        weights[HIPPOCAMPUS][TEMPORAL] = 0.5;  // memoire semantique

        // CPF dorsolateral → inhibition amygdale (regulation emotionnelle)
        weights[PFC_DORSO][AMYGDALA] = -0.3;
        weights[PFC_DORSO][ACC] = 0.4;
        weights[PFC_DORSO][BASAL_GANGLIA] = 0.3;
        weights[PFC_DORSO][PARIETAL] = 0.3;

        // CPF ventromedian → valeur, decision
        weights[PFC_VENTRO][OFC] = 0.5;
        weights[PFC_VENTRO][AMYGDALA] = -0.2;  // regulation emotionnelle
        weights[PFC_VENTRO][BASAL_GANGLIA] = 0.4;

        // CCA → detection de conflit
        weights[ACC][PFC_DORSO] = 0.5;   // recrute le CPF en cas de conflit
        weights[ACC][AMYGDALA] = 0.2;    // alerte emotionnelle
        weights[ACC][BRAINSTEM] = 0.3;   // arousal

        // Noyaux de la base → boucle cortico-basale
        weights[BASAL_GANGLIA][PFC_DORSO] = 0.3;
        weights[BASAL_GANGLIA][PFC_VENTRO] = 0.3;
        weights[BASAL_GANGLIA][CEREBELLUM] = 0.2;

        // Tronc cerebral → arousal global
        weights[BRAINSTEM][AMYGDALA] = 0.2;
        weights[BRAINSTEM][PFC_DORSO] = 0.3;
        weights[BRAINSTEM][PARIETAL] = 0.2;

        // COF → valeur
        weights[OFC][AMYGDALA] = 0.3;
        weights[OFC][BASAL_GANGLIA] = 0.4;

        // Insula → conscience corporelle
        weights[INSULA][ACC] = 0.4;
        weights[INSULA][AMYGDALA] = 0.3;

        // Cervelet → prediction
        weights[CEREBELLUM][PFC_DORSO] = 0.2;
        weights[CEREBELLUM][TEMPORAL] = 0.2;

        Self {
            regions,
            weights,
            workspace_winner: None,
            workspace_strength: 0.0,
            workspace_history: Vec::new(),
            global_surprise: 0.0,
        }
    }

    /// Met a jour les activations des regions en fonction de la chimie et des sens.
    /// 1. Chaque region recoit l'influence chimique via sa sensibilite NT
    /// 1b. Les sens actifs boostent directement leurs regions anatomiques
    /// 2. Les connexions inter-regions propagent l'activation
    /// 3. Le Global Workspace selectionne le signal dominant
    ///
    /// sensory_input : [lecture, ecoute, contact, saveur, ambiance] (intensites 0-1)
    pub fn tick(&mut self, chemistry: &crate::neurochemistry::NeuroChemicalState, sensory_input: [f64; 5]) {
        let chem9 = chemistry.to_vec9();

        // Phase 1 : Activation chimique directe
        let mut new_activations = [0.0f64; NUM_REGIONS];
        for (i, region) in self.regions.iter().enumerate() {
            let mut chem_input = 0.0;
            for (j, &conc) in chem9.iter().enumerate() {
                chem_input += conc * region.nt_sensitivity[j];
            }
            new_activations[i] = region.activation * 0.7 + chem_input * 0.3;
        }

        // Phase 1b : Boost sensoriel direct (anatomiquement correct)
        // Coefficient modere (0.08) pour colorer l'activite sans la dominer.
        // Chaque sens stimule ses regions anatomiques naturelles :
        //   Lecture (vue)    → Temporal (stream ventral) + Parietal (stream dorsal)
        //   Ecoute (ouie)    → Temporal (cortex auditif)
        //   Contact (toucher) → Insula (interoception) + Parietal (somatosensoriel)
        //   Saveur (gout)    → Insula (cortex gustatif) + COF
        //   Ambiance (odorat) → COF (cortex olfactif) + Amygdale (valence emotionnelle)
        const SENS_COEFF: f64 = 0.08;
        let [reading, listening, contact, taste, ambiance] = sensory_input;

        new_activations[TEMPORAL]  += reading * SENS_COEFF * 0.7;
        new_activations[PARIETAL]  += reading * SENS_COEFF * 0.3;

        new_activations[TEMPORAL]  += listening * SENS_COEFF;

        new_activations[INSULA]    += contact * SENS_COEFF * 0.6;
        new_activations[PARIETAL]  += contact * SENS_COEFF * 0.4;

        new_activations[INSULA]    += taste * SENS_COEFF * 0.5;
        new_activations[OFC]       += taste * SENS_COEFF * 0.5;

        new_activations[OFC]       += ambiance * SENS_COEFF * 0.6;
        new_activations[AMYGDALA]  += ambiance * SENS_COEFF * 0.4;

        // Phase 2 : Propagation inter-regions
        let mut propagated = new_activations;
        for i in 0..NUM_REGIONS {
            let mut input = 0.0;
            for j in 0..NUM_REGIONS {
                if i != j {
                    input += new_activations[j] * self.weights[j][i];
                }
            }
            propagated[i] += input * 0.15; // facteur d'influence inter-regionale
        }

        // Phase 3 : Normalisation et mise a jour
        for (i, region) in self.regions.iter_mut().enumerate() {
            region.prev_activation = region.activation;
            region.activation = propagated[i].clamp(0.0, 1.0);
        }

        // Phase 4 : Global Workspace — competition pour le broadcast
        self.compute_global_workspace();
    }

    /// Global Workspace Theory : la region la plus active au-dessus de son
    /// seuil de broadcast gagne la competition et diffuse son signal.
    pub fn compute_global_workspace(&mut self) {
        let mut best_idx = None;
        let mut best_score = 0.0;

        for (i, region) in self.regions.iter().enumerate() {
            if region.activation > region.broadcast_threshold {
                let score = region.activation;
                if score > best_score {
                    best_score = score;
                    best_idx = Some(i);
                }
            }
        }

        // Calculer la surprise : ecart par rapport a la prediction
        if let Some(prev_winner) = self.workspace_winner {
            if let Some(current_winner) = best_idx {
                self.global_surprise = if prev_winner != current_winner { 0.8 } else { 0.1 };
            } else {
                self.global_surprise = 0.5; // pas de gagnant = moderement surprenant
            }
        }

        self.workspace_winner = best_idx;
        self.workspace_strength = best_score;

        // Historique (garder les 50 derniers)
        if let Some(idx) = best_idx {
            self.workspace_history.push(idx);
            if self.workspace_history.len() > 50 {
                self.workspace_history.remove(0);
            }
        }
    }

    /// Retourne le nom de la region dominant le Global Workspace.
    pub fn workspace_region_name(&self) -> &str {
        match self.workspace_winner {
            Some(idx) if idx < NUM_REGIONS => REGION_NAMES[idx],
            _ => "Aucune",
        }
    }

    /// Calcule la diversite du Global Workspace sur les derniers cycles.
    /// 0.0 = toujours la meme region, 1.0 = toutes les regions representees.
    pub fn workspace_diversity(&self) -> f64 {
        if self.workspace_history.is_empty() { return 0.0; }
        let unique: std::collections::HashSet<&usize> = self.workspace_history.iter().collect();
        unique.len() as f64 / NUM_REGIONS as f64
    }

    /// Mapping vers l'ancien modele triunique pour compatibilite.
    /// Reptilien = moyenne(Amygdale, Tronc cerebral)
    /// Limbique = moyenne(Hippocampe, Noyaux de la base, Insula)
    /// Neocortex = moyenne(CPF-Dorso, CPF-Ventro, CCA, COF, Temporal, Parietal)
    pub fn triune_compat(&self) -> [f64; 3] {
        let r = &self.regions;
        let reptilian = (r[AMYGDALA].activation + r[BRAINSTEM].activation) / 2.0;
        let limbic = (r[HIPPOCAMPUS].activation + r[BASAL_GANGLIA].activation + r[INSULA].activation) / 3.0;
        let neocortex = (r[PFC_DORSO].activation + r[PFC_VENTRO].activation
            + r[ACC].activation + r[OFC].activation
            + r[TEMPORAL].activation + r[PARIETAL].activation) / 6.0;
        [reptilian, limbic, neocortex]
    }

    /// Resume pour le dashboard.
    pub fn summary(&self) -> BrainNetworkSummary {
        BrainNetworkSummary {
            activations: self.regions.iter().map(|r| (r.name.clone(), r.activation)).collect(),
            workspace_winner: self.workspace_region_name().to_string(),
            workspace_strength: self.workspace_strength,
            workspace_diversity: self.workspace_diversity(),
            global_surprise: self.global_surprise,
            triune_compat: self.triune_compat(),
        }
    }

    /// Serialise l'etat pour persistance.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "activations": self.regions.iter().map(|r| r.activation).collect::<Vec<_>>(),
            "workspace_winner": self.workspace_winner,
            "workspace_strength": self.workspace_strength,
            "global_surprise": self.global_surprise,
        })
    }

    /// Restaure les activations depuis un JSON persiste.
    pub fn restore_from_json(&mut self, json: &serde_json::Value) {
        if let Some(activations) = json.get("activations").and_then(|v| v.as_array()) {
            for (i, val) in activations.iter().enumerate().take(NUM_REGIONS) {
                if let Some(a) = val.as_f64() {
                    if i < self.regions.len() {
                        self.regions[i].activation = a.clamp(0.0, 1.0);
                        self.regions[i].prev_activation = a.clamp(0.0, 1.0);
                    }
                }
            }
        }
    }
}

/// Resume du reseau cerebral pour le dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainNetworkSummary {
    pub activations: Vec<(String, f64)>,
    pub workspace_winner: String,
    pub workspace_strength: f64,
    pub workspace_diversity: f64,
    pub global_surprise: f64,
    /// Compatibilite triunique [reptilien, limbique, neocortex]
    pub triune_compat: [f64; 3],
}

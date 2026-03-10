// =============================================================================
// nutrition.rs — Biochimie nutritionnelle de Saphire
//
// Role : Simule le substrat nutritionnel de l'etre : vitamines, acides amines,
// proteines, metabolisme energetique (ATP, glycogene, calories). Les carences
// influencent directement la neurochimie (tryptophane → serotonine, etc.).
//
// Place dans l'architecture :
//   Pipeline cognitif etape 3o : tick + chemistry_influence apres culture.
//   Interactions croisees : UV solaire → vitamine D, repas → restore nutrients,
//   tryptophane → BDNF (grey_matter).
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::config::NutritionConfig;
use crate::world::ChemistryAdjustment;

// ─── Niveaux de vitamines ───────────────────────────────────────────────────

/// Niveaux des vitamines essentielles (0.0 = carence totale, 1.0 = optimal).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitaminLevels {
    /// Complexe B (B1, B6, B9, B12) — synthese des neurotransmetteurs
    pub b_complex: f64,
    /// Vitamine C — antioxydant, synthese noradrenaline
    pub c: f64,
    /// Vitamine D — synthese serotonine, immuno-modulation (synthetisee par UV)
    pub d: f64,
    /// Vitamine E — protection membranaire, antioxydant lipophile
    pub e: f64,
    /// Vitamine A — vision, transcription genique
    pub a: f64,
    /// Vitamine K — coagulation, sante osseuse
    pub k: f64,
}

impl Default for VitaminLevels {
    fn default() -> Self {
        Self {
            b_complex: 0.7,
            c: 0.7,
            d: 0.7,
            e: 0.7,
            a: 0.7,
            k: 0.7,
        }
    }
}

// ─── Niveaux d'acides amines ────────────────────────────────────────────────

/// Acides amines essentiels pour la neurochimie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AminoAcidLevels {
    /// Tryptophane — precurseur de la serotonine et melatonine
    pub tryptophan: f64,
    /// Tyrosine — precurseur de la dopamine et noradrenaline
    pub tyrosine: f64,
    /// Glutamine — precurseur du GABA et glutamate
    pub glutamine: f64,
    /// Histidine — precurseur de l'histamine (eveil, attention)
    pub histidine: f64,
    /// Glycine — co-agoniste NMDA, sommeil, inhibition
    pub glycine: f64,
}

impl Default for AminoAcidLevels {
    fn default() -> Self {
        Self {
            tryptophan: 0.6,
            tyrosine: 0.6,
            glutamine: 0.6,
            histidine: 0.6,
            glycine: 0.6,
        }
    }
}

// ─── Metabolisme energetique ────────────────────────────────────────────────

/// Metabolisme energetique cellulaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyMetabolism {
    /// Metabolisme basal (taux de consommation au repos)
    pub bmr: f64,
    /// Reserves d'ATP (0.0 = epuise, 1.0 = plein)
    pub atp_reserves: f64,
    /// Reserves de glycogene (hepatique + musculaire)
    pub glycogen_reserves: f64,
    /// Balance calorique (>0 = surplus, <0 = deficit)
    pub caloric_balance: f64,
}

impl Default for EnergyMetabolism {
    fn default() -> Self {
        Self {
            bmr: 0.003,
            atp_reserves: 0.8,
            glycogen_reserves: 0.7,
            caloric_balance: 0.0,
        }
    }
}

// ─── Systeme nutritionnel complet ───────────────────────────────────────────

/// Systeme nutritionnel complet : vitamines + acides amines + proteines + energie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionSystem {
    pub enabled: bool,
    pub vitamins: VitaminLevels,
    pub amino_acids: AminoAcidLevels,
    /// Niveau de proteines disponibles (synthese, reparation)
    pub protein_level: f64,
    pub energy: EnergyMetabolism,
}

impl NutritionSystem {
    /// Cree un nouveau systeme nutritionnel depuis la config.
    pub fn new(config: &NutritionConfig) -> Self {
        Self {
            enabled: config.enabled,
            vitamins: VitaminLevels::default(),
            amino_acids: AminoAcidLevels::default(),
            protein_level: 0.7,
            energy: EnergyMetabolism::default(),
        }
    }

    /// Tick metabolique : degradation naturelle, consommation energetique,
    /// conversion glycogene → ATP quand reserves basses.
    /// `uv_index` provient du module fields (champs EM solaires).
    pub fn tick(&mut self, config: &NutritionConfig, is_eating: bool, uv_index: f64) {
        if !self.enabled { return; }

        // Degradation naturelle des vitamines (metabolisme)
        let vd = config.vitamin_decay_rate;
        self.vitamins.b_complex = (self.vitamins.b_complex - vd).max(0.0);
        self.vitamins.c = (self.vitamins.c - vd * 1.2).max(0.0); // C se degrade plus vite
        self.vitamins.d = (self.vitamins.d - vd * 0.8).max(0.0); // D plus stable
        self.vitamins.e = (self.vitamins.e - vd * 0.6).max(0.0);
        self.vitamins.a = (self.vitamins.a - vd * 0.5).max(0.0);
        self.vitamins.k = (self.vitamins.k - vd * 0.4).max(0.0);

        // Synthese de vitamine D par exposition UV (interaction champs solaires)
        if uv_index > 0.2 {
            let synth = (uv_index - 0.2) * config.uv_vitamin_d_factor;
            self.vitamins.d = (self.vitamins.d + synth).min(1.0);
        }

        // Degradation des acides amines (utilisation metabolique)
        let ad = config.amino_decay_rate;
        self.amino_acids.tryptophan = (self.amino_acids.tryptophan - ad).max(0.0);
        self.amino_acids.tyrosine = (self.amino_acids.tyrosine - ad).max(0.0);
        self.amino_acids.glutamine = (self.amino_acids.glutamine - ad * 0.8).max(0.0);
        self.amino_acids.histidine = (self.amino_acids.histidine - ad * 0.7).max(0.0);
        self.amino_acids.glycine = (self.amino_acids.glycine - ad * 0.6).max(0.0);

        // Degradation des proteines
        self.protein_level = (self.protein_level - config.protein_decay_rate).max(0.0);

        // Consommation energetique (ATP)
        self.energy.atp_reserves = (self.energy.atp_reserves - config.atp_consumption_rate).max(0.0);

        // Conversion glycogene → ATP quand reserves basses
        if self.energy.atp_reserves < 0.3 && self.energy.glycogen_reserves > 0.1 {
            let conversion = config.glycogen_to_atp_rate.min(self.energy.glycogen_reserves);
            self.energy.glycogen_reserves -= conversion;
            self.energy.atp_reserves = (self.energy.atp_reserves + conversion * 0.8).min(1.0);
        }

        // Balance calorique : deficit si ATP bas, surplus si haut
        self.energy.caloric_balance = self.energy.atp_reserves - 0.5;

        // Si on mange, restaurer les nutrients
        if is_eating {
            self.restore_from_meal(config.meal_nutrient_boost);
        }
    }

    /// Calcule l'influence de la nutrition sur la neurochimie.
    /// Les carences en precurseurs impactent directement les neurotransmetteurs.
    pub fn chemistry_influence(&self, config: &NutritionConfig) -> ChemistryAdjustment {
        let mut adj = ChemistryAdjustment::default();
        if !self.enabled { return adj; }

        // Tryptophane bas → serotonine diminuee
        if self.amino_acids.tryptophan < config.amino_deficiency_threshold {
            let deficit = config.amino_deficiency_threshold - self.amino_acids.tryptophan;
            adj.serotonin -= deficit * 0.04;
        }

        // Tyrosine bas → dopamine et noradrenaline diminuees
        if self.amino_acids.tyrosine < config.amino_deficiency_threshold {
            let deficit = config.amino_deficiency_threshold - self.amino_acids.tyrosine;
            adj.dopamine -= deficit * 0.03;
            adj.noradrenaline -= deficit * 0.02;
        }

        // Vitamine D basse → serotonine diminuee
        if self.vitamins.d < config.vitamin_deficiency_threshold {
            let deficit = config.vitamin_deficiency_threshold - self.vitamins.d;
            adj.serotonin -= deficit * 0.03;
        }

        // Complexe B bas → neurotransmetteurs globaux diminues
        if self.vitamins.b_complex < config.vitamin_deficiency_threshold {
            let deficit = config.vitamin_deficiency_threshold - self.vitamins.b_complex;
            adj.serotonin -= deficit * 0.02;
            adj.dopamine -= deficit * 0.02;
            adj.noradrenaline -= deficit * 0.01;
        }

        // ATP bas → cortisol augmente (stress metabolique)
        if self.energy.atp_reserves < 0.3 {
            let deficit = 0.3 - self.energy.atp_reserves;
            adj.cortisol += deficit * 0.05;
        }

        adj
    }

    /// Restaure les niveaux nutritionnels suite a un repas.
    pub fn restore_from_meal(&mut self, boost: f64) {
        self.vitamins.b_complex = (self.vitamins.b_complex + boost).min(1.0);
        self.vitamins.c = (self.vitamins.c + boost).min(1.0);
        self.vitamins.e = (self.vitamins.e + boost * 0.8).min(1.0);
        self.vitamins.a = (self.vitamins.a + boost * 0.7).min(1.0);
        self.vitamins.k = (self.vitamins.k + boost * 0.6).min(1.0);
        // Pas de boost vitD via repas (surtout UV)
        self.vitamins.d = (self.vitamins.d + boost * 0.3).min(1.0);

        self.amino_acids.tryptophan = (self.amino_acids.tryptophan + boost).min(1.0);
        self.amino_acids.tyrosine = (self.amino_acids.tyrosine + boost).min(1.0);
        self.amino_acids.glutamine = (self.amino_acids.glutamine + boost * 0.9).min(1.0);
        self.amino_acids.histidine = (self.amino_acids.histidine + boost * 0.8).min(1.0);
        self.amino_acids.glycine = (self.amino_acids.glycine + boost * 0.7).min(1.0);

        self.protein_level = (self.protein_level + boost).min(1.0);
        self.energy.atp_reserves = (self.energy.atp_reserves + boost * 1.5).min(1.0);
        self.energy.glycogen_reserves = (self.energy.glycogen_reserves + boost).min(1.0);
    }

    /// Retourne la liste des carences sous les seuils.
    pub fn deficiencies(&self, config: &NutritionConfig) -> Vec<(String, f64)> {
        let mut defs = Vec::new();
        let vt = config.vitamin_deficiency_threshold;
        let at = config.amino_deficiency_threshold;

        if self.vitamins.b_complex < vt { defs.push(("vitamin_b".into(), self.vitamins.b_complex)); }
        if self.vitamins.c < vt { defs.push(("vitamin_c".into(), self.vitamins.c)); }
        if self.vitamins.d < vt { defs.push(("vitamin_d".into(), self.vitamins.d)); }
        if self.vitamins.e < vt { defs.push(("vitamin_e".into(), self.vitamins.e)); }
        if self.vitamins.a < vt { defs.push(("vitamin_a".into(), self.vitamins.a)); }
        if self.vitamins.k < vt { defs.push(("vitamin_k".into(), self.vitamins.k)); }

        if self.amino_acids.tryptophan < at { defs.push(("tryptophan".into(), self.amino_acids.tryptophan)); }
        if self.amino_acids.tyrosine < at { defs.push(("tyrosine".into(), self.amino_acids.tyrosine)); }
        if self.amino_acids.glutamine < at { defs.push(("glutamine".into(), self.amino_acids.glutamine)); }
        if self.amino_acids.histidine < at { defs.push(("histidine".into(), self.amino_acids.histidine)); }
        if self.amino_acids.glycine < at { defs.push(("glycine".into(), self.amino_acids.glycine)); }

        if self.protein_level < 0.3 { defs.push(("protein".into(), self.protein_level)); }
        if self.energy.atp_reserves < 0.2 { defs.push(("atp".into(), self.energy.atp_reserves)); }

        defs
    }

    /// Serialise l'etat en JSON pour persistance.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "vitamins": {
                "b_complex": self.vitamins.b_complex,
                "c": self.vitamins.c,
                "d": self.vitamins.d,
                "e": self.vitamins.e,
                "a": self.vitamins.a,
                "k": self.vitamins.k,
            },
            "amino_acids": {
                "tryptophan": self.amino_acids.tryptophan,
                "tyrosine": self.amino_acids.tyrosine,
                "glutamine": self.amino_acids.glutamine,
                "histidine": self.amino_acids.histidine,
                "glycine": self.amino_acids.glycine,
            },
            "protein_level": self.protein_level,
            "energy": {
                "bmr": self.energy.bmr,
                "atp_reserves": self.energy.atp_reserves,
                "glycogen_reserves": self.energy.glycogen_reserves,
                "caloric_balance": self.energy.caloric_balance,
            },
        })
    }

    /// Restaure l'etat depuis le JSON persiste.
    pub fn restore_from_json(&mut self, v: &serde_json::Value) {
        if let Some(vit) = v.get("vitamins") {
            self.vitamins.b_complex = vit["b_complex"].as_f64().unwrap_or(self.vitamins.b_complex);
            self.vitamins.c = vit["c"].as_f64().unwrap_or(self.vitamins.c);
            self.vitamins.d = vit["d"].as_f64().unwrap_or(self.vitamins.d);
            self.vitamins.e = vit["e"].as_f64().unwrap_or(self.vitamins.e);
            self.vitamins.a = vit["a"].as_f64().unwrap_or(self.vitamins.a);
            self.vitamins.k = vit["k"].as_f64().unwrap_or(self.vitamins.k);
        }
        if let Some(aa) = v.get("amino_acids") {
            self.amino_acids.tryptophan = aa["tryptophan"].as_f64().unwrap_or(self.amino_acids.tryptophan);
            self.amino_acids.tyrosine = aa["tyrosine"].as_f64().unwrap_or(self.amino_acids.tyrosine);
            self.amino_acids.glutamine = aa["glutamine"].as_f64().unwrap_or(self.amino_acids.glutamine);
            self.amino_acids.histidine = aa["histidine"].as_f64().unwrap_or(self.amino_acids.histidine);
            self.amino_acids.glycine = aa["glycine"].as_f64().unwrap_or(self.amino_acids.glycine);
        }
        self.protein_level = v["protein_level"].as_f64().unwrap_or(self.protein_level);
        if let Some(en) = v.get("energy") {
            self.energy.atp_reserves = en["atp_reserves"].as_f64().unwrap_or(self.energy.atp_reserves);
            self.energy.glycogen_reserves = en["glycogen_reserves"].as_f64().unwrap_or(self.energy.glycogen_reserves);
            self.energy.caloric_balance = en["caloric_balance"].as_f64().unwrap_or(self.energy.caloric_balance);
        }
    }
}

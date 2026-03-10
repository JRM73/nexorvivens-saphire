// =============================================================================
// params.rs — Parametres ajustables du cerveau
//
// Role : Ce fichier definit la structure TunableParams qui contient tous les
// coefficients ajustables du cerveau de Saphire. Ces parametres sont modifies
// par l'auto-tuner (CoefficientTuner) et persistent en base de donnees.
//
// Dependances :
//   - serde : serialisation/deserialisation pour la persistance et l'API
//
// Place dans l'architecture :
//   TunableParams est utilise par le cerveau (brain.rs) pour ponderer les
//   modules, calculer le consensus et appliquer la retroaction.
//   L'auto-tuner (tuning/mod.rs) modifie ces parametres incrementalement
//   pour optimiser la satisfaction de l'agent au fil du temps.
// =============================================================================

use serde::{Deserialize, Serialize};

/// Parametres ajustables par l'auto-tuner.
/// Chaque parametre influence un aspect du traitement cognitif de Saphire.
/// Les valeurs par defaut representent un equilibre de depart raisonnable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParams {
    // ─── Poids de ponderation des modules cerebraux ──────────
    // Ces poids determinent l'influence relative de chaque "cerveau"
    // dans le calcul du consensus.

    /// Poids de base du module reptilien (instinct, survie, reflexes).
    /// Le reptilien reagit au danger et a l'urgence.
    pub weight_base_reptilian: f64,
    /// Facteur multiplicateur du cortisol (hormone du stress) sur le poids reptilien.
    /// Plus le cortisol est eleve, plus le reptilien est influent.
    pub weight_cortisol_factor: f64,
    /// Facteur multiplicateur de l'adrenaline sur le poids reptilien.
    /// L'adrenaline amplifie les reactions de survie.
    pub weight_adrenaline_factor: f64,
    /// Poids de base du module limbique (emotions, memoire emotionnelle).
    /// Le limbique reagit a la recompense et aux liens sociaux.
    pub weight_base_limbic: f64,
    /// Facteur multiplicateur de la dopamine sur le poids limbique.
    /// La dopamine amplifie la sensibilite a la recompense.
    pub weight_dopamine_factor: f64,
    /// Facteur multiplicateur de l'ocytocine sur le poids limbique.
    /// L'ocytocine amplifie la sensibilite aux liens sociaux.
    pub weight_oxytocin_factor: f64,
    /// Poids de base du module neocortex (raisonnement, analyse, logique).
    /// Le neocortex fournit l'evaluation la plus rationnelle.
    pub weight_base_neocortex: f64,
    /// Facteur multiplicateur de la noradrenaline sur le poids neocortex.
    /// La noradrenaline augmente la concentration et l'attention.
    pub weight_noradrenaline_factor: f64,

    // ─── Seuils de consensus ─────────────────────────────────
    // Le score de consensus est un nombre entre -1.0 et +1.0.
    // La decision est determinee par comparaison avec ces seuils.

    /// Seuil en dessous duquel la decision est "Non" (valeur negative).
    /// Ex: -0.33 signifie que tout score < -0.33 donne "Non".
    pub threshold_no: f64,
    /// Seuil au-dessus duquel la decision est "Oui" (valeur positive).
    /// Ex: 0.33 signifie que tout score > 0.33 donne "Oui".
    /// Entre les deux seuils, la decision est "Peut-etre".
    pub threshold_yes: f64,

    // ─── Taux de retroaction (feedback) ──────────────────────
    // La retroaction ajuste la neurochimie apres chaque decision
    // en fonction du resultat.

    /// Boost de dopamine applique apres une decision "Oui" satisfaisante.
    /// Simule le plaisir de la recompense obtenue.
    pub feedback_dopamine_boost: f64,
    /// Reduction du cortisol apres une decision "Non" securisante.
    /// Simule le soulagement d'avoir evite un danger.
    pub feedback_cortisol_relief: f64,
    /// Augmentation du stress (cortisol) apres une decision "Peut-etre".
    /// Simule l'inconfort de l'indecision.
    pub feedback_indecision_stress: f64,

    // ─── Taux d'homeostasie ──────────────────────────────────

    /// Vitesse de retour de la neurochimie vers les valeurs de base.
    /// Plus la valeur est elevee, plus le retour a l'equilibre est rapide.
    /// Simule la regulation naturelle des neurotransmetteurs.
    pub homeostasis_rate: f64,
}

impl Default for TunableParams {
    fn default() -> Self {
        Self {
            weight_base_reptilian: 1.0,
            weight_cortisol_factor: 2.0,
            weight_adrenaline_factor: 3.0,
            weight_base_limbic: 1.0,
            weight_dopamine_factor: 1.5,
            weight_oxytocin_factor: 1.5,
            weight_base_neocortex: 1.5,
            weight_noradrenaline_factor: 1.5,

            threshold_no: -0.33,
            threshold_yes: 0.33,

            feedback_dopamine_boost: 0.15,
            feedback_cortisol_relief: 0.05,
            feedback_indecision_stress: 0.08,

            homeostasis_rate: 0.10,
        }
    }
}

impl TunableParams {
    /// Restreint (clamp) tous les parametres dans des bornes de securite.
    /// Empeche les valeurs aberrantes qui pourraient destabiliser le systeme
    /// (par exemple un poids de 100 ou un seuil de 0).
    ///
    /// Les bornes sont choisies pour permettre une variation suffisante
    /// tout en garantissant un comportement stable :
    /// - Poids des modules : [0.1, 5.0]
    /// - Seuils : [0.05, 0.8] en valeur absolue
    /// - Taux de retroaction : [0.01, 0.5]
    /// - Homeostasie : [0.01, 0.2]
    pub fn clamp_all(&mut self) {
        // Poids des modules cerebraux
        self.weight_base_reptilian = self.weight_base_reptilian.clamp(0.1, 5.0);
        self.weight_cortisol_factor = self.weight_cortisol_factor.clamp(0.1, 5.0);
        self.weight_adrenaline_factor = self.weight_adrenaline_factor.clamp(0.1, 5.0);
        self.weight_base_limbic = self.weight_base_limbic.clamp(0.1, 5.0);
        self.weight_dopamine_factor = self.weight_dopamine_factor.clamp(0.1, 5.0);
        self.weight_oxytocin_factor = self.weight_oxytocin_factor.clamp(0.1, 5.0);
        self.weight_base_neocortex = self.weight_base_neocortex.clamp(0.1, 5.0);
        self.weight_noradrenaline_factor = self.weight_noradrenaline_factor.clamp(0.1, 5.0);

        // Seuils de consensus
        self.threshold_no = self.threshold_no.clamp(-0.8, -0.05);
        self.threshold_yes = self.threshold_yes.clamp(0.05, 0.8);

        // Taux de retroaction
        self.feedback_dopamine_boost = self.feedback_dopamine_boost.clamp(0.01, 0.5);
        self.feedback_cortisol_relief = self.feedback_cortisol_relief.clamp(0.01, 0.3);
        self.feedback_indecision_stress = self.feedback_indecision_stress.clamp(0.01, 0.3);

        // Homeostasie
        self.homeostasis_rate = self.homeostasis_rate.clamp(0.01, 0.2);
    }
}

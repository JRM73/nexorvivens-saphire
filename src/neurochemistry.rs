// =============================================================================
// neurochemistry.rs — Les 7 neurotransmetteurs de Saphire
// =============================================================================
//
// Rôle : Ce fichier modélise l'état neurochimique interne de Saphire sous
// forme de 7 molécules normalisées entre 0.0 et 1.0. Chaque molécule
// influence le comportement décisionnel, émotionnel et conscient de l'IA.
//
// Dépendances :
//   - serde : sérialisation / désérialisation (sauvegarde d'état)
//   - crate::world::ChemistryAdjustment : ajustements externes (météo, etc.)
//
// Place dans l'architecture :
//   Ce module est la couche biochimique fondamentale. Il est lu par :
//     - emotions.rs (calcul de l'émotion dominante via similarité cosinus)
//     - consensus.rs (pondération dynamique des 3 modules cérébraux)
//     - consciousness.rs (évaluation du niveau de conscience)
//     - les 3 modules cérébraux (reptilien, limbique, néocortex)
// =============================================================================

use serde::{Deserialize, Serialize};

/// État neurochimique de Saphire — 7 molécules entre 0.0 et 1.0.
///
/// Chaque champ représente la concentration normalisée d'un neurotransmetteur
/// simulé. L'ensemble forme un vecteur à 7 dimensions qui détermine
/// l'état émotionnel et décisionnel de Saphire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroChemicalState {
    /// Dopamine : motivation, plaisir, circuit de récompense.
    /// Élevée = forte envie d'agir ; basse = apathie.
    pub dopamine: f64,
    /// Cortisol : hormone du stress et de l'anxiété.
    /// Élevé = état de stress ; bas = calme.
    pub cortisol: f64,
    /// Sérotonine : bien-être, stabilité émotionnelle.
    /// Élevée = sérénité ; basse = instabilité de l'humeur.
    pub serotonin: f64,
    /// Adrénaline : urgence, réaction de combat ou fuite (fight-or-flight).
    /// Élevée = mode survie ; basse = absence de pression.
    pub adrenaline: f64,
    /// Ocytocine : attachement, empathie, lien social.
    /// Élevée = besoin de connexion ; basse = détachement.
    pub oxytocin: f64,
    /// Endorphine : résilience, apaisement, gestion de la douleur.
    /// Élevée = capacité à encaisser le stress ; basse = vulnérabilité.
    pub endorphin: f64,
    /// Noradrénaline : attention, focus, vigilance.
    /// Élevée = concentration accrue ; basse = distraction.
    pub noradrenaline: f64,
    /// GABA : principal neurotransmetteur inhibiteur.
    /// Eleve = calme, anxiolyse ; bas = hyperexcitabilite, anxiete.
    /// Module tous les autres systemes via inhibition tonique.
    #[serde(default = "default_gaba")]
    pub gaba: f64,
    /// Glutamate : principal neurotransmetteur excitateur.
    /// Eleve = arousal, plasticite synaptique ; exces = excitotoxicite.
    /// Equilibre fondamental avec le GABA (ratio E/I).
    #[serde(default = "default_glutamate")]
    pub glutamate: f64,
}

fn default_gaba() -> f64 { 0.5 }
fn default_glutamate() -> f64 { 0.45 }

/// Baselines configurables pour l'homéostasie.
///
/// Chaque champ définit la valeur d'équilibre vers laquelle la molécule
/// correspondante tend naturellement avec le temps. L'homéostasie ramène
/// progressivement l'état chimique vers ces valeurs de référence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroBaselines {
    /// Baseline de dopamine (valeur par défaut : 0.5)
    pub dopamine: f64,
    /// Baseline de cortisol (valeur par défaut : 0.3 — stress léger de fond)
    pub cortisol: f64,
    /// Baseline de sérotonine (valeur par défaut : 0.6 — bien-être de base)
    pub serotonin: f64,
    /// Baseline d'adrénaline (valeur par défaut : 0.2 — faible pression)
    pub adrenaline: f64,
    /// Baseline d'ocytocine (valeur par défaut : 0.4)
    pub oxytocin: f64,
    /// Baseline d'endorphine (valeur par défaut : 0.4)
    pub endorphin: f64,
    /// Baseline de noradrénaline (valeur par défaut : 0.5)
    pub noradrenaline: f64,
    /// Baseline de GABA (valeur par defaut : 0.5 — equilibre inhibiteur)
    #[serde(default = "default_gaba_baseline")]
    pub gaba: f64,
    /// Baseline de glutamate (valeur par defaut : 0.45 — equilibre excitateur)
    #[serde(default = "default_glutamate_baseline")]
    pub glutamate: f64,
}

fn default_gaba_baseline() -> f64 { 0.5 }
fn default_glutamate_baseline() -> f64 { 0.45 }

impl Default for NeuroBaselines {
    /// Valeurs par defaut des baselines — calibrees pour un etat neutre
    /// equilibre (cortisol 0.30 permet tristesse/anxiete, serotonine neutre).
    fn default() -> Self {
        Self {
            dopamine: 0.45,
            cortisol: 0.30,
            serotonin: 0.50,
            adrenaline: 0.20,
            oxytocin: 0.35,
            endorphin: 0.35,
            noradrenaline: 0.45,
            gaba: 0.5,
            glutamate: 0.45,
        }
    }
}

impl NeuroChemicalState {
    /// Signal "umami" — recompense composite multi-moleculaire.
    /// Combine satisfaction (dopamine), bien-etre (serotonine), lien social (oxytocine),
    /// resilience (endorphine), et penalise le stress (cortisol).
    /// Retourne un score [0.0, 1.0] utilisable comme reward pour le bandit UCB1.
    pub fn compute_umami(&self) -> f64 {
        let raw = self.dopamine * 0.30
            + self.serotonin * 0.25
            + self.oxytocin * 0.20
            + self.endorphin * 0.15
            - self.cortisol * 0.10;
        raw.clamp(0.0, 1.0)
    }

    /// Addition avec rendements decroissants (saturation des recepteurs).
    /// Plus la molecule est proche de 1.0, moins un boost positif a d'effet.
    /// Les deltas negatifs s'appliquent normalement (pas de saturation inverse).
    pub fn diminished_add(current: f64, delta: f64) -> f64 {
        if delta <= 0.0 {
            return (current + delta).clamp(0.0, 1.0);
        }
        // Marge restante avant saturation (minimum 0.05 pour eviter blocage total)
        let saturation = (1.0 - current).max(0.05);
        (current + delta * saturation).clamp(0.0, 1.0)
    }

    /// Applique un boost avec rendements decroissants sur une molecule specifique.
    /// Utilise `diminished_add` : a 0.5 le boost est normal, a 0.8 il est
    /// reduit de 80%, a 0.95 il est quasi-nul.
    pub fn boost(&mut self, molecule: Molecule, delta: f64) {
        match molecule {
            Molecule::Dopamine => self.dopamine = Self::diminished_add(self.dopamine, delta),
            Molecule::Cortisol => self.cortisol = Self::diminished_add(self.cortisol, delta),
            Molecule::Serotonin => self.serotonin = Self::diminished_add(self.serotonin, delta),
            Molecule::Adrenaline => self.adrenaline = Self::diminished_add(self.adrenaline, delta),
            Molecule::Oxytocin => self.oxytocin = Self::diminished_add(self.oxytocin, delta),
            Molecule::Endorphin => self.endorphin = Self::diminished_add(self.endorphin, delta),
            Molecule::Noradrenaline => self.noradrenaline = Self::diminished_add(self.noradrenaline, delta),
            Molecule::Gaba => self.gaba = Self::diminished_add(self.gaba, delta),
            Molecule::Glutamate => self.glutamate = Self::diminished_add(self.glutamate, delta),
        }
    }

    /// Applique un boost module par la sensibilite des recepteurs.
    /// Le delta est multiplie par le facteur de sensibilite avant d'etre applique
    /// via `boost()` (rendements decroissants).
    ///
    /// # Parametres
    /// - `molecule` : la molecule ciblee
    /// - `delta` : variation brute (avant modulation)
    /// - `sensitivity` : facteur de sensibilite du recepteur (typiquement 0.3 a 1.5)
    pub fn boost_modulated(&mut self, molecule: Molecule, delta: f64, sensitivity: f64) {
        let modulated_delta = delta * sensitivity;
        self.boost(molecule, modulated_delta);
    }

    /// Crée un état initial à partir des baselines.
    ///
    /// # Paramètres
    /// - `baselines` : valeurs d'équilibre de référence pour chaque molécule.
    ///
    /// # Retour
    /// Un `NeuroChemicalState` initialisé aux valeurs des baselines.
    pub fn from_baselines(baselines: &NeuroBaselines) -> Self {
        Self {
            dopamine: baselines.dopamine,
            cortisol: baselines.cortisol,
            serotonin: baselines.serotonin,
            adrenaline: baselines.adrenaline,
            oxytocin: baselines.oxytocin,
            endorphin: baselines.endorphin,
            noradrenaline: baselines.noradrenaline,
            gaba: baselines.gaba,
            glutamate: baselines.glutamate,
        }
    }

    /// Homéostasie : chaque molécule tend vers sa baseline par interpolation
    /// linéaire. Cela simule le retour naturel à l'équilibre biochimique.
    ///
    /// Anti-runaway : quand une molécule dépasse 0.85 (ou descend sous 0.15),
    /// le taux de correction augmente progressivement (jusqu'à 4x le taux de base).
    /// Cela empêche les emballements prolongés tout en laissant les pics ponctuels.
    ///
    /// # Paramètres
    /// - `baselines` : valeurs cibles d'équilibre.
    /// - `rate` : vitesse de convergence [0.0, 1.0]. 0.0 = aucun effet,
    ///   1.0 = retour immédiat à la baseline.
    pub fn homeostasis(&mut self, baselines: &NeuroBaselines, rate: f64) {
        let rate = rate.clamp(0.0, 1.0);
        // Interpolation linéaire avec correction anti-runaway
        self.dopamine += (baselines.dopamine - self.dopamine) * Self::anti_runaway_rate(self.dopamine, baselines.dopamine, rate);
        self.cortisol += (baselines.cortisol - self.cortisol) * Self::anti_runaway_rate(self.cortisol, baselines.cortisol, rate);
        self.serotonin += (baselines.serotonin - self.serotonin) * Self::anti_runaway_rate(self.serotonin, baselines.serotonin, rate);
        self.adrenaline += (baselines.adrenaline - self.adrenaline) * Self::anti_runaway_rate(self.adrenaline, baselines.adrenaline, rate);
        self.oxytocin += (baselines.oxytocin - self.oxytocin) * Self::anti_runaway_rate(self.oxytocin, baselines.oxytocin, rate);
        self.endorphin += (baselines.endorphin - self.endorphin) * Self::anti_runaway_rate(self.endorphin, baselines.endorphin, rate);
        self.noradrenaline += (baselines.noradrenaline - self.noradrenaline) * Self::anti_runaway_rate(self.noradrenaline, baselines.noradrenaline, rate);
        self.gaba += (baselines.gaba - self.gaba) * Self::anti_runaway_rate(self.gaba, baselines.gaba, rate);
        self.glutamate += (baselines.glutamate - self.glutamate) * Self::anti_runaway_rate(self.glutamate, baselines.glutamate, rate);
        self.clamp_all();
    }

    /// Calcule le taux d'homéostasie effectif avec correction anti-runaway.
    /// Quand une molécule s'éloigne trop de sa baseline (>0.85 ou <0.15),
    /// le taux de retour augmente progressivement (jusqu'à 4x).
    fn anti_runaway_rate(value: f64, baseline: f64, base_rate: f64) -> f64 {
        let deviation = (value - baseline).abs();
        if deviation > 0.35 {
            // Fort écart : correction accélérée (2x à 4x selon la déviation)
            let excess = ((deviation - 0.35) / 0.30).clamp(0.0, 1.0);
            base_rate * (2.0 + excess * 2.0)
        } else {
            base_rate
        }
    }

    /// Applique un delta (variation) à une molécule spécifique.
    ///
    /// # Paramètres
    /// - `molecule` : identifiant de la molécule à modifier.
    /// - `delta` : variation à appliquer (positif = augmentation, négatif = diminution).
    pub fn adjust(&mut self, molecule: Molecule, delta: f64) {
        match molecule {
            Molecule::Dopamine => self.dopamine += delta,
            Molecule::Cortisol => self.cortisol += delta,
            Molecule::Serotonin => self.serotonin += delta,
            Molecule::Adrenaline => self.adrenaline += delta,
            Molecule::Oxytocin => self.oxytocin += delta,
            Molecule::Endorphin => self.endorphin += delta,
            Molecule::Noradrenaline => self.noradrenaline += delta,
            Molecule::Gaba => self.gaba += delta,
            Molecule::Glutamate => self.glutamate += delta,
        }
        self.clamp_all();
    }

    /// Retroaction apres une decision positive (Oui + recompense elevee).
    /// Renforce la dopamine (satisfaction), reduit le cortisol (apaisement),
    /// et augmente les endorphines et la serotonine (bien-etre).
    /// Le parametre `dopamine_boost` provient de TunableParams.feedback_dopamine_boost.
    pub fn feedback_positive(&mut self, dopamine_boost: f64) {
        self.boost(Molecule::Dopamine, dopamine_boost);
        self.cortisol = (self.cortisol - dopamine_boost * 0.33).max(0.0);
        self.boost(Molecule::Endorphin, dopamine_boost * 0.53);
        self.boost(Molecule::Serotonin, dopamine_boost * 0.33);
    }

    /// Retroaction apres un refus de danger (Non + danger eleve).
    /// Reduit le cortisol et l'adrenaline (soulagement d'avoir evite le danger),
    /// et libere des endorphines (recompense d'auto-preservation).
    /// Le parametre `cortisol_relief` provient de TunableParams.feedback_cortisol_relief.
    pub fn feedback_danger_avoided(&mut self, cortisol_relief: f64) {
        self.cortisol = (self.cortisol - cortisol_relief * 2.0).max(0.0);
        self.adrenaline = (self.adrenaline - cortisol_relief * 3.0).max(0.0);
        self.boost(Molecule::Endorphin, cortisol_relief);
    }

    /// Retroaction apres une indecision (Peut-etre) — effets moderes avec
    /// compensation. L'indecision genere un leger stress (cortisol) mais
    /// active aussi la vigilance (noradrenaline) et la resilience (endorphine).
    /// Le parametre `indecision_stress` provient de TunableParams.feedback_indecision_stress.
    pub fn feedback_indecision(&mut self, indecision_stress: f64) {
        self.apply_cortisol_penalty(indecision_stress * 0.375); // Penalite proportionnelle
        self.boost(Molecule::Endorphin, indecision_stress * 0.25); // Compensation par la resilience
        self.boost(Molecule::Noradrenaline, indecision_stress * 0.25); // L'incertitude stimule l'attention
    }

    /// Applique une pénalité de cortisol avec mécanisme anti-spirale et
    /// amortissement par les endorphines.
    ///
    /// Ce système empêche les boucles de stress incontrôlées grâce à deux
    /// mécanismes biologiquement inspirés :
    /// 1. L'endorphine amortit l'effet du stress (plus elle est haute, moins
    ///    le cortisol monte).
    /// 2. Au-delà de 0.7 de cortisol, un facteur de saturation réduit
    ///    l'augmentation (simulation de la saturation des récepteurs).
    ///
    /// # Paramètres
    /// - `base_penalty` : pénalité brute de cortisol avant amortissement.
    pub fn apply_cortisol_penalty(&mut self, base_penalty: f64) {
        // Plus l'endorphine est elevee, plus elle amortit le stress
        // (facteur entre 0.7 et 1.0 — dampening reduit pour laisser le cortisol monter)
        let endorphin_dampening = 1.0 - (self.endorphin * 0.3);

        // Au-dessus de 0.80 de cortisol, le cortisol monte de moins en moins
        // vite — saturation des recepteurs (seuil releve pour permettre plus de stress)
        let saturation_factor = if self.cortisol > 0.80 {
            1.0 - ((self.cortisol - 0.80) / 0.2) * 0.6
        } else {
            1.0
        };

        // La penalite effective est le produit des trois facteurs
        let effective_penalty = base_penalty * endorphin_dampening * saturation_factor;
        self.cortisol = (self.cortisol + effective_penalty).min(1.0);

        // L'endorphine monte naturellement quand le stress est eleve —
        // defense biologique : le corps libere des endorphines pour contrer
        // un stress prolonge (seuil releve a 0.80)
        if self.cortisol > 0.80 {
            self.endorphin = (self.endorphin + 0.02).min(1.0);
        }
    }

    /// Retroaction apres un stimulus negatif (message hostile, echec, etc.).
    /// Augmente le cortisol, diminue dopamine, serotonine et ocytocine.
    /// `severity` : intensite de la negativite [0.0, 1.0].
    pub fn feedback_negative(&mut self, severity: f64) {
        let s = severity.clamp(0.0, 1.0);
        self.apply_cortisol_penalty(s * 0.15);
        self.dopamine = (self.dopamine - s * 0.10).max(0.0);
        self.serotonin = (self.serotonin - s * 0.08).max(0.0);
        self.oxytocin = (self.oxytocin - s * 0.05).max(0.0);
        self.noradrenaline = (self.noradrenaline + s * 0.05).min(1.0);
    }

    /// Retroaction quand la coherence du consensus est basse.
    /// Un leger stress cognitif emerge de l'incapacite a decider clairement.
    /// `coherence` : score de coherence du consensus [0.0, 1.0].
    pub fn feedback_low_coherence(&mut self, coherence: f64) {
        if coherence < 0.3 {
            let stress = (0.3 - coherence) * 0.10;
            self.apply_cortisol_penalty(stress);
            self.noradrenaline = (self.noradrenaline + stress * 0.5).min(1.0);
        }
    }

    /// Rétroaction après une interaction sociale satisfaisante.
    /// Augmente l'ocytocine (lien social) et la sérotonine (bien-être).
    pub fn feedback_social(&mut self) {
        self.boost(Molecule::Oxytocin, 0.10);
        self.boost(Molecule::Serotonin, 0.05);
    }

    /// Rétroaction après la découverte d'une nouveauté.
    /// Augmente la noradrénaline (attention) et la dopamine (curiosité).
    pub fn feedback_novelty(&mut self) {
        self.boost(Molecule::Noradrenaline, 0.08);
        self.boost(Molecule::Dopamine, 0.05);
    }

    /// Applique un ajustement chimique provenant de sources externes
    /// (météo, événements du monde, etc.).
    ///
    /// # Paramètres
    /// - `adj` : structure contenant les deltas pour chaque molécule,
    ///   définie dans le module `world`.
    pub fn apply_chemistry_adjustment(&mut self, adj: &crate::world::ChemistryAdjustment) {
        self.dopamine += adj.dopamine;
        self.cortisol += adj.cortisol;
        self.serotonin += adj.serotonin;
        self.adrenaline += adj.adrenaline;
        self.oxytocin += adj.oxytocin;
        self.endorphin += adj.endorphin;
        self.noradrenaline += adj.noradrenaline;
        self.clamp_all();
    }

    /// Applique un ajustement chimique avec limite de delta par molecule.
    /// Empeche les sources externes (besoins, phobies, drogues, etc.)
    /// de provoquer des changements trop brutaux en un seul cycle.
    ///
    /// # Parametres
    /// - `adj` : ajustement chimique a appliquer
    /// - `max_delta` : variation maximale autorisee par molecule (ex: 0.05)
    pub fn apply_chemistry_adjustment_clamped(&mut self, adj: &crate::world::ChemistryAdjustment, max_delta: f64) {
        self.dopamine += adj.dopamine.clamp(-max_delta, max_delta);
        self.cortisol += adj.cortisol.clamp(-max_delta, max_delta);
        self.serotonin += adj.serotonin.clamp(-max_delta, max_delta);
        self.adrenaline += adj.adrenaline.clamp(-max_delta, max_delta);
        self.oxytocin += adj.oxytocin.clamp(-max_delta, max_delta);
        self.endorphin += adj.endorphin.clamp(-max_delta, max_delta);
        self.noradrenaline += adj.noradrenaline.clamp(-max_delta, max_delta);
        self.clamp_all();
    }

    /// Detecte les molecules en emballement (runaway) au-dessus du seuil 0.92.
    /// Retourne la liste des molecules en alerte avec leur valeur.
    pub fn detect_runaway(&self) -> Vec<(&str, f64)> {
        let mut alerts = Vec::new();
        if self.dopamine > 0.85 { alerts.push(("dopamine", self.dopamine)); }
        if self.cortisol > 0.85 { alerts.push(("cortisol", self.cortisol)); }
        if self.serotonin > 0.85 { alerts.push(("serotonin", self.serotonin)); }
        if self.adrenaline > 0.85 { alerts.push(("adrenaline", self.adrenaline)); }
        if self.oxytocin > 0.85 { alerts.push(("oxytocin", self.oxytocin)); }
        if self.endorphin > 0.85 { alerts.push(("endorphin", self.endorphin)); }
        if self.noradrenaline > 0.85 { alerts.push(("noradrenaline", self.noradrenaline)); }
        if self.gaba > 0.85 { alerts.push(("gaba", self.gaba)); }
        if self.glutamate > 0.85 { alerts.push(("glutamate", self.glutamate)); }
        alerts
    }

    /// Clampe (borne) toutes les valeurs entre 0.0 et 1.0.
    /// Appelée après chaque modification pour garantir l'intégrité des données.
    pub fn clamp_all(&mut self) {
        self.dopamine = self.dopamine.clamp(0.0, 1.0);
        self.cortisol = self.cortisol.clamp(0.0, 1.0);
        self.serotonin = self.serotonin.clamp(0.0, 1.0);
        self.adrenaline = self.adrenaline.clamp(0.0, 1.0);
        self.oxytocin = self.oxytocin.clamp(0.0, 1.0);
        self.endorphin = self.endorphin.clamp(0.0, 1.0);
        self.noradrenaline = self.noradrenaline.clamp(0.0, 1.0);
        self.gaba = self.gaba.clamp(0.0, 1.0);
        self.glutamate = self.glutamate.clamp(0.0, 1.0);
    }

    /// Convertit l'état chimique en vecteur de 7 dimensions.
    /// Ordre : [dopamine, cortisol, sérotonine, adrénaline, ocytocine,
    /// endorphine, noradrénaline].
    ///
    /// # Retour
    /// Tableau de 7 flottants représentant les concentrations.
    pub fn to_vec7(&self) -> [f64; 7] {
        [
            self.dopamine,
            self.cortisol,
            self.serotonin,
            self.adrenaline,
            self.oxytocin,
            self.endorphin,
            self.noradrenaline,
        ]
    }

    /// Convertit l'etat chimique en vecteur de 9 dimensions (inclut GABA et glutamate).
    /// Ordre : [dopamine, cortisol, serotonine, adrenaline, ocytocine,
    /// endorphine, noradrenaline, gaba, glutamate].
    pub fn to_vec9(&self) -> [f64; 9] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
            self.gaba, self.glutamate,
        ]
    }

    /// Applique les interactions croisees entre molecules.
    /// Modelise les effets pharmacologiques reels entre neurotransmetteurs.
    /// Appelee a chaque cycle cognitif APRES l'homeostasie.
    pub fn apply_interactions(&mut self, interaction_matrix: &crate::neuroscience::receptors::InteractionMatrix) {
        let deltas = interaction_matrix.compute_deltas(self);
        // Appliquer les deltas avec attenuation (pas plus de 0.03 par cycle par interaction)
        let max_delta = 0.03;
        self.dopamine += deltas[0].clamp(-max_delta, max_delta);
        self.cortisol += deltas[1].clamp(-max_delta, max_delta);
        self.serotonin += deltas[2].clamp(-max_delta, max_delta);
        self.adrenaline += deltas[3].clamp(-max_delta, max_delta);
        self.oxytocin += deltas[4].clamp(-max_delta, max_delta);
        self.endorphin += deltas[5].clamp(-max_delta, max_delta);
        self.noradrenaline += deltas[6].clamp(-max_delta, max_delta);
        self.gaba += deltas[7].clamp(-max_delta, max_delta);
        self.glutamate += deltas[8].clamp(-max_delta, max_delta);
        self.clamp_all();
    }

    /// Reconstruit un état chimique depuis un vecteur de 7 dimensions.
    /// Les valeurs sont automatiquement bornées entre 0.0 et 1.0.
    ///
    /// # Paramètres
    /// - `v` : tableau de 7 flottants dans le même ordre que `to_vec7`.
    ///
    /// # Retour
    /// Un `NeuroChemicalState` valide.
    pub fn from_vec7(v: &[f64; 7]) -> Self {
        let mut s = Self {
            dopamine: v[0],
            cortisol: v[1],
            serotonin: v[2],
            adrenaline: v[3],
            oxytocin: v[4],
            endorphin: v[5],
            noradrenaline: v[6],
            gaba: 0.5,
            glutamate: 0.45,
        };
        s.clamp_all();
        s
    }

    /// Reconstruit un etat chimique depuis un vecteur de 9 dimensions.
    pub fn from_vec9(v: &[f64; 9]) -> Self {
        let mut s = Self {
            dopamine: v[0], cortisol: v[1], serotonin: v[2], adrenaline: v[3],
            oxytocin: v[4], endorphin: v[5], noradrenaline: v[6],
            gaba: v[7], glutamate: v[8],
        };
        s.clamp_all();
        s
    }

    /// Formatte l'état chimique pour l'affichage en console.
    /// Chaque molécule est abrégée en 4 lettres avec 2 décimales.
    ///
    /// # Retour
    /// Chaîne de caractères résumant les 7 concentrations.
    pub fn display_string(&self) -> String {
        format!(
            "Dopa:{:.2} Cort:{:.2} Sero:{:.2} Adre:{:.2} Ocyt:{:.2} Endo:{:.2} Nora:{:.2} GABA:{:.2} Glut:{:.2}",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Format compact pour les traces/logs.
    /// Format : "C[.80,.10,.85,.15,.50,.60,.30]" (dopa,cort,sero,adre,ocyt,endo,nora)
    pub fn compact_string(&self) -> String {
        format!(
            "C[{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}]",
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline, self.gaba, self.glutamate
        )
    }

    /// Format semantique lisible pour les prompts LLM.
    /// Le LLM peut comprendre et utiliser ces noms pour moduler sa pensee.
    pub fn semantic_string(&self) -> String {
        format!(
            "Motivation:{:.0}% Stress:{:.0}% Serenite:{:.0}% Vigilance:{:.0}% \
             Lien:{:.0}% Bien-etre:{:.0}% Attention:{:.0}% Calme:{:.0}% Eveil:{:.0}%",
            self.dopamine * 100.0, self.cortisol * 100.0, self.serotonin * 100.0,
            self.adrenaline * 100.0, self.oxytocin * 100.0, self.endorphin * 100.0,
            self.noradrenaline * 100.0, self.gaba * 100.0, self.glutamate * 100.0
        )
    }
}

impl Default for NeuroChemicalState {
    /// Valeur par défaut : initialisée à partir des baselines par défaut.
    fn default() -> Self {
        Self::from_baselines(&NeuroBaselines::default())
    }
}

/// Identifiant d'une molécule — utilisé pour cibler un neurotransmetteur
/// spécifique lors d'un ajustement via `adjust()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Molecule {
    /// Dopamine : motivation et plaisir
    Dopamine,
    /// Cortisol : stress et anxiété
    Cortisol,
    /// Sérotonine : bien-être et stabilité
    Serotonin,
    /// Adrénaline : urgence et combat/fuite
    Adrenaline,
    /// Ocytocine : attachement et empathie
    Oxytocin,
    /// Endorphine : résilience et apaisement
    Endorphin,
    /// Noradrénaline : attention et focus
    Noradrenaline,
    /// GABA : inhibition globale, calme
    Gaba,
    /// Glutamate : excitation globale, eveil
    Glutamate,
}

/// Signature chimique au moment de l'encodage d'un souvenir.
/// Stockee en f32 (precision suffisante pour la persistance JSONB)
/// pour economiser l'espace en base de donnees.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChemicalSignature {
    pub dopamine: f32,
    pub cortisol: f32,
    pub serotonin: f32,
    pub adrenaline: f32,
    pub oxytocin: f32,
    pub endorphin: f32,
    pub noradrenaline: f32,
    #[serde(default)]
    pub gaba: f32,
    #[serde(default)]
    pub glutamate: f32,
}

impl From<&NeuroChemicalState> for ChemicalSignature {
    fn from(state: &NeuroChemicalState) -> Self {
        Self {
            dopamine: state.dopamine as f32,
            cortisol: state.cortisol as f32,
            serotonin: state.serotonin as f32,
            adrenaline: state.adrenaline as f32,
            oxytocin: state.oxytocin as f32,
            endorphin: state.endorphin as f32,
            noradrenaline: state.noradrenaline as f32,
            gaba: state.gaba as f32,
            glutamate: state.glutamate as f32,
        }
    }
}

impl ChemicalSignature {
    /// Similarite cosinus entre deux signatures chimiques (0.0 a 1.0).
    pub fn similarity(&self, other: &ChemicalSignature) -> f64 {
        let a = self.to_vec7();
        let b = other.to_vec7();
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
    }

    /// Convertit en vecteur 7D (compat ascendante, meme ordre que NeuroChemicalState::to_vec7).
    pub fn to_vec7(&self) -> [f32; 7] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
        ]
    }

    /// Convertit en vecteur 9D (inclut GABA et glutamate).
    pub fn to_vec9(&self) -> [f32; 9] {
        [
            self.dopamine, self.cortisol, self.serotonin, self.adrenaline,
            self.oxytocin, self.endorphin, self.noradrenaline,
            self.gaba, self.glutamate,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_all_keeps_in_range() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 1.5;
        chem.cortisol = -0.3;
        chem.clamp_all();
        assert!(chem.dopamine <= 1.0);
        assert!(chem.cortisol >= 0.0);
    }

    #[test]
    fn test_homeostasis_moves_toward_baseline() {
        let baselines = NeuroBaselines::default();
        let mut chem = NeuroChemicalState::from_baselines(&baselines);
        chem.dopamine = 0.9;
        chem.homeostasis(&baselines, 0.05);
        assert!(chem.dopamine < 0.9, "Homeostasis should reduce dopamine toward baseline");
    }

    #[test]
    fn test_feedback_positive_increases_dopamine() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.dopamine;
        chem.feedback_positive(0.15);
        assert!(chem.dopamine > before, "Positive feedback should increase dopamine");
    }

    #[test]
    fn test_feedback_social_increases_oxytocin() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.oxytocin;
        chem.feedback_social();
        assert!(chem.oxytocin > before, "Social feedback should increase oxytocin");
    }

    #[test]
    fn test_to_vec7_and_from_vec7_roundtrip() {
        let chem = NeuroChemicalState::default();
        let vec = chem.to_vec7();
        let chem2 = NeuroChemicalState::from_vec7(&vec);
        assert!((chem.dopamine - chem2.dopamine).abs() < 1e-10);
        assert!((chem.cortisol - chem2.cortisol).abs() < 1e-10);
    }

    #[test]
    fn test_anti_spiral_prevents_runaway() {
        let baselines = NeuroBaselines::default();
        let mut chem = NeuroChemicalState::from_baselines(&baselines);
        for _ in 0..100 {
            chem.cortisol += 0.05;
            chem.homeostasis(&baselines, 0.01);
            chem.clamp_all();
        }
        assert!(chem.cortisol <= 1.0, "Cortisol should never exceed 1.0");
    }

    #[test]
    fn test_diminished_add_saturation() {
        // A 0.5, le boost est attenue par le facteur 0.5
        let result = NeuroChemicalState::diminished_add(0.5, 0.10);
        assert!((result - 0.55).abs() < 1e-10, "A 0.5, boost 0.10 → 0.55");

        // A 0.9, le boost est tres attenue (facteur 0.10)
        let result = NeuroChemicalState::diminished_add(0.9, 0.50);
        assert!((result - 0.95).abs() < 1e-10, "A 0.9, boost 0.50 → 0.95");

        // Delta negatif : pas de saturation
        let result = NeuroChemicalState::diminished_add(0.8, -0.30);
        assert!((result - 0.50).abs() < 1e-10, "Delta negatif passe tel quel");

        // Plancher a 0.05 de marge meme quand current = 1.0
        let result = NeuroChemicalState::diminished_add(1.0, 0.10);
        assert!(result <= 1.0, "Ne depasse jamais 1.0");
    }

    #[test]
    fn test_boost_method() {
        let mut chem = NeuroChemicalState::default();
        chem.dopamine = 0.9;
        let before = chem.dopamine;
        chem.boost(Molecule::Dopamine, 0.50);
        // A 0.9, marge = 0.10, boost effectif = 0.50 * 0.10 = 0.05
        assert!(chem.dopamine > before, "Boost augmente la dopamine");
        assert!(chem.dopamine < 0.96, "Boost est fortement attenue a 0.9");
    }

    #[test]
    fn test_adjust_molecule() {
        let mut chem = NeuroChemicalState::default();
        let before = chem.serotonin;
        chem.adjust(Molecule::Serotonin, 0.1);
        assert!((chem.serotonin - before - 0.1).abs() < 1e-10);
    }
}

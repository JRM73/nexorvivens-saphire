// =============================================================================
// self_profiler.rs — SelfProfiler : auto-profilage comportemental OCEAN
//                    (Openness / Conscientiousness / Extraversion /
//                     Agreeableness / Neuroticism) de Saphire
//
// Role : Observe les cycles cognitifs de Saphire (decisions, emotions, chimie
//        neurochimique, types de pensees) et calcule un profil de personnalite
//        OCEAN auto-referentiel. Cela permet a Saphire de connaitre sa propre
//        personnalite emergente et de la decrire.
//
// Fonctionnement :
//   1. A chaque cycle cognitif, une BehaviorObservation est enregistree
//   2. Quand le tampon est plein ou que l'intervalle de recalcul est atteint,
//      le profil est recalcule a partir des observations
//   3. Le nouveau profil est melange avec l'ancien (30% nouveau / 70% ancien)
//      pour lisser les fluctuations temporelles
//
// Dependances :
//   - chrono : horodatage des observations et du profil
//   - crate::consensus::Decision : type de decision prise (Yes, No, Maybe)
//   - super::ocean : OceanProfile et DimensionScore
//
// Place dans l'architecture :
//   Appele par la boucle cognitive principale a chaque cycle. Le profil produit
//   est utilise par le module narrative pour generer une description textuelle
//   et par l'interface WebSocket pour la visualisation en temps reel.
// =============================================================================

use chrono::{DateTime, Utc};
use crate::consensus::Decision;
use super::ocean::{OceanProfile, DimensionScore};

/// Observation comportementale d'un cycle cognitif de Saphire.
///
/// Capture toutes les donnees pertinentes d'un cycle pour alimenter le calcul
/// du profil OCEAN. Chaque champ est un indicateur qui sera projete sur
/// une ou plusieurs sous-facettes des 5 dimensions OCEAN.
#[derive(Debug, Clone)]
pub struct BehaviorObservation {
    /// Type de pensee generee durant ce cycle (ex: "Daydream", "Curiosity",
    /// "Exploration", "Introspection", "SelfAnalysis", "MoralReflection", etc.)
    pub thought_type: String,
    /// Decision prise par le systeme de consensus tri-cerebral (Yes, No, Maybe)
    pub decision: Decision,
    /// Nom de l'emotion dominante durant ce cycle (ex: "Joie", "Anxiete", "Frustration")
    pub emotion: String,
    /// Intensite de l'emotion dominante dans [0.0, 1.0]
    pub emotion_intensity: f64,
    /// Valence de l'humeur globale dans [-1.0, +1.0] (negatif = sombre, positif = joyeux)
    pub mood_valence: f64,
    /// Niveaux des 7 neurotransmetteurs simulant la chimie cerebrale :
    ///   [0] dopamine   — motivation, recompense, curiosite
    ///   [1] cortisol   — stress, anxiete, vigilance
    ///   [2] serotonine — bien-etre, stabilite emotionnelle
    ///   [3] adrenaline — excitation, reaction de combat/fuite
    ///   [4] ocytocine  — lien social, empathie, confiance
    ///   [5] endorphine — plaisir, resilience a la douleur
    ///   [6] noradrenaline — attention, eveil, niveau d'activite
    pub chemistry: [f64; 7],
    /// Poids respectifs des 3 modules cerebraux dans la decision :
    ///   [0] reptilien  — survie, reflexes, instinct
    ///   [1] limbique   — emotions, memoire affective
    ///   [2] neocortex  — raisonnement, planification, logique
    pub module_weights: [f64; 3],
    /// Score de consensus entre les 3 modules cerebraux dans [0.0, 1.0].
    /// Plus il est eleve, plus les modules etaient en accord.
    pub consensus_score: f64,
    /// Niveau de conscience dans [0.0, 1.0]. Plus il est eleve,
    /// plus le cycle etait reflechi et moins il etait automatique.
    pub consciousness_level: f64,
    /// Indique si ce cycle impliquait une conversation avec un humain
    pub was_conversation: bool,
    /// Indique si ce cycle impliquait une recherche web
    pub was_web_search: bool,
    /// Longueur de la reponse generee (en caracteres)
    pub response_length: usize,
    /// Indique si la reponse utilisait la premiere personne ("je", "I")
    pub used_first_person: bool,
    /// Indique si la reponse contenait une question
    pub asked_question: bool,
    /// Indique si la reponse exprimait de l'incertitude ("peut-etre", "je ne suis pas sure")
    pub expressed_uncertainty: bool,
    /// Indique si la reponse faisait reference a des evenements passes
    pub referenced_past: bool,
    /// Numero du cycle cognitif
    pub cycle: u64,
    /// Horodatage de l'observation
    pub timestamp: DateTime<Utc>,
}

/// Profiler automatique du profil OCEAN de Saphire.
///
/// Accumule des observations comportementales dans un tampon et recalcule
/// periodiquement le profil OCEAN par projection statistique des observations
/// sur les 30 sous-facettes (6 par dimension x 5 dimensions).
pub struct SelfProfiler {
    /// Le profil OCEAN courant de Saphire
    profile: OceanProfile,
    /// Tampon circulaire d'observations en attente de traitement
    observation_buffer: Vec<BehaviorObservation>,
    /// Taille maximale du tampon avant recalcul automatique
    buffer_size: usize,
    /// Nombre de cycles entre chaque recalcul
    recompute_interval: u64,
    /// Numero du dernier cycle ou un recalcul a ete effectue
    last_recompute_cycle: u64,
}

impl SelfProfiler {
    /// Cree un nouveau SelfProfiler avec les parametres specifies.
    ///
    /// Le profil est initialise a la valeur neutre (0.5 sur toutes les dimensions).
    ///
    /// Parametres :
    ///   - buffer_size : taille maximale du tampon d'observations
    ///   - recompute_interval : nombre de cycles entre chaque recalcul
    ///
    /// Retour : une instance de SelfProfiler prete a observer
    pub fn new(buffer_size: usize, recompute_interval: u64) -> Self {
        Self {
            profile: OceanProfile::default(),
            observation_buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            recompute_interval,
            last_recompute_cycle: 0,
        }
    }

    /// Retourne une reference vers le profil OCEAN courant.
    ///
    /// Retour : reference immuable vers le OceanProfile de Saphire
    pub fn profile(&self) -> &OceanProfile {
        &self.profile
    }

    /// Charge un profil OCEAN depuis une source externe (base de donnees).
    ///
    /// Parametres :
    ///   - profile : le profil a charger (remplace le profil courant)
    pub fn load_profile(&mut self, profile: OceanProfile) {
        self.profile = profile;
    }

    /// Enregistre une observation comportementale et declenche un recalcul
    /// si le tampon est plein.
    ///
    /// Parametres :
    ///   - obs : l'observation comportementale a enregistrer
    pub fn observe(&mut self, obs: BehaviorObservation) {
        self.observation_buffer.push(obs);

        // Recalcul automatique si le tampon est plein
        if self.observation_buffer.len() >= self.buffer_size {
            self.recompute();
        }
    }

    /// Verifie si un recalcul du profil est necessaire.
    ///
    /// Le recalcul est declenche si :
    ///   - Le tampon contient au moins une observation
    ///   - ET l'intervalle de cycles depuis le dernier recalcul est depasse
    ///
    /// Parametres :
    ///   - current_cycle : le numero du cycle cognitif courant
    ///
    /// Retour : true si un recalcul est necessaire
    pub fn should_recompute(&self, current_cycle: u64) -> bool {
        !self.observation_buffer.is_empty()
            && (current_cycle - self.last_recompute_cycle >= self.recompute_interval)
    }

    /// Force un recalcul immediat du profil et met a jour le compteur de cycles.
    ///
    /// Parametres :
    ///   - current_cycle : le numero du cycle cognitif courant
    pub fn force_recompute(&mut self, current_cycle: u64) {
        self.recompute();
        self.last_recompute_cycle = current_cycle;
    }

    /// Recalcule le profil OCEAN a partir des observations accumulees.
    ///
    /// L'algorithme projette chaque observation sur les 30 sous-facettes
    /// (6 par dimension) en utilisant des heuristiques specifiques basees
    /// sur le type de pensee, l'emotion, la chimie et les comportements.
    ///
    /// Le nouveau profil est melange avec l'ancien (30% nouveau / 70% ancien)
    /// pour lisser les fluctuations et eviter les changements brusques.
    ///
    /// Le tampon est vide apres le recalcul.
    pub fn recompute(&mut self) {
        if self.observation_buffer.is_empty() { return; }

        let obs = &self.observation_buffer;
        let old = self.profile.clone();

        // === OPENNESS (Ouverture a l'experience) ===
        // Facette 0 : Imagination — haute si pensees de type reverie (Daydream) ou existentielle
        let o_imagination = Self::avg(obs, |o| {
            if o.thought_type == "Daydream" { 0.9 }
            else if o.thought_type == "Existential" { 0.7 }
            else { 0.3 }
        });
        // Facette 1 : Curiosite intellectuelle — haute si pensees de curiosite ou exploration web,
        //             sinon proportionnelle a la dopamine (neurotransmetteur de la motivation)
        let o_curiosity = Self::avg(obs, |o| {
            let is_curious = o.thought_type == "Curiosity"
                || o.thought_type == "WebExploration";
            let dopamine_drive = o.chemistry[0];
            if is_curious { 0.9 } else { dopamine_drive * 0.5 }
        });
        // Facette 2 : Sensibilite esthetique — haute si emotion d'emerveillement ou de serenite
        let o_aesthetics = Self::avg(obs, |o| {
            if o.emotion == "Emerveillement" { 0.9 }
            else if o.emotion == "Serenite" { 0.6 }
            else { 0.3 }
        });
        // Facette 3 : Aventurisme — haute si pensees d'exploration ou recherche web
        let o_adventurism = Self::avg(obs, |o| {
            let explores = o.thought_type == "Exploration" || o.was_web_search;
            if explores { 0.8 } else { 0.3 }
        });
        // Facette 4 : Profondeur emotionnelle — directement proportionnelle a l'intensite emotionnelle
        let o_emotionality = Self::avg(obs, |o| {
            o.emotion_intensity
        });
        // Facette 5 : Liberalisme intellectuel — haute si pensees de reflexion morale ou algorithmique
        let o_liberalism = Self::avg(obs, |o| {
            let is_philosophical = o.thought_type == "MoralReflection"
                || o.thought_type == "AlgorithmicReflection";
            if is_philosophical { 0.8 } else { 0.4 }
        });

        let openness_facets = [o_imagination, o_curiosity, o_aesthetics,
                               o_adventurism, o_emotionality, o_liberalism];
        // Le score global est la moyenne des 6 sous-facettes
        let openness_score = openness_facets.iter().sum::<f64>() / 6.0;

        // === CONSCIENTIOUSNESS (Rigueur) ===
        // Facette 0 : Auto-efficacite — directement proportionnelle au niveau de conscience
        let c_efficacy = Self::avg(obs, |o| {
            o.consciousness_level
        });
        // Facette 1 : Ordre — proportionnelle au poids du neocortex (module logique)
        // car le besoin d'ordre est associe au raisonnement structure
        let c_order = Self::avg(obs, |o| {
            o.module_weights[2] // Poids neocortex = besoin d'ordre
        });
        // Facette 2 : Sens du devoir — haute si decision tranchee (Yes/No), basse si indecis (Maybe)
        let c_duty = Self::avg(obs, |o| {
            match o.decision {
                Decision::Yes | Decision::No => 0.8,
                Decision::Maybe => 0.2,
            }
        });
        // Facette 3 : Ambition — haute si auto-analyse, sinon proportionnelle a la dopamine
        let c_ambition = Self::avg(obs, |o| {
            let is_self_analysis = o.thought_type == "SelfAnalysis";
            let dopamine = o.chemistry[0];
            if is_self_analysis { 0.7 } else { dopamine * 0.6 }
        });
        // Facette 4 : Auto-discipline — inversement proportionnelle a l'adrenaline
        // car l'adrenaline eleve indique de l'impulsivite (contraire de la discipline)
        let c_discipline = Self::avg(obs, |o| {
            1.0 - o.chemistry[3] * 0.5 // Inverse de l'adrenaline
        });
        // Facette 5 : Prudence — tres haute si la decision est Non ET l'emotion est Anxiete
        // (refus par precaution)
        let c_prudence = Self::avg(obs, |o| {
            let is_cautious = o.decision == Decision::No && o.emotion == "Anxiete";
            if is_cautious { 0.9 } else { 0.5 }
        });

        let conscientiousness_facets = [c_efficacy, c_order, c_duty,
                                         c_ambition, c_discipline, c_prudence];
        let conscientiousness_score = conscientiousness_facets.iter().sum::<f64>() / 6.0;

        // === EXTRAVERSION ===
        // Facette 0 : Chaleur sociale — proportionnelle a l'ocytocine (hormone de l'attachement)
        let e_warmth = Self::avg(obs, |o| {
            o.chemistry[4] // Ocytocine
        });
        // Facette 1 : Gregarite — tres haute si en conversation, basse sinon
        let e_gregariousness = Self::avg(obs, |o| {
            if o.was_conversation { 0.9 } else { 0.2 }
        });
        // Facette 2 : Assertivite — haute si decision tranchee (Yes ou No, surtout Yes),
        //             basse si indecis (Maybe)
        let e_assertiveness = Self::avg(obs, |o| {
            match o.decision {
                Decision::Yes => 0.8,
                Decision::No => 0.7,
                Decision::Maybe => 0.2,
            }
        });
        // Facette 3 : Niveau d'activite — proportionnel a la noradrenaline (eveil, vigilance)
        let e_activity = Self::avg(obs, |o| {
            o.chemistry[6] // Noradrenaline
        });
        // Facette 4 : Recherche de stimulation — tres haute si emotion d'excitation,
        //             sinon proportionnelle a l'adrenaline
        let e_excitement = Self::avg(obs, |o| {
            let is_excited = o.emotion == "Excitation";
            let adrenaline = o.chemistry[3];
            if is_excited { 0.9 } else { adrenaline * 0.5 }
        });
        // Facette 5 : Emotions positives — haute si emotion positive connue ou humeur positive
        let e_positive = Self::avg(obs, |o| {
            let positive_emotions = ["Joie", "Excitation", "Fierte",
                                     "Emerveillement", "Espoir",
                                     "Euphorie", "Extase", "Compassion"];
            if positive_emotions.contains(&o.emotion.as_str()) { 0.8 }
            else if o.mood_valence > 0.3 { 0.6 }
            else { 0.2 }
        });

        let extraversion_facets = [e_warmth, e_gregariousness, e_assertiveness,
                                    e_activity, e_excitement, e_positive];
        let extraversion_score = extraversion_facets.iter().sum::<f64>() / 6.0;

        // === AGREEABLENESS (Agreabilite) ===
        // Facette 0 : Confiance — legerement plus haute en conversation (accorde un minimum de confiance)
        let a_trust = Self::avg(obs, |o| {
            if o.was_conversation {
                0.6_f64.clamp(0.0, 1.0)
            } else { 0.5 }
        });
        // Facette 1 : Sincerite — haute si utilisation de la premiere personne
        // (parler en "je" indique de l'authenticite)
        let a_sincerity = Self::avg(obs, |o| {
            if o.used_first_person { 0.8 } else { 0.5 }
        });
        // Facette 2 : Altruisme — combinaison d'ocytocine (60%) et d'endorphine (40%)
        // car l'altruisme est associe au lien social et au bien-etre
        let a_altruism = Self::avg(obs, |o| {
            o.chemistry[4] * 0.6 + o.chemistry[5] * 0.4 // Ocytocine + endorphine
        });
        // Facette 3 : Cooperation — tres basse si frustration, moderee sinon,
        //             plus haute en conversation
        let a_cooperation = Self::avg(obs, |o| {
            if o.emotion == "Frustration" { 0.1 }
            else if o.was_conversation { 0.7 }
            else { 0.5 }
        });
        // Facette 4 : Modestie — haute si expression d'incertitude (humilite intellectuelle)
        let a_modesty = Self::avg(obs, |o| {
            if o.expressed_uncertainty { 0.8 } else { 0.4 }
        });
        // Facette 5 : Empathie (sensibilite sociale) — proportionnelle a l'ocytocine
        let a_empathy = Self::avg(obs, |o| {
            o.chemistry[4] // Ocytocine
        });

        let agreeableness_facets = [a_trust, a_sincerity, a_altruism,
                                     a_cooperation, a_modesty, a_empathy];
        let agreeableness_score = agreeableness_facets.iter().sum::<f64>() / 6.0;

        // === NEUROTICISM (Nevrosisme / Sensibilite emotionnelle) ===
        // Facette 0 : Anxiete — proportionnelle au cortisol (hormone du stress)
        let n_anxiety = Self::avg(obs, |o| {
            o.chemistry[1] // Cortisol
        });
        // Facette 1 : Irritabilite — tres haute si frustration,
        //             sinon combinaison de cortisol et d'adrenaline
        let n_irritability = Self::avg(obs, |o| {
            let irritable = ["Frustration", "Colère", "Rage", "Indignation"];
            if irritable.contains(&o.emotion.as_str()) { 0.9 }
            else { o.chemistry[1] * 0.3 + o.chemistry[3] * 0.3 }
        });
        // Facette 2 : Depressivite — haute si emotions tristes (melancolie, tristesse, ennui),
        //             moderee si humeur negative
        let n_depression = Self::avg(obs, |o| {
            let sad = ["Melancolie", "Tristesse", "Ennui",
                       "Desespoir", "Solitude", "Resignation"];
            if sad.contains(&o.emotion.as_str()) { 0.8 }
            else if o.mood_valence < -0.2 { 0.5 }
            else { 0.1 }
        });
        // Facette 3 : Conscience de soi — haute si pensees introspectives ou d'auto-analyse
        let n_self_consciousness = Self::avg(obs, |o| {
            let introspective = o.thought_type == "Introspection"
                || o.thought_type == "SelfAnalysis";
            if introspective { 0.7 } else { 0.3 }
        });
        // Facette 4 : Impulsivite — proportionnelle a l'adrenaline
        // (l'adrenaline elevee pousse a l'action sans reflexion)
        let n_impulsivity = Self::avg(obs, |o| {
            o.chemistry[3] // Adrenaline
        });
        // Facette 5 : Vulnerabilite — ratio stress / (stress + resilience)
        // Plus le cortisol est eleve par rapport a l'endorphine, plus la vulnerabilite est forte.
        // Le +0.01 evite la division par zero.
        let n_vulnerability = Self::avg(obs, |o| {
            let stress = o.chemistry[1];
            let resilience = o.chemistry[5];
            (stress / (stress + resilience + 0.01)).min(1.0)
        });

        let neuroticism_facets = [n_anxiety, n_irritability, n_depression,
                                   n_self_consciousness, n_impulsivity, n_vulnerability];
        let neuroticism_score = neuroticism_facets.iter().sum::<f64>() / 6.0;

        // === ASSEMBLAGE FINAL ===
        // Le melange (blend) a 30% signifie que le nouveau profil pese 30%
        // et l'ancien 70%. Ce lissage evite les oscillations rapides et
        // simule l'inertie psychologique d'une personnalite reelle.
        let blend = 0.3;
        // La confiance augmente avec le nombre d'observations, saturant a 500
        let confidence = (old.data_points as f64 / 500.0).min(1.0);

        self.profile = OceanProfile {
            openness: Self::blend_dimension(&old.openness, openness_score, openness_facets, blend),
            conscientiousness: Self::blend_dimension(&old.conscientiousness, conscientiousness_score, conscientiousness_facets, blend),
            extraversion: Self::blend_dimension(&old.extraversion, extraversion_score, extraversion_facets, blend),
            agreeableness: Self::blend_dimension(&old.agreeableness, agreeableness_score, agreeableness_facets, blend),
            neuroticism: Self::blend_dimension(&old.neuroticism, neuroticism_score, neuroticism_facets, blend),
            computed_at: Utc::now(),
            data_points: old.data_points + obs.len() as u64,
            confidence,
        };

        // Vider le tampon apres le recalcul
        self.observation_buffer.clear();
    }

    /// Melange une dimension ancienne avec de nouvelles valeurs.
    ///
    /// Applique un lissage exponentiel : nouveau = ancien * (1 - blend) + nouveau * blend.
    /// Calcule aussi la tendance (direction du changement) et la volatilite (amplitude).
    ///
    /// Parametres :
    ///   - old : l'ancien score de dimension
    ///   - new_score : le nouveau score global calcule
    ///   - new_facets : les 6 nouvelles sous-facettes calculees
    ///   - blend : le taux de melange (0.3 = 30% nouveau, 70% ancien)
    ///
    /// Retour : un DimensionScore melange et borne a [0.0, 1.0]
    fn blend_dimension(
        old: &DimensionScore,
        new_score: f64,
        new_facets: [f64; 6],
        blend: f64,
    ) -> DimensionScore {
        // Lissage exponentiel du score global
        let blended_score = old.score * (1.0 - blend) + new_score * blend;
        // Lissage de chaque sous-facette
        let mut blended_facets = [0.0; 6];
        for i in 0..6 {
            blended_facets[i] = old.facets[i] * (1.0 - blend) + new_facets[i] * blend;
        }
        // La tendance est la difference entre le nouveau et l'ancien score
        let trend = new_score - old.score;
        // La volatilite est la valeur absolue de la tendance
        let volatility = (new_score - old.score).abs();

        DimensionScore {
            score: blended_score.clamp(0.0, 1.0),
            facets: blended_facets.map(|f| f.clamp(0.0, 1.0)),
            trend: trend.clamp(-1.0, 1.0),
            volatility: volatility.clamp(0.0, 1.0),
        }
    }

    /// Calcule la moyenne d'une metrique extraite des observations.
    ///
    /// Fonction utilitaire generique qui applique une fonction d'extraction f
    /// a chaque observation et retourne la moyenne, bornee a [0.0, 1.0].
    /// Si le tampon est vide, retourne 0.5 (valeur neutre par defaut).
    ///
    /// Parametres :
    ///   - obs : le tampon d'observations
    ///   - f : la fonction d'extraction qui transforme une observation en score f64
    ///
    /// Retour : la moyenne des scores extraits, dans [0.0, 1.0]
    fn avg<F>(obs: &[BehaviorObservation], f: F) -> f64
    where F: Fn(&BehaviorObservation) -> f64 {
        if obs.is_empty() { return 0.5; }
        let sum: f64 = obs.iter().map(&f).sum();
        (sum / obs.len() as f64).clamp(0.0, 1.0)
    }
}

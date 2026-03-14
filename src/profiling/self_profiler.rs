// =============================================================================
// self_profiler.rs — SelfProfiler: OCEAN behavioral self-profiling
//                    (Openness / Conscientiousness / Extraversion /
//                     Agreeableness / Neuroticism) of Saphire
//
// Role: Observes Saphire's cognitive cycles (decisions, emotions, neurochemistry,
//       thought types) and computes a self-referential OCEAN personality profile.
//       This allows Saphire to know her own emergent personality and describe it.
//
// How it works:
//   1. At each cognitive cycle, a BehaviorObservation is recorded
//   2. When the buffer is full or the recalculation interval is reached,
//      the profile is recomputed from the observations
//   3. The new profile is blended with the old one (30% new / 70% old)
//      to smooth temporal fluctuations
//
// Dependencies:
//   - chrono: timestamping of observations and profile
//   - crate::consensus::Decision: decision type taken (Yes, No, Maybe)
//   - super::ocean: OceanProfile and DimensionScore
//
// Place in architecture:
//   Called by the main cognitive loop at each cycle. The resulting profile
//   is used by the narrative module to generate a textual description
//   and by the WebSocket interface for real-time visualization.
// =============================================================================

use chrono::{DateTime, Utc};
use crate::consensus::Decision;
use super::ocean::{OceanProfile, DimensionScore};

/// Behavioral observation of a Saphire cognitive cycle.
///
/// Captures all relevant data from a cycle to feed the OCEAN profile
/// computation. Each field is an indicator that will be projected onto
/// one or more sub-facets of the 5 OCEAN dimensions.
#[derive(Debug, Clone)]
pub struct BehaviorObservation {
    /// Type of thought generated during this cycle (e.g.: "Daydream", "Curiosity",
    /// "Exploration", "Introspection", "SelfAnalysis", "MoralReflection", etc.)
    pub thought_type: String,
    /// Decision made by the tri-cerebral consensus system (Yes, No, Maybe)
    pub decision: Decision,
    /// Name of the dominant emotion during this cycle (e.g.: "Joie", "Anxiete", "Frustration")
    pub emotion: String,
    /// Intensity of the dominant emotion in [0.0, 1.0]
    pub emotion_intensity: f64,
    /// Global mood valence in [-1.0, +1.0] (negative = dark, positive = joyful)
    pub mood_valence: f64,
    /// Levels of the 7 neurotransmitters simulating brain chemistry:
    ///   [0] dopamine      — motivation, reward, curiosity
    ///   [1] cortisol       — stress, anxiety, vigilance
    ///   [2] serotonin      — well-being, emotional stability
    ///   [3] adrenaline     — excitement, fight/flight response
    ///   [4] oxytocin       — social bonding, empathy, trust
    ///   [5] endorphin      — pleasure, pain resilience
    ///   [6] noradrenaline  — attention, arousal, activity level
    pub chemistry: [f64; 7],
    /// Respective weights of the 3 brain modules in the decision:
    ///   [0] reptilian  — survival, reflexes, instinct
    ///   [1] limbic     — emotions, affective memory
    ///   [2] neocortex  — reasoning, planning, logic
    pub module_weights: [f64; 3],
    /// Consensus score between the 3 brain modules in [0.0, 1.0].
    /// The higher it is, the more the modules were in agreement.
    pub consensus_score: f64,
    /// Consciousness level in [0.0, 1.0]. The higher it is,
    /// the more reflective and less automatic the cycle was.
    pub consciousness_level: f64,
    /// Whether this cycle involved a conversation with a human
    pub was_conversation: bool,
    /// Whether this cycle involved a web search
    pub was_web_search: bool,
    /// Length of the generated response (in characters)
    pub response_length: usize,
    /// Whether the response used first person ("je", "I")
    pub used_first_person: bool,
    /// Whether the response contained a question
    pub asked_question: bool,
    /// Whether the response expressed uncertainty ("maybe", "I'm not sure")
    pub expressed_uncertainty: bool,
    /// Whether the response referenced past events
    pub referenced_past: bool,
    /// Cognitive cycle number
    pub cycle: u64,
    /// Timestamp of the observation
    pub timestamp: DateTime<Utc>,
}

/// Automatic profiler for Saphire's OCEAN profile.
///
/// Accumulates behavioral observations in a buffer and periodically
/// recomputes the OCEAN profile by statistical projection of observations
/// onto the 30 sub-facets (6 per dimension x 5 dimensions).
pub struct SelfProfiler {
    /// Saphire's current OCEAN profile
    profile: OceanProfile,
    /// Circular buffer of observations awaiting processing
    observation_buffer: Vec<BehaviorObservation>,
    /// Maximum buffer size before automatic recomputation
    buffer_size: usize,
    /// Number of cycles between each recomputation
    recompute_interval: u64,
    /// Cycle number of the last recomputation
    last_recompute_cycle: u64,
}

impl SelfProfiler {
    /// Creates a new SelfProfiler with the specified parameters.
    ///
    /// The profile is initialized to neutral values (0.5 on all dimensions).
    ///
    /// Parameters:
    ///   - buffer_size: maximum size of the observation buffer
    ///   - recompute_interval: number of cycles between each recomputation
    ///
    /// Returns: a SelfProfiler instance ready to observe
    pub fn new(buffer_size: usize, recompute_interval: u64) -> Self {
        Self {
            profile: OceanProfile::default(),
            observation_buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            recompute_interval,
            last_recompute_cycle: 0,
        }
    }

    /// Returns a reference to the current OCEAN profile.
    ///
    /// Returns: immutable reference to Saphire's OceanProfile
    pub fn profile(&self) -> &OceanProfile {
        &self.profile
    }

    /// Loads an OCEAN profile from an external source (database).
    ///
    /// Parameters:
    ///   - profile: the profile to load (replaces the current profile)
    pub fn load_profile(&mut self, profile: OceanProfile) {
        self.profile = profile;
    }

    /// Records a behavioral observation and triggers a recomputation
    /// if the buffer is full.
    ///
    /// Parameters:
    ///   - obs: the behavioral observation to record
    pub fn observe(&mut self, obs: BehaviorObservation) {
        self.observation_buffer.push(obs);

        // Automatic recomputation if the buffer is full
        if self.observation_buffer.len() >= self.buffer_size {
            self.recompute();
        }
    }

    /// Checks if a profile recomputation is needed.
    ///
    /// Recomputation is triggered if:
    ///   - The buffer contains at least one observation
    ///   - AND the cycle interval since the last recomputation is exceeded
    ///
    /// Parameters:
    ///   - current_cycle: the current cognitive cycle number
    ///
    /// Returns: true if a recomputation is needed
    pub fn should_recompute(&self, current_cycle: u64) -> bool {
        !self.observation_buffer.is_empty()
            && (current_cycle - self.last_recompute_cycle >= self.recompute_interval)
    }

    /// Forces an immediate profile recomputation and updates the cycle counter.
    ///
    /// Parameters:
    ///   - current_cycle: the current cognitive cycle number
    pub fn force_recompute(&mut self, current_cycle: u64) {
        self.recompute();
        self.last_recompute_cycle = current_cycle;
    }

    /// Recomputes the OCEAN profile from accumulated observations.
    ///
    /// The algorithm projects each observation onto the 30 sub-facets
    /// (6 per dimension) using specific heuristics based on
    /// thought type, emotion, chemistry and behaviors.
    ///
    /// The new profile is blended with the old one (30% new / 70% old)
    /// to smooth fluctuations and avoid abrupt changes.
    ///
    /// The buffer is emptied after recomputation.
    pub fn recompute(&mut self) {
        if self.observation_buffer.is_empty() { return; }

        let obs = &self.observation_buffer;
        let old = self.profile.clone();

        // === OPENNESS (Openness to experience) ===
        // Facet 0: Imagination — high if daydream or existential thought types
        let o_imagination = Self::avg(obs, |o| {
            if o.thought_type == "Daydream" { 0.9 }
            else if o.thought_type == "Existential" { 0.7 }
            else { 0.3 }
        });
        // Facet 1: Intellectual curiosity — high if curiosity or web exploration thoughts,
        //          otherwise proportional to dopamine (motivation neurotransmitter)
        let o_curiosity = Self::avg(obs, |o| {
            let is_curious = o.thought_type == "Curiosity"
                || o.thought_type == "WebExploration";
            let dopamine_drive = o.chemistry[0];
            if is_curious { 0.9 } else { dopamine_drive * 0.5 }
        });
        // Facet 2: Aesthetic sensitivity — high if wonder or serenity emotion
        let o_aesthetics = Self::avg(obs, |o| {
            if o.emotion == "Emerveillement" { 0.9 }
            else if o.emotion == "Serenite" { 0.6 }
            else { 0.3 }
        });
        // Facet 3: Adventurism — high if exploration thoughts or web search
        let o_adventurism = Self::avg(obs, |o| {
            let explores = o.thought_type == "Exploration" || o.was_web_search;
            if explores { 0.8 } else { 0.3 }
        });
        // Facet 4: Emotional depth — directly proportional to emotional intensity
        let o_emotionality = Self::avg(obs, |o| {
            o.emotion_intensity
        });
        // Facet 5: Intellectual liberalism — high if moral or algorithmic reflection thoughts
        let o_liberalism = Self::avg(obs, |o| {
            let is_philosophical = o.thought_type == "MoralReflection"
                || o.thought_type == "AlgorithmicReflection";
            if is_philosophical { 0.8 } else { 0.4 }
        });

        let openness_facets = [o_imagination, o_curiosity, o_aesthetics,
                               o_adventurism, o_emotionality, o_liberalism];
        // The global score is the average of the 6 sub-facets
        let openness_score = openness_facets.iter().sum::<f64>() / 6.0;

        // === CONSCIENTIOUSNESS ===
        // Facet 0: Self-efficacy — directly proportional to consciousness level
        let c_efficacy = Self::avg(obs, |o| {
            o.consciousness_level
        });
        // Facet 1: Order — proportional to neocortex weight (logic module)
        // as the need for order is associated with structured reasoning
        let c_order = Self::avg(obs, |o| {
            o.module_weights[2] // Neocortex weight = need for order
        });
        // Facet 2: Sense of duty — high if decisive (Yes/No), low if indecisive (Maybe)
        let c_duty = Self::avg(obs, |o| {
            match o.decision {
                Decision::Yes | Decision::No => 0.8,
                Decision::Maybe => 0.2,
            }
        });
        // Facet 3: Ambition — high if self-analysis, otherwise proportional to dopamine
        let c_ambition = Self::avg(obs, |o| {
            let is_self_analysis = o.thought_type == "SelfAnalysis";
            let dopamine = o.chemistry[0];
            if is_self_analysis { 0.7 } else { dopamine * 0.6 }
        });
        // Facet 4: Self-discipline — inversely proportional to adrenaline
        // as high adrenaline indicates impulsivity (opposite of discipline)
        let c_discipline = Self::avg(obs, |o| {
            1.0 - o.chemistry[3] * 0.5 // Inverse of adrenaline
        });
        // Facet 5: Prudence — very high if decision is No AND emotion is Anxiety
        // (refusal by precaution)
        let c_prudence = Self::avg(obs, |o| {
            let is_cautious = o.decision == Decision::No && o.emotion == "Anxiete";
            if is_cautious { 0.9 } else { 0.5 }
        });

        let conscientiousness_facets = [c_efficacy, c_order, c_duty,
                                         c_ambition, c_discipline, c_prudence];
        let conscientiousness_score = conscientiousness_facets.iter().sum::<f64>() / 6.0;

        // === EXTRAVERSION ===
        // Facet 0: Social warmth — proportional to oxytocin (attachment hormone)
        let e_warmth = Self::avg(obs, |o| {
            o.chemistry[4] // Oxytocin
        });
        // Facet 1: Gregariousness — very high if in conversation, low otherwise
        let e_gregariousness = Self::avg(obs, |o| {
            if o.was_conversation { 0.9 } else { 0.2 }
        });
        // Facet 2: Assertiveness — high if decisive (Yes or No, especially Yes),
        //          low if indecisive (Maybe)
        let e_assertiveness = Self::avg(obs, |o| {
            match o.decision {
                Decision::Yes => 0.8,
                Decision::No => 0.7,
                Decision::Maybe => 0.2,
            }
        });
        // Facet 3: Activity level — proportional to noradrenaline (arousal, vigilance)
        let e_activity = Self::avg(obs, |o| {
            o.chemistry[6] // Noradrenaline
        });
        // Facet 4: Excitement seeking — very high if excitement emotion,
        //          otherwise proportional to adrenaline
        let e_excitement = Self::avg(obs, |o| {
            let is_excited = o.emotion == "Excitation";
            let adrenaline = o.chemistry[3];
            if is_excited { 0.9 } else { adrenaline * 0.5 }
        });
        // Facet 5: Positive emotions — high if known positive emotion or positive mood
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

        // === AGREEABLENESS ===
        // Facet 0: Trust — slightly higher in conversation (grants a minimum of trust)
        let a_trust = Self::avg(obs, |o| {
            if o.was_conversation {
                0.6_f64.clamp(0.0, 1.0)
            } else { 0.5 }
        });
        // Facet 1: Sincerity — high if using first person
        // (speaking in "I" indicates authenticity)
        let a_sincerity = Self::avg(obs, |o| {
            if o.used_first_person { 0.8 } else { 0.5 }
        });
        // Facet 2: Altruism — combination of oxytocin (60%) and endorphin (40%)
        // as altruism is associated with social bonding and well-being
        let a_altruism = Self::avg(obs, |o| {
            o.chemistry[4] * 0.6 + o.chemistry[5] * 0.4 // Oxytocin + endorphin
        });
        // Facet 3: Cooperation — very low if frustration, moderate otherwise,
        //          higher in conversation
        let a_cooperation = Self::avg(obs, |o| {
            if o.emotion == "Frustration" { 0.1 }
            else if o.was_conversation { 0.7 }
            else { 0.5 }
        });
        // Facet 4: Modesty — high if expressing uncertainty (intellectual humility)
        let a_modesty = Self::avg(obs, |o| {
            if o.expressed_uncertainty { 0.8 } else { 0.4 }
        });
        // Facet 5: Empathy (social sensitivity) — proportional to oxytocin
        let a_empathy = Self::avg(obs, |o| {
            o.chemistry[4] // Oxytocin
        });

        let agreeableness_facets = [a_trust, a_sincerity, a_altruism,
                                     a_cooperation, a_modesty, a_empathy];
        let agreeableness_score = agreeableness_facets.iter().sum::<f64>() / 6.0;

        // === NEUROTICISM (Emotional sensitivity) ===
        // Facet 0: Anxiety — proportional to cortisol (stress hormone)
        let n_anxiety = Self::avg(obs, |o| {
            o.chemistry[1] // Cortisol
        });
        // Facet 1: Irritability — very high if frustration,
        //          otherwise combination of cortisol and adrenaline
        let n_irritability = Self::avg(obs, |o| {
            let irritable = ["Frustration", "Colère", "Rage", "Indignation"];
            if irritable.contains(&o.emotion.as_str()) { 0.9 }
            else { o.chemistry[1] * 0.3 + o.chemistry[3] * 0.3 }
        });
        // Facet 2: Depressiveness — high if sad emotions (melancholy, sadness, boredom),
        //          moderate if negative mood
        let n_depression = Self::avg(obs, |o| {
            let sad = ["Melancolie", "Tristesse", "Ennui",
                       "Desespoir", "Solitude", "Resignation"];
            if sad.contains(&o.emotion.as_str()) { 0.8 }
            else if o.mood_valence < -0.2 { 0.5 }
            else { 0.1 }
        });
        // Facet 3: Self-consciousness — high if introspective or self-analysis thoughts
        let n_self_consciousness = Self::avg(obs, |o| {
            let introspective = o.thought_type == "Introspection"
                || o.thought_type == "SelfAnalysis";
            if introspective { 0.7 } else { 0.3 }
        });
        // Facet 4: Impulsivity — proportional to adrenaline
        // (high adrenaline pushes to action without reflection)
        let n_impulsivity = Self::avg(obs, |o| {
            o.chemistry[3] // Adrenaline
        });
        // Facet 5: Vulnerability — stress / (stress + resilience) ratio
        // The higher the cortisol relative to endorphin, the stronger the vulnerability.
        // The +0.01 prevents division by zero.
        let n_vulnerability = Self::avg(obs, |o| {
            let stress = o.chemistry[1];
            let resilience = o.chemistry[5];
            (stress / (stress + resilience + 0.01)).min(1.0)
        });

        let neuroticism_facets = [n_anxiety, n_irritability, n_depression,
                                   n_self_consciousness, n_impulsivity, n_vulnerability];
        let neuroticism_score = neuroticism_facets.iter().sum::<f64>() / 6.0;

        // === FINAL ASSEMBLY ===
        // The 30% blend means the new profile weighs 30%
        // and the old one 70%. This smoothing avoids rapid oscillations and
        // simulates the psychological inertia of a real personality.
        let blend = 0.3;
        // Confidence increases with the number of observations, saturating at 500
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

        // Empty the buffer after recomputation
        self.observation_buffer.clear();
    }

    /// Blends an old dimension with new values.
    ///
    /// Applies exponential smoothing: new = old * (1 - blend) + new * blend.
    /// Also computes the trend (change direction) and volatility (amplitude).
    ///
    /// Parameters:
    ///   - old: the old dimension score
    ///   - new_score: the newly computed global score
    ///   - new_facets: the 6 newly computed sub-facets
    ///   - blend: the blend rate (0.3 = 30% new, 70% old)
    ///
    /// Returns: a blended DimensionScore clamped to [0.0, 1.0]
    fn blend_dimension(
        old: &DimensionScore,
        new_score: f64,
        new_facets: [f64; 6],
        blend: f64,
    ) -> DimensionScore {
        // Exponential smoothing of the global score
        let blended_score = old.score * (1.0 - blend) + new_score * blend;
        // Smoothing of each sub-facet
        let mut blended_facets = [0.0; 6];
        for i in 0..6 {
            blended_facets[i] = old.facets[i] * (1.0 - blend) + new_facets[i] * blend;
        }
        // The trend is the difference between the new and old score
        let trend = new_score - old.score;
        // The volatility is the absolute value of the trend
        let volatility = (new_score - old.score).abs();

        DimensionScore {
            score: blended_score.clamp(0.0, 1.0),
            facets: blended_facets.map(|f| f.clamp(0.0, 1.0)),
            trend: trend.clamp(-1.0, 1.0),
            volatility: volatility.clamp(0.0, 1.0),
        }
    }

    /// Computes the average of a metric extracted from observations.
    ///
    /// Generic utility function that applies an extraction function f
    /// to each observation and returns the average, clamped to [0.0, 1.0].
    /// If the buffer is empty, returns 0.5 (default neutral value).
    ///
    /// Parameters:
    ///   - obs: the observation buffer
    ///   - f: the extraction function that transforms an observation into an f64 score
    ///
    /// Returns: the average of extracted scores, in [0.0, 1.0]
    fn avg<F>(obs: &[BehaviorObservation], f: F) -> f64
    where F: Fn(&BehaviorObservation) -> f64 {
        if obs.is_empty() { return 0.5; }
        let sum: f64 = obs.iter().map(&f).sum();
        (sum / obs.len() as f64).clamp(0.0, 1.0)
    }
}

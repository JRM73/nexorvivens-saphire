// =============================================================================
// human_profiler.rs — HumanProfiler : profilage psychologique des interlocuteurs
//
// Role : Analyse les messages des interlocuteurs humains pour construire et
//        mettre a jour leur profil psychologique OCEAN (Openness /
//        Conscientiousness / Extraversion / Agreeableness / Neuroticism)
//        et leur style de communication.
//
// Fonctionnement :
//   A chaque message humain, le profiler met a jour :
//     1. Le style de communication (verbosite, formalite, emotionnalite, etc.)
//     2. Les dimensions OCEAN (par inference a partir du contenu et du style)
//     3. Les patterns emotionnels (historique des polarites de sentiment)
//     4. Le score de rapport (qualite de la relation)
//
//   Le lissage exponentiel (80% ancien + 20% nouveau) est utilise pour le style
//   de communication, et (90% ancien + 10% nouveau) pour les dimensions OCEAN,
//   afin d'eviter les fluctuations brusques.
//
// Dependances :
//   - std::collections::HashMap : stockage des profils et des patterns emotionnels
//   - chrono : horodatage des interactions
//   - serde : serialisation pour la persistance
//   - crate::nlp : NlpResult (resultat de l'analyse NLP du message)
//   - crate::nlp::intent : Intent (intention detectee du message)
//   - super::ocean : OceanProfile
//
// Place dans l'architecture :
//   Appele par la boucle cognitive principale quand un message humain est recu.
//   Le profil humain produit est consomme par le module adaptation pour generer
//   des instructions de style adaptees a l'interlocuteur.
// =============================================================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::nlp::NlpResult;
use crate::nlp::intent::Intent;
use super::ocean::OceanProfile;

/// Profil psychologique d'un interlocuteur humain construit par observation.
///
/// Accumule les donnees au fil des interactions pour affiner progressivement
/// le portrait psychologique et le style de communication de l'humain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanProfile {
    /// Nom ou identifiant de l'humain profile
    pub name: String,
    /// Profil OCEAN (Big Five) estime par observation des messages
    pub ocean: OceanProfile,
    /// Style de communication observe (verbosite, formalite, etc.)
    pub communication_style: CommunicationStyle,
    /// Nombre total d'interactions enregistrees avec cet humain
    pub interaction_count: u64,
    /// Horodatage de la premiere interaction
    pub first_seen: DateTime<Utc>,
    /// Horodatage de la derniere interaction
    pub last_seen: DateTime<Utc>,
    /// Liste des sujets preferes detectes dans les conversations
    pub preferred_topics: Vec<String>,
    /// Historique des patterns emotionnels : cle = label de polarite
    /// ("positif", "negatif", "neutre"), valeur = nombre d'occurrences
    pub emotional_patterns: HashMap<String, u32>,
    /// Score de rapport dans [0.0, 1.0] : qualite estimee de la relation.
    /// Augmente quand les interactions sont positives, stagne ou diminue sinon.
    pub rapport_score: f64,
}

impl HumanProfile {
    /// Cree un nouveau profil humain avec les valeurs par defaut.
    ///
    /// Le profil OCEAN est initialise a neutre (0.5), le style de communication
    /// est a la valeur mediane, et le rapport est a 0.5.
    ///
    /// Parametres :
    ///   - name : le nom ou identifiant de l'humain
    ///
    /// Retour : un HumanProfile initialise
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ocean: OceanProfile::default(),
            communication_style: CommunicationStyle::default(),
            interaction_count: 0,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            preferred_topics: Vec::new(),
            emotional_patterns: HashMap::new(),
            rapport_score: 0.5,
        }
    }
}

/// Style de communication observe d'un humain.
///
/// Chaque dimension est une valeur dans [0.0, 1.0] qui est mise a jour
/// par lissage exponentiel (80% ancien + 20% nouveau) a chaque message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Verbosite : tendance a ecrire des messages longs (0 = tres concis, 1 = tres verbeux).
    /// Calculee a partir du nombre de mots par message, normalisee a 50 mots = 1.0.
    pub verbosity: f64,
    /// Formalite : registre de langue (0 = tres familier/tutoiement, 1 = tres formel/vouvoiement).
    /// Estimee par le ratio marqueurs formels / (marqueurs formels + informels).
    pub formality: f64,
    /// Emotionnalite : tendance a exprimer des emotions dans les messages.
    /// Basee sur la valeur absolue du score compose de sentiment (compound).
    pub emotionality: f64,
    /// Directivite : tendance a donner des ordres ou des instructions directes.
    /// Haute si le message commence par un verbe imperatif ou si l'intention est Command.
    pub directness: f64,
    /// Taux de questionnement : proportion de messages contenant des questions.
    /// 1.0 si le message contient '?', 0.0 sinon.
    pub questioning_rate: f64,
    /// Langue preferee de l'interlocuteur ("fr" pour francais, "en" pour anglais).
    pub preferred_language: String,
}

impl Default for CommunicationStyle {
    /// Style de communication par defaut : toutes les dimensions a 0.5 (neutre),
    /// langue preferee = francais.
    fn default() -> Self {
        Self {
            verbosity: 0.5,
            formality: 0.5,
            emotionality: 0.5,
            directness: 0.5,
            questioning_rate: 0.5,
            preferred_language: "fr".into(),
        }
    }
}

/// Profiler des interlocuteurs humains interagissant avec Saphire.
///
/// Maintient un dictionnaire de profils indexes par identifiant humain.
/// Chaque nouveau message est analyse pour mettre a jour le profil correspondant.
pub struct HumanProfiler {
    /// Dictionnaire des profils humains : cle = identifiant, valeur = profil
    profiles: HashMap<String, HumanProfile>,
}

impl Default for HumanProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl HumanProfiler {
    /// Cree un nouveau HumanProfiler avec un dictionnaire vide.
    ///
    /// Retour : une instance de HumanProfiler prete a observer
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Retourne le profil d'un humain identifie par son ID.
    ///
    /// Parametres :
    ///   - human_id : l'identifiant de l'humain
    ///
    /// Retour : Some(&HumanProfile) si le profil existe, None sinon
    pub fn get_profile(&self, human_id: &str) -> Option<&HumanProfile> {
        self.profiles.get(human_id)
    }

    /// Retourne le profil actif (l'humain vu le plus recemment).
    ///
    /// Utile pour obtenir le profil de l'interlocuteur courant sans connaitre
    /// son identifiant.
    ///
    /// Retour : Some(&HumanProfile) du profil le plus recent, None si aucun profil
    pub fn current_profile(&self) -> Option<&HumanProfile> {
        // Selectionner le profil avec le last_seen le plus recent
        self.profiles.values()
            .max_by_key(|p| p.last_seen)
    }

    /// Charge des profils depuis une source externe (base de donnees).
    ///
    /// Parametres :
    ///   - profiles : liste de tuples (identifiant, profil) a charger
    pub fn load_profiles(&mut self, profiles: Vec<(String, HumanProfile)>) {
        for (id, profile) in profiles {
            self.profiles.insert(id, profile);
        }
    }

    /// Retourne une reference vers le dictionnaire complet des profils.
    ///
    /// Utile pour la sauvegarde periodique en base de donnees.
    ///
    /// Retour : reference vers le HashMap de tous les profils
    pub fn all_profiles(&self) -> &HashMap<String, HumanProfile> {
        &self.profiles
    }

    /// Analyse un message humain et met a jour le profil de l'interlocuteur.
    ///
    /// Le traitement se fait en 4 etapes :
    ///   1. Mise a jour du style de communication (verbosite, formalite, etc.)
    ///   2. Estimation des dimensions OCEAN par inference
    ///   3. Mise a jour des patterns emotionnels
    ///   4. Mise a jour du score de rapport
    ///
    /// Parametres :
    ///   - human_id : l'identifiant de l'humain
    ///   - message : le texte brut du message
    ///   - nlp_result : le resultat de l'analyse NLP du message
    pub fn observe_message(
        &mut self,
        human_id: &str,
        message: &str,
        nlp_result: &NlpResult,
    ) {
        // Obtenir ou creer le profil de l'humain
        let profile = self.profiles.entry(human_id.to_string())
            .or_insert_with(|| HumanProfile::new(human_id));

        profile.interaction_count += 1;
        profile.last_seen = Utc::now();

        // === Etape 1 : Estimer le style de communication ===
        let word_count = message.split_whitespace().count() as f64;
        let style = &mut profile.communication_style;

        // Verbosite : nombre de mots normalise a 50 (50 mots = verbosite maximale)
        // Lissage exponentiel : 80% ancien + 20% nouveau
        let msg_verbosity = (word_count / 50.0).min(1.0);
        style.verbosity = style.verbosity * 0.8 + msg_verbosity * 0.2;

        // Formalite : ratio marqueurs formels / (formels + informels)
        // Marqueurs formels : vouvoiement, formules de politesse
        // Marqueurs informels : tutoiement, argot, abreviations
        let formal_markers = ["vous", "veuillez", "cordialement", "merci de",
                              "pourriez-vous", "je vous prie", "would you"];
        let informal_markers = ["tu", "salut", "cool", "mdr", "lol", "hey",
                                "ouais", "trop", "grave"];
        let lower = message.to_lowercase();
        let formal_count = formal_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f64;
        let informal_count = informal_markers.iter()
            .filter(|m| lower.contains(*m)).count() as f64;
        let msg_formality = if formal_count + informal_count > 0.0 {
            formal_count / (formal_count + informal_count)
        } else { 0.5 };
        style.formality = style.formality * 0.8 + msg_formality * 0.2;

        // Emotionnalite : basee sur la valeur absolue du score compose de sentiment
        let msg_emotionality = nlp_result.sentiment.compound.abs();
        style.emotionality = style.emotionality * 0.8 + msg_emotionality * 0.2;

        // Directivite : haute si le message commence par un verbe imperatif
        // ou si l'intention NLP detectee est un ordre (Command)
        let directive_markers = ["fais", "cree", "montre", "donne", "change",
                                 "arrete", "do", "make", "show", "give"];
        let is_directive = directive_markers.iter()
            .any(|m| lower.starts_with(m))
            || nlp_result.intent.primary_intent == Intent::Command;
        let msg_directness = if is_directive { 0.9 } else { 0.4 };
        style.directness = style.directness * 0.8 + msg_directness * 0.2;

        // Taux de questionnement : 1.0 si le message contient '?', 0.0 sinon
        let has_question = message.contains('?');
        let msg_questioning = if has_question { 1.0 } else { 0.0 };
        style.questioning_rate = style.questioning_rate * 0.8 + msg_questioning * 0.2;

        // Langue preferee : mise a jour si la langue est detectee
        style.preferred_language = match nlp_result.language {
            crate::nlp::preprocessor::Language::French => "fr".into(),
            crate::nlp::preprocessor::Language::English => "en".into(),
            crate::nlp::preprocessor::Language::Unknown => style.preferred_language.clone(),
        };

        // === Etape 2 : Estimer les dimensions OCEAN de l'humain ===
        // Les estimations utilisent un lissage plus lent (90% ancien + 10% nouveau)
        // car le profil OCEAN doit etre stable dans le temps.
        let ocean = &mut profile.ocean;

        // Ouverture (Openness) : augmente si le message est philosophique ou interrogatif,
        // car la curiosite et la reflexion sont des marqueurs d'ouverture.
        if nlp_result.intent.primary_intent == Intent::Philosophical
           || nlp_result.intent.primary_intent == Intent::Question {
            ocean.openness.score = (ocean.openness.score * 0.9 + 0.8 * 0.1).min(1.0);
        }

        // Extraversion : augmente si le message est long (> 30 mots) ou contient des '!',
        // car l'expressivite et l'enthousiasme sont des marqueurs d'extraversion.
        if word_count > 30.0 || message.contains('!') {
            ocean.extraversion.score = (ocean.extraversion.score * 0.9 + 0.7 * 0.1).min(1.0);
        }

        // Agreabilite : augmente si le message contient des mots chaleureux
        // (remerciements, compliments, confiance).
        let warm_words = ["merci", "bravo", "super", "bien", "confiance",
                          "j'apprecie", "thanks", "great", "love"];
        if warm_words.iter().any(|w| lower.contains(w)) {
            ocean.agreeableness.score = (ocean.agreeableness.score * 0.9 + 0.8 * 0.1).min(1.0);
        }

        // Nevrosisme (Neuroticism) : augmente si le stimulus du message a une
        // forte urgence (> 0.5) ou un danger significatif (> 0.3), car l'anxiete
        // et le stress sont des marqueurs de nevrosisme.
        if nlp_result.stimulus.urgency > 0.5 || nlp_result.stimulus.danger > 0.3 {
            ocean.neuroticism.score = (ocean.neuroticism.score * 0.9 + 0.6 * 0.1).min(1.0);
        }

        // === Etape 3 : Patterns emotionnels ===
        // Classe le sentiment du message en 3 categories et incremente le compteur
        let polarity_label = if nlp_result.sentiment.compound > 0.2 { "positif" }
            else if nlp_result.sentiment.compound < -0.2 { "negatif" }
            else { "neutre" };
        let emotion_entry = profile.emotional_patterns
            .entry(polarity_label.to_string()).or_insert(0);
        *emotion_entry += 1;

        // === Etape 4 : Score de rapport ===
        // Le rapport augmente legerement (+0.02) a chaque interaction positive.
        // Il est plafonne a 1.0. Les interactions negatives ne diminuent pas le rapport
        // (stabilite relationnelle).
        if nlp_result.sentiment.compound > 0.0 {
            profile.rapport_score = (profile.rapport_score + 0.02).min(1.0);
        }
    }
}

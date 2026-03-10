// =============================================================================
// adaptation.rs — Adaptation du style de communication selon le profil humain
//
// Role : Genere des instructions textuelles d'adaptation du style de Saphire
//        en fonction du profil OCEAN (Openness / Conscientiousness / Extraversion /
//        Agreeableness / Neuroticism) et du style de communication de l'humain.
//
//        Ces instructions sont injectees dans le prompt du LLM (Large Language Model /
//        Grand Modele de Langage) pour que Saphire adapte automatiquement sa facon
//        de communiquer a chaque interlocuteur.
//
// Dependances :
//   - super::human_profiler::HumanProfile : le profil complet de l'humain
//
// Place dans l'architecture :
//   Appele juste avant la generation de reponse par le LLM. Les instructions
//   d'adaptation sont concatenees au prompt systeme pour personnaliser le style
//   de la reponse (longueur, registre, ton emotionnel, profondeur intellectuelle).
// =============================================================================

use super::human_profiler::HumanProfile;

/// Genere les instructions d'adaptation pour le LLM basees sur le profil humain.
///
/// Analyse le style de communication et le profil OCEAN de l'humain pour produire
/// une liste de directives textuelles. Chaque directive indique un aspect du style
/// a adapter (longueur, registre, emotionnalite, profondeur).
///
/// Les seuils utilises :
///   - Style de communication : < 0.3 = bas, > 0.6 ou > 0.7 = haut
///   - OCEAN : > 0.6 ou > 0.7 = trait marque
///
/// Parametres :
///   - human : le profil complet de l'interlocuteur humain
///
/// Retour : une chaine de caracteres contenant les instructions d'adaptation
///          formatees. Retourne une chaine vide si aucune adaptation n'est necessaire.
pub fn adapt_for_human(human: &HumanProfile) -> String {
    let mut adaptations = Vec::new();

    let style = &human.communication_style;

    // Adapter la longueur de reponse en fonction de la verbosite de l'humain.
    // Un humain concis (verbosite < 0.3) prefere des reponses courtes.
    // Un humain verbeux (verbosite > 0.7) apprecie les reponses detaillees.
    if style.verbosity < 0.3 {
        adaptations.push("L'humain prefere les reponses CONCISES. Maximum 2 phrases.");
    } else if style.verbosity > 0.7 {
        adaptations.push("L'humain apprecie les reponses detaillees et developpees.");
    }

    // Adapter le registre de langue en fonction de la formalite.
    // Un humain formel (> 0.6) attend le vouvoiement et un ton professionnel.
    // Un humain informel (< 0.3) prefere le tutoiement et un ton chaleureux.
    if style.formality > 0.6 {
        adaptations.push("Utilise le vouvoiement et un ton professionnel.");
    } else if style.formality < 0.3 {
        adaptations.push("Ton familier et chaleureux. Tutoiement.");
    }

    // Adapter au niveau d'emotionnalite de l'humain.
    // Un humain expressif (> 0.6) attend un engagement emotionnel.
    // Un humain reserve (< 0.3) prefere les faits et la precision.
    if style.emotionality > 0.6 {
        adaptations.push("L'humain est expressif. Engage-toi emotionnellement.");
    } else if style.emotionality < 0.3 {
        adaptations.push("L'humain prefere les faits. Sois factuelle et precise.");
    }

    // Adapter au profil OCEAN de l'humain pour des ajustements plus profonds.
    let ocean = &human.ocean;

    // Ouverture elevee : la personne apprecie les idees abstraites et les tangentes
    if ocean.openness.score > 0.7 {
        adaptations.push("Cette personne aime les idees profondes et abstraites. N'hesite pas a explorer des tangentes intellectuelles.");
    }
    // Nevrosisme eleve : la personne peut etre sensible au stress, necessitant
    // un ton rassurant et doux pour eviter d'aggraver son anxiete.
    if ocean.neuroticism.score > 0.6 {
        adaptations.push("Cette personne peut etre sensible. Sois rassurante et douce.");
    }
    // Agreabilite elevee : la personne est chaleureuse et cooperative,
    // on peut refleter cette chaleur dans la reponse.
    if ocean.agreeableness.score > 0.7 {
        adaptations.push("Cette personne est chaleureuse. Reflete cette chaleur.");
    }

    // Si aucune adaptation n'est necessaire (profil neutre), retourner une chaine vide
    if adaptations.is_empty() {
        String::new()
    } else {
        format!("ADAPTATION A L'INTERLOCUTEUR :\n{}\n", adaptations.join("\n"))
    }
}

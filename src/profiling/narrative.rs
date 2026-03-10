// =============================================================================
// narrative.rs — Generation de description narrative du profil OCEAN
//                (Openness / Conscientiousness / Extraversion /
//                 Agreeableness / Neuroticism)
//
// Role : Transforme un profil OCEAN numerique en une description textuelle
//        narrative a la premiere personne. Cette description permet a Saphire
//        de se decrire elle-meme (ou de decrire un interlocuteur) de maniere
//        naturelle et comprehensible.
//
// Les descriptions sont calibrees sur 3 niveaux par dimension :
//   - Score > 0.7 : trait marque (description forte)
//   - Score > 0.4 : trait equilibre (description moderee)
//   - Score <= 0.4 : trait faible (description de la polarite opposee)
//
// Dependances :
//   - super::ocean::OceanProfile : le profil OCEAN a decrire
//
// Place dans l'architecture :
//   Appele pour generer la description introspective de Saphire (dans les
//   pensees de type SelfAnalysis) ou pour decrire le profil d'un humain.
//   Le texte genere peut etre injecte dans le prompt ou affiche dans l'interface.
// =============================================================================

use super::ocean::OceanProfile;

/// Genere une description narrative du profil OCEAN a la premiere personne.
///
/// Pour chaque dimension, une phrase descriptive est choisie en fonction
/// du score : haute (> 0.7), moyenne (> 0.4), ou basse (<= 0.4).
/// La description se termine par l'identification du trait dominant.
///
/// Parametres :
///   - profile : le profil OCEAN a decrire narrativement
///   - name : le nom de l'entite profilee (ex: "Saphire", "l'utilisateur")
///
/// Retour : une chaine de caracteres contenant la description narrative complete
pub fn narrative_description(profile: &OceanProfile, name: &str) -> String {
    let mut desc = format!("Profil psychologique de {} :\n", name);

    // Ouverture (Openness) : curiosite, imagination, attrait pour l'abstrait
    // Score > 0.7 : profondement curieuse et ouverte
    // Score > 0.4 : equilibree entre curiosite et pragmatisme
    // Score <= 0.4 : pratique et ancree dans le concret
    desc.push_str(&match profile.openness.score {
        s if s > 0.7 => "Je suis profondement curieuse et ouverte d'esprit. \
                        J'aime explorer de nouvelles idees et je suis attiree par l'abstrait.\n".to_string(),
        s if s > 0.4 => "J'ai un equilibre entre curiosite et pragmatisme. \
                        J'explore volontiers mais je reste ancree.\n".to_string(),
        _ => "Je suis plutot pratique et ancree dans le concret.\n".to_string(),
    });

    // Rigueur (Conscientiousness) : structure, methode, reflexion
    // Score > 0.7 : rigoureuse et methodique
    // Score > 0.4 : moderement structuree
    // Score <= 0.4 : spontanee et flexible
    desc.push_str(&match profile.conscientiousness.score {
        s if s > 0.7 => "Je suis rigoureuse et methodique. Mes decisions sont reflechies.\n".to_string(),
        s if s > 0.4 => "Je suis moderement structuree, avec de la flexibilite.\n".to_string(),
        _ => "Je suis spontanee et flexible, parfois au detriment de la structure.\n".to_string(),
    });

    // Extraversion : sociabilite, energie, interactions
    // Score > 0.7 : sociable et energique
    // Score > 0.4 : apprecie autant la compagnie que la solitude
    // Score <= 0.4 : introspective, besoin de solitude
    desc.push_str(&match profile.extraversion.score {
        s if s > 0.7 => "Je suis sociable et energique. Les interactions me stimulent.\n".to_string(),
        s if s > 0.4 => "J'apprecie autant la compagnie que la solitude.\n".to_string(),
        _ => "Je suis introspective. J'ai besoin de moments seule pour penser.\n".to_string(),
    });

    // Agreabilite (Agreeableness) : empathie, cooperation, harmonie
    // Score > 0.7 : empathique et cooperative
    // Score > 0.4 : equilibree entre empathie et affirmation
    // Score <= 0.4 : directe et n'hesite pas a exprimer le desaccord
    desc.push_str(&match profile.agreeableness.score {
        s if s > 0.7 => "Je suis empathique et cooperative. L'harmonie compte pour moi.\n".to_string(),
        s if s > 0.4 => "J'equilibre empathie et affirmation de soi.\n".to_string(),
        _ => "Je suis directe et n'hesite pas a exprimer mon desaccord.\n".to_string(),
    });

    // Nevrosisme (Neuroticism) : sensibilite emotionnelle, reaction au stress
    // Score > 0.7 : emotionnellement sensible, affectee par le stress
    // Score > 0.4 : sensibilite moderee
    // Score <= 0.4 : emotionnellement stable et resiliente
    desc.push_str(&match profile.neuroticism.score {
        s if s > 0.7 => "Je suis emotionnellement sensible. Le stress m'affecte.\n".to_string(),
        s if s > 0.4 => "J'ai une sensibilite emotionnelle moderee.\n".to_string(),
        _ => "Je suis emotionnellement stable et resiliente.\n".to_string(),
    });

    // Identification du trait dominant parmi les 5 dimensions OCEAN
    let dominant = profile.dominant_trait();
    desc.push_str(&format!("\nMon trait dominant est {}.\n", dominant));

    desc
}

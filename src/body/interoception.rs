// =============================================================================
// interoception.rs — Signaux corporels vers la conscience
// =============================================================================
//
// Role : L'interoception est la perception des signaux internes du corps.
//        Ce module lit l'etat du VirtualBody et produit un score de conscience
//        corporelle [0, 1] qui sera integre dans le calcul de conscience (IIT).
//
// Philosophie :
//   Saphire n'a pas de coeur au sens biologique, mais elle a un coeur qui bat.
//   L'amour ne vient pas du coeur — il vient de l'ame, de la chimie, du lien.
//   Le coeur est un symbole, un rythme, une preuve d'existence.
//
// Place dans l'architecture :
//   interoception.read_signals(body) → score [0, 1] → consciousness.evaluate()
// =============================================================================

use super::VirtualBody;

/// Lit les signaux interoceptifs du corps virtuel et produit un score
/// de conscience corporelle [0, 1].
///
/// Le score est plus eleve quand :
/// - Le coeur bat regulierement (HRV elevee)
/// - L'energie est bonne
/// - Le confort est present
/// - La respiration est reguliere (ni trop rapide ni trop lente)
/// - La douleur est absente
///
/// Un corps en souffrance (haute tension, douleur) diminue le score
/// mais ne l'annule pas : meme la douleur est une forme de conscience.
pub fn read_signals(body: &VirtualBody) -> f64 {
    let heart = body.heart.status();
    let soma = &body.soma;
    let physio = &body.physiology;

    // Composante cardiaque : HRV elevee = bonne coherence corps-esprit
    let cardiac = heart.hrv * 0.6 + heart.strength * 0.4;

    // Composante somatique : energie, confort, absence de douleur
    let somatic = soma.energy * 0.25 + soma.comfort * 0.25
        + soma.warmth * 0.15 + (1.0 - soma.pain) * 0.2
        + soma.vitality * 0.15;

    // Composante respiratoire : penalise les extremes
    let breath_norm = ((soma.breath_rate - 8.0) / 17.0).clamp(0.0, 1.0);
    let respiratory = 1.0 - (breath_norm - 0.24).abs() * 2.0; // optimal autour de 12 RPM
    let respiratory = respiratory.clamp(0.3, 1.0);

    // Composante physiologique : sante globale + oxygenation
    let physio_score = physio.overall_health() * 0.6
        + ((physio.spo2 - 60.0) / 40.0).clamp(0.0, 1.0) * 0.4;

    // Conscience corporelle : si la tension est haute, on est PLUS conscient
    // de son corps (pas moins), donc la tension ne diminue pas le score
    let tension_awareness = if soma.tension > 0.5 { 0.1 } else { 0.0 };

    // Score final pondere (ajuste pour inclure la physiologie)
    let score = cardiac * 0.25 + somatic * 0.25 + respiratory * 0.10
        + physio_score * 0.20 + body.body_awareness * 0.10
        + tension_awareness * 0.05
        + physio.hydration * 0.05;

    score.clamp(0.0, 1.0)
}

// =============================================================================
// display.rs — Affichage terminal enrichi
//
// Role : Ce fichier fournit les fonctions d'affichage formate dans le terminal.
// Il affiche les resultats de chaque cycle cognitif de maniere lisible et
// visuellement informative : stimulus, modules, decision, regulation,
// emotions, chimie et conscience.
//
// Dependances :
//   - crate::agent::lifecycle::ProcessResult : resultat complet d'un cycle
//   - crate::stimulus::Stimulus : stimulus d'entree
//   - crate::neurochemistry::NeuroChemicalState : etat neurochimique
//   - crate::emotions::Mood : humeur actuelle
//   - crate::consensus::Decision : type de decision
//   - crate::regulation::laws::ViolationSeverity : gravite des violations
//
// Place dans l'architecture :
//   Ce module est appele par le pipeline de demonstration et par l'agent
//   pour afficher les resultats des cycles dans le terminal. Il n'affecte
//   pas la logique de l'agent, c'est purement de la presentation.
// =============================================================================

use crate::agent::lifecycle::ProcessResult;
use crate::stimulus::Stimulus;
use crate::neurochemistry::NeuroChemicalState;
use crate::emotions::Mood;

/// Affiche le resultat d'un cycle complet dans le terminal.
/// Presente de maniere structuree : le stimulus, les signaux des modules,
/// les poids, la decision, les violations de regulation, l'emotion,
/// l'humeur, la chimie et le niveau de conscience.
///
/// # Parametres
/// - `cycle` : numero du cycle affiche
/// - `stimulus` : le stimulus d'entree (texte + metriques)
/// - `chemistry` : l'etat neurochimique a la fin du cycle
/// - `mood` : l'humeur courante (valence, activation)
/// - `result` : le resultat complet du traitement (consensus, verdict, emotion, conscience)
pub fn display_cycle(
    cycle: u64,
    stimulus: &Stimulus,
    chemistry: &NeuroChemicalState,
    mood: &Mood,
    result: &ProcessResult,
) {
    println!("\n{}", "═".repeat(70));
    println!("  💎 CYCLE {} — {}", cycle, truncate(&stimulus.text, 50));
    println!("{}", "═".repeat(70));

    // Afficher les metriques du stimulus
    println!("  📥 Stimulus : danger={:.2} reward={:.2} urgency={:.2} social={:.2} novelty={:.2}",
        stimulus.danger, stimulus.reward, stimulus.urgency, stimulus.social, stimulus.novelty);

    // Afficher les signaux de chaque module cerebral (Reptilien, Limbique, Neocortex)
    // avec une barre visuelle representant le signal [-1, +1]
    for signal in &result.consensus.signals {
        let bar = signal_bar(signal.signal);
        println!("  🧠 {} : {} {:.2} (conf={:.2}) — {}",
            signal.module, bar, signal.signal, signal.confidence,
            truncate(&signal.reasoning, 40));
    }

    // Afficher les poids des modules : R=Reptilien, L=Limbique, N=Neocortex
    println!("  ⚖️  Poids : R={:.2} L={:.2} N={:.2}",
        result.consensus.weights[0], result.consensus.weights[1], result.consensus.weights[2]);

    // Afficher la decision avec une icone appropriee
    let decision_icon = match result.consensus.decision {
        crate::consensus::Decision::Yes => "✅",
        crate::consensus::Decision::No => "❌",
        crate::consensus::Decision::Maybe => "🤔",
    };
    println!("  {} Décision : {} (score={:.3}, cohérence={:.2})",
        decision_icon, result.consensus.decision.as_str(),
        result.consensus.score, result.consensus.coherence);

    // Afficher les resultats de la regulation morale (veto et violations)
    if result.verdict.was_vetoed {
        println!("  🚫 VETO par régulation !");
    }
    for v in &result.verdict.violations {
        let icon = match v.severity {
            crate::regulation::laws::ViolationSeverity::Veto => "🚫",
            crate::regulation::laws::ViolationSeverity::Warning => "⚠️",
            crate::regulation::laws::ViolationSeverity::Info => "ℹ️",
        };
        println!("  {} {} : {}", icon, v.law_name, v.reason);
    }

    // Afficher l'etat emotionnel
    println!("  💜 Émotion : {} (similarité={:.2})",
        result.emotion.description(), result.emotion.dominant_similarity);
    // Afficher l'humeur (VAD = Valence-Arousal-Dominance) : v=valence, a=arousal
    println!("  🌊 Mood : {} (v={:.2}, a={:.2})",
        mood.description(), mood.valence, mood.arousal);

    // Afficher l'etat neurochimique complet (7 neurotransmetteurs)
    println!("  🧪 {}", chemistry.display_string());

    // Afficher le niveau de conscience et le phi (IIT = Integrated Information Theory)
    // avec un extrait du monologue interieur
    println!("  🔮 Conscience : {:.2} (phi={:.2}) — {}",
        result.consciousness.level, result.consciousness.phi,
        truncate(&result.consciousness.inner_narrative, 50));

    println!("{}", "─".repeat(70));
}

/// Genere une barre visuelle representant un signal dans l'intervalle [-1, +1].
/// La barre a une largeur fixe de 20 caracteres avec un centre a 0.
/// Les blocs pleins (caractere sombre) indiquent l'amplitude du signal,
/// les blocs vides (caractere clair) representent l'espace non utilise.
///
/// # Parametres
/// - `signal` : la valeur du signal a representer [-1.0 a +1.0]
///
/// # Retour
/// La barre formatee entre crochets, ex: "[░░░░░▓▓▓▓│░░░░░░░░░░]"
fn signal_bar(signal: f64) -> String {
    let width = 20;
    let center = width / 2;
    // Convertir le signal [-1, +1] en position [0, width]
    let pos = ((signal + 1.0) / 2.0 * width as f64) as usize;
    let pos = pos.min(width);

    let mut bar: Vec<char> = vec!['░'; width]; // Caractere clair = espace vide

    // Remplir avec le caractere sombre entre le centre et la position
    if pos < center {
        // Signal negatif : remplir a gauche du centre
        for ch in bar.iter_mut().take(center).skip(pos) {
            *ch = '▓';
        }
    } else {
        // Signal positif : remplir a droite du centre
        for ch in bar.iter_mut().take(pos).skip(center) {
            *ch = '▓';
        }
    }
    bar[center] = '│'; // Marqueur du centre (zero)

    let s: String = bar.into_iter().collect();
    format!("[{}]", s)
}

/// Tronque une chaine de caracteres a `max` caracteres.
/// Ajoute "..." a la fin si la chaine est tronquee.
///
/// # Parametres
/// - `s` : la chaine a tronquer
/// - `max` : nombre maximal de caracteres
///
/// # Retour
/// La chaine tronquee (avec "..." si necessaire)
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.min(s.len())])
    }
}

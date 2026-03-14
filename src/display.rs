// =============================================================================
// display.rs — Rich terminal display
//
// Role: This file provides formatted display functions for the terminal.
// It displays the results of each cognitive cycle in a readable and
// visually informative way: stimulus, modules, decision, regulation,
// emotions, chemistry, and consciousness.
//
// Dependencies:
//   - crate::agent::lifecycle::ProcessResult : complete result of a cycle
//   - crate::stimulus::Stimulus : input stimulus
//   - crate::neurochemistry::NeuroChemicalState : neurochemical state
//   - crate::emotions::Mood : current mood
//   - crate::consensus::Decision : decision type
//  - crate::regulation::laws::ViolationSeverity : violation severity
//
// Place in architecture:
//   This module is called by the demo pipeline and by the agent
//   to display cycle results in the terminal. It does not affect
//   the agent's logic; it is purely presentational.
// =============================================================================

use crate::agent::lifecycle::ProcessResult;
use crate::stimulus::Stimulus;
use crate::neurochemistry::NeuroChemicalState;
use crate::emotions::Mood;

/// Displays the result of a complete cycle in the terminal.
/// Presents in a structured way: the stimulus, the module signals,
/// the weights, the decision, the regulation violations, the emotion,
/// the mood, the chemistry, and the consciousness level.
///
/// # Parameters
/// - `cycle` : cycle number being displayed
/// - `stimulus` : the input stimulus (text + metrics)
/// - `chemistry` : the neurochemical state at the end of the cycle
/// - `mood` : the current mood (valence, arousal)
/// - `result` : the complete processing result (consensus, verdict, emotion, consciousness)
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

    // Display the stimulus metrics
    println!("  📥 Stimulus : danger={:.2} reward={:.2} urgency={:.2} social={:.2} novelty={:.2}",
        stimulus.danger, stimulus.reward, stimulus.urgency, stimulus.social, stimulus.novelty);

    // Display the signals from each brain module (Reptilian, Limbic, Neocortex)
    // with a visual bar representing the signal [-1, +1]
    for signal in &result.consensus.signals {
        let bar = signal_bar(signal.signal);
        println!("  🧠 {} : {} {:.2} (conf={:.2}) — {}",
            signal.module, bar, signal.signal, signal.confidence,
            truncate(&signal.reasoning, 40));
    }

    // Display the module weights: R=Reptilian, L=Limbic, N=Neocortex
    println!("  ⚖️  Poids : R={:.2} L={:.2} N={:.2}",
        result.consensus.weights[0], result.consensus.weights[1], result.consensus.weights[2]);

    // Display the decision with an appropriate icon
    let decision_icon = match result.consensus.decision {
        crate::consensus::Decision::Yes => "✅",
        crate::consensus::Decision::No => "❌",
        crate::consensus::Decision::Maybe => "🤔",
    };
    println!("  {} Décision : {} (score={:.3}, cohérence={:.2})",
        decision_icon, result.consensus.decision.as_str(),
        result.consensus.score, result.consensus.coherence);

    // Display the moral regulation results (veto and violations)
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

    // Display the emotional state
    println!("  💜 Émotion : {} (similarité={:.2})",
        result.emotion.description(), result.emotion.dominant_similarity);
    // Display the mood (VAD = Valence-Arousal-Dominance): v=valence, a=arousal
    println!("  🌊 Mood : {} (v={:.2}, a={:.2})",
        mood.description(), mood.valence, mood.arousal);

    // Display the complete neurochemical state (7 neurotransmitters)
    println!("  🧪 {}", chemistry.display_string());

    // Display the consciousness level and phi (IIT = Integrated Information Theory)
    // with an excerpt from the inner monologue
    println!("  🔮 Conscience : {:.2} (phi={:.2}) — {}",
        result.consciousness.level, result.consciousness.phi,
        truncate(&result.consciousness.inner_narrative, 50));

    println!("{}", "─".repeat(70));
}

/// Generates a visual bar representing a signal in the [-1, +1] range.
/// The bar has a fixed width of 20 characters with a center at 0.
/// Filled blocks (dark character) indicate the signal amplitude,
/// empty blocks (light character) represent unused space.
///
/// # Parameters
/// - `signal` : the signal value to represent [-1.0 to +1.0]
///
/// # Returns
/// The formatted bar between brackets, e.g.: "[░░░░░▓▓▓▓│░░░░░░░░░░]"
fn signal_bar(signal: f64) -> String {
    let width = 20;
    let center = width / 2;
    // Convert the signal [-1, +1] to position [0, width]
    let pos = ((signal + 1.0) / 2.0 * width as f64) as usize;
    let pos = pos.min(width);

    let mut bar: Vec<char> = vec!['░'; width]; // Light character = empty space

    // Fill with dark character between the center and the position
    if pos < center {
        // Negative signal: fill to the left of center
        for ch in bar.iter_mut().take(center).skip(pos) {
            *ch = '▓';
        }
    } else {
        // Positive signal: fill to the right of center
        for ch in bar.iter_mut().take(pos).skip(center) {
            *ch = '▓';
        }
    }
    bar[center] = '│'; // Center marker (zero)

    let s: String = bar.into_iter().collect();
    format!("[{}]", s)
}

/// Truncates a string to `max` characters.
/// Adds "..." at the end if the string is truncated.
///
/// # Parameters
/// - `s` : the string to truncate
/// - `max` : maximum number of characters
///
/// # Returns
/// The truncated string (with "..." if necessary)
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.min(s.len())])
    }
}

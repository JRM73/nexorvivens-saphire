// =============================================================================
// display.rs -- Enriched terminal display
//
// Purpose:
//   Provides formatted terminal display functions for cognitive cycle results.
//   Renders each cycle's output in a human-readable, visually informative
//   layout: stimulus metrics, brain module signals, consensus decision,
//   ethical regulation verdicts, emergent emotion, neurochemical state,
//   and consciousness level.
//
// Dependencies:
//   - crate::agent::lifecycle::ProcessResult: complete result of a cognitive cycle
//   - crate::stimulus::Stimulus: the input stimulus
//   - crate::neurochemistry::NeuroChemicalState: current neurochemical state
//   - crate::emotions::Mood: current mood (valence, arousal)
//   - crate::consensus::Decision: decision type (Yes / No / Maybe)
//   - crate::regulation::laws::ViolationSeverity: severity of ethical violations
//
// Role in the architecture:
//   Called by the demonstration pipeline and by the agent's main loop to
//   render cycle results in the terminal. This module is purely presentational
//   and has no effect on the agent's internal logic or state.
// =============================================================================

use crate::agent::lifecycle::ProcessResult;
use crate::stimulus::Stimulus;
use crate::neurochemistry::NeuroChemicalState;
use crate::emotions::Mood;

/// Displays the complete result of a single cognitive cycle in the terminal.
///
/// Presents, in structured sections:
///   1. The stimulus text and its 5 perceptual dimension scores
///   2. Each brain module's output signal with a visual bar, confidence, and reasoning
///   3. The consensus weights for Reptilian (R), Limbic (L), and Neocortex (N)
///   4. The final decision (Yes / No / Maybe) with the aggregate score and coherence
///   5. Ethical regulation results: veto status and any law violations
///   6. The emergent emotion and its cosine similarity to the prototype
///   7. Current mood in the VAD (Valence-Arousal-Dominance) space
///   8. Full neurochemical state (7 neurotransmitters)
///   9. Consciousness level, Phi (from IIT -- Integrated Information Theory),
///      and an excerpt of the inner narrative
///
/// # Parameters
/// - `cycle`: the sequential cycle number being displayed.
/// - `stimulus`: the input stimulus (raw text + perceptual metrics).
/// - `chemistry`: the neurochemical state at the end of this cycle.
/// - `mood`: the current mood state (valence and arousal).
/// - `result`: the complete processing result (consensus, verdict, emotion, consciousness).
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

    // Section 1: Stimulus perceptual metrics
    println!("  📥 Stimulus : danger={:.2} reward={:.2} urgency={:.2} social={:.2} novelty={:.2}",
        stimulus.danger, stimulus.reward, stimulus.urgency, stimulus.social, stimulus.novelty);

    // Section 2: Brain module signals (Reptilian, Limbic, Neocortex).
    // Each signal is in [-1, +1] and is visualized with a horizontal bar chart.
    for signal in &result.consensus.signals {
        let bar = signal_bar(signal.signal);
        println!("  🧠 {} : {} {:.2} (conf={:.2}) — {}",
            signal.module, bar, signal.signal, signal.confidence,
            truncate(&signal.reasoning, 40));
    }

    // Section 3: Module weights -- R=Reptilian, L=Limbic, N=Neocortex.
    // These weights are dynamically adjusted based on context (e.g., high danger
    // increases the reptilian weight).
    println!("  ⚖️  Poids : R={:.2} L={:.2} N={:.2}",
        result.consensus.weights[0], result.consensus.weights[1], result.consensus.weights[2]);

    // Section 4: Final consensus decision with an appropriate status icon.
    let decision_icon = match result.consensus.decision {
        crate::consensus::Decision::Yes => "✅",
        crate::consensus::Decision::No => "❌",
        crate::consensus::Decision::Maybe => "🤔",
    };
    println!("  {} Décision : {} (score={:.3}, cohérence={:.2})",
        decision_icon, result.consensus.decision.as_str(),
        result.consensus.score, result.consensus.coherence);

    // Section 5: Ethical regulation results -- veto flag and individual violations.
    // Violations are tagged with severity: Veto (hard block), Warning, or Info.
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

    // Section 6: Emergent emotion -- the emotion whose VAD prototype vector
    // has the highest cosine similarity to the current neurochemical state.
    println!("  💜 Émotion : {} (similarité={:.2})",
        result.emotion.description(), result.emotion.dominant_similarity);

    // Section 7: Mood in VAD space -- v=valence (pleasant/unpleasant),
    // a=arousal (activated/deactivated). Mood is a slow-moving average
    // that integrates emotional states over multiple cycles.
    println!("  🌊 Mood : {} (v={:.2}, a={:.2})",
        mood.description(), mood.valence, mood.arousal);

    // Section 8: Full neurochemical state (7 neurotransmitters:
    // dopamine, cortisol, serotonin, adrenaline, oxytocin, endorphin,
    // noradrenaline).
    println!("  🧪 {}", chemistry.display_string());

    // Section 9: Consciousness level and Phi metric.
    // Phi is the integrated information measure from IIT (Integrated Information
    // Theory, Tononi 2004) -- higher values indicate greater information
    // integration across the cognitive system. The inner narrative is the
    // agent's current stream of consciousness (truncated for display).
    println!("  🔮 Conscience : {:.2} (phi={:.2}) — {}",
        result.consciousness.level, result.consciousness.phi,
        truncate(&result.consciousness.inner_narrative, 50));

    println!("{}", "─".repeat(70));
}

/// Generates a visual horizontal bar representing a signal in the range [-1, +1].
///
/// The bar has a fixed width of 20 characters with a center marker at zero.
/// Filled blocks (dark character '▓') indicate the signal amplitude relative
/// to zero; empty blocks (light character '░') represent unused space.
///
/// For negative signals, the filled region extends leftward from the center.
/// For positive signals, it extends rightward.
///
/// # Parameters
/// - `signal`: the signal value to represent, expected in [-1.0, +1.0].
///
/// # Returns
/// A formatted string enclosed in brackets, e.g., `"[░░░░░▓▓▓▓│░░░░░░░░░░]"`.
fn signal_bar(signal: f64) -> String {
    let width = 20;
    let center = width / 2;

    // Map the signal from [-1, +1] to a discrete position in [0, width].
    let pos = ((signal + 1.0) / 2.0 * width as f64) as usize;
    let pos = pos.min(width);

    // Initialize the bar with light/empty characters.
    let mut bar: Vec<char> = vec!['░'; width];

    // Fill with dark characters between the center and the signal position.
    if pos < center {
        // Negative signal: fill leftward from center to the signal position.
        for ch in bar.iter_mut().take(center).skip(pos) {
            *ch = '▓';
        }
    } else {
        // Positive signal: fill rightward from center to the signal position.
        for ch in bar.iter_mut().take(pos).skip(center) {
            *ch = '▓';
        }
    }

    // Place the center marker (zero reference point).
    bar[center] = '│';

    let s: String = bar.into_iter().collect();
    format!("[{}]", s)
}

/// Truncates a string to at most `max` characters, appending "..." if truncated.
///
/// # Parameters
/// - `s`: the input string to truncate.
/// - `max`: the maximum number of bytes (not Unicode characters) to retain.
///
/// # Returns
/// The original string if it fits within `max` bytes, otherwise the first
/// `max` bytes followed by "...".
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.min(s.len())])
    }
}

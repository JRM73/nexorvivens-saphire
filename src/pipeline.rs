// =============================================================================
// pipeline.rs -- Complete processing pipeline (demonstration mode)
// =============================================================================
//
// Purpose:
//   Orchestrates Saphire's demonstration mode. This mode allows testing the
//   entire cognitive system without an LLM (Large Language Model) backend or
//   a database connection, by executing 8 predefined scenarios and displaying
//   the results of each cognitive processing cycle.
//
// Dependencies:
//   - crate::agent::lifecycle::SaphireAgent: the main cognitive agent
//   - crate::display: formatted terminal display functions
//   - crate::scenarios: the 8 demonstration scenarios and their expected outcomes
//
// Role in the architecture:
//   Top-level module in src/. Called from main.rs when demonstration mode is
//   selected (--demo flag, no Ollama/LLM connection required). Uses
//   SaphireAgent::process_stimulus() to run each scenario through the full
//   cognitive pipeline and display::display_cycle() for terminal output.
// =============================================================================

use crate::agent::lifecycle::SaphireAgent;
use crate::display;

/// Runs the demonstration mode (no LLM, no database).
///
/// Iterates through 8 predefined scenarios (defined in `scenarios.rs`),
/// processes each one through Saphire's full cognitive pipeline
/// (neurochemistry update, brain module consensus, ethical regulation,
/// emotion emergence, consciousness evaluation), and displays the results
/// alongside the expected outcome for manual comparison.
///
/// # Parameters
/// - `agent`: mutable reference to the `SaphireAgent` instance used to
///   process each stimulus through the cognitive pipeline.
pub fn run_demo(agent: &mut SaphireAgent) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  💎 SAPHIRE — Mode Démonstration (8 scénarios)                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // Load the 8 demonstration scenarios and their expected behavioral outcomes.
    let scenarios = crate::scenarios::demo_scenarios();
    let expected = crate::scenarios::expected_outcomes();

    // Process each scenario sequentially through the full cognitive pipeline.
    for (i, stimulus) in scenarios.iter().enumerate() {
        // Run the stimulus through the complete cognitive pipeline:
        // NLP scoring -> brain modules -> consensus -> regulation -> emotion -> consciousness
        let result = agent.process_stimulus(stimulus);

        // Display the cycle results in a structured terminal format:
        // stimulus metrics, module signals, decision, violations, emotion, chemistry, consciousness.
        display::display_cycle(
            agent.cycle_count,
            stimulus,
            &agent.chemistry,
            &agent.mood,
            &result,
        );

        // Display the expected outcome for side-by-side manual comparison.
        println!("  📋 Attendu : {}", expected[i]);
        println!();
    }

    // Display a summary footer with the total number of cycles processed
    // and the agent's final dominant emotion.
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Démonstration terminée — {} cycles traités                     ║", agent.cycle_count);
    println!("║  Émotion finale : {}                                            ║", agent.identity.dominant_emotion);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

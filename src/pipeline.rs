// =============================================================================
// pipeline.rs — Complete processing pipeline (for demo mode)
// =============================================================================
//
// Role: Orchestrates the Saphire demo mode. This mode allows testing
//        the cognitive system without LLM (Large Language Model) or database,
//        by executing 8 predetermined scenarios and displaying the results
//        of each processing cycle.
//
// Dependencies:
//   - crate::agent::lifecycle::SaphireAgent : the main cognitive agent
//   - crate::display : formatted display functions for the terminal
//   - crate::scenarios : the 8 demo scenarios and their expected results
//
// Place in architecture:
//   High-level module in src/. Called from main.rs when demo mode
//   is selected (without Ollama connection). Uses
//   SaphireAgent.process_stimulus() to process each scenario and
//   display::display_cycle() for display.
// =============================================================================

use crate::agent::lifecycle::SaphireAgent;
use crate::display;

/// Executes the demo mode (without LLM or DB).
///
/// Runs through the 8 predetermined scenarios (defined in scenarios.rs),
/// processes each one via SaphireAgent's complete cognitive pipeline, and
/// displays the results (chemistry, mood, decision) along with the
/// expected result for comparison.
///
/// Parameter `agent` : mutable reference to the Saphire agent to use
///                      for stimulus processing
pub fn run_demo(agent: &mut SaphireAgent) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  💎 SAPHIRE — Mode Démonstration (8 scénarios)                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // Load the 8 demo scenarios and their expected results
    let scenarios = crate::scenarios::demo_scenarios();
    let expected = crate::scenarios::expected_outcomes();

    // Process each scenario sequentially
    for (i, stimulus) in scenarios.iter().enumerate() {
        // Process the stimulus via the complete cognitive pipeline
        let result = agent.process_stimulus(stimulus);

        // Display the cycle results (chemistry, mood, decision)
        display::display_cycle(
            agent.cycle_count,
            stimulus,
            &agent.chemistry,
            &agent.mood,
            &result,
        );

        // Display the expected result for manual comparison
        println!("  📋 Attendu : {}", expected[i]);
        println!();
    }

    // Display the final summary of the demonstration
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Démonstration terminée — {} cycles traités                     ║", agent.cycle_count);
    println!("║  Émotion finale : {}                                            ║", agent.identity.dominant_emotion);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// =============================================================================
// pipeline.rs — Pipeline de traitement complet (pour le mode demo)
// =============================================================================
//
// Role : Orchestre le mode demonstration de Saphire. Ce mode permet de tester
//        le systeme cognitif sans LLM (Large Language Model) ni base de donnees,
//        en executant 8 scenarios predetermines et en affichant les resultats
//        de chaque cycle de traitement.
//
// Dependances :
//   - crate::agent::lifecycle::SaphireAgent : l'agent cognitif principal
//   - crate::display : fonctions d'affichage formatees pour le terminal
//   - crate::scenarios : les 8 scenarios de demonstration et leurs resultats attendus
//
// Place dans l'architecture :
//   Module de haut niveau dans src/. Appele depuis main.rs lorsque le mode
//   demonstration est selectionne (sans connexion Ollama). Utilise
//   SaphireAgent.process_stimulus() pour traiter chaque scenario et
//   display::display_cycle() pour l'affichage.
// =============================================================================

use crate::agent::lifecycle::SaphireAgent;
use crate::display;

/// Execute le mode demonstration (sans LLM ni DB).
///
/// Deroule les 8 scenarios predetermines (definis dans scenarios.rs),
/// traite chacun via le pipeline cognitif complet de SaphireAgent, et
/// affiche les resultats (chimie, humeur, decision) ainsi que le resultat
/// attendu pour comparaison.
///
/// Parametre `agent` : reference mutable vers l'agent Saphire a utiliser
///                      pour le traitement des stimuli
pub fn run_demo(agent: &mut SaphireAgent) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  💎 SAPHIRE — Mode Démonstration (8 scénarios)                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    // Charger les 8 scenarios de demonstration et leurs resultats attendus
    let scenarios = crate::scenarios::demo_scenarios();
    let expected = crate::scenarios::expected_outcomes();

    // Traiter chaque scenario sequentiellement
    for (i, stimulus) in scenarios.iter().enumerate() {
        // Traiter le stimulus via le pipeline cognitif complet
        let result = agent.process_stimulus(stimulus);

        // Afficher les resultats du cycle (chimie, humeur, decision)
        display::display_cycle(
            agent.cycle_count,
            stimulus,
            &agent.chemistry,
            &agent.mood,
            &result,
        );

        // Afficher le resultat attendu pour comparaison manuelle
        println!("  📋 Attendu : {}", expected[i]);
        println!();
    }

    // Afficher le resume final de la demonstration
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Démonstration terminée — {} cycles traités                     ║", agent.cycle_count);
    println!("║  Émotion finale : {}                                            ║", agent.identity.dominant_emotion);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

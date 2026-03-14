// =============================================================================
// api/neuroscience.rs — Handlers for the modules neuroscientifiques avances
//
// Role: Endpoints for recepteurs pharmacologiques, regions cerebrales,
// predictive processing, reconsolidation memorielle, metriques de conscience.
// =============================================================================

use axum::extract::State;
use axum::response::IntoResponse;

use super::state::AppState;

/// GET /api/receptors — Etat de la banque de recepteurs pharmacologiques.
pub async fn api_receptors_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "summary": agent.receptor_bank.summary(),
        "pharmacological_subtypes": agent.receptor_bank.to_json(),
        "hormonal_sensitivity": agent.hormonal_system.receptors.to_snapshot_json(),
        "sensitivity_factors": {
            "dopamine": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Dopamine),
            "cortisol": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Cortisol),
            "serotonin": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Serotonin),
            "adrenaline": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Adrenaline),
            "oxytocin": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Oxytocin),
            "endorphin": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Endorphin),
            "noradrenaline": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Noradrenaline),
            "gaba": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Gaba),
            "glutamate": agent.hormonal_system.receptors.factor_for(crate::neurochemistry::Molecule::Glutamate),
        },
        "desensitized": agent.hormonal_system.receptors.describe_desensitized(),
    }))
}

/// GET /api/brain-regions — Activations des 12 regions cerebrales + GWT.
pub async fn api_brain_regions_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "summary": agent.brain_network.summary(),
        "regions": agent.brain_network.regions.iter().map(|r| serde_json::json!({
            "name": r.name,
            "activation": r.activation,
        })).collect::<Vec<_>>(),
    }))
}

/// GET /api/predictive — Moteur de prediction (Friston).
pub async fn api_predictive_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    axum::Json(serde_json::json!({
        "summary": agent.predictive_engine.summary(),
        "error_history_len": agent.predictive_engine.error_history.len(),
    }))
}

/// GET /api/reconsolidation — Moteur de reconsolidation memorielle.
pub async fn api_reconsolidation_status(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let engine = &agent.reconsolidation;
    axum::Json(serde_json::json!({
        "active_labile_count": engine.labile_memories.len(),
        "labile_memories": engine.labile_memories.iter().take(20).map(|(id, state)| {
            serde_json::json!({
                "memory_id": id,
                "lability_remaining": state.lability_remaining,
                "recall_count": state.recall_count,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// GET /api/receptors/sensitivity — Facteurs de sensibilite par molecule(s).
pub async fn api_receptors_sensitivity(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let receptors = &agent.hormonal_system.receptors;
    axum::Json(serde_json::json!({
        "factors": {
            "dopamine": receptors.factor_for(crate::neurochemistry::Molecule::Dopamine),
            "cortisol": receptors.factor_for(crate::neurochemistry::Molecule::Cortisol),
            "serotonin": receptors.factor_for(crate::neurochemistry::Molecule::Serotonin),
            "adrenaline": receptors.factor_for(crate::neurochemistry::Molecule::Adrenaline),
            "oxytocin": receptors.factor_for(crate::neurochemistry::Molecule::Oxytocin),
            "endorphin": receptors.factor_for(crate::neurochemistry::Molecule::Endorphin),
            "noradrenaline": receptors.factor_for(crate::neurochemistry::Molecule::Noradrenaline),
            "gaba": receptors.factor_for(crate::neurochemistry::Molecule::Gaba),
            "glutamate": receptors.factor_for(crate::neurochemistry::Molecule::Glutamate),
        },
        "detailed": receptors.to_snapshot_json(),
        "desensitized": receptors.describe_desensitized(),
    }))
}

/// GET /api/consciousness-metrics — Metriques scientifiques (LZC, PCI, Phi*).
pub async fn api_consciousness_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let agent = state.agent.lock().await;
    let cs = &agent.consciousness;
    // Compute the metrics en direct
    let metrics = cs.compute_scientific_metrics(
        &agent.brain_network,
        &agent.chemistry,
    );
    axum::Json(serde_json::json!({
        "lzc": metrics.lzc,
        "pci": {
            "value": metrics.pci.pci,
            "entropy": metrics.pci.source_entropy,
            "lzc_response": metrics.pci.lzc,
            "target_region": metrics.pci.target_region,
            "regions_affected": metrics.pci.regions_affected,
        },
        "phi_star": {
            "value": metrics.phi_star.phi_star,
            "mi_whole": metrics.phi_star.mi_whole,
            "mi_minimum_partition": metrics.phi_star.mi_minimum_partition,
            "minimum_information_partition": metrics.phi_star.minimum_information_partition,
            "phi_raw": metrics.phi_star.phi_raw,
        },
        "composite_score": metrics.composite_score,
        "interpretation": metrics.interpretation,
    }))
}

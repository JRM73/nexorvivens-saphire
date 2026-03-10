// algorithms_integration.rs — Stub for the lite edition
// Full algorithm integration (LLM-driven algorithms, body context builder) not ported.

use super::SaphireAgent;

impl SaphireAgent {
    /// Builds a body context string for the LLM substrate prompt.
    pub(crate) fn build_body_context(&self) -> String {
        if !self.config.body.enabled {
            return String::new();
        }
        let bpm = self.body.heart.bpm();
        let energy = self.body.soma.energy;
        let vitality = self.body.soma.vitality;
        format!(
            "[CORPS] BPM: {:.0}, Energie: {:.0}%, Vitalite: {:.0}%",
            bpm, energy * 100.0, vitality * 100.0
        )
    }

    /// Handles an algorithm request from the LLM response.
    pub(crate) async fn handle_algorithm_request(
        &mut self,
        _request: &crate::algorithms::orchestrator::AlgorithmRequest,
    ) {
        // stub — no algorithm execution in lite
    }

    /// Runs automatic algorithms (smoothing, clustering, etc.).
    pub(crate) async fn run_auto_algorithms(&mut self) {
        // stub — no auto algorithms in lite
    }
}

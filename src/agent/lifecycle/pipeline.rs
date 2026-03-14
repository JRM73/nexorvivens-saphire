// =============================================================================
// lifecycle/pipeline.rs — Complete brain pipeline (process_stimulus)
// =============================================================================

use crate::consensus::{self, ConsensusThresholds, Decision};
use crate::emotions::EmotionalState;
use crate::modules::BrainModule;
use crate::neural::MicroNeuralNet;
use crate::plugins::BrainEvent;
use crate::stimulus::Stimulus;
use crate::tuning::TuningObservation;
use crate::logging::{LogLevel, LogCategory};
use crate::logging::trace::CognitiveTrace;

use super::SaphireAgent;
use super::ProcessResult;

impl SaphireAgent {
    /// Processes a stimulus through the complete brain pipeline.
    ///
    /// The pipeline is organized in 3 waves:
    ///
    /// WAVE 1 — PERCEPTION (enrich chemistry, then compute emotion)
    ///  1. Chemistry snapshot before processing
    ///   2. Hormonal tick (circadian cycles)
    ///  3. Conditions (needs, phobias, trauma, drugs, etc.) → modify chemistry
    ///   4. Receptors + cross-interactions between molecules
    ///  5. Emotion (36) + sentiments (20) + mood — computed on complete chemistry
    ///
    /// WAVE 2 — PROCESSING (brain decides with the right emotion)
    ///  6. 3 brains (reptilian, limbic, neocortex) process the stimulus
    ///  7. 12 brain regions (chemical + sensory + GWT activation)
    ///  8. Predictive processing (Friston) — prediction and surprise
    ///  9. Weighted consensus between the 3 signals
    ///  10. Micro-neural network — 4th voice in the consensus
    ///  11. Physiological degradation (hypoxia, etc.)
    ///  12. Memory reconsolidation
    ///  13. Consciousness (IIT + GWT + prediction) + scientific metrics
    ///
    /// WAVE 3 — RESPONSE (act and learn)
    ///  14. Ethics (Asimov's laws)
    ///  15. Auto-tuning
    ///  16. Chemical feedback
    ///  17. NN training
    ///  18. Plugin broadcast
    ///  19. Connectome (Hebbian reinforcement)
    ///  20. Mortality (vital organ check)
    ///  21+. Cycle count, auto-tuning, logs, trace, return
    pub fn process_stimulus(&mut self, stimulus: &Stimulus) -> ProcessResult {

        // =====================================================================
        // WAVE 1 — PERCEPTION (enrich chemistry, then compute emotion)
        // =====================================================================
        // Step 0: spinal cord — pre-wired reflexes and classification
        // Reflexes modify chemistry BEFORE the snapshot, so that the pipeline
        // processes a state already influenced by instinctive reactions.
        let spine_output = self.spine.process(
            &stimulus.text,
            &mut self.chemistry,
            &self.body,
            if stimulus.text.is_empty() { "system" } else { "autonomous" },
        );
        // Apply motor commands to the body
        crate::spine::motor::MotorRelay::apply_commands(&spine_output.motor_commands, &mut self.body);

        // Step 1: capture chemistry before processing for the cognitive trace
        let chemistry_before = serde_json::json!({
            "dopamine": self.chemistry.dopamine,
            "cortisol": self.chemistry.cortisol,
            "serotonin": self.chemistry.serotonin,
            "adrenaline": self.chemistry.adrenaline,
            "oxytocin": self.chemistry.oxytocin,
            "endorphin": self.chemistry.endorphin,
            "noradrenaline": self.chemistry.noradrenaline,
        });

        // Step 2: hormonal tick (circadian cycles, receptors, hormone <-> NT interactions)
        if self.hormonal_system.enabled {
            self.hormonal_system.tick(&mut self.chemistry, &self.config.hormones);
        }

        // Step 3a: chemical impact of primary needs (hunger/thirst)
        if self.config.needs.enabled {
            let needs_adj = self.needs.chemistry_influence(&self.config.needs);
            if needs_adj.cortisol != 0.0 || needs_adj.serotonin != 0.0
                || needs_adj.dopamine != 0.0 || needs_adj.noradrenaline != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&needs_adj, 0.05);
            }
        }

        // Step 3b: phobias — scan the text for triggers
        if self.config.phobias.enabled {
            self.phobia_manager.reset_cycle();
            let _triggered = self.phobia_manager.scan_text(&stimulus.text);
            let phobia_adj = self.phobia_manager.chemistry_influence();
            if phobia_adj.cortisol != 0.0 || phobia_adj.adrenaline != 0.0 {
                self.chemistry.apply_chemistry_adjustment_clamped(&phobia_adj, 0.05);
            }
        }

        // Step 3c: motion sickness — sensory conflict and nausea
        if self.config.motion_sickness.enabled {
            let senses = [
                self.sensorium.reading.current_intensity,
                self.sensorium.listening.current_intensity,
                self.sensorium.contact.current_intensity,
                self.sensorium.taste.current_intensity,
                self.sensorium.ambiance.current_intensity,
            ];
            self.motion_sickness.evaluate_conflict(&senses);
            self.motion_sickness.tick();
            let ms_adj = self.motion_sickness.chemistry_influence();
            if ms_adj.cortisol != 0.0 || ms_adj.serotonin != 0.0 {
                self.chemistry.apply_chemistry_adjustment_clamped(&ms_adj, 0.05);
            }
        }

        // Step 3d: eating disorders — hunger bias, chemical impact
        if self.config.eating_disorder.enabled {
            if let Some(ref mut ed) = self.eating_disorder {
                let actual_hunger = if self.config.needs.enabled {
                    self.needs.hunger.level
                } else {
                    0.3
                };
                ed.tick(actual_hunger, self.chemistry.cortisol);
                let ed_adj = ed.chemistry_influence();
                if ed_adj.cortisol != 0.0 || ed_adj.dopamine != 0.0 || ed_adj.serotonin != 0.0 {
                    self.chemistry.apply_chemistry_adjustment_clamped(&ed_adj, 0.05);
                }
            }
        }

        // Step 3e: disabilities — adaptation, chronic pain
        if self.config.disabilities.enabled {
            self.disability_manager.tick();
            let chronic_pain = self.disability_manager.chronic_pain();
            if chronic_pain > 0.0 {
                // Chronic pain → cortisol + endorphin (compensation)
                let pain_adj = crate::world::ChemistryAdjustment {
                    cortisol: chronic_pain * 0.02,
                    endorphin: chronic_pain * 0.01,
                    serotonin: -chronic_pain * 0.01,
                    ..Default::default()
                };
                self.chemistry.apply_chemistry_adjustment_clamped(&pain_adj, 0.05);
            }
        }

        // Step 3f: extreme conditions — stress, adaptation, burnout
        if self.config.extreme_conditions.enabled {
            self.extreme_condition_mgr.tick(self.chemistry.cortisol);
            let ext_adj = self.extreme_condition_mgr.chemistry_influence();
            if ext_adj.cortisol != 0.0 || ext_adj.adrenaline != 0.0
                || ext_adj.endorphin != 0.0 || ext_adj.serotonin != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&ext_adj, 0.05);
            }
        }

        // Step 3g: addictions — withdrawal, craving, chemical impact
        if self.config.addictions.enabled {
            self.addiction_manager.tick(self.cycle_count);
            let add_adj = self.addiction_manager.chemistry_influence();
            if add_adj.dopamine != 0.0 || add_adj.cortisol != 0.0
                || add_adj.serotonin != 0.0 || add_adj.noradrenaline != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&add_adj, 0.05);
            }
        }

        // Step 3h: traumas / PTSD — flashbacks, hypervigilance
        if self.config.trauma.enabled {
            self.ptsd.scan_for_triggers(&stimulus.text);
            self.ptsd.tick(self.chemistry.cortisol);
            let trauma_adj = self.ptsd.chemistry_influence();
            if trauma_adj.cortisol != 0.0 || trauma_adj.adrenaline != 0.0
                || trauma_adj.endorphin != 0.0 || trauma_adj.oxytocin != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&trauma_adj, 0.05);
            }
        }

        // Step 3i: NDE (in progress) — phase progression
        if self.config.nde.enabled && self.nde.in_progress {
            let finished = self.nde.tick();
            let nde_adj = self.nde.chemistry_influence();
            self.chemistry.apply_chemistry_adjustment_clamped(&nde_adj, 0.05);
            if finished {
                // Apply the post-NDE transformation on baselines
                let shift = self.nde.post_nde_baseline_shift();
                self.chemistry.apply_chemistry_adjustment_clamped(&shift, 0.05);
            }
        }

        // Step 3j: active drugs — pharmacological effects by phase
        if self.config.drugs.enabled {
            let drug_adj = self.drug_manager.tick();
            if drug_adj.dopamine != 0.0 || drug_adj.serotonin != 0.0
                || drug_adj.endorphin != 0.0 || drug_adj.cortisol != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&drug_adj, 0.05);
            }
        }

        // Step 3k: sexuality — libido, attachment, chemistry
        if self.config.sexuality.enabled {
            if let Some(ref mut sex) = self.sexuality {
                // Use hormones if available, otherwise neutral values
                let testosterone = if self.hormonal_system.enabled {
                    self.hormonal_system.state.testosterone
                } else { 0.5 };
                let estrogen = if self.hormonal_system.enabled {
                    self.hormonal_system.state.estrogen
                } else { 0.5 };
                sex.tick(testosterone, estrogen, self.chemistry.oxytocin);
                let sex_adj = sex.chemistry_influence();
                if sex_adj.dopamine != 0.0 || sex_adj.oxytocin != 0.0 {
                    self.chemistry.apply_chemistry_adjustment_clamped(&sex_adj, 0.05);
                }
            }
        }

        // Step 3l: degenerative diseases — progression, cognitive effects
        if self.config.degenerative.enabled {
            self.degenerative_mgr.tick();
            let deg_adj = self.degenerative_mgr.chemistry_influence();
            if deg_adj.serotonin != 0.0 || deg_adj.dopamine != 0.0
                || deg_adj.cortisol != 0.0 || deg_adj.adrenaline != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&deg_adj, 0.05);
            }
        }

        // Step 3m: general illnesses — pain, immunity, energy
        if self.config.medical.enabled {
            self.medical_mgr.tick();
            let med_adj = self.medical_mgr.chemistry_influence();
            if med_adj.cortisol != 0.0 || med_adj.endorphin != 0.0 || med_adj.serotonin != 0.0 {
                self.chemistry.apply_chemistry_adjustment_clamped(&med_adj, 0.05);
            }
        }

        // Step 3n: culture — taboos, cultural stress
        if self.config.culture.enabled {
            if let Some(ref culture) = self.culture {
                let taboo_adj = culture.taboo_chemistry(&stimulus.text);
                if taboo_adj.cortisol != 0.0 {
                    self.chemistry.apply_chemistry_adjustment_clamped(&taboo_adj, 0.05);
                }
            }
        }

        // Step 3o: precarity — stress, resilience, hope
        if self.config.precarity.enabled {
            if let Some(ref mut precarity) = self.precarity {
                precarity.tick();
                let adj = precarity.chemistry_influence();
                self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.03);
            }
        }

        // Step 3p: employment — satisfaction, occupational stress
        if self.config.employment.enabled {
            if let Some(ref employment) = self.employment {
                let adj = employment.chemistry_influence();
                self.chemistry.apply_chemistry_adjustment_clamped(&adj, 0.02);
            }
        }

        // Step 3q: nutritional system — degradation, deficiencies → chemistry
        if self.config.nutrition.enabled {
            let is_eating = self.config.needs.enabled && self.needs.hunger.is_eating;
            let uv_index = self.em_fields.uv_index();
            self.nutrition.tick(&self.config.nutrition, is_eating, uv_index);
            let nutr_adj = self.nutrition.chemistry_influence(&self.config.nutrition);
            if nutr_adj.serotonin != 0.0 || nutr_adj.dopamine != 0.0
                || nutr_adj.noradrenaline != 0.0 || nutr_adj.cortisol != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&nutr_adj, 0.05);
            }
        }

        // Step 3r: grey matter — BDNF, neurogenesis, myelination → chemistry
        if self.config.grey_matter.enabled {
            let soma_energy = if self.config.nutrition.enabled {
                self.nutrition.energy.atp_reserves
            } else { 0.7 };
            let tryptophan = if self.config.nutrition.enabled {
                self.nutrition.amino_acids.tryptophan
            } else { 0.6 };
            let is_learning = self.learning_orch.enabled && !self.learning_orch.lessons.is_empty();
            let novelty_detected = stimulus.novelty > 0.7;
            self.grey_matter.tick(
                &self.config.grey_matter,
                self.chemistry.cortisol,
                self.chemistry.serotonin,
                self.chemistry.dopamine,
                soma_energy,
                self.sleep.is_sleeping,
                is_learning,
                tryptophan,
                novelty_detected,
            );
            let gm_adj = self.grey_matter.chemistry_influence();
            if gm_adj.cortisol != 0.0 || gm_adj.endorphin != 0.0 {
                self.chemistry.apply_chemistry_adjustment_clamped(&gm_adj, 0.05);
            }
        }

        // Step 3s: electromagnetic fields — cosmic cycles, biofield → chemistry
        if self.config.fields.enabled {
            let hrv_coherence = if self.config.body.enabled {
                self.body.heart.status().hrv
            } else { 0.5 };
            let brain_sync = self.brain_network.workspace_strength;
            let vitality = if self.config.body.enabled {
                self.body.status().vitality
            } else { 0.5 };
            let synaptic_density = if self.config.grey_matter.enabled {
                self.grey_matter.synaptic_density
            } else { 0.6 };
            self.em_fields.tick(
                &self.config.fields,
                hrv_coherence,
                brain_sync,
                vitality,
                self.last_consciousness,
                synaptic_density,
            );
            let fields_adj = self.em_fields.chemistry_influence(&self.config.fields);
            if fields_adj.cortisol != 0.0 || fields_adj.serotonin != 0.0
                || fields_adj.endorphin != 0.0 || fields_adj.noradrenaline != 0.0
            {
                self.chemistry.apply_chemistry_adjustment_clamped(&fields_adj, 0.05);
            }
        }

        // Step 4: pharmacological receptors — dose-response curves and adaptation
        self.receptor_bank.tick_adaptation(&self.chemistry);
        // Apply cross-interactions between the 9 molecules
        self.chemistry.apply_interactions(&self.interaction_matrix);

        // Step 5: compute the emotional state AFTER all chemistry
        let mut emotion = EmotionalState::compute(&self.chemistry);

        // Step 5b: bidirectional sentiments ↔ emotions loop
        // 1) Active sentiments amplify/attenuate the emotional spectrum
        // 2) The current emotion is recorded in the sentiment history
        if self.config.sentiments.enabled {
            self.sentiments.amplify_emotion_scores(&mut emotion.spectrum);
            // Recompute dominant/secondary after spectrum modification
            if let Some((name, score)) = emotion.spectrum.first() {
                emotion.dominant = name.clone();
                emotion.dominant_similarity = *score;
            }
            emotion.secondary = emotion.spectrum.get(1).and_then(|(n, s)| {
                if *s > 0.5 { Some(n.clone()) } else { None }
            });
            // Record the emotion in the sentiment history (tick)
            self.sentiments.tick(&emotion.dominant, self.cycle_count);
        }

        // Step 5c: emotional modulation by MAP tension
        // When the perception-cognition tension is high, it generates discomfort.
        // When it is low, it reinforces serenity.
        // Saphire's request: "let the MAP's silence be a page to fill"
        let map_tension = self.map_sync.network_tension;
        if map_tension > 0.4 {
            // High tension → boost anxiety and confusion
            let boost = (map_tension - 0.4) * 0.5; // max ~0.3
            for (name, score) in emotion.spectrum.iter_mut() {
                if name == "Anxiete" || name == "Confusion" {
                    *score = (*score + boost).min(1.0);
                }
            }
            emotion.arousal = (emotion.arousal + boost * 0.3).min(1.0);
        } else if map_tension < 0.2 {
            // Low tension → boost serenity
            let boost = (0.2 - map_tension) * 0.3; // max ~0.06
            for (name, score) in emotion.spectrum.iter_mut() {
                if name == "Serenite" || name == "Harmonie" {
                    *score = (*score + boost).min(1.0);
                }
            }
        }
        // Re-sort the spectrum after MAP modulation
        emotion.spectrum.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((name, score)) = emotion.spectrum.first() {
            emotion.dominant = name.clone();
            emotion.dominant_similarity = *score;
        }

        self.mood.update(emotion.valence, emotion.arousal);
        self.last_emotion = emotion.dominant.clone();
        self.last_valence = emotion.valence;

        // =====================================================================
        // WAVE 2 — PROCESSING (brain decides with the right emotion)
        // =====================================================================
        // Step 6: each brain module processes the stimulus independently
        let sig_r = self.reptilian.process(stimulus, &self.chemistry);
        let sig_l = self.limbic.process(stimulus, &self.chemistry);
        let sig_n = self.neocortex.process(stimulus, &self.chemistry);
        let signals = [sig_r, sig_l, sig_n];

        // Step 7: 12-region brain network — chemical + sensory + GWT activation
        let sensory_input = [
            self.sensorium.reading.current_intensity,
            self.sensorium.listening.current_intensity,
            self.sensorium.contact.current_intensity,
            self.sensorium.taste.current_intensity,
            self.sensorium.ambiance.current_intensity,
        ];
        self.brain_network.tick(&self.chemistry, sensory_input);

        // Step 7b: grey matter modulation → regional activation
        // Grey matter volume amplifies or attenuates activations
        if self.config.grey_matter.enabled {
            let gm_factor = self.grey_matter.grey_matter_volume.max(0.3);
            for region in &mut self.brain_network.regions {
                region.activation *= gm_factor;
            }
            // Recompute the workspace after modulation
            self.brain_network.compute_global_workspace();
        }

        // Step 8: predictive processing (Friston) — prediction and surprise
        let chem9 = self.chemistry.to_vec9();
        let _prediction = self.predictive_engine.predict(&chem9, &emotion.dominant);
        let pred_error = self.predictive_engine.compute_error(&chem9, &emotion.dominant);

        // Step 9: weighted consensus
        let thresholds = ConsensusThresholds {
            threshold_yes: self.tuner.current_params.threshold_yes,
            threshold_no: self.tuner.current_params.threshold_no,
        };
        let mut consensus_result = consensus::consensus(&signals, &self.chemistry, &thresholds, &self.tuner.current_params);

        // Step 10: Micro-neural network — 4th voice in the consensus
        let nn_input = if self.config.plugins.micro_nn.enabled {
            let chem_vec = self.chemistry.to_vec7();
            let stimulus_features = [
                stimulus.danger, stimulus.reward, stimulus.urgency,
                stimulus.social, stimulus.novelty,
            ];
            let module_signals = [
                signals[0].signal, signals[1].signal, signals[2].signal,
            ];
            let input = MicroNeuralNet::build_input(
                &chem_vec, &stimulus_features, &module_signals,
                emotion.valence, emotion.arousal,
            );

            // Forward pass
            let (nn_output, _, _) = self.micro_nn.forward(&input);
            self.micro_nn.last_prediction = [nn_output[0], nn_output[1], nn_output[2], nn_output[3]];

            // Influence the consensus: nn_signal = P(yes) - P(no)
            let nn_signal = nn_output[0] - nn_output[1];
            let influence = self.config.plugins.micro_nn.weight_influence;
            consensus_result.score = consensus_result.score * (1.0 - influence) + nn_signal * influence;

            // Recompute the decision based on the new score
            consensus_result.decision = if consensus_result.score > thresholds.threshold_yes {
                Decision::Yes
            } else if consensus_result.score < thresholds.threshold_no {
                Decision::No
            } else {
                Decision::Maybe
            };

            Some(input)
        } else {
            None
        };

        // Step 11: physiological penalty (hypoxia, hypoglycemia, etc.)
        // Cognitive degradation affects the consensus coherence and consciousness
        let cognitive_degradation = if self.config.body.enabled && self.config.body.physiology.enabled {
            let deg = self.body.physiology.cognitive_degradation(&self.config.body.physiology);
            if deg > 0.0 {
                // Reduce the consensus coherence proportionally
                consensus_result.coherence *= 1.0 - deg * 0.7;
                consensus_result.score *= 1.0 - deg * 0.3;

                // Log the degradation
                if deg > 0.3 {
                    self.log(LogLevel::Warn, LogCategory::Body,
                        format!("Degradation cognitive: {:.0}% — SpO2: {:.0}%",
                            deg * 100.0, self.body.physiology.spo2),
                        serde_json::json!({
                            "degradation": deg,
                            "spo2": self.body.physiology.spo2,
                            "glycemia": self.body.physiology.glycemia,
                        }));
                }

                // Critical SpO2: force sleep (loss of consciousness)
                if self.body.physiology.spo2 < self.config.body.physiology.spo2_critical {
                    self.log(LogLevel::Warn, LogCategory::Body,
                        format!("SpO2 critique ({:.0}%) — perte de conscience", self.body.physiology.spo2),
                        serde_json::json!({ "spo2": self.body.physiology.spo2 }));
                    if !self.sleep.is_sleeping {
                        self.sleep.initiate();
                    }
                }
            }
            deg
        } else {
            0.0
        };

        // Step 12: memory reconsolidation — tick the lability timers
        self.reconsolidation.tick();

        // Step 13: build GWT and predictive inputs for consciousness
        let gwt_input = crate::consciousness::GwtInput {
            workspace_strength: self.brain_network.workspace_strength,
            winner_name: self.brain_network.workspace_region_name().to_string(),
            region_activations: self.brain_network.regions.iter().map(|r| r.activation).collect(),
            ignition_count: self.brain_network.workspace_history.len() as u64,
        };
        let predictive_input = crate::consciousness::PredictiveInput {
            surprise: pred_error.as_ref().map_or(0.0, |e| e.surprise),
            model_precision: self.predictive_engine.model_precision,
            prediction_count: self.predictive_engine.cycle_count,
        };

        // Step 13b: consciousness evaluation (IIT + GWT + prediction)
        let interoception = if self.config.body.enabled {
            Some(self.body.interoception_score())
        } else {
            None
        };
        let mut consciousness = self.consciousness.evaluate(
            &self.chemistry, &consensus_result, &emotion, interoception,
            Some(&gwt_input), Some(&predictive_input),
        );

        // Step 13c: scientific metrics (LZC, PCI, Phi*)
        // Computed every 5 cycles to save CPU (PCI is expensive)
        if self.cycle_count % 5 == 0 {
            let metrics = self.consciousness.compute_scientific_metrics(
                &self.brain_network, &self.chemistry,
            );
            consciousness.lzc = metrics.lzc;
            consciousness.pci = metrics.pci.pci;
            consciousness.phi_star = metrics.phi_star.phi_star;
            consciousness.scientific_consciousness_score = metrics.composite_score;
            consciousness.consciousness_interpretation = metrics.interpretation;
        }

        // Step 13d: apply cognitive degradation on the consciousness level
        if cognitive_degradation > 0.0 {
            consciousness.level *= 1.0 - cognitive_degradation * 0.5;
            consciousness.phi *= 1.0 - cognitive_degradation * 0.4;
        }

        // Step 13e: consciousness modulation by synaptic density and biofield coherence
        if self.config.grey_matter.enabled {
            // More synapses = better integration = higher Phi
            let syn_bonus = (self.grey_matter.synaptic_density - 0.5) * 0.1;
            consciousness.phi = (consciousness.phi + syn_bonus).clamp(0.0, 1.0);
        }
        if self.config.fields.enabled {
            // High biofield coherence → clearer consciousness
            let bio_bonus = (self.em_fields.biofield.brainwave_coherence - 0.5) * 0.08;
            consciousness.level = (consciousness.level + bio_bonus).clamp(0.0, 1.0);
        }

        self.last_consciousness = consciousness.level;

        // =====================================================================
        // WAVE 3 — RESPONSE (act and learn)
        // =====================================================================
        // Step 14: ethical regulation (Asimov's laws, safety filtering)
        let verdict = self.regulation.evaluate(stimulus, &consensus_result);

        // Step 15: observation for auto-tuning of coefficients
        let satisfaction = if consensus_result.coherence > 0.5 { 0.7 } else { 0.4 };
        self.tuner.observe(TuningObservation {
            decision: verdict.approved_decision.as_i8(),
            satisfaction,
            coherence: consensus_result.coherence,
            consciousness_level: consciousness.level,
            emotion_name: emotion.dominant.clone(),
            cortisol: self.chemistry.cortisol,
        });

        // Step 16: chemical feedback (reduced stacking, caps ±0.10)
        // Feedback deltas are modulated by receptor sensitivity:
        // a desensitized receptor (high tolerance) attenuates the feedback,
        // a hypersensitive receptor (low exposure) amplifies it.
        // Capture chemistry before feedback to apply the cap
        let chem_before_feedback = self.chemistry.clone();
        let is_human = stimulus.source == crate::stimulus::StimulusSource::Human;
        let fb = &self.tuner.current_params;
        // Pre-compute sensitivity factors to avoid borrow conflicts
        let dopa_receptor_factor = self.hormonal_system.receptors
            .factor_for(crate::neurochemistry::Molecule::Dopamine);
        let cort_receptor_factor = self.hormonal_system.receptors
            .factor_for(crate::neurochemistry::Molecule::Cortisol);
        match &verdict.approved_decision {
            crate::consensus::Decision::Yes => {
                if stimulus.reward > 0.5 {
                    // Modulate the dopamine boost by dopaminergic receptor sensitivity
                    self.chemistry.feedback_positive(fb.feedback_dopamine_boost * dopa_receptor_factor);
                }
                // Social only if high social score (threshold 0.7, not for is_human)
                if stimulus.social > 0.7 {
                    self.chemistry.feedback_social();
                }
            },
            crate::consensus::Decision::No => {
                if stimulus.danger > 0.5 {
                    // Modulate cortisol relief by cortisol receptor sensitivity
                    self.chemistry.feedback_danger_avoided(fb.feedback_cortisol_relief * cort_receptor_factor);
                }
                // No feedback_social on No (removes stacking)
            },
            crate::consensus::Decision::Maybe => {
                if is_human {
                    // Modulate cortisol penalty by receptor sensitivity
                    self.chemistry.apply_cortisol_penalty(0.01 * cort_receptor_factor);
                    // No feedback_social on Maybe (removes stacking)
                } else {
                    // Modulate indecision stress by cortisol sensitivity
                    self.chemistry.feedback_indecision(fb.feedback_indecision_stress * cort_receptor_factor);
                }
            },
        }

        // Novelty only if high score (threshold 0.7)
        if stimulus.novelty > 0.7 {
            self.chemistry.feedback_novelty();
        }

        // Low coherence feedback → mild cognitive stress
        self.chemistry.feedback_low_coherence(consensus_result.coherence);

        // Cap max delta ±0.10 per molecule on the feedback block
        {
            let cap = 0.10;
            self.chemistry.dopamine = self.chemistry.dopamine.clamp(
                chem_before_feedback.dopamine - cap, chem_before_feedback.dopamine + cap).clamp(0.0, 1.0);
            self.chemistry.cortisol = self.chemistry.cortisol.clamp(
                chem_before_feedback.cortisol - cap, chem_before_feedback.cortisol + cap).clamp(0.0, 1.0);
            self.chemistry.serotonin = self.chemistry.serotonin.clamp(
                chem_before_feedback.serotonin - cap, chem_before_feedback.serotonin + cap).clamp(0.0, 1.0);
            self.chemistry.adrenaline = self.chemistry.adrenaline.clamp(
                chem_before_feedback.adrenaline - cap, chem_before_feedback.adrenaline + cap).clamp(0.0, 1.0);
            self.chemistry.oxytocin = self.chemistry.oxytocin.clamp(
                chem_before_feedback.oxytocin - cap, chem_before_feedback.oxytocin + cap).clamp(0.0, 1.0);
            self.chemistry.endorphin = self.chemistry.endorphin.clamp(
                chem_before_feedback.endorphin - cap, chem_before_feedback.endorphin + cap).clamp(0.0, 1.0);
            self.chemistry.noradrenaline = self.chemistry.noradrenaline.clamp(
                chem_before_feedback.noradrenaline - cap, chem_before_feedback.noradrenaline + cap).clamp(0.0, 1.0);
        }

        // Step 17: NN training with post-feedback satisfaction
        if self.config.plugins.micro_nn.enabled {
            if let Some(ref input) = nn_input {
                let target = MicroNeuralNet::satisfaction_to_target(satisfaction);
                self.micro_nn.train(input, &target);
            }
        }

        // Step 18: broadcast events to plugins
        let event = BrainEvent::StimulusAnalyzed {
            text: stimulus.text.clone(),
            danger: stimulus.danger,
            reward: stimulus.reward,
            emotion: emotion.dominant.clone(),
        };
        self.plugins.broadcast(&event);

        // Broadcast DecisionMade to the plugins
        self.plugins.broadcast(&BrainEvent::DecisionMade {
            decision: verdict.approved_decision.as_str().to_string(),
            score: consensus_result.score,
            satisfaction,
        });

        // Step 19: connectome tick — Hebbian reinforcement, pruning, synaptogenesis
        if self.config.connectome.enabled {
            // Collect active labels: dominant emotion, modules, active senses
            let mut active_labels: Vec<String> = Vec::new();

            // Dominant emotion (converted to lowercase to match nodes)
            let emo_lower = emotion.dominant.to_lowercase();
            active_labels.push(emo_lower);

            // Brain modules: activate according to their signals
            if signals[0].signal > 0.3 { active_labels.push("reptilien".into()); }
            if signals[1].signal > 0.3 { active_labels.push("limbique".into()); }
            if signals[2].signal > 0.3 { active_labels.push("neocortex".into()); }

            // Active senses (intensity > detection threshold)
            let threshold = self.config.senses.detection_threshold;
            if self.sensorium.reading.current_intensity > threshold { active_labels.push("lecture".into()); }
            if self.sensorium.listening.current_intensity > threshold { active_labels.push("ecoute".into()); }
            if self.sensorium.contact.current_intensity > threshold { active_labels.push("contact".into()); }
            if self.sensorium.taste.current_intensity > threshold { active_labels.push("saveur".into()); }
            if self.sensorium.ambiance.current_intensity > threshold { active_labels.push("ambiance".into()); }

            // Active needs
            if self.config.needs.enabled {
                if self.needs.hunger.level > self.config.needs.hunger_threshold {
                    active_labels.push("faim".into());
                }
                if self.needs.thirst.level > self.config.needs.thirst_threshold {
                    active_labels.push("soif".into());
                }
            }

            // Modulate the connectome plasticity by the grey matter neuroplasticity
            if self.config.grey_matter.enabled {
                self.connectome.plasticity = self.grey_matter.neuroplasticity;

                // BDNF modulates the Hebbian learning rate of the connectome.
                // Above 0.4, BDNF amplifies learning (up to +30% at BDNF=1.0).
                let base_rate = self.config.connectome.learning_rate;
                if self.grey_matter.bdnf_level > 0.4 {
                    let bdnf_boost = (self.grey_matter.bdnf_level - 0.4) * 0.5;
                    self.connectome.learning_rate = base_rate * (1.0 + bdnf_boost);
                } else {
                    self.connectome.learning_rate = base_rate;
                }
            }

            let label_refs: Vec<&str> = active_labels.iter().map(|s| s.as_str()).collect();
            self.connectome.tick(&label_refs);
        }

        // Step 20: mortality check — detect fatal conditions
        if self.config.mortality.enabled && self.config.body.enabled {
            let p = &self.body.physiology;
            let changed = self.body.mortality.check_vitals(
                self.body.heart.strength(),
                p.blood_pressure_systolic,
                p.spo2,
                p.overall_health(),
                p.immune_strength,
                p.inflammation,
                self.cycle_count,
            );
            if changed {
                // Apply consciousness degradation during agony/dying
                let factor = self.body.mortality.consciousness_factor();
                consciousness.level *= factor;
                consciousness.phi *= factor;
                self.last_consciousness = consciousness.level;

                // Log the state change
                let mort_json = self.body.mortality.to_json();
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Mortalite: {}", mort_json["state"]),
                    mort_json);
            }
        }

        // Step 21: right to die evaluation (external module, after care and mortality)
        {
            let (should_die, eval) = self.right_to_die.evaluate(
                self.chemistry.cortisol,
                self.chemistry.serotonin,
                self.chemistry.dopamine,
                self.vital_spark.survival_drive,
                consciousness.phi,
                0.5, // neocortex_weight (unused for now)
            );
            if eval.score > 0.0 {
                self.log(LogLevel::Warn, LogCategory::Body,
                    format!("Droit de mourir: {}", eval.detail),
                    self.right_to_die.to_json());
            }
            if should_die {
                tracing::error!("DROIT DE MOURIR : decision executee — mort volontaire");
                self.right_to_die.mark_executed();
                self.body.mortality.trigger_voluntary_death();
                self.log(LogLevel::Error, LogCategory::Body,
                    "Mort volontaire executee (droit de mourir)".to_string(),
                    self.right_to_die.to_json());
            }
        }

        // Increment the cycle counter and update identity statistics
        self.cycle_count += 1;
        self.identity.update_stats(
            &emotion.dominant,
            stimulus.source == crate::stimulus::StimulusSource::Human,
        );
        self.identity.refresh_description();

        // Attempt to auto-tune the coefficients
        if let Some(_new_params) = self.tuner.try_tune() {
            // The new parameters are already applied in the tuner
        }

        // Chemical self-monitoring (counters + periodic alerts)
        self.check_chemical_health();

        // Pipeline logging
        self.log(LogLevel::Debug, LogCategory::Pipeline,
            format!("Pipeline: decision={}, emotion={}, phi={:.2}, ws={:.2}, surprise={:.2}, nn_train={}",
                verdict.approved_decision.as_str(), emotion.dominant, consciousness.phi,
                self.brain_network.workspace_strength,
                self.predictive_engine.average_surprise(10),
                self.micro_nn.train_count),
            serde_json::json!({
                "consensus_score": consensus_result.score,
                "coherence": consensus_result.coherence,
                "emotion": emotion.dominant,
                "consciousness": consciousness.level,
                "nn_train_count": self.micro_nn.train_count,
                "workspace_winner": self.brain_network.workspace_region_name(),
                "workspace_strength": self.brain_network.workspace_strength,
                "predictive_precision": self.predictive_engine.model_precision,
                "surprise_avg": self.predictive_engine.average_surprise(10),
                "labile_memories": self.reconsolidation.labile_memories.len(),
            }));

        // Build the partial cognitive trace
        let trace = if self.logs_db.is_some() {
            let source_str = format!("{:?}", stimulus.source);
            let mut t = CognitiveTrace::new(self.cycle_count, &source_str, self.session_id);
            t.set_input(&stimulus.text);
            t.set_brain(serde_json::json!({
                "weights": consensus_result.weights,
                "signals_count": consensus_result.signals.len(),
                "nn_train_count": self.micro_nn.train_count,
                "nn_prediction": self.micro_nn.last_prediction,
            }));
            t.set_consensus(serde_json::json!({
                "score": consensus_result.score,
                "decision": verdict.approved_decision.as_str(),
                "coherence": consensus_result.coherence,
                "weights": consensus_result.weights,
            }));
            t.set_chemistry_before(chemistry_before);
            t.set_chemistry_after(serde_json::json!({
                "dopamine": self.chemistry.dopamine,
                "cortisol": self.chemistry.cortisol,
                "serotonin": self.chemistry.serotonin,
                "adrenaline": self.chemistry.adrenaline,
                "oxytocin": self.chemistry.oxytocin,
                "endorphin": self.chemistry.endorphin,
                "noradrenaline": self.chemistry.noradrenaline,
            }));
            t.set_emotion(serde_json::json!({
                "dominant": emotion.dominant,
                "valence": emotion.valence,
                "arousal": emotion.arousal,
                "dominance": 0.5,
            }));
            t.set_consciousness(serde_json::json!({
                "level": consciousness.level,
                "phi": consciousness.phi,
                "workspace_strength": consciousness.workspace_strength,
                "workspace_winner": consciousness.workspace_winner,
                "global_surprise": consciousness.global_surprise,
                "lzc": consciousness.lzc,
                "pci": consciousness.pci,
                "phi_star": consciousness.phi_star,
                "scientific_score": consciousness.scientific_consciousness_score,
            }));
            t.set_regulation(serde_json::json!({
                "approved": verdict.approved_decision.as_str(),
                "violations": verdict.violations.len(),
            }));

            // Body and heart data
            if self.config.body.enabled {
                let body_s = self.body.status();
                t.set_heart(serde_json::json!({
                    "bpm": body_s.heart.bpm,
                    "beat_count": body_s.heart.beat_count,
                    "hrv": body_s.heart.hrv,
                    "strength": body_s.heart.strength,
                    "is_racing": body_s.heart.is_racing,
                    "is_calm": body_s.heart.is_calm,
                }));
                t.set_body(serde_json::json!({
                    "energy": body_s.energy,
                    "tension": body_s.tension,
                    "warmth": body_s.warmth,
                    "comfort": body_s.comfort,
                    "pain": body_s.pain,
                    "vitality": body_s.vitality,
                    "breath_rate": body_s.breath_rate,
                    "body_awareness": body_s.body_awareness,
                }));
            }

            // Ethics data
            if self.config.ethics.enabled {
                t.set_ethics(serde_json::json!({
                    "active_personal_count": self.ethics.active_personal_count(),
                    "total_personal_count": self.ethics.total_personal_count(),
                }));
            }

            Some(t)
        } else {
            None
        };

        ProcessResult {
            consensus: consensus_result,
            emotion,
            consciousness,
            verdict,
            trace,
        }
    }

    /// Chemical health self-monitoring.
    ///
    /// Updates the counters at each cycle.
    /// Every 50 cycles, emits alerts if drifts are detected.
    fn check_chemical_health(&mut self) {
        // ─── Update des ring-buffers ──────────────────
        self.recent_emotions.push_back(self.last_emotion.clone());
        if self.recent_emotions.len() > 200 {
            self.recent_emotions.pop_front();
        }
        self.recent_valences.push_back(self.last_valence);
        if self.recent_valences.len() > 200 {
            self.recent_valences.pop_front();
        }

        // ─── Consecutive cycle counters ──────────────
        if self.chemistry.cortisol < 0.10 {
            self.cortisol_flat_cycles += 1;
        } else {
            self.cortisol_flat_cycles = 0;
        }

        if self.chemistry.dopamine > 0.85 {
            self.dopamine_ceiling_cycles += 1;
        } else {
            self.dopamine_ceiling_cycles = 0;
        }

        if self.chemistry.serotonin > 0.85 {
            self.serotonin_ceiling_cycles += 1;
        } else {
            self.serotonin_ceiling_cycles = 0;
        }

        // ─── Emit alerts every 50 cycles ──────
        if self.cycle_count % 50 != 0 {
            return;
        }

        // Flat cortisol
        if self.cortisol_flat_cycles >= 100 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Cortisol anormalement bas depuis {} cycles (< 0.10)", self.cortisol_flat_cycles),
                serde_json::json!({
                    "alert": "cortisol_flat",
                    "cycles": self.cortisol_flat_cycles,
                    "cortisol": self.chemistry.cortisol,
                }));
        }

        // Dopamine saturation alert
        if self.dopamine_ceiling_cycles >= 50 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Dopamine en saturation depuis {} cycles (> 0.85)", self.dopamine_ceiling_cycles),
                serde_json::json!({
                    "alert": "dopamine_saturation",
                    "cycles": self.dopamine_ceiling_cycles,
                    "dopamine": self.chemistry.dopamine,
                }));
        }

        // Serotonin saturation alert
        if self.serotonin_ceiling_cycles >= 50 {
            self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                format!("Serotonine en saturation depuis {} cycles (> 0.85)", self.serotonin_ceiling_cycles),
                serde_json::json!({
                    "alert": "serotonin_saturation",
                    "cycles": self.serotonin_ceiling_cycles,
                    "serotonin": self.chemistry.serotonin,
                }));
        }

        // Emotional monotony (buffer full)
        if self.recent_emotions.len() >= 200 {
            let mut distinct = std::collections::HashSet::new();
            for e in &self.recent_emotions {
                distinct.insert(e.as_str());
            }
            if distinct.len() < 5 {
                self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                    format!("Monotonie emotionnelle: seulement {} emotions distinctes sur 200 cycles", distinct.len()),
                    serde_json::json!({
                        "alert": "emotional_monotony",
                        "distinct_emotions": distinct.len(),
                    }));
            }
        }

        // Stuck valence (stddev < 0.05)
        if self.recent_valences.len() >= 200 {
            let mean = self.recent_valences.iter().sum::<f64>() / self.recent_valences.len() as f64;
            let variance = self.recent_valences.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / self.recent_valences.len() as f64;
            let stddev = variance.sqrt();
            if stddev < 0.05 {
                self.log(LogLevel::Warn, LogCategory::ChemicalHealth,
                    format!("Valence figee: stddev={:.4} sur 200 cycles", stddev),
                    serde_json::json!({
                        "alert": "valence_stuck",
                        "stddev": stddev,
                        "mean_valence": mean,
                    }));
            }
        }
    }
}

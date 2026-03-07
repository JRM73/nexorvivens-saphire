// consolidation.rs — Consolidation process: episodic to long-term memory transfer
//
// This module implements the memory consolidation process, analogous to the role
// of slow-wave sleep (SWS) and REM sleep in human memory consolidation. It
// transfers sufficiently important episodic memories to long-term memory (LTM),
// applies strength decay to remaining episodic memories, prunes those that have
// become too weak, and archives excess LTM memories in compressed batches.
//
// The consolidation process mirrors the complementary learning systems (CLS)
// theory (McClelland et al., 1995): the hippocampus (episodic store) rapidly
// encodes experiences, and during consolidation (sleep replay), the most
// valuable traces are gradually transferred to the neocortex (LTM) for
// permanent storage.
//
// The process runs in 7 sequential steps:
//   1. Retrieve unconsolidated episodic candidates.
//   2. Compute consolidation score and generate the vector embedding.
//   3. Store in LTM via the `memories` table (PostgreSQL + pgvector).
//   4. Mark the episodic memory as consolidated.
//   5. Apply strength decay to unconsolidated episodic memories.
//   6. Prune episodic memories if the count exceeds the configured maximum.
//   7. Prune LTM with protection rules and archive excess memories.

use crate::db::{SaphireDb, NewMemory};
use crate::db::archives::NewArchive;
use crate::memory::episodic::consolidation_score;
use crate::memory::long_term::LocalEncoder;

/// Grouped parameters for the consolidation process.
/// Replaces the previous 6+ positional parameters with a single named struct.
pub struct ConsolidationParams {
    /// Minimum consolidation score for transfer to LTM (0.0 to 1.0).
    pub threshold: f64,
    /// Per-cycle strength decay rate for episodic memories.
    pub decay_rate: f64,
    /// Maximum number of episodic memories allowed before pruning.
    pub max_episodic: usize,
    /// Target number of episodic memories after a pruning pass.
    pub episodic_prune_target: usize,
    /// Maximum number of LTM memories (beyond this, pruning + archiving kicks in).
    pub ltm_max: usize,
    /// Target number of LTM memories after a pruning pass.
    pub ltm_prune_target: usize,
    /// Minimum access count for an LTM memory to be protected from pruning.
    /// Models the testing effect: frequently retrieved traces resist forgetting.
    pub ltm_protection_access_count: i32,
    /// Minimum emotional weight for an LTM memory to be protected from pruning.
    /// Models amygdala-mediated consolidation enhancement for emotional memories.
    pub ltm_protection_emotional_weight: f32,
    /// Batch size for archiving pruned LTM memories into compressed summaries.
    pub archive_batch_size: usize,
}

/// Report produced by a single consolidation run, summarizing all actions taken.
#[derive(Debug, Default)]
pub struct ConsolidationReport {
    /// Number of episodic memories transferred to LTM.
    pub consolidated: u64,
    /// Number of episodic memories that underwent strength decay.
    pub decayed: u64,
    /// Number of episodic memories deleted during pruning.
    pub pruned: u64,
    /// Number of LTM memories pruned (transferred to archives).
    pub ltm_pruned: u64,
    /// Number of archive entries created from pruned LTM memories.
    pub archived: u64,
}

/// Executes the complete 7-step consolidation process.
///
/// This is the central function of the memory consolidation pipeline, called
/// periodically (every `consolidation_interval_cycles` cognitive cycles) and
/// optionally during Saphire's sleep phase.
///
/// # Parameters
/// - `db`: database access handle for all persistence operations.
/// - `encoder`: local TF-IDF vector encoder for generating embeddings.
/// - `params`: consolidation parameters (thresholds, limits, rates).
///
/// # Returns
/// A `ConsolidationReport` summarizing the number of memories consolidated,
/// decayed, pruned, and archived.
pub async fn consolidate(
    db: &SaphireDb,
    encoder: &LocalEncoder,
    params: &ConsolidationParams,
) -> ConsolidationReport {
    let mut report = ConsolidationReport::default();

    // Step 1: Retrieve episodic memories that are candidates for consolidation
    // (i.e., not yet consolidated and still present in the episodic store).
    let candidates = match db.episodic_consolidation_candidates().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Consolidation: error retrieving candidates: {}", e);
            return report;
        }
    };

    tracing::info!(
        "Consolidation: {} candidates found, threshold={}",
        candidates.len(), params.threshold
    );

    for candidate in &candidates {
        // Compute the multi-factor consolidation score for this episodic memory.
        let score = consolidation_score(candidate);

        tracing::debug!(
            "Consolidation candidate [{}]: score={:.3} (emotion={:.2}, strength={:.2}, access={}), source={}",
            candidate.id, score, candidate.emotional_intensity,
            candidate.strength, candidate.access_count, candidate.source_type
        );

        if score >= params.threshold {
            // Step 2: Generate the vector embedding for the textual content.
            // The encoder produces a TF-IDF feature vector with FNV-1a hashing,
            // which is then stored alongside the memory for cosine similarity search.
            let embedding_f64 = encoder.encode(&candidate.content);
            // Convert from f64 to f32 for pgvector storage efficiency.
            let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

            // Step 3: Store the memory in LTM (the `memories` table with pgvector index).
            let memory = NewMemory {
                embedding: embedding_f32,
                text_summary: candidate.content.clone(),
                stimulus_json: candidate.stimulus_json.clone().unwrap_or_default(),
                decision: candidate.decision.unwrap_or(0),
                chemistry_json: candidate.chemistry_json.clone().unwrap_or_default(),
                emotion: candidate.emotion.clone(),
                mood_valence: 0.0,
                satisfaction: candidate.satisfaction,
                emotional_weight: score as f32,
                source_episodic_id: Some(candidate.id),
                chemical_signature: candidate.chemical_signature.clone(),
            };

            match db.store_memory(&memory).await {
                Ok(_ltm_id) => {
                    // Step 4: Mark the episodic source as consolidated so it
                    // is not re-processed in future consolidation runs.
                    let _ = db.mark_episodic_consolidated(candidate.id).await;
                    report.consolidated += 1;
                },
                Err(e) => {
                    tracing::warn!("Consolidation: error storing LTM memory: {}", e);
                }
            }
        }
    }

    // Step 4b: Clean up episodic memories that were already consolidated.
    // These are no longer needed in the episodic store since their content
    // now lives in LTM.
    match db.cleanup_consolidated_episodic().await {
        Ok(n) if n > 0 => tracing::info!("Consolidation: {} consolidated episodic memories cleaned up", n),
        Ok(_) => {},
        Err(e) => tracing::warn!("Consolidation: error cleaning up consolidated memories: {}", e),
    }

    // Step 5: Apply strength decay to unconsolidated episodic memories.
    // Each memory's strength is reduced by decay_rate, modeling the natural
    // fading of hippocampal traces over time (Ebbinghaus-style exponential decay).
    match db.decay_episodic(params.decay_rate).await {
        Ok(n) => report.decayed = n,
        Err(e) => tracing::warn!("Consolidation: error during episodic decay: {}", e),
    }

    // Step 6: Prune episodic memories if the count exceeds max_episodic.
    // The weakest (lowest-strength) memories are removed first, down to the
    // episodic_prune_target count.
    match db.count_episodic().await {
        Ok(count) if count as usize > params.max_episodic => {
            let to_prune = (count as usize - params.episodic_prune_target) as i64;
            if to_prune > 0 {
                match db.prune_episodic(to_prune).await {
                    Ok(n) => report.pruned = n,
                    Err(e) => tracing::warn!("Consolidation: error during episodic pruning: {}", e),
                }
            }
        },
        _ => {}
    }

    // Step 7: Prune LTM with protection rules and archive excess memories.
    // Protected memories (access_count >= threshold OR emotional_weight >= threshold)
    // are spared. Pruned memories are archived in compressed batch summaries —
    // they are never silently deleted. This ensures no information is permanently
    // lost, only compressed and moved to a lower-priority retrieval tier.
    match db.count_ltm().await {
        Ok(ltm_count) if ltm_count as usize > params.ltm_max => {
            let to_prune = ltm_count as usize - params.ltm_prune_target;
            tracing::info!(
                "LTM pruning: {} memories (max={}), pruning {} to reach target={}",
                ltm_count, params.ltm_max, to_prune, params.ltm_prune_target
            );

            // Retrieve the weakest unprotected LTM memories for pruning.
            match db.fetch_ltm_weakest_unprotected(
                to_prune as i64,
                params.ltm_protection_access_count,
                params.ltm_protection_emotional_weight,
            ).await {
                Ok(weak_memories) => {
                    if weak_memories.is_empty() {
                        tracing::info!("LTM pruning: no unprotected memories available to prune");
                    } else {
                        let batch_size = params.archive_batch_size;
                        // Process in batches to control archive granularity.
                        for batch in weak_memories.chunks(batch_size) {
                            // Build a concatenated summary for the batch.
                            let summary: String = batch.iter()
                                .map(|m| {
                                    let preview: String = m.text_summary.chars().take(120).collect();
                                    preview
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");

                            // Compute the L2-normalized average embedding for the batch.
                            // This centroid vector enables cosine similarity search
                            // against the archive, preserving approximate semantic access.
                            let mut avg_embedding = vec![0.0f32; 64];
                            let mut embedding_count = 0usize;
                            for m in batch {
                                // Re-generate embedding from text (not stored in the record).
                                let emb_f64 = encoder.encode(&m.text_summary);
                                for (i, &v) in emb_f64.iter().enumerate() {
                                    if i < 64 {
                                        avg_embedding[i] += v as f32;
                                    }
                                }
                                embedding_count += 1;
                            }
                            if embedding_count > 0 {
                                // Element-wise mean of all embeddings in the batch.
                                for v in &mut avg_embedding {
                                    *v /= embedding_count as f32;
                                }
                                // L2 normalization to produce a unit vector for cosine similarity.
                                let norm: f32 = avg_embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
                                if norm > 0.0 {
                                    for v in &mut avg_embedding {
                                        *v /= norm;
                                    }
                                }
                            }

                            // Deduplicate emotions across the batch.
                            let mut emotions: Vec<String> = batch.iter()
                                .map(|m| m.emotion.clone())
                                .filter(|e| !e.is_empty())
                                .collect();
                            emotions.sort();
                            emotions.dedup();

                            // Determine the time period spanned by this batch
                            // (earliest to latest created_at timestamp).
                            let period_start = batch.iter()
                                .map(|m| m.created_at)
                                .min()
                                .unwrap_or_else(chrono::Utc::now);
                            let period_end = batch.iter()
                                .map(|m| m.created_at)
                                .max()
                                .unwrap_or_else(chrono::Utc::now);

                            // Compute the average emotional weight across the batch.
                            let avg_weight: f32 = batch.iter()
                                .map(|m| m.emotional_weight)
                                .sum::<f32>() / batch.len() as f32;

                            let source_ids: Vec<i64> = batch.iter().map(|m| m.id).collect();

                            let archive = NewArchive {
                                summary,
                                source_count: batch.len() as i32,
                                source_ids: source_ids.clone(),
                                emotions,
                                period_start,
                                period_end,
                                avg_emotional_weight: avg_weight,
                                embedding: avg_embedding,
                            };

                            match db.store_archive(&archive).await {
                                Ok(archive_id) => {
                                    tracing::info!(
                                        "Archive #{} created: {} memories compressed",
                                        archive_id, batch.len()
                                    );
                                    report.archived += 1;

                                    // Delete the source memories now that they are archived.
                                    match db.delete_memories_by_ids(&source_ids).await {
                                        Ok(n) => report.ltm_pruned += n,
                                        Err(e) => tracing::warn!(
                                            "LTM pruning: error deleting batch: {}", e
                                        ),
                                    }
                                },
                                Err(e) => {
                                    tracing::warn!("LTM pruning: error creating archive: {}", e);
                                    // Do NOT delete source memories if archiving failed —
                                    // this prevents data loss.
                                }
                            }
                        }

                        tracing::info!(
                            "LTM pruning complete: {} memories pruned, {} archives created",
                            report.ltm_pruned, report.archived
                        );
                    }
                },
                Err(e) => tracing::warn!("LTM pruning: error retrieving weak memories: {}", e),
            }
        },
        _ => {}
    }

    report
}

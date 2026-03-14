// consolidation.rs — Episodic-to-long-term consolidation process
//
// This module implements the memory consolidation process, analogous
// to the role of sleep in human mnesic consolidation. It transfers
// sufficiently important episodic memories to long-term memory (LTM),
// applies decay to remaining memories, prunes those that have become
// too weak, and archives excess LTM memories.
//
// The process runs in 7 sequential steps:
//   1. Retrieve episodic candidates not yet consolidated.
//   2. Compute consolidation score and generate the vector embedding.
//   3. Store in LTM via the `memories` table (PostgreSQL + pgvector).
//   4. Mark the episodic memory as consolidated.
//   5. Decay the strength of unconsolidated memories.
//   6. Prune episodic memories if count exceeds the maximum.
//   7. Prune LTM with protection and archiving if count > ltm_max.

use crate::db::{SaphireDb, NewMemory, MemoryRecord};
use crate::db::archives::NewArchive;
use crate::memory::episodic::consolidation_score;
use crate::vectorstore::encoder::TextEncoder;
use crate::vectorstore::cosine_similarity;

/// Consolidation parameters grouped in a single structure.
/// Replaces the 6 positional parameters of the former signature.
pub struct ConsolidationParams {
    /// Minimum consolidation score for transfer to LTM
    pub threshold: f64,
    /// Decay rate for episodic memories
    pub decay_rate: f64,
    /// Maximum number of episodic memories
    pub max_episodic: usize,
    /// Target number of episodic memories after pruning
    pub episodic_prune_target: usize,
    /// Maximum number of LTM memories (beyond this, pruning + archiving occurs)
    pub ltm_max: usize,
    /// Target number of LTM memories after pruning
    pub ltm_prune_target: usize,
    /// Minimum access count to protect an LTM memory from pruning
    pub ltm_protection_access_count: i32,
    /// Minimum emotional weight to protect an LTM memory from pruning
    pub ltm_protection_emotional_weight: f32,
    /// Batch size for archiving
    pub archive_batch_size: usize,
    /// Current BDNF level (0.0 - 1.0) — modulates consolidation strength
    pub bdnf_level: f64,
}

/// Report produced by the consolidation process.
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
    /// Number of archives created from pruned memories.
    pub archived: u64,
}

/// Runs the complete memory consolidation process (7 steps).
pub async fn consolidate(
    db: &SaphireDb,
    encoder: &dyn TextEncoder,
    params: &ConsolidationParams,
) -> ConsolidationReport {
    let mut report = ConsolidationReport::default();

    // Step 1: Retrieve episodic candidates for consolidation.
    let candidates = match db.episodic_consolidation_candidates().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Consolidation: erreur recuperation candidats: {}", e);
            return report;
        }
    };

    tracing::info!(
        "Consolidation: {} candidats trouves, seuil={}",
        candidates.len(), params.threshold
    );

    for candidate in &candidates {
        let score = consolidation_score(candidate, params.bdnf_level);

        tracing::debug!(
            "Consolidation candidat [{}]: score={:.3} (emotion={:.2}, strength={:.2}, access={}), source={}",
            candidate.id, score, candidate.emotional_intensity,
            candidate.strength, candidate.access_count, candidate.source_type
        );

        if score >= params.threshold {
            // Step 2: Generate the vector embedding for the textual content.
            let embedding_f64 = encoder.encode(&candidate.content);
            let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

            // Step 3: Store in LTM.
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
                    // Step 4: Mark as consolidated
                    let _ = db.mark_episodic_consolidated(candidate.id).await;
                    report.consolidated += 1;
                },
                Err(e) => {
                    tracing::warn!("Consolidation: erreur stockage LTM: {}", e);
                }
            }
        }
    }

    // Step 4b: Clean up episodic memories already consolidated.
    match db.cleanup_consolidated_episodic().await {
        Ok(n) if n > 0 => tracing::info!("Consolidation: {} souvenirs consolides nettoyes", n),
        Ok(_) => {},
        Err(e) => tracing::warn!("Consolidation: erreur nettoyage consolides: {}", e),
    }

    // Step 5: Strength decay for unconsolidated episodic memories.
    match db.decay_episodic(params.decay_rate).await {
        Ok(n) => report.decayed = n,
        Err(e) => tracing::warn!("Consolidation: erreur decay: {}", e),
    }

    // Step 6: Episodic pruning if count > max_episodic.
    match db.count_episodic().await {
        Ok(count) if count as usize > params.max_episodic => {
            let to_prune = (count as usize - params.episodic_prune_target) as i64;
            if to_prune > 0 {
                match db.prune_episodic(to_prune).await {
                    Ok(n) => report.pruned = n,
                    Err(e) => tracing::warn!("Consolidation: erreur prune episodique: {}", e),
                }
            }
        },
        _ => {}
    }

    // Step 7: LTM pruning with protection and archiving.
    // Protected memories (access_count >= threshold OR emotional_weight >= threshold)
    // are spared. Pruned memories are archived in compressed batches,
    // never silently deleted.
    match db.count_ltm().await {
        Ok(ltm_count) if ltm_count as usize > params.ltm_max => {
            let to_prune = ltm_count as usize - params.ltm_prune_target;
            tracing::info!(
                "LTM pruning: {} souvenirs (max={}), elagage de {} vers cible={}",
                ltm_count, params.ltm_max, to_prune, params.ltm_prune_target
            );

            // Retrieve the weakest unprotected memories
            match db.fetch_ltm_weakest_unprotected(
                to_prune as i64,
                params.ltm_protection_access_count,
                params.ltm_protection_emotional_weight,
            ).await {
                Ok(weak_memories) => {
                    if weak_memories.is_empty() {
                        tracing::info!("LTM pruning: aucun souvenir non protege a elaguer");
                    } else {
                        // Simple clustering: group memories by semantic similarity
                        // before creating archives. Each cluster produces a coherent archive.
                        let clusters = cluster_by_similarity(&weak_memories, encoder, 0.4);
                        for batch in &clusters {
                            // Build the batch summary
                            let summary: String = batch.iter()
                                .map(|m| {
                                    let preview: String = m.text_summary.chars().take(120).collect();
                                    preview
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");

                            // L2-normalized average embedding
                            let dim = encoder.dim();
                            let mut avg_embedding = vec![0.0f32; dim];
                            let mut embedding_count = 0usize;
                            for m in batch {
                                let emb_f64 = encoder.encode(&m.text_summary);
                                for (i, &v) in emb_f64.iter().enumerate() {
                                    if i < dim {
                                        avg_embedding[i] += v as f32;
                                    }
                                }
                                embedding_count += 1;
                            }
                            if embedding_count > 0 {
                                for v in &mut avg_embedding {
                                    *v /= embedding_count as f32;
                                }
                                // L2 normalization
                                let norm: f32 = avg_embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
                                if norm > 0.0 {
                                    for v in &mut avg_embedding {
                                        *v /= norm;
                                    }
                                }
                            }

                            // Emotion deduplication
                            let mut emotions: Vec<String> = batch.iter()
                                .map(|m| m.emotion.clone())
                                .filter(|e| !e.is_empty())
                                .collect();
                            emotions.sort();
                            emotions.dedup();

                            // Period (min/max created_at)
                            let period_start = batch.iter()
                                .map(|m| m.created_at)
                                .min()
                                .unwrap_or_else(chrono::Utc::now);
                            let period_end = batch.iter()
                                .map(|m| m.created_at)
                                .max()
                                .unwrap_or_else(chrono::Utc::now);

                            // Average emotional weight
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
                                        "Archive #{} creee: {} souvenirs compresses",
                                        archive_id, batch.len()
                                    );
                                    report.archived += 1;

                                    // Delete the source memories
                                    match db.delete_memories_by_ids(&source_ids).await {
                                        Ok(n) => report.ltm_pruned += n,
                                        Err(e) => tracing::warn!(
                                            "LTM pruning: erreur suppression lot: {}", e
                                        ),
                                    }
                                },
                                Err(e) => {
                                    tracing::warn!("LTM pruning: erreur creation archive: {}", e);
                                    // Do NOT delete memories if archiving fails
                                }
                            }
                        }

                        tracing::info!(
                            "LTM pruning termine: {} souvenirs elagués, {} archives creees",
                            report.ltm_pruned, report.archived
                        );
                    }
                },
                Err(e) => tracing::warn!("LTM pruning: erreur recuperation faibles: {}", e),
            }
        },
        _ => {}
    }

    report
}

/// Groups memories by semantic similarity (greedy clustering).
///
/// Algorithm: for each memory, compute its embedding then compare it to
/// the centroid of each existing cluster. If it is sufficiently similar
/// (>= threshold), add it to the closest cluster. Otherwise, create a
/// new cluster.
///
/// Complexity: O(n * k * d) where n = memories, k = clusters, d = dimension.
fn cluster_by_similarity<'a>(
    memories: &'a [MemoryRecord],
    encoder: &dyn TextEncoder,
    threshold: f64,
) -> Vec<Vec<&'a MemoryRecord>> {
    if memories.is_empty() {
        return vec![];
    }

    // Pre-encode all memories
    let embeddings: Vec<Vec<f64>> = memories.iter()
        .map(|m| encoder.encode(&m.text_summary))
        .collect();

    // Clusters: (centroid, indices)
    let mut clusters: Vec<(Vec<f64>, Vec<usize>)> = Vec::new();

    for (i, emb) in embeddings.iter().enumerate() {
        // Find the most similar cluster
        let mut best_cluster = None;
        let mut best_sim = 0.0_f64;

        for (ci, (centroid, _)) in clusters.iter().enumerate() {
            let sim = cosine_similarity(emb, centroid);
            if sim > best_sim {
                best_sim = sim;
                best_cluster = Some(ci);
            }
        }

        if best_sim >= threshold {
            // Add to existing cluster and update centroid (running average)
            let ci = best_cluster.unwrap();
            let (centroid, members) = &mut clusters[ci];
            let n = members.len() as f64;
            for (j, v) in centroid.iter_mut().enumerate() {
                if j < emb.len() {
                    *v = (*v * n + emb[j]) / (n + 1.0);
                }
            }
            members.push(i);
        } else {
            // Create a new cluster
            clusters.push((emb.clone(), vec![i]));
        }
    }

    // Convert indices to references
    clusters.into_iter()
        .map(|(_, indices)| {
            indices.into_iter().map(|i| &memories[i]).collect()
        })
        .collect()
}

// consolidation.rs — Processus de consolidation episodique vers long terme
//
// Ce module implemente le processus de consolidation des souvenirs, analogue
// au role du sommeil dans la consolidation mnesique humaine. Il transfere
// les souvenirs episodiques suffisamment importants vers la memoire a long
// terme (LTM), applique la decroissance sur les souvenirs restants, elague
// ceux devenus trop faibles, et archive les souvenirs LTM excedentaires.
//
// Le processus se deroule en 7 etapes sequentielles :
//   1. Recuperation des candidats episodiques non encore consolides.
//   2. Calcul du score de consolidation et generation de l'embedding vectoriel.
//   3. Stockage en LTM via la table `memories` (PostgreSQL + pgvector).
//   4. Marquage du souvenir episodique comme consolide.
//   5. Decroissance de la force des souvenirs non consolides.
//   6. Elagage (pruning) episodique si le nombre depasse le maximum.
//   7. Elagage LTM avec protection et archivage si count > ltm_max.

use crate::db::{SaphireDb, NewMemory, MemoryRecord};
use crate::db::archives::NewArchive;
use crate::memory::episodic::consolidation_score;
use crate::vectorstore::encoder::TextEncoder;
use crate::vectorstore::cosine_similarity;

/// Parametres de consolidation regroupes en une seule structure.
/// Remplace les 6 parametres positionnels de l'ancienne signature.
pub struct ConsolidationParams {
    /// Score minimum de consolidation pour transfert vers LTM
    pub threshold: f64,
    /// Taux de decroissance des souvenirs episodiques
    pub decay_rate: f64,
    /// Nombre max de souvenirs episodiques
    pub max_episodic: usize,
    /// Nombre cible de souvenirs episodiques apres elagage
    pub episodic_prune_target: usize,
    /// Nombre max de souvenirs LTM (au-dela, elagage + archivage)
    pub ltm_max: usize,
    /// Nombre cible de souvenirs LTM apres elagage
    pub ltm_prune_target: usize,
    /// Nombre min d'acces pour proteger un souvenir LTM
    pub ltm_protection_access_count: i32,
    /// Poids emotionnel min pour proteger un souvenir LTM
    pub ltm_protection_emotional_weight: f32,
    /// Taille des lots d'archivage
    pub archive_batch_size: usize,
    /// Niveau courant de BDNF (0.0 - 1.0) — module la force de consolidation
    pub bdnf_level: f64,
}

/// Rapport produit par le processus de consolidation.
#[derive(Debug, Default)]
pub struct ConsolidationReport {
    /// Nombre de souvenirs episodiques transferes vers la LTM.
    pub consolidated: u64,
    /// Nombre de souvenirs episodiques ayant subi une decroissance de force.
    pub decayed: u64,
    /// Nombre de souvenirs episodiques supprimes lors de l'elagage.
    pub pruned: u64,
    /// Nombre de souvenirs LTM elagués (transferes vers les archives).
    pub ltm_pruned: u64,
    /// Nombre d'archives creees a partir des souvenirs elagués.
    pub archived: u64,
}

/// Execute le processus complet de consolidation des souvenirs (7 etapes).
pub async fn consolidate(
    db: &SaphireDb,
    encoder: &dyn TextEncoder,
    params: &ConsolidationParams,
) -> ConsolidationReport {
    let mut report = ConsolidationReport::default();

    // Etape 1 : Recuperer les candidats episodiques a la consolidation.
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
            // Etape 2 : Generer l'embedding vectoriel du contenu textuel.
            let embedding_f64 = encoder.encode(&candidate.content);
            let embedding_f32: Vec<f32> = embedding_f64.iter().map(|&v| v as f32).collect();

            // Etape 3 : Stocker en LTM.
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
                    // Etape 4 : Marquer comme consolide
                    let _ = db.mark_episodic_consolidated(candidate.id).await;
                    report.consolidated += 1;
                },
                Err(e) => {
                    tracing::warn!("Consolidation: erreur stockage LTM: {}", e);
                }
            }
        }
    }

    // Etape 4b : Nettoyer les souvenirs episodiques deja consolides.
    match db.cleanup_consolidated_episodic().await {
        Ok(n) if n > 0 => tracing::info!("Consolidation: {} souvenirs consolides nettoyes", n),
        Ok(_) => {},
        Err(e) => tracing::warn!("Consolidation: erreur nettoyage consolides: {}", e),
    }

    // Etape 5 : Decroissance de force des souvenirs episodiques non consolides.
    match db.decay_episodic(params.decay_rate).await {
        Ok(n) => report.decayed = n,
        Err(e) => tracing::warn!("Consolidation: erreur decay: {}", e),
    }

    // Etape 6 : Elagage episodique si nombre > max_episodic.
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

    // Etape 7 : Elagage LTM avec protection et archivage.
    // Les souvenirs proteges (access_count >= seuil OU emotional_weight >= seuil)
    // sont epargnes. Les souvenirs elagués sont archivés en lots compresses,
    // jamais supprimes silencieusement.
    match db.count_ltm().await {
        Ok(ltm_count) if ltm_count as usize > params.ltm_max => {
            let to_prune = ltm_count as usize - params.ltm_prune_target;
            tracing::info!(
                "LTM pruning: {} souvenirs (max={}), elagage de {} vers cible={}",
                ltm_count, params.ltm_max, to_prune, params.ltm_prune_target
            );

            // Recuperer les souvenirs les plus faibles non proteges
            match db.fetch_ltm_weakest_unprotected(
                to_prune as i64,
                params.ltm_protection_access_count,
                params.ltm_protection_emotional_weight,
            ).await {
                Ok(weak_memories) => {
                    if weak_memories.is_empty() {
                        tracing::info!("LTM pruning: aucun souvenir non protege a elaguer");
                    } else {
                        // Clustering simple : regrouper les souvenirs par similarite semantique
                        // avant de creer les archives. Chaque cluster produit une archive cohérente.
                        let clusters = cluster_by_similarity(&weak_memories, encoder, 0.4);
                        for batch in &clusters {
                            // Construire le resume du lot
                            let summary: String = batch.iter()
                                .map(|m| {
                                    let preview: String = m.text_summary.chars().take(120).collect();
                                    preview
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");

                            // Embedding moyen normalise L2
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
                                // Normalisation L2
                                let norm: f32 = avg_embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
                                if norm > 0.0 {
                                    for v in &mut avg_embedding {
                                        *v /= norm;
                                    }
                                }
                            }

                            // Deduplication des emotions
                            let mut emotions: Vec<String> = batch.iter()
                                .map(|m| m.emotion.clone())
                                .filter(|e| !e.is_empty())
                                .collect();
                            emotions.sort();
                            emotions.dedup();

                            // Periode (min/max created_at)
                            let period_start = batch.iter()
                                .map(|m| m.created_at)
                                .min()
                                .unwrap_or_else(chrono::Utc::now);
                            let period_end = batch.iter()
                                .map(|m| m.created_at)
                                .max()
                                .unwrap_or_else(chrono::Utc::now);

                            // Poids emotionnel moyen
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

                                    // Supprimer les souvenirs source
                                    match db.delete_memories_by_ids(&source_ids).await {
                                        Ok(n) => report.ltm_pruned += n,
                                        Err(e) => tracing::warn!(
                                            "LTM pruning: erreur suppression lot: {}", e
                                        ),
                                    }
                                },
                                Err(e) => {
                                    tracing::warn!("LTM pruning: erreur creation archive: {}", e);
                                    // On ne supprime PAS les souvenirs si l'archivage echoue
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

/// Regroupe des souvenirs par similarite semantique (clustering glouton).
///
/// Algorithme : pour chaque souvenir, on calcule son embedding puis on le
/// compare au centroide de chaque cluster existant. S'il est suffisamment
/// similaire (>= threshold), on l'ajoute au cluster le plus proche.
/// Sinon, on cree un nouveau cluster.
///
/// Complexite : O(n * k * d) ou n = souvenirs, k = clusters, d = dimension.
fn cluster_by_similarity<'a>(
    memories: &'a [MemoryRecord],
    encoder: &dyn TextEncoder,
    threshold: f64,
) -> Vec<Vec<&'a MemoryRecord>> {
    if memories.is_empty() {
        return vec![];
    }

    // Pre-encoder tous les souvenirs
    let embeddings: Vec<Vec<f64>> = memories.iter()
        .map(|m| encoder.encode(&m.text_summary))
        .collect();

    // Clusters : (centroid, indices)
    let mut clusters: Vec<(Vec<f64>, Vec<usize>)> = Vec::new();

    for (i, emb) in embeddings.iter().enumerate() {
        // Trouver le cluster le plus similaire
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
            // Ajouter au cluster existant et mettre a jour le centroide (moyenne courante)
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
            // Creer un nouveau cluster
            clusters.push((emb.clone(), vec![i]));
        }
    }

    // Convertir les indices en references
    clusters.into_iter()
        .map(|(_, indices)| {
            indices.into_iter().map(|i| &memories[i]).collect()
        })
        .collect()
}

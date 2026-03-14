// =============================================================================
// implementations.rs — Implementations of the AlgorithmExecutor trait
//
// Each struct wraps an existing (or new) algorithm and translates its
// result into natural language so the LLM can understand it.
// =============================================================================

use std::collections::HashMap;
use super::orchestrator::{AlgorithmExecutor, AlgorithmInput, AlgorithmOutput};

// ─── K-Means ───────────────────────────────────────────────────────────────

pub struct KMeansExecutor;

impl AlgorithmExecutor for KMeansExecutor {
    fn id(&self) -> &str { "kmeans" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("K-Means necessite des vecteurs en entree")?;
        if vectors.is_empty() {
            return Err("Pas de donnees pour K-Means".into());
        }

        let k = input.params.get("k").map(|v| *v as usize).unwrap_or(5);
        let labels = super::kmeans::kmeans(&vectors, k, 100);

        // Count elements per cluster
        let mut cluster_sizes: HashMap<usize, usize> = HashMap::new();
        for &label in &labels {
            *cluster_sizes.entry(label).or_insert(0) += 1;
        }

        let mut sorted_clusters: Vec<(usize, usize)> = cluster_sizes.into_iter().collect();
        sorted_clusters.sort_by(|a, b| b.1.cmp(&a.1));

        let total = vectors.len();
        let mut desc = format!("J'ai regroupe mes {} elements en {} categories :\n", total, k);
        for (i, (cluster_id, size)) in sorted_clusters.iter().enumerate() {
            let pct = (*size as f64 / total as f64 * 100.0) as u32;
            desc.push_str(&format!("  - Groupe {} ({} elements, {}%)\n", i + 1, size, pct));
            // We don't know semantic labels here — the LLM will interpret
            let _ = cluster_id; // used for the JSON        }

        let biggest = sorted_clusters.first().map(|(_, s)| *s).unwrap_or(0);
        let smallest = sorted_clusters.last().map(|(_, s)| *s).unwrap_or(0);
        desc.push_str(&format!(
            "Le plus grand groupe contient {} elements, le plus petit {}.",
            biggest, smallest
        ));

        Ok(AlgorithmOutput {
            algorithm_id: "kmeans".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "k": k,
                "total_points": total,
                "clusters": sorted_clusters.iter().enumerate().map(|(i, (cid, size))| {
                    serde_json::json!({"id": cid, "rank": i, "size": size})
                }).collect::<Vec<_>>(),
                "labels": labels,
            }),
            metrics: [
                ("k".into(), k as f64),
                ("total_points".into(), total as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── DBSCAN ────────────────────────────────────────────────────────────────

pub struct DbscanExecutor;

impl AlgorithmExecutor for DbscanExecutor {
    fn id(&self) -> &str { "dbscan" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("DBSCAN necessite des vecteurs en entree")?;
        if vectors.is_empty() {
            return Err("Pas de donnees pour DBSCAN".into());
        }

        let eps = input.params.get("eps").copied().unwrap_or(1.0);
        let min_pts = input.params.get("min_points").map(|v| *v as usize).unwrap_or(3);
        let n = vectors.len();

        // Simplified DBSCAN implementation
        let mut labels = vec![-1i32; n]; // -1 = noise
        let mut cluster_id = 0i32;

        for i in 0..n {
            if labels[i] != -1 { continue; }

            // Find neighbors of i
            let neighbors = region_query(&vectors, i, eps);
            if neighbors.len() < min_pts {
                continue; // Noise point            }

            // New cluster
            labels[i] = cluster_id;
            let mut queue = neighbors.clone();
            let mut qi = 0;
            while qi < queue.len() {
                let j = queue[qi];
                qi += 1;
                if labels[j] == -1 || labels[j] >= 0 && labels[j] != cluster_id {
                    if labels[j] == -1 { // was noise, now border point
                        labels[j] = cluster_id;
                    }
                    if labels[j] != -1 { continue; } // already in a cluster                }
                labels[j] = cluster_id;
                let j_neighbors = region_query(&vectors, j, eps);
                if j_neighbors.len() >= min_pts {
                    for &nn in &j_neighbors {
                        if !queue.contains(&nn) {
                            queue.push(nn);
                        }
                    }
                }
            }
            cluster_id += 1;
        }

        let noise_count = labels.iter().filter(|&&l| l == -1).count();
        let num_clusters = if cluster_id > 0 { cluster_id as usize } else { 0 };

        let mut desc = format!(
            "DBSCAN a identifie {} groupes naturels parmi {} elements.\n",
            num_clusters, n
        );
        if noise_count > 0 {
            desc.push_str(&format!(
                "{} elements isoles (outliers) ne rentrent dans aucun groupe.\n",
                noise_count
            ));
        }

        for c in 0..cluster_id {
            let size = labels.iter().filter(|&&l| l == c).count();
            desc.push_str(&format!("  - Groupe {} : {} elements\n", c + 1, size));
        }

        Ok(AlgorithmOutput {
            algorithm_id: "dbscan".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "num_clusters": num_clusters,
                "noise_count": noise_count,
                "labels": labels,
            }),
            metrics: [
                ("num_clusters".into(), num_clusters as f64),
                ("noise_count".into(), noise_count as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

fn region_query(data: &[Vec<f64>], point_idx: usize, eps: f64) -> Vec<usize> {
    let point = &data[point_idx];
    data.iter().enumerate()
        .filter(|(i, other)| {
            *i != point_idx && euclidean_dist(point, other) <= eps
        })
        .map(|(i, _)| i)
        .collect()
}

fn euclidean_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

// ─── Naive Bayes ───────────────────────────────────────────────────────────

pub struct NaiveBayesExecutor;

impl AlgorithmExecutor for NaiveBayesExecutor {
    fn id(&self) -> &str { "naive_bayes" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let texts = input.texts.ok_or("Naive Bayes necessite des textes en entree")?;
        let labels = input.labels.ok_or("Naive Bayes necessite des labels pour l'entrainement")?;

        if texts.len() != labels.len() || texts.is_empty() {
            return Err("Textes et labels doivent avoir la meme taille et etre non-vides".into());
        }

        // Train the classifier (tokenize each text into words)
        let mut classifier = super::naive_bayes::NaiveBayesClassifier::new();
        for (text, label) in texts.iter().zip(labels.iter()) {
            let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_lowercase()).collect();
            classifier.train(&tokens, label);
        }

        // Predict on the last text (or a specific text)
        let test_text = texts.last().unwrap();
        let test_tokens: Vec<String> = test_text.split_whitespace().map(|s| s.to_lowercase()).collect();
        let (predicted_class, confidence) = classifier.predict(&test_tokens);

        // Stats per class
        let mut class_counts: HashMap<String, usize> = HashMap::new();
        for label in &labels {
            *class_counts.entry(label.clone()).or_insert(0) += 1;
        }

        let desc = format!(
            "Classification terminee sur {} exemples en {} categories.\n\
             Prediction pour le dernier texte : '{}' (confiance {:.0}%)\n\
             Distribution : {}",
            texts.len(),
            class_counts.len(),
            predicted_class,
            confidence * 100.0,
            class_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(AlgorithmOutput {
            algorithm_id: "naive_bayes".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "predicted_class": predicted_class,
                "confidence": confidence,
                "class_counts": class_counts,
                "training_size": texts.len(),
            }),
            metrics: [
                ("confidence".into(), confidence),
                ("num_classes".into(), class_counts.len() as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── KNN ───────────────────────────────────────────────────────────────────

pub struct KnnExecutor;

impl AlgorithmExecutor for KnnExecutor {
    fn id(&self) -> &str { "knn" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("KNN necessite des vecteurs en entree")?;
        let labels = input.labels.ok_or("KNN necessite des labels")?;

        if vectors.len() < 2 || vectors.len() != labels.len() {
            return Err("KNN necessite au moins 2 points avec labels".into());
        }

        let k = input.params.get("k").map(|v| *v as usize).unwrap_or(5).min(vectors.len() - 1);

        // The last point is the query, the others are the database
        let query = vectors.last().unwrap();
        let base = &vectors[..vectors.len() - 1];
        let base_labels = &labels[..labels.len() - 1];

        // Compute distances
        let mut distances: Vec<(usize, f64)> = base.iter().enumerate()
            .map(|(i, v)| (i, euclidean_dist(query, v)))
            .collect();
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take the K nearest
        let nearest: Vec<(usize, f64)> = distances.into_iter().take(k).collect();

        // Majority vote
        let mut votes: HashMap<&str, usize> = HashMap::new();
        for &(idx, _) in &nearest {
            *votes.entry(&base_labels[idx]).or_insert(0) += 1;
        }
        let predicted = votes.iter()
            .max_by_key(|(_, &v)| v)
            .map(|(k, _)| k.to_string())
            .unwrap_or_default();

        let confidence = votes.get(predicted.as_str()).copied().unwrap_or(0) as f64 / k as f64;

        let desc = format!(
            "Les {} voisins les plus proches suggerent la classe '{}' (confiance {:.0}%).\n\
             Voisins : {}",
            k, predicted, confidence * 100.0,
            nearest.iter().take(3).map(|(idx, dist)| {
                format!("{} (dist={:.3})", base_labels[*idx], dist)
            }).collect::<Vec<_>>().join(", ")
        );

        Ok(AlgorithmOutput {
            algorithm_id: "knn".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "predicted_class": predicted,
                "confidence": confidence,
                "k": k,
                "nearest": nearest.iter().map(|(idx, dist)| serde_json::json!({
                    "label": base_labels[*idx],
                    "distance": dist,
                })).collect::<Vec<_>>(),
            }),
            metrics: [("confidence".into(), confidence), ("k".into(), k as f64)].into(),
            execution_ms: 0,
        })
    }
}

// ─── Decision Tree (simplified) ─────────────────────────────────────────────

pub struct DecisionTreeExecutor;

impl AlgorithmExecutor for DecisionTreeExecutor {
    fn id(&self) -> &str { "decision_tree" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("Decision Tree necessite des vecteurs")?;
        let labels = input.labels.ok_or("Decision Tree necessite des labels")?;

        if vectors.is_empty() || vectors.len() != labels.len() {
            return Err("Donnees invalides pour Decision Tree".into());
        }

        let dim = vectors[0].len();
        let n = vectors.len();

        // Find the best feature (the one that best separates classes)
        let mut best_feature = 0;
        let mut best_threshold = 0.0;
        let mut best_gini = f64::MAX;

        for feat in 0..dim {
            let mean: f64 = vectors.iter().map(|v| v[feat]).sum::<f64>() / n as f64;
            let gini = gini_impurity(&vectors, &labels, feat, mean);
            if gini < best_gini {
                best_gini = gini;
                best_feature = feat;
                best_threshold = mean;
            }
        }

        // Split and count
        let mut left_counts: HashMap<&str, usize> = HashMap::new();
        let mut right_counts: HashMap<&str, usize> = HashMap::new();
        for (v, l) in vectors.iter().zip(labels.iter()) {
            if v[best_feature] <= best_threshold {
                *left_counts.entry(l.as_str()).or_insert(0) += 1;
            } else {
                *right_counts.entry(l.as_str()).or_insert(0) += 1;
            }
        }

        let left_majority = left_counts.iter().max_by_key(|(_, &v)| v)
            .map(|(k, _)| *k).unwrap_or("?");
        let right_majority = right_counts.iter().max_by_key(|(_, &v)| v)
            .map(|(k, _)| *k).unwrap_or("?");

        let desc = format!(
            "Regle de decision trouvee :\n\
             SI feature[{}] <= {:.3} ALORS '{}' ({} elements)\n\
             SINON '{}' ({} elements)\n\
             Impurete de Gini : {:.3}",
            best_feature, best_threshold,
            left_majority, left_counts.values().sum::<usize>(),
            right_majority, right_counts.values().sum::<usize>(),
            best_gini
        );

        Ok(AlgorithmOutput {
            algorithm_id: "decision_tree".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "best_feature": best_feature,
                "threshold": best_threshold,
                "gini": best_gini,
                "left": left_counts,
                "right": right_counts,
            }),
            metrics: [("gini".into(), best_gini)].into(),
            execution_ms: 0,
        })
    }
}

fn gini_impurity(
    vectors: &[Vec<f64>],
    labels: &[String],
    feature: usize,
    threshold: f64,
) -> f64 {
    let n = vectors.len() as f64;
    let mut left_counts: HashMap<&str, f64> = HashMap::new();
    let mut right_counts: HashMap<&str, f64> = HashMap::new();
    let mut left_total = 0.0;
    let mut right_total = 0.0;

    for (v, l) in vectors.iter().zip(labels.iter()) {
        if v[feature] <= threshold {
            *left_counts.entry(l.as_str()).or_insert(0.0) += 1.0;
            left_total += 1.0;
        } else {
            *right_counts.entry(l.as_str()).or_insert(0.0) += 1.0;
            right_total += 1.0;
        }
    }

    let gini = |counts: &HashMap<&str, f64>, total: f64| -> f64 {
        if total == 0.0 { return 0.0; }
        1.0 - counts.values().map(|c| (c / total).powi(2)).sum::<f64>()
    };

    (left_total / n) * gini(&left_counts, left_total)
        + (right_total / n) * gini(&right_counts, right_total)
}

// ─── Isolation Forest (via Z-Score) ────────────────────────────────────────

pub struct IsolationForestExecutor;

impl AlgorithmExecutor for IsolationForestExecutor {
    fn id(&self) -> &str { "isolation_forest" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("Isolation Forest necessite des vecteurs")?;
        if vectors.is_empty() {
            return Err("Pas de donnees pour la detection d'anomalies".into());
        }

        let threshold = input.params.get("threshold").copied().unwrap_or(2.0);
        let dim = vectors[0].len();
        let n = vectors.len();

        // Detect anomalies via multivariate Z-Score
        let mut anomalies = Vec::new();
        let mol_names = ["dopamine", "cortisol", "serotonine", "adrenaline",
                         "ocytocine", "endorphine", "noradrenaline"];

        for d in 0..dim {
            let values: Vec<f64> = vectors.iter().map(|v| v[d]).collect();
            let mean = values.iter().sum::<f64>() / n as f64;
            let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
            let std_dev = variance.sqrt();

            if std_dev < 1e-10 { continue; }

            for (i, v) in values.iter().enumerate() {
                let z = (v - mean) / std_dev;
                if z.abs() > threshold {
                    let mol_name = mol_names.get(d).copied().unwrap_or("dim");
                    anomalies.push((i, d, mol_name, z, *v));
                }
            }
        }

        let anomaly_count = anomalies.len();
        let mut desc = format!(
            "J'ai analyse {} points sur {} dimensions.\n",
            n, dim
        );

        if anomalies.is_empty() {
            desc.push_str("Aucune anomalie detectee. Tout est dans les normes.");
        } else {
            desc.push_str(&format!("J'ai detecte {} anomalies :\n", anomaly_count));
            for (i, (_idx, _d, mol, z, val)) in anomalies.iter().take(5).enumerate() {
                let direction = if *z > 0.0 { "anormalement haut" } else { "anormalement bas" };
                desc.push_str(&format!(
                    "  - Point {} : {} {} ({:.3}, z={:.2})\n",
                    i + 1, mol, direction, val, z
                ));
            }
            if anomaly_count > 5 {
                desc.push_str(&format!("  ... et {} autres.\n", anomaly_count - 5));
            }
        }

        Ok(AlgorithmOutput {
            algorithm_id: "isolation_forest".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "anomaly_count": anomaly_count,
                "anomalies": anomalies.iter().map(|(idx, d, mol, z, val)| serde_json::json!({
                    "point_index": idx,
                    "dimension": d,
                    "molecule": mol,
                    "z_score": z,
                    "value": val,
                })).collect::<Vec<_>>(),
                "total_points": n,
                "dimensions": dim,
            }),
            metrics: [
                ("anomalies_found".into(), anomaly_count as f64),
                ("critical".into(), if anomaly_count > 3 { 1.0 } else { 0.0 }),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── PCA ───────────────────────────────────────────────────────────────────

pub struct PcaExecutor;

impl AlgorithmExecutor for PcaExecutor {
    fn id(&self) -> &str { "pca" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let vectors = input.vectors.ok_or("PCA necessite des vecteurs en entree")?;
        if vectors.len() < 3 {
            return Err("PCA necessite au moins 3 points".into());
        }

        let n_components = input.params.get("n_components").map(|v| *v as usize).unwrap_or(3);
        let result = super::pca::pca(&vectors, n_components)
            .ok_or("PCA a echoue (donnees insuffisantes ou invalides)")?;

        let total_variance: f64 = result.explained_variance.iter().sum();
        let dim = vectors[0].len();

        let mut desc = format!(
            "PCA de {} points en {} dimensions reduit a {} composantes.\n",
            vectors.len(), dim, n_components
        );
        for (i, var) in result.explained_variance.iter().enumerate() {
            let pct = if total_variance > 0.0 { var / total_variance * 100.0 } else { 0.0 };
            desc.push_str(&format!(
                "  - Composante {} : {:.1}% de la variance\n",
                i + 1, pct
            ));
        }

        let explained_ratio: f64 = if total_variance > 0.0 {
            result.explained_variance.iter().sum::<f64>() / total_variance
        } else {
            0.0
        };
        desc.push_str(&format!(
            "Ces {} composantes capturent {:.1}% de l'information totale.",
            n_components, explained_ratio * 100.0
        ));

        Ok(AlgorithmOutput {
            algorithm_id: "pca".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "n_components": n_components,
                "explained_variance": result.explained_variance,
                "explained_ratio": explained_ratio,
                "n_points": vectors.len(),
                "original_dim": dim,
            }),
            metrics: [
                ("explained_ratio".into(), explained_ratio),
                ("n_components".into(), n_components as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── Association Rules ─────────────────────────────────────────────────────

pub struct AssociationRulesExecutor;

impl AlgorithmExecutor for AssociationRulesExecutor {
    fn id(&self) -> &str { "association_rules" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let texts = input.texts.ok_or("Association Rules necessite des textes")?;
        if texts.len() < 5 {
            return Err("Au moins 5 sequences necessaires pour trouver des patterns".into());
        }

        let min_support = input.params.get("min_support").copied().unwrap_or(0.1);
        let min_confidence = input.params.get("min_confidence").copied().unwrap_or(0.5);
        let n = texts.len() as f64;

        // Count antecedent → consequent pairs
        // Expected format: "antecedent→consequent" (separator →)
        let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut antecedent_counts: HashMap<String, usize> = HashMap::new();

        for text in &texts {
            let parts: Vec<&str> = text.splitn(2, ['→', '>']).collect();
            if parts.len() == 2 {
                let ante = parts[0].trim().to_string();
                let cons = parts[1].trim().to_string();
                *pair_counts.entry((ante.clone(), cons)).or_insert(0) += 1;
                *antecedent_counts.entry(ante).or_insert(0) += 1;
            }
        }

        // Compute rules with support and confidence
        let mut rules: Vec<(String, String, f64, f64)> = Vec::new(); // ante, cons, support, confidence
        for ((ante, cons), count) in &pair_counts {
            let support = *count as f64 / n;
            let confidence = *count as f64 / *antecedent_counts.get(ante).unwrap_or(&1) as f64;

            if support >= min_support && confidence >= min_confidence {
                rules.push((ante.clone(), cons.clone(), support, confidence));
            }
        }

        rules.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

        let mut desc = format!("J'ai trouve {} regles dans {} sequences :\n", rules.len(), texts.len());
        for (ante, cons, _support, confidence) in rules.iter().take(5) {
            desc.push_str(&format!(
                "  - Quand '{}' → souvent '{}' ({:.0}% du temps)\n",
                ante, cons, confidence * 100.0
            ));
        }
        if rules.is_empty() {
            desc.push_str("  Aucun pattern suffisamment frequent n'a ete trouve.");
        }

        Ok(AlgorithmOutput {
            algorithm_id: "association_rules".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "rules_count": rules.len(),
                "rules": rules.iter().map(|(a, c, s, conf)| serde_json::json!({
                    "antecedent": a,
                    "consequent": c,
                    "support": s,
                    "confidence": conf,
                })).collect::<Vec<_>>(),
                "total_sequences": texts.len(),
            }),
            metrics: [
                ("rules_found".into(), rules.len() as f64),
                ("best_confidence".into(), rules.first().map(|r| r.3).unwrap_or(0.0)),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── Exponential Smoothing (EMA) ───────────────────────────────────────────

pub struct ExponentialSmoothingExecutor;

impl AlgorithmExecutor for ExponentialSmoothingExecutor {
    fn id(&self) -> &str { "exponential_smoothing" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let series = input.time_series.ok_or("Lissage exponentiel necessite une serie temporelle")?;
        if series.len() < 3 {
            return Err("Au moins 3 points necessaires pour le lissage".into());
        }

        let alpha = input.params.get("alpha").copied().unwrap_or(0.3);

        // Compute the smoothed series (EMA)
        let mut smoothed = Vec::with_capacity(series.len());
        smoothed.push(series[0]);
        for i in 1..series.len() {
            let prev = smoothed[i - 1];
            smoothed.push(alpha * series[i] + (1.0 - alpha) * prev);
        }

        // Analyze the trend
        let first_half_avg = smoothed[..smoothed.len() / 2].iter().sum::<f64>()
            / (smoothed.len() / 2) as f64;
        let second_half_avg = smoothed[smoothed.len() / 2..].iter().sum::<f64>()
            / (smoothed.len() - smoothed.len() / 2) as f64;
        let trend = second_half_avg - first_half_avg;

        let current = *smoothed.last().unwrap_or(&0.0);
        let trend_word = if trend.abs() < 0.02 {
            "stable"
        } else if trend > 0.0 {
            "en hausse"
        } else {
            "en baisse"
        };

        let desc = format!(
            "Tendance lissee sur {} points (alpha={:.2}) :\n\
             Valeur actuelle : {:.3}\n\
             Tendance : {} (delta={:+.3})\n\
             Minimum : {:.3}, Maximum : {:.3}",
            series.len(), alpha, current, trend_word, trend,
            smoothed.iter().cloned().fold(f64::INFINITY, f64::min),
            smoothed.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        );

        Ok(AlgorithmOutput {
            algorithm_id: "exponential_smoothing".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "smoothed": smoothed,
                "trend": trend,
                "current": current,
                "alpha": alpha,
            }),
            metrics: [
                ("trend".into(), trend),
                ("current".into(), current),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── Changepoint Detection ─────────────────────────────────────────────────

pub struct ChangepointDetectionExecutor;

impl AlgorithmExecutor for ChangepointDetectionExecutor {
    fn id(&self) -> &str { "changepoint_detection" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let series = input.time_series.ok_or("Detection de rupture necessite une serie temporelle")?;
        if series.len() < 10 {
            return Err("Au moins 10 points necessaires pour la detection de rupture".into());
        }

        let window = input.params.get("window").map(|v| *v as usize).unwrap_or(5);
        let threshold = input.params.get("threshold").copied().unwrap_or(2.0);

        // Detection via sliding mean comparison
        let mut changepoints: Vec<(usize, f64)> = Vec::new(); // (index, amplitude)

        for i in window..series.len().saturating_sub(window) {
            let before_mean = series[i - window..i].iter().sum::<f64>() / window as f64;
            let after_mean = series[i..i + window].iter().sum::<f64>() / window as f64;
            let amplitude = (after_mean - before_mean).abs();

            // Compute local standard deviation for normalization
            let local = &series[i.saturating_sub(window * 2)..=(i + window).min(series.len() - 1)];
            let local_mean = local.iter().sum::<f64>() / local.len() as f64;
            let local_std = (local.iter().map(|v| (v - local_mean).powi(2)).sum::<f64>()
                / local.len() as f64).sqrt();

            if local_std > 1e-10 && amplitude / local_std > threshold {
                // Avoid close duplicates
                if changepoints.last().map(|(last_i, _)| i - last_i > window).unwrap_or(true) {
                    changepoints.push((i, after_mean - before_mean));
                }
            }
        }

        let mut desc = if changepoints.is_empty() {
            format!("Aucun point de rupture detecte dans les {} points analyses. La serie est stable.", series.len())
        } else {
            let mut d = format!(
                "J'ai detecte {} points de rupture dans {} points :\n",
                changepoints.len(), series.len()
            );
            for (i, (idx, amplitude)) in changepoints.iter().enumerate() {
                let direction = if *amplitude > 0.0 { "hausse" } else { "baisse" };
                d.push_str(&format!(
                    "  - Point {} : {} brutale de {:+.3} au point {}\n",
                    i + 1, direction, amplitude, idx
                ));
            }
            d
        };

        let _ = &mut desc; // avoid unused warning

        Ok(AlgorithmOutput {
            algorithm_id: "changepoint_detection".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "changepoints": changepoints.iter().map(|(idx, amp)| serde_json::json!({
                    "index": idx,
                    "amplitude": amp,
                })).collect::<Vec<_>>(),
                "total_points": series.len(),
            }),
            metrics: [
                ("changepoints_found".into(), changepoints.len() as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── UCB1 ──────────────────────────────────────────────────────────────────

pub struct Ucb1Executor;

impl AlgorithmExecutor for Ucb1Executor {
    fn id(&self) -> &str { "ucb1" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let texts = input.texts.ok_or("UCB1 necessite les noms des options (texts)")?;
        if texts.is_empty() {
            return Err("Au moins 1 option necessaire pour UCB1".into());
        }

        // Create a temporary bandit with the options
        let arm_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let mut bandit = super::bandit::UCB1Bandit::new(&arm_refs);

        // If past rewards are provided via params (by index)
        for (name, &reward) in input.params.iter() {
            if let Some(idx) = texts.iter().position(|t| t == name) {
                bandit.update(idx, reward);
            }
        }

        let selected_idx = bandit.select();
        let selected = texts.get(selected_idx).cloned().unwrap_or_default();

        let desc = format!(
            "Parmi {} options, UCB1 recommande '{}'. \
             L'algorithme equilibre entre exploiter les options connues \
             et explorer les options peu testees.",
            texts.len(), selected
        );

        Ok(AlgorithmOutput {
            algorithm_id: "ucb1".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "selected": selected,
                "options": texts,
            }),
            metrics: HashMap::new(),
            execution_ms: 0,
        })
    }
}

// ─── Q-Learning (simplified) ────────────────────────────────────────────────

pub struct QLearningExecutor;

impl AlgorithmExecutor for QLearningExecutor {
    fn id(&self) -> &str { "q_learning" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let texts = input.texts.ok_or("Q-Learning necessite des donnees etat|action|recompense")?;
        if texts.is_empty() {
            return Err("Pas de donnees pour Q-Learning".into());
        }

        let alpha = input.params.get("alpha").copied().unwrap_or(0.1);
        let gamma = input.params.get("gamma").copied().unwrap_or(0.9);

        // Parse state|action|reward tuples
        let mut q_table: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for text in &texts {
            let parts: Vec<&str> = text.split('|').collect();
            if parts.len() >= 3 {
                let state = parts[0].trim().to_string();
                let action = parts[1].trim().to_string();
                let reward: f64 = parts[2].trim().parse().unwrap_or(0.0);

                let q = q_table.entry(state).or_default()
                    .entry(action).or_insert(0.5);
                *q = *q + alpha * (reward + gamma * 0.5 - *q); // simplified: max Q(s',a') = 0.5
            }
        }

        // Find the best actions per state
        let mut best_actions: Vec<(String, String, f64)> = Vec::new();
        for (state, actions) in &q_table {
            if let Some((best_action, &best_q)) = actions.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                best_actions.push((state.clone(), best_action.clone(), best_q));
            }
        }
        best_actions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let mut desc = format!(
            "Q-Learning a analyse {} experiences sur {} etats :\n",
            texts.len(), q_table.len()
        );
        for (state, action, q) in best_actions.iter().take(5) {
            desc.push_str(&format!(
                "  - Etat '{}' → meilleure action '{}' (Q={:.3})\n",
                state, action, q
            ));
        }

        Ok(AlgorithmOutput {
            algorithm_id: "q_learning".into(),
            natural_language_result: desc,
            structured_result: serde_json::json!({
                "q_table": q_table,
                "best_actions": best_actions.iter().map(|(s, a, q)| serde_json::json!({
                    "state": s, "action": a, "q_value": q
                })).collect::<Vec<_>>(),
                "states_count": q_table.len(),
                "experiences": texts.len(),
            }),
            metrics: [
                ("states_count".into(), q_table.len() as f64),
                ("experiences".into(), texts.len() as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── Sentiment VADER (simplified French lexicon) ───────────────────────────

pub struct SentimentVaderExecutor;

impl AlgorithmExecutor for SentimentVaderExecutor {
    fn id(&self) -> &str { "sentiment_vader" }

    fn execute(&self, input: AlgorithmInput) -> Result<AlgorithmOutput, String> {
        let texts = input.texts.ok_or("Sentiment VADER necessite des textes")?;
        if texts.is_empty() { return Err("Pas de textes".into()); }

        // Simplified French lexicon (positive/negative)
        let positive = ["joie","bonheur","amour","rire","paix","reve","lumiere",
            "doux","chaleur","espoir","liberte","beaute","plaisir","calme","harmonie"];
        let negative = ["peur","douleur","mort","ombre","sang","cri","angoisse",
            "froid","vide","solitude","chaos","destruction","terreur","tristesse","rage"];

        let mut scores = Vec::new();
        let mut total_compound = 0.0;
        for text in &texts {
            let lower = text.to_lowercase();
            let words: Vec<&str> = lower.split_whitespace().collect();
            let word_count = words.len().max(1) as f64;
            let pos: f64 = words.iter().filter(|w| positive.contains(w)).count() as f64 / word_count;
            let neg: f64 = words.iter().filter(|w| negative.contains(w)).count() as f64 / word_count;
            let compound = (pos - neg).clamp(-1.0, 1.0);
            scores.push(serde_json::json!({"positive": pos, "negative": neg, "compound": compound}));
            total_compound += compound;
        }
        let avg_compound = total_compound / texts.len() as f64;
        let sentiment_label = if avg_compound > 0.1 { "positif" }
            else if avg_compound < -0.1 { "negatif" } else { "neutre" };

        Ok(AlgorithmOutput {
            algorithm_id: "sentiment_vader".into(),
            natural_language_result: format!(
                "Analyse de sentiment sur {} textes : sentiment moyen {} ({:.2})",
                texts.len(), sentiment_label, avg_compound),
            structured_result: serde_json::json!({
                "scores": scores, "average_compound": avg_compound,
                "label": sentiment_label, "texts_analyzed": texts.len(),
            }),
            metrics: [
                ("average_compound".into(), avg_compound),
                ("texts_analyzed".into(), texts.len() as f64),
            ].into(),
            execution_ms: 0,
        })
    }
}

// ─── Registration of all implementations ──────────────────────────
/// Creates and registers all algorithm implementations
pub fn register_all_implementations() -> HashMap<String, Box<dyn AlgorithmExecutor>> {
    let mut map: HashMap<String, Box<dyn AlgorithmExecutor>> = HashMap::new();
    map.insert("kmeans".into(), Box::new(KMeansExecutor));
    map.insert("dbscan".into(), Box::new(DbscanExecutor));
    map.insert("naive_bayes".into(), Box::new(NaiveBayesExecutor));
    map.insert("knn".into(), Box::new(KnnExecutor));
    map.insert("decision_tree".into(), Box::new(DecisionTreeExecutor));
    map.insert("isolation_forest".into(), Box::new(IsolationForestExecutor));
    map.insert("pca".into(), Box::new(PcaExecutor));
    map.insert("association_rules".into(), Box::new(AssociationRulesExecutor));
    map.insert("exponential_smoothing".into(), Box::new(ExponentialSmoothingExecutor));
    map.insert("changepoint_detection".into(), Box::new(ChangepointDetectionExecutor));
    map.insert("ucb1".into(), Box::new(Ucb1Executor));
    map.insert("q_learning".into(), Box::new(QLearningExecutor));
    map.insert("sentiment_vader".into(), Box::new(SentimentVaderExecutor));
    map
}

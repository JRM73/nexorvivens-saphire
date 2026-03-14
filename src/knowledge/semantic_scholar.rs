// =============================================================================
// knowledge/semantic_scholar.rs — Semantic Scholar API client
// =============================================================================
//
// Purpose: Searches academic articles on Semantic Scholar, a free database
//          covering 200M+ scientific papers (computer science, neuroscience,
//          psychology, philosophy, biology, etc.).
//
// API: https://api.semanticscholar.org/graph/v1/paper/search
//      - Free, no API key required
//      - Limit: 100 requests / 5 minutes
//      - Requested fields: title, abstract, url, authors, year, citationCount
//      - Returns up to 3 results per query
//
// Translation: French queries are translated to English via
//              translate_query_to_english() (shared with arxiv.rs).
//
// Relevance score: 0.85 + citation bonus (max +0.10 for 1000+ citations)
//   Formula: 0.85 + min(citationCount / 1000, 0.10)
//   A paper with 500 citations -> score 0.90
//   A paper with 2000 citations -> score 0.95 (cap)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for academic articles on Semantic Scholar.
    ///
    /// Process:
    ///   1. Translate the query FR -> EN (dictionary of common terms)
    ///   2. URL-encode and send the request to the Graph v1 API
    ///   3. Parse JSON results (title, abstract, authors, year, citations)
    ///   4. Filter articles without a substantial abstract (<50 chars)
    ///   5. Compute relevance score (base + citation bonus)
    ///
    /// Returns a Vec since the API may return up to 3 results.
    pub fn search_semantic_scholar(&self, query: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // Translate the query to English (most articles are in EN)
        // Uses the same function as arxiv.rs (pub(crate))
        let english_query = Self::translate_query_to_english(query);
        let encoded = Self::url_encode(&english_query);

        // Build the API URL with required fields
        let url = format!(
            "https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit=3&fields=title,abstract,url,authors,year,citationCount",
            encoded
        );

        // Safety: verify the domain is in the whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Send the API request
        let resp_str = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; academic research)")
            .call()
            .map_err(|e| {
                tracing::warn!("Semantic Scholar echoue pour '{}': {}", english_query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parse the JSON response
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Results are in the "data" field (array of papers)
        let papers = resp["data"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        let max_chars = self.config.max_content_chars;

        // Transform each paper into a KnowledgeResult
        let results: Vec<KnowledgeResult> = papers.iter()
            .filter_map(|paper| {
                let title = paper["title"].as_str()?.to_string();
                let abstract_text = paper["abstract"].as_str()
                    .unwrap_or("").to_string();
                let paper_url = paper["url"].as_str()
                    .unwrap_or("").to_string();
                let year = paper["year"].as_u64().unwrap_or(0);
                let citations = paper["citationCount"].as_u64().unwrap_or(0);

                // Extract author names
                let authors: Vec<String> = paper["authors"]
                    .as_array()
                    .map(|a| a.iter()
                        .filter_map(|auth| auth["name"].as_str().map(String::from))
                        .collect()
                    ).unwrap_or_default();

                // Skip articles without a substantial abstract
                if abstract_text.len() < 50 { return None; }

                let first_author = authors.first()
                    .cloned()
                    .unwrap_or_else(|| "Unknown".into());

                Some(KnowledgeResult {
                    source: format!("Semantic Scholar ({}, {} citations)", year, citations),
                    title: format!("{} — {}", title, first_author),
                    url: paper_url,
                    extract: abstract_text.chars().take(max_chars).collect(),
                    section_titles: vec![], // Abstracts have no sections
                    total_length: abstract_text.len(),
                    // Relevance score: base 0.85 + citation-proportional bonus
                    // Bonus is capped at 0.10 (for 1000+ citations)
                    relevance_score: 0.85 + (citations as f64 / 1000.0).min(0.1),
                    fetched_at: Utc::now(),
                })
            })
            .collect();

        if results.is_empty() {
            tracing::info!("Semantic Scholar: aucun resultat pour '{}' (traduit: '{}')",
                query, english_query);
            Err(KnowledgeError::NotFound)
        } else {
            tracing::info!("Semantic Scholar: {} resultats pour '{}' (traduit: '{}')",
                results.len(), query, english_query);
            Ok(results)
        }
    }
}

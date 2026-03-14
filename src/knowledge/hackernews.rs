// =============================================================================
// knowledge/hackernews.rs — HackerNews client via Algolia API
// =============================================================================
//
// Purpose: Searches popular tech/science articles on HackerNews
//          via the Algolia API (full-text search, relevance ranking).
//
// API: https://hn.algolia.com/api/v1/search?query=...&tags=story
//      Returns JSON with hits[].title, hits[].url, hits[].story_text, etc.
//
// Relevance score: 0.75
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for articles on HackerNews via the Algolia API.
    ///
    /// Returns up to 3 results sorted by relevance.
    pub fn search_hackernews(&self, query: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let encoded = Self::url_encode(query);
        let url = format!(
            "https://hn.algolia.com/api/v1/search?query={}&tags=story&hitsPerPage=3",
            encoded,
        );

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity)")
            .call()?;

        let body = response.into_string()?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| KnowledgeError::Parse(format!("JSON HackerNews: {}", e)))?;

        let hits = json["hits"].as_array()
            .ok_or_else(|| KnowledgeError::Parse("HackerNews: pas de hits".into()))?;

        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        for hit in hits.iter().take(3) {
            let title = hit["title"].as_str().unwrap_or_default().to_string();
            let story_url = hit["url"].as_str().unwrap_or_default().to_string();
            let object_id = hit["objectID"].as_str().unwrap_or_default();

            // The text may be in story_text or comment_text
            let story_text = hit["story_text"].as_str().unwrap_or_default();
            let extract = if story_text.len() > 20 {
                Self::strip_html_tags(story_text)
                    .chars()
                    .take(max_chars)
                    .collect::<String>()
            } else {
                // Fallback: use title + author + points as context
                let author = hit["author"].as_str().unwrap_or("anonyme");
                let points = hit["points"].as_u64().unwrap_or(0);
                let num_comments = hit["num_comments"].as_u64().unwrap_or(0);
                format!(
                    "{} — par {} | {} points, {} commentaires sur HackerNews",
                    title, author, points, num_comments
                )
            };

            if !title.is_empty() && extract.len() > 20 {
                let hn_url = if story_url.is_empty() {
                    format!("https://news.ycombinator.com/item?id={}", object_id)
                } else {
                    story_url
                };

                results.push(KnowledgeResult {
                    source: "HackerNews".to_string(),
                    title,
                    url: hn_url,
                    extract,
                    section_titles: vec![],
                    total_length: story_text.len(),
                    relevance_score: 0.75,
                    fetched_at: Utc::now(),
                });
            }
        }

        if results.is_empty() {
            Err(KnowledgeError::NotFound)
        } else {
            Ok(results)
        }
    }
}

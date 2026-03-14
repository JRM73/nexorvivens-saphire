// =============================================================================
// knowledge/internet_archive.rs — Internet Archive client (web archives)
// =============================================================================
//
// Purpose: Searches documents on Internet Archive via the search API.
//          Allows Saphire to access digitized books, articles and
//          historical documents.
//
// API: https://archive.org/advancedsearch.php?q=...&output=json
//      Returns JSON with response.docs[].title, description, etc.
//
// Relevance score: 0.70
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for documents on Internet Archive.
    ///
    /// Uses the advanced search API (JSON).
    /// Filters results to keep texts and books.
    pub fn search_internet_archive(&self, query: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let encoded = Self::url_encode(query);
        let url = format!(
            "https://archive.org/advancedsearch.php?q={}&fl[]=identifier&fl[]=title&fl[]=description&fl[]=creator&fl[]=mediatype&rows=3&output=json",
            encoded,
        );

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; educational research)")
            .call()?;

        let body = response.into_string()?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| KnowledgeError::Parse(format!("JSON Internet Archive: {}", e)))?;

        let docs = json["response"]["docs"].as_array()
            .ok_or_else(|| KnowledgeError::Parse("Internet Archive: pas de docs".into()))?;

        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        for doc in docs.iter().take(3) {
            let title = doc["title"].as_str().unwrap_or_default().to_string();
            let identifier = doc["identifier"].as_str().unwrap_or_default();
            let creator = doc["creator"].as_str().unwrap_or("Inconnu");
            let media_type = doc["mediatype"].as_str().unwrap_or("unknown");

            // Extract the description (may be an array or a string)
            let description = if let Some(desc) = doc["description"].as_str() {
                desc.to_string()
            } else if let Some(arr) = doc["description"].as_array() {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ")
            } else {
                String::new()
            };

            let doc_url = format!("https://archive.org/details/{}", identifier);

            let extract = if description.len() > 50 {
                let clean = Self::strip_html_tags(&description);
                format!(
                    "par {} | type: {}\n\n{}",
                    creator,
                    media_type,
                    clean.chars().take(max_chars).collect::<String>(),
                )
            } else {
                format!("{} — par {} ({})", title, creator, media_type)
            };

            if !title.is_empty() && extract.len() > 20 {
                results.push(KnowledgeResult {
                    source: "Internet Archive".to_string(),
                    title,
                    url: doc_url,
                    extract,
                    section_titles: vec![],
                    total_length: description.len(),
                    relevance_score: 0.70,
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

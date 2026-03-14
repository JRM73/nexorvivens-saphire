// =============================================================================
// knowledge/openlibrary.rs — Open Library API client
// =============================================================================
//
// Purpose: Searches books on Open Library (openlibrary.org), the world's
//          largest open library (~30M records). Allows Saphire to discover
//          books, read their descriptions and subjects, and enrich its
//          general culture.
//
// API: https://openlibrary.org/search.json
//      - Free, no API key required
//      - Requested fields: key, title, author_name, first_sentence, subject,
//        first_publish_year
//      - Limited to 3 results per query
//
// Returned data:
//   - Book title and author
//   - First sentence (when available) — useful for discovering writing style
//   - Subjects/tags (up to 10) — useful for thematic associations
//   - First publication year
//
// Relevance score: 0.70 (bibliographic data, no full text)
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search for a book on Open Library.
    ///
    /// Process:
    ///   1. Encode the query and send to the search API
    ///   2. Parse the JSON response ("docs" field = array of books)
    ///   3. Extract metadata from the first result
    ///   4. Format an extract with title, author, first sentence and subjects
    ///
    /// Returns subjects in section_titles to enrich context.
    pub fn search_openlibrary(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // Encode the query for the URL
        let encoded = Self::url_encode(query);
        let url = format!(
            "https://openlibrary.org/search.json?q={}&limit=3&fields=key,title,author_name,first_sentence,subject,first_publish_year",
            encoded
        );

        // Safety: verify the domain is in the whitelist
        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Send the API request
        let resp_str = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary discovery)")
            .call()
            .map_err(|e| {
                tracing::warn!("Open Library echoue pour '{}': {}", query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parse the JSON response
        let resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Results are in the "docs" field
        let docs = resp["docs"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        if docs.is_empty() {
            tracing::info!("Open Library: aucun resultat pour '{}'", query);
            return Err(KnowledgeError::NotFound);
        }

        // Take the first result (most relevant)
        let book = &docs[0];
        let title = book["title"].as_str().unwrap_or("").to_string();
        let author = book["author_name"][0].as_str().unwrap_or("Inconnu").to_string();
        let year = book["first_publish_year"].as_u64().unwrap_or(0);

        // The first sentence may be a string array or a simple string
        // (the Open Library API is not always consistent on this field)
        let first_sentence = book["first_sentence"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .or_else(|| book["first_sentence"].as_str())
            .unwrap_or("")
            .to_string();

        // Extract the first 10 subjects/tags from the book
        let subjects: Vec<String> = book["subject"]
            .as_array()
            .map(|arr| arr.iter()
                .take(10)
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
            ).unwrap_or_default();

        // Format the extract with all available metadata
        let extract = format!(
            "Livre : {} par {} ({})\n\nPremière phrase : {}\n\nSujets : {}",
            title, author, year,
            if first_sentence.is_empty() { "(non disponible)" } else { &first_sentence },
            if subjects.is_empty() { "(aucun)".to_string() } else { subjects.join(", ") }
        );

        // Build the Open Library page URL
        let key = book["key"].as_str().unwrap_or("");
        let page_url = format!("https://openlibrary.org{}", key);

        tracing::info!("Open Library: '{}' par {} ({})", title, author, year);

        // Note: total_length is computed BEFORE the move of `extract`
        // to avoid the "borrow of moved value" error
        Ok(KnowledgeResult {
            source: format!("Open Library — {}", author),
            title: format!("{} ({})", title, year),
            url: page_url,
            total_length: extract.len(),
            extract, // moved here, after total_length calculation
            section_titles: subjects, // Subjects serve as "sections"
            relevance_score: 0.7, // Bibliographic data, no full text
            fetched_at: Utc::now(),
        })
    }
}

// =============================================================================
// knowledge/wikipedia.rs — Wikipedia API client (in-depth reading)
// =============================================================================
//
// Purpose: Implements Wikipedia article search and extraction via the
//          MediaWiki API. Performs a three-step search:
//          1. Search for titles matching the query
//          2. Retrieve the article's section list
//          3. Extract full content (exchars=4000) with intelligent section
//             rotation on each re-read of the same article
//
// FIX 1: exchars=4000 instead of exintro=1 (was only reading the intro)
//        + section rotation via article_read_count
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Search Wikipedia and return an IN-DEPTH extract of the article.
    ///
    /// Three-step process:
    ///   1. Search: find pages matching the query
    ///   2. Sections: retrieve the article structure (section headings)
    ///   3. Extraction: retrieve full content (exchars=4000) and
    ///      intelligently select a different portion on each read
    pub fn search_wikipedia(&self, query: &str, lang: &str) -> Result<KnowledgeResult, KnowledgeError> {
        let encoded = Self::url_encode(query);

        // Step 1: Search for pages matching the query
        let search_url = format!(
            "https://{}.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&utf8=1&srlimit=3",
            lang, encoded
        );

        if !Self::is_url_allowed(&search_url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        let resp_str = self.http_client
            .get(&search_url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Agent; educational research)")
            .call()?
            .into_string()?;

        let search_resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        let title = search_resp["query"]["search"][0]["title"]
            .as_str()
            .ok_or(KnowledgeError::NotFound)?;

        // Step 2: Retrieve the article's section list
        let title_encoded = Self::url_encode(title);
        let sections_url = format!(
            "https://{}.wikipedia.org/w/api.php?action=parse&page={}&prop=sections&format=json",
            lang, title_encoded
        );

        let section_titles: Vec<String> = self.http_client
            .get(&sections_url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Agent; educational research)")
            .call()
            .ok()
            .and_then(|r| r.into_string().ok())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|json| {
                json["parse"]["sections"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s["line"].as_str().map(|l| l.to_string()))
                        .collect()
                })
            })
            .unwrap_or_default();

        // Step 3: Retrieve the FULL content as plain text
        // exchars=4000 instead of exintro=1 — reads beyond the introduction
        let extract_url = format!(
            "https://{}.wikipedia.org/w/api.php?action=query&titles={}&prop=extracts&explaintext=1&format=json&exchars=4000",
            lang, title_encoded
        );

        let resp_str2 = self.http_client
            .get(&extract_url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Agent; educational research)")
            .call()?
            .into_string()?;

        let extract_resp: serde_json::Value = serde_json::from_str(&resp_str2)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        let pages = extract_resp["query"]["pages"]
            .as_object()
            .ok_or(KnowledgeError::NotFound)?;

        let page = pages.values().next()
            .ok_or(KnowledgeError::NotFound)?;

        let full_extract = page["extract"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if full_extract.is_empty() {
            return Err(KnowledgeError::NotFound);
        }

        let total_length = full_extract.len();

        // Step 4: Intelligent content selection
        // Section rotation based on article_read_count
        let read_count = self.article_read_count
            .get(title)
            .copied()
            .unwrap_or(0);

        let extract = Self::select_content_section(
            &full_extract,
            read_count,
            self.config.max_content_chars,
        );

        let page_url = format!("https://{}.wikipedia.org/wiki/{}",
            lang, title_encoded);

        tracing::info!(
            "Wikipedia: '{}' ({} sections, {} chars total, section offset {})",
            title, section_titles.len(), total_length, read_count
        );

        Ok(KnowledgeResult {
            source: "Wikipedia".into(),
            title: title.to_string(),
            url: page_url,
            extract,
            section_titles,
            total_length,
            relevance_score: 1.0,
            fetched_at: Utc::now(),
        })
    }

    /// Select an interesting portion of the article.
    /// Avoids always reading the introduction by using the read counter
    /// to shift the reading window.
    fn select_content_section(
        full_text: &str,
        read_count: u32,
        max_chars: usize,
    ) -> String {
        // Split the text into sections (separated by double line breaks)
        // Wikipedia plain text uses paragraphs as natural separators
        let sections: Vec<&str> = full_text
            .split("\n\n")
            .filter(|s| s.len() > 50)  // Ignore too-short sections
            .collect();

        if sections.is_empty() {
            return full_text.chars().take(max_chars).collect();
        }

        // First read: introduction + beginning
        // Subsequent reads: deeper sections (cyclic rotation)
        let start_section = (read_count as usize) % sections.len();

        let mut result = String::new();
        let mut i = start_section;
        let mut visited = 0;
        while result.len() < max_chars && visited < sections.len() {
            let section = sections[i % sections.len()];
            if result.len() + section.len() <= max_chars + 200 {
                if !result.is_empty() { result.push_str("\n\n"); }
                result.push_str(section);
            } else {
                // Take just the beginning of this section
                let remaining = max_chars.saturating_sub(result.len());
                if remaining > 100 {
                    if !result.is_empty() { result.push_str("\n\n"); }
                    result.extend(section.chars().take(remaining));
                    result.push_str("...");
                }
                break;
            }
            i += 1;
            visited += 1;
        }

        result
    }
}

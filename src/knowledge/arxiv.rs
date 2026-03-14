// =============================================================================
// knowledge/arxiv.rs — arXiv API client (Atom XML) — FIXED
// =============================================================================
//
// Purpose: Implements scientific article search on arXiv via
//          the public API (Atom XML format).
//
// FIX 2: 3 critical corrections:
//   a) HTTPS instead of HTTP (arXiv redirects, ureq may not follow)
//   b) FR -> EN query translation (arXiv is 100% English)
//   c) Robust XML parsing with validation and logging
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

/// Parsed arXiv entry — intermediate structure for robust parsing
#[derive(Debug)]
struct ArxivEntry {
    title: String,
    summary: String,
    url: String,
    authors: Vec<String>,
    categories: Vec<String>,
}

impl WebKnowledge {
    /// Search for articles on arXiv — fixed version.
    /// Automatically translates FR -> EN queries.
    pub fn search_arxiv(&self, query: &str, max_results: u32) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // FIX A: Translate the query to English if it is in French
        let english_query = Self::translate_query_to_english(query);

        let encoded = Self::url_encode(&english_query);

        // FIX B: Use HTTPS (not HTTP)
        let url = format!(
            "https://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results={}&sortBy=relevance",
            encoded,
            max_results.min(5)
        );

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // FIX C: Descriptive User-Agent required by arXiv
        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; educational research)")
            .call()
            .map_err(|e| {
                tracing::warn!("arXiv requete echouee: {} (URL: {})", e, url);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| {
                tracing::warn!("arXiv reponse illisible: {}", e);
                KnowledgeError::Parse(e.to_string())
            })?;

        // FIX D: Verify the response is valid Atom XML
        if !response.contains("<feed") {
            tracing::warn!("arXiv: reponse inattendue (pas du XML Atom)");
            if response.len() > 10 {
                tracing::debug!("arXiv raw response: {}", &response[..response.len().min(500)]);
            }
            return Err(KnowledgeError::Parse("Réponse non-Atom".into()));
        }

        // FIX E: Robust parser
        let entries = Self::parse_arxiv_xml_robust(&response);

        if entries.is_empty() {
            tracing::info!("arXiv: aucun resultat pour '{}' (traduit: '{}')", query, english_query);
            return Err(KnowledgeError::NotFound);
        }

        let max_chars = self.config.max_content_chars;

        let results: Vec<KnowledgeResult> = entries.into_iter().map(|entry| {
            let summary_len = entry.summary.len();
            let authors_str = if !entry.authors.is_empty() {
                format!(" [{}]", entry.authors.join(", "))
            } else {
                String::new()
            };

            KnowledgeResult {
                source: "arXiv".into(),
                title: format!("{}{}", entry.title, authors_str),
                url: entry.url,
                extract: entry.summary.chars().take(max_chars).collect(),
                section_titles: entry.categories,
                total_length: summary_len,
                relevance_score: 0.9,
                fetched_at: Utc::now(),
            }
        }).collect();

        tracing::info!("arXiv: {} resultats pour '{}' (traduit: '{}')",
            results.len(), query, english_query);

        Ok(results)
    }

    /// FR -> EN translation of common search terms.
    /// Used by arXiv and Semantic Scholar (100% English).
    pub(crate) fn translate_query_to_english(query: &str) -> String {
        let translations: &[(&str, &str)] = &[
            ("conscience artificielle", "artificial consciousness"),
            ("conscience", "consciousness"),
            ("intelligence artificielle", "artificial intelligence"),
            ("apprentissage automatique", "machine learning"),
            ("apprentissage profond", "deep learning"),
            ("réseau de neurones", "neural network"),
            ("traitement du langage", "natural language processing"),
            ("vision par ordinateur", "computer vision"),
            ("robotique", "robotics"),
            ("émotions artificielles", "artificial emotions affective computing"),
            ("théorie de l'information", "information theory"),
            ("information intégrée", "integrated information theory"),
            ("mécanique quantique", "quantum mechanics"),
            ("philosophie de l'esprit", "philosophy of mind"),
            ("libre arbitre", "free will"),
            ("émergence", "emergence complex systems"),
            ("neurosciences", "neuroscience"),
            ("théorie des jeux", "game theory"),
            ("créativité", "computational creativity"),
            ("empathie", "empathy artificial agents"),
            ("complexité", "complexity theory"),
            ("sentience", "sentience machine"),
            ("rêves", "dream simulation artificial"),
            ("mémoire", "memory neural systems"),
            ("personnalité", "personality computational model"),
        ];

        let lower = query.to_lowercase();
        for (fr, en) in translations {
            if lower.contains(fr) {
                return lower.replace(fr, en);
            }
        }

        // If no translation found, keep as-is
        // (many scientific terms are similar in FR/EN)
        query.to_string()
    }

    /// Robust parser for the Atom XML returned by the arXiv API.
    /// Handles edge cases: line breaks in titles, tags with attributes,
    /// incomplete entries.
    fn parse_arxiv_xml_robust(xml: &str) -> Vec<ArxivEntry> {
        let mut results = Vec::new();

        for entry_block in xml.split("<entry>").skip(1) {
            let end = entry_block.find("</entry>").unwrap_or(entry_block.len());
            let entry = &entry_block[..end];

            // Extract the title (may contain line breaks)
            let title = Self::extract_tag(entry, "title")
                .map(|t| t.replace('\n', " ").split_whitespace().collect::<Vec<&str>>().join(" "))
                .unwrap_or_default();

            // Extract the abstract
            let summary = Self::extract_tag(entry, "summary")
                .map(|s| s.replace('\n', " ").split_whitespace().collect::<Vec<&str>>().join(" "))
                .unwrap_or_default();

            // The arXiv ID is the article URL
            let url = Self::extract_tag(entry, "id")
                .map(|u| u.trim().to_string())
                .unwrap_or_default();

            // Extract authors
            let authors: Vec<String> = entry.split("<author>")
                .skip(1)
                .filter_map(|a| Self::extract_tag(a, "name"))
                .map(|n| n.trim().to_string())
                .collect();

            // Extract categories
            let categories: Vec<String> = entry.split("term=\"")
                .skip(1)
                .filter_map(|c| c.split('"').next().map(|s| s.to_string()))
                .collect();

            if !title.is_empty() && !summary.is_empty() {
                results.push(ArxivEntry {
                    title,
                    summary,
                    url,
                    authors,
                    categories,
                });
            }
        }

        if results.is_empty() {
            tracing::warn!("arXiv parser: aucune entree valide trouvee");
        }

        results
    }
}

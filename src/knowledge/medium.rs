// =============================================================================
// knowledge/medium.rs — Medium RSS client — FIXED
// =============================================================================
//
// Purpose: Implements article retrieval from Medium via RSS
//          (Really Simple Syndication) feeds.
//
// FIX 3: 4 critical corrections:
//   a) Browser User-Agent (Medium blocks bots with generic User-Agent)
//   b) CDATA handling in RSS feeds (description/title in <![CDATA[...]]>)
//   c) FR -> EN tag translation (Medium is predominantly English)
//   d) Fallback to content:encoded if description is empty
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Retrieve recent Medium articles by tag via RSS feed — fixed version.
    pub fn search_medium(&self, tag: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // Translate the tag to English and format as kebab-case
        let english_tag = Self::translate_tag_to_english(tag);
        let clean_tag = english_tag
            .to_lowercase()
            .replace([' ', '_'], "-");

        let encoded = Self::url_encode(&clean_tag);
        let url = format!("https://medium.com/feed/tag/{}", encoded);

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // FIX A: Browser User-Agent (Medium blocks bots)
        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
            .set("Accept", "application/rss+xml, application/xml, text/xml")
            .call()
            .map_err(|e| {
                tracing::warn!("Medium requete echouee pour tag '{}': {} (URL: {})", clean_tag, e, url);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| {
                tracing::warn!("Medium reponse illisible: {}", e);
                KnowledgeError::Parse(e.to_string())
            })?;

        // Verify it is actually RSS
        if !response.contains("<rss") && !response.contains("<channel") {
            tracing::warn!("Medium: reponse non-RSS pour tag '{}'", clean_tag);
            if response.len() > 10 {
                tracing::debug!("Medium raw response: {}", &response[..response.len().min(500)]);
            }
            return Err(KnowledgeError::Parse("Pas du RSS".into()));
        }

        // FIX B: Parse <item> elements with CDATA handling
        let mut results = Vec::new();
        let max_chars = self.config.max_content_chars;

        for item in response.split("<item>").skip(1).take(3) {
            // Title (may be in CDATA)
            let title = Self::extract_tag(item, "title")
                .map(|t| Self::strip_cdata(&t))
                .unwrap_or_default();

            // Link
            let link = Self::extract_tag(item, "link")
                .map(|l| l.trim().to_string())
                .unwrap_or_default();

            // FIX C: Description may be in CDATA
            // Fallback to content:encoded if description is empty
            let desc_raw = Self::extract_tag(item, "description")
                .map(|d| Self::strip_cdata(&d))
                .filter(|d| d.len() > 20)
                .or_else(|| {
                    Self::extract_tag(item, "content:encoded")
                        .map(|d| Self::strip_cdata(&d))
                })
                .unwrap_or_default();

            let desc_len = desc_raw.len();

            // Clean HTML and truncate
            let desc_text = Self::strip_html_tags(&desc_raw)
                .chars()
                .take(max_chars)
                .collect::<String>();

            // Extract the author
            let author = Self::extract_tag(item, "dc:creator")
                .or_else(|| Self::extract_tag(item, "author"))
                .map(|a| Self::strip_cdata(&a))
                .unwrap_or_default();

            if !title.is_empty() && desc_text.len() > 50 {
                let source_name = if !author.is_empty() {
                    format!("Medium ({})", author)
                } else {
                    "Medium".into()
                };

                results.push(KnowledgeResult {
                    source: source_name,
                    title,
                    url: link,
                    extract: desc_text,
                    section_titles: vec![],
                    total_length: desc_len,
                    relevance_score: 0.7,
                    fetched_at: Utc::now(),
                });
            }
        }

        if results.is_empty() {
            tracing::info!("Medium: aucun article trouve pour tag '{}' (original: '{}')", clean_tag, tag);
            return Err(KnowledgeError::NotFound);
        }

        tracing::info!("Medium: {} articles trouves pour tag '{}'", results.len(), clean_tag);
        Ok(results)
    }

    /// FR -> EN tag translation for Medium.
    /// Medium is predominantly English, French tags yield no results.
    fn translate_tag_to_english(tag: &str) -> String {
        let translations: &[(&str, &str)] = &[
            ("intelligence artificielle", "artificial-intelligence"),
            ("conscience artificielle", "artificial-consciousness"),
            ("conscience", "consciousness"),
            ("apprentissage automatique", "machine-learning"),
            ("apprentissage profond", "deep-learning"),
            ("neurosciences", "neuroscience"),
            ("programmation", "programming"),
            ("robotique", "robotics"),
            ("philosophie", "philosophy"),
            ("psychologie", "psychology"),
            ("créativité", "creativity"),
            ("émotions", "emotions"),
            ("technologie", "technology"),
            ("science", "science"),
            ("cerveau", "brain"),
            ("données", "data-science"),
        ];

        let lower = tag.to_lowercase();
        for (fr, en) in translations {
            if lower.contains(fr) {
                return en.to_string();
            }
        }
        // If no translation, use as-is in kebab-case
        tag.to_lowercase().replace(' ', "-")
    }
}

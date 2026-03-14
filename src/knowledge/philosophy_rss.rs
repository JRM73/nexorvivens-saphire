// =============================================================================
// knowledge/philosophy_rss.rs — RSS client for philosophical essays
// =============================================================================
//
// Purpose: Retrieves recent philosophical essays from RSS feeds.
//          Unlike other sources (keyword search), this module fetches the
//          LATEST published articles without any search filter.
//          It serves as a source of "intellectual serendipity" for Saphire.
//
// RSS Sources:
//   - Aeon.co: long-form essays on consciousness, existence, beauty,
//     ethics, identity. Published by philosophers, scientists and writers.
//   - Daily Nous: academic blog on philosophical news, debates,
//     publications and events in the philosophy world.
//
// RSS Format:
//   - <item><title>...<description>...<link>...</item>
//   - Content may be wrapped in CDATA: <![CDATA[...]]>
//   - Descriptions often contain HTML (tags like <p>, <a>, etc.)
//   - Cleaned with strip_cdata() then strip_html_tags()
//
// User-Agent note:
//   We use a browser User-Agent (Mozilla/5.0) because some RSS servers
//   reject requests with generic User-Agents.
//
// Relevance score: 0.80
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Retrieve recent philosophical essays via RSS (Aeon + Daily Nous).
    ///
    /// Process:
    ///   1. Iterate over the 2 RSS feeds
    ///   2. For each feed, download the XML
    ///   3. Parse the first 3 <item> elements from each feed
    ///   4. Extract title, link and description (with fallback to content:encoded)
    ///   5. Clean the content (CDATA + HTML)
    ///   6. Filter out articles with too short descriptions (<100 chars)
    ///
    /// Returns a Vec since we retrieve up to 6 articles (3 per feed).
    /// Returns KnowledgeError::NotFound if no articles are found.
    pub fn search_philosophy_rss(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // The 2 RSS feeds to query
        let feeds: &[(&str, &str)] = &[
            ("https://aeon.co/feed.rss", "Aeon"),
            ("https://dailynous.com/feed/", "Daily Nous"),
        ];

        let mut results = Vec::new();
        let max_chars = self.config.max_content_chars;

        for (url, source_name) in feeds {
            // Safety: verify the domain is in the whitelist
            if !Self::is_url_allowed(url) {
                continue;
            }

            // Download the RSS feed
            // Note: browser User-Agent to avoid server rejections
            let response = match self.http_client
                .get(url)
                .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
                .set("Accept", "application/rss+xml, application/xml, text/xml")
                .call()
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("RSS {} reponse illisible : {}", source_name, e);
                        continue; // Skip to the next feed
                    }
                },
                Err(e) => {
                    tracing::warn!("RSS {} echoue : {}", source_name, e);
                    continue; // Skip to the next feed
                }
            };

            // Parse the RSS items (take the first 3)
            // The XML is split by "<item>" — skip(1) because the first fragment
            // is the feed header (before the first <item>)
            for item in response.split("<item>").skip(1).take(3) {
                // Extract the title (may be in CDATA)
                let title = Self::extract_tag(item, "title")
                    .map(|t| Self::strip_cdata(&t))
                    .unwrap_or_default();

                // Extract the article link
                let link = Self::extract_tag(item, "link")
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default();

                // Extract description with fallback to content:encoded
                // (some feeds put full content in content:encoded
                //  and a short summary in description)
                let desc_raw = Self::extract_tag(item, "description")
                    .map(|d| Self::strip_cdata(&d))
                    .filter(|d| d.len() > 20) // Ignore too-short descriptions
                    .or_else(|| {
                        // Fallback: use content:encoded (full content)
                        Self::extract_tag(item, "content:encoded")
                            .map(|d| Self::strip_cdata(&d))
                    })
                    .unwrap_or_default();

                // Clean HTML and truncate to max_chars
                let desc_text = Self::strip_html_tags(&desc_raw)
                    .chars()
                    .take(max_chars)
                    .collect::<String>();

                // Only keep articles with a title and substantial content
                if !title.is_empty() && desc_text.len() > 100 {
                    results.push(KnowledgeResult {
                        source: source_name.to_string(),
                        title: title.clone(),
                        url: link,
                        extract: desc_text,
                        section_titles: vec![], // RSS feeds have no sections
                        total_length: desc_raw.len(),
                        relevance_score: 0.80,
                        fetched_at: Utc::now(),
                    });
                }
            }
        }

        if results.is_empty() {
            tracing::info!("Philosophy RSS: aucun article trouve");
            Err(KnowledgeError::NotFound)
        } else {
            tracing::info!("Philosophy RSS: {} articles trouves", results.len());
            Ok(results)
        }
    }
}

// =============================================================================
// knowledge/custom_rss.rs — Configurable generic RSS client
// =============================================================================
//
// Purpose: Allows the user to configure custom RSS feeds in saphire.toml.
//          Saphire periodically browses them to discover new and diverse content.
//
// Configuration in saphire.toml:
//   [knowledge.sources]
//   custom_rss = true
//
//   [knowledge.custom_rss_feeds]
//   urls = ["https://example.com/feed.xml", "https://other.com/rss"]
//
// RSS Format:
//   - Standard RSS 2.0: <item><title>...<description>...<link>...</item>
//   - Content may be in CDATA
//   - Descriptions may contain HTML
//
// Relevance score: 0.70
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Retrieve recent articles from configured RSS feeds.
    ///
    /// Iterates over all custom_rss_feeds URLs and extracts articles.
    /// Takes the first 2 articles from each feed (max 6 total).
    pub fn search_custom_rss(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let feed_urls = &self.config.custom_rss_feeds.urls;

        if feed_urls.is_empty() {
            return Err(KnowledgeError::NotFound);
        }

        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        for feed_url in feed_urls.iter().take(5) {
            // Note: custom RSS feeds are not in ALLOWED_DOMAINS
            // We trust the user who configured them

            let response = match self.http_client
                .get(feed_url)
                .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
                .set("Accept", "application/rss+xml, application/xml, text/xml, application/atom+xml")
                .call()
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("Custom RSS {} illisible: {}", feed_url, e);
                        continue;
                    }
                },
                Err(e) => {
                    tracing::warn!("Custom RSS {} echoue: {}", feed_url, e);
                    continue;
                }
            };

            // Determine the source name from the feed title
            let feed_title = Self::extract_tag(&response, "title")
                .map(|t| Self::strip_cdata(&t))
                .unwrap_or_else(|| {
                    // Extract the domain from the URL as a name
                    feed_url
                        .split("//")
                        .nth(1)
                        .and_then(|s| s.split('/').next())
                        .unwrap_or("RSS")
                        .to_string()
                });

            let source_name = format!("RSS: {}", feed_title);

            // Parse items (RSS 2.0: <item>, Atom: <entry>)
            let is_atom = response.contains("<feed") && response.contains("<entry>");

            if is_atom {
                // Atom format
                for entry in response.split("<entry>").skip(1).take(2) {
                    let title = Self::extract_tag(entry, "title")
                        .map(|t| Self::strip_cdata(&t))
                        .unwrap_or_default();

                    // Atom link: <link href="..." />
                    let link = if let Some(href_start) = entry.find("href=\"") {
                        let rest = &entry[href_start + 6..];
                        if let Some(href_end) = rest.find('"') {
                            rest[..href_end].to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    let content = Self::extract_tag(entry, "content")
                        .or_else(|| Self::extract_tag(entry, "summary"))
                        .map(|c| Self::strip_cdata(&c))
                        .unwrap_or_default();

                    let extract = Self::strip_html_tags(&content)
                        .chars()
                        .take(max_chars)
                        .collect::<String>();

                    if !title.is_empty() && extract.len() > 50 {
                        results.push(KnowledgeResult {
                            source: source_name.clone(),
                            title,
                            url: link,
                            extract,
                            section_titles: vec![],
                            total_length: content.len(),
                            relevance_score: 0.70,
                            fetched_at: Utc::now(),
                        });
                    }
                }
            } else {
                // RSS 2.0 format
                for item in response.split("<item>").skip(1).take(2) {
                    let title = Self::extract_tag(item, "title")
                        .map(|t| Self::strip_cdata(&t))
                        .unwrap_or_default();

                    let link = Self::extract_tag(item, "link")
                        .map(|l| l.trim().to_string())
                        .unwrap_or_default();

                    let desc_raw = Self::extract_tag(item, "description")
                        .map(|d| Self::strip_cdata(&d))
                        .filter(|d| d.len() > 20)
                        .or_else(|| {
                            Self::extract_tag(item, "content:encoded")
                                .map(|d| Self::strip_cdata(&d))
                        })
                        .unwrap_or_default();

                    let extract = Self::strip_html_tags(&desc_raw)
                        .chars()
                        .take(max_chars)
                        .collect::<String>();

                    if !title.is_empty() && extract.len() > 50 {
                        results.push(KnowledgeResult {
                            source: source_name.clone(),
                            title,
                            url: link,
                            extract,
                            section_titles: vec![],
                            total_length: desc_raw.len(),
                            relevance_score: 0.70,
                            fetched_at: Utc::now(),
                        });
                    }
                }
            }
        }

        if results.is_empty() {
            Err(KnowledgeError::NotFound)
        } else {
            Ok(results)
        }
    }
}

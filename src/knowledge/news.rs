// =============================================================================
// knowledge/news.rs — Real-time news (Swiss + international media)
// =============================================================================
//
// Purpose: Allows Saphire to check world news for a sense of time
//          and awareness of its environment.
//          Priority given to Swiss media, category mix to balance
//          mood impact (general, tech, economy, science).
//
// Built-in RSS feeds (~10 feeds):
//   - Swissinfo FR, RTS Info, Le Temps (general CH)
//   - Ars Technica, The Verge (tech)
//   - Les Echos (economy)
//   - Futura Sciences, Heidi.news (science)
//   - L'Agefi (finance CH)
//   - France24 (international)
//
// Relevance score: 0.75
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

/// Pre-configured news RSS feeds
const NEWS_FEEDS: &[(&str, &str)] = &[
    // General CH (Swiss priority)
    ("Swissinfo", "https://www.swissinfo.ch/fre/rss"),
    ("RTS Info", "https://www.rts.ch/info/rss"),
    ("Le Temps", "https://www.letemps.ch/rss"),
    // Tech
    ("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index"),
    ("The Verge", "https://www.theverge.com/rss/index.xml"),
    // Economy
    ("Les Echos", "https://syndication.lesechos.fr/rss/rss_une.xml"),
    // Science
    ("Futura Sciences", "https://www.futura-sciences.com/rss/actualites.xml"),
    ("Heidi.news", "https://www.heidi.news/feed"),
    // Finance CH
    ("L'Agefi", "https://www.agefi.com/feeds/agefi-rss.xml"),
    // International
    ("France24", "https://www.france24.com/fr/rss"),
];

impl WebKnowledge {
    /// Retrieve recent news from pre-configured RSS feeds.
    ///
    /// Selects 2 feeds at random from the 10 available and extracts
    /// the 2-3 most recent articles from each.
    pub fn search_news(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        // Select 2 feeds at random via pseudo-random rotation
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;

        let feed_count = NEWS_FEEDS.len();
        let idx1 = now_nanos % feed_count;
        let idx2 = (now_nanos / 7 + 3) % feed_count; // offset to avoid duplicates

        let selected = if idx1 == idx2 {
            vec![idx1, (idx1 + 1) % feed_count]
        } else {
            vec![idx1, idx2]
        };

        for &idx in &selected {
            let (feed_name, feed_url) = NEWS_FEEDS[idx];
            let source_name = format!("News: {}", feed_name);

            let response = match self.http_client
                .get(feed_url)
                .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
                .set("Accept", "application/rss+xml, application/xml, text/xml, application/atom+xml")
                .call()
            {
                Ok(r) => match r.into_string() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("News {} illisible: {}", feed_name, e);
                        continue;
                    }
                },
                Err(e) => {
                    tracing::warn!("News {} echoue: {}", feed_name, e);
                    continue;
                }
            };

            // Detect the format (Atom vs RSS 2.0)
            let is_atom = response.contains("<feed") && response.contains("<entry>");

            if is_atom {
                // Atom format
                for entry in response.split("<entry>").skip(1).take(3) {
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

                    if !title.is_empty() && extract.len() > 30 {
                        results.push(KnowledgeResult {
                            source: source_name.clone(),
                            title,
                            url: link,
                            extract,
                            section_titles: vec![],
                            total_length: content.len(),
                            relevance_score: 0.75,
                            fetched_at: Utc::now(),
                        });
                    }
                }
            } else {
                // RSS 2.0 format
                for item in response.split("<item>").skip(1).take(3) {
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

                    if !title.is_empty() && extract.len() > 30 {
                        results.push(KnowledgeResult {
                            source: source_name.clone(),
                            title,
                            url: link,
                            extract,
                            section_titles: vec![],
                            total_length: desc_raw.len(),
                            relevance_score: 0.75,
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

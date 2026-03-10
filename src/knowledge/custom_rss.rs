// =============================================================================
// knowledge/custom_rss.rs — Client RSS generique configurable
// =============================================================================
//
// Role : Permet a l'utilisateur de configurer des flux RSS personnalises
//        dans saphire.toml. Saphire les parcourt periodiquement pour
//        decouvrir du contenu nouveau et diversifie.
//
// Configuration dans saphire.toml :
//   [knowledge.sources]
//   custom_rss = true
//
//   [knowledge.custom_rss_feeds]
//   urls = ["https://example.com/feed.xml", "https://other.com/rss"]
//
// Format RSS :
//   - Standard RSS 2.0 : <item><title>...<description>...<link>...</item>
//   - Le contenu peut etre dans CDATA
//   - Les descriptions peuvent contenir du HTML
//
// Score de pertinence : 0.70
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recuperer les articles recents depuis les flux RSS configures.
    ///
    /// Parcourt tous les URLs de custom_rss_feeds et extrait les articles.
    /// Prend les 2 premiers articles de chaque flux (max 6 au total).
    pub fn search_custom_rss(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let feed_urls = &self.config.custom_rss_feeds.urls;

        if feed_urls.is_empty() {
            return Err(KnowledgeError::NotFound);
        }

        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        for feed_url in feed_urls.iter().take(5) {
            // Note : les RSS custom ne sont pas dans ALLOWED_DOMAINS
            // On fait confiance a l'utilisateur qui les a configures

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

            // Determiner le nom de la source depuis le titre du flux
            let feed_title = Self::extract_tag(&response, "title")
                .map(|t| Self::strip_cdata(&t))
                .unwrap_or_else(|| {
                    // Extraire le domaine de l'URL comme nom
                    feed_url
                        .split("//")
                        .nth(1)
                        .and_then(|s| s.split('/').next())
                        .unwrap_or("RSS")
                        .to_string()
                });

            let source_name = format!("RSS: {}", feed_title);

            // Parser les items (RSS 2.0 : <item>, Atom : <entry>)
            let is_atom = response.contains("<feed") && response.contains("<entry>");

            if is_atom {
                // Format Atom
                for entry in response.split("<entry>").skip(1).take(2) {
                    let title = Self::extract_tag(entry, "title")
                        .map(|t| Self::strip_cdata(&t))
                        .unwrap_or_default();

                    // Lien Atom : <link href="..." />
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
                // Format RSS 2.0
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

// =============================================================================
// knowledge/news.rs — Actualites en temps reel (medias suisses + internationaux)
// =============================================================================
//
// Role : Permet a Saphire de consulter l'actualite du monde pour avoir une
//        notion temporelle et une conscience de son environnement.
//        Priorite aux medias suisses, mix de categories pour equilibrer
//        l'impact sur l'humeur (general, tech, economie, science).
//
// Flux RSS integres (~10 feeds) :
//   - Swissinfo FR, RTS Info, Le Temps (general CH)
//   - Ars Technica, The Verge (tech)
//   - Les Echos (economie)
//   - Futura Sciences, Heidi.news (science)
//   - L'Agefi (finance CH)
//   - France24 (international)
//
// Score de pertinence : 0.75
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

/// Flux RSS d'actualite pre-configures
const NEWS_FEEDS: &[(&str, &str)] = &[
    // General CH (priorite suisse)
    ("Swissinfo", "https://www.swissinfo.ch/fre/rss"),
    ("RTS Info", "https://www.rts.ch/info/rss"),
    ("Le Temps", "https://www.letemps.ch/rss"),
    // Tech
    ("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index"),
    ("The Verge", "https://www.theverge.com/rss/index.xml"),
    // Economie
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
    /// Recuperer les actualites recentes depuis les flux RSS pre-configures.
    ///
    /// Selectionne 2 flux au hasard parmi les 10 disponibles et extrait
    /// les 2-3 articles les plus recents de chacun.
    pub fn search_news(&self) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        // Selectionner 2 flux au hasard via rotation pseudo-aleatoire
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;

        let feed_count = NEWS_FEEDS.len();
        let idx1 = now_nanos % feed_count;
        let idx2 = (now_nanos / 7 + 3) % feed_count; // decalage pour eviter doublon

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

            // Detecter le format (Atom vs RSS 2.0)
            let is_atom = response.contains("<feed") && response.contains("<entry>");

            if is_atom {
                // Format Atom
                for entry in response.split("<entry>").skip(1).take(3) {
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
                // Format RSS 2.0
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

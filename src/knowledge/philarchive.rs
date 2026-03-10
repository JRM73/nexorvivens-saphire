// =============================================================================
// knowledge/philarchive.rs — Client PhilArchive (philosophie academique)
// =============================================================================
//
// Role : Recherche d'articles de philosophie academique sur PhilArchive,
//        un depot ouvert d'articles de philosophie (companion de PhilPapers).
//
// API : https://philarchive.org/oai.pl?verb=... (OAI-PMH) ou recherche HTML.
//       On utilise la recherche HTML simple car l'API OAI est moins pratique
//       pour la recherche par mots-cles.
//
// Methode : scraping leger de la page de recherche
//           https://philarchive.org/s/<query>
//
// Score de pertinence : 0.80
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Cherche des articles de philosophie sur PhilArchive.
    ///
    /// Utilise la page de recherche HTML et extrait les resultats.
    pub fn search_philarchive(&self, query: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        let encoded = Self::url_encode(query);
        let url = format!("https://philarchive.org/s/{}", encoded);

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        let response = self.http_client
            .get(&url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
            .set("Accept", "text/html")
            .call()?;

        let body = response.into_string()?;
        let max_chars = self.config.max_content_chars;
        let mut results = Vec::new();

        // Parser les resultats : chercher les blocs d'entrees
        // PhilArchive utilise des <span class="title">...</span> et
        // <span class="abstract">...</span> dans ses resultats
        for entry in body.split("class=\"entryList\"").skip(1).take(1) {
            // Extraire les items individuels
            for item in entry.split("class=\"philtitle\"").skip(1).take(3) {
                // Extraire le titre (dans <a ...>TITRE</a>)
                let title = if let Some(a_start) = item.find('>') {
                    let rest = &item[a_start + 1..];
                    if let Some(a_end) = rest.find("</a>") {
                        Self::strip_html_tags(&rest[..a_end]).trim().to_string()
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                // Extraire le lien
                let link = if let Some(href_start) = item.find("href=\"") {
                    let rest = &item[href_start + 6..];
                    if let Some(href_end) = rest.find('"') {
                        let path = &rest[..href_end];
                        if path.starts_with("http") {
                            path.to_string()
                        } else {
                            format!("https://philarchive.org{}", path)
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Extraire l'abstract s'il existe
                let abstract_text = if let Some(abs_start) = item.find("class=\"abstract\"") {
                    let rest = &item[abs_start..];
                    if let Some(tag_end) = rest.find('>') {
                        let content = &rest[tag_end + 1..];
                        if let Some(div_end) = content.find("</") {
                            Self::strip_html_tags(&content[..div_end])
                                .chars()
                                .take(max_chars)
                                .collect::<String>()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Extraire les auteurs
                let authors = if let Some(auth_start) = item.find("class=\"author\"") {
                    let rest = &item[auth_start..];
                    if let Some(tag_end) = rest.find('>') {
                        let content = &rest[tag_end + 1..];
                        if let Some(end) = content.find("</") {
                            Self::strip_html_tags(&content[..end]).trim().to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let extract = if abstract_text.len() > 50 {
                    format!("{}\n\n{}", authors, abstract_text)
                } else {
                    format!("{} — {}", title, authors)
                };

                if !title.is_empty() && extract.len() > 20 {
                    results.push(KnowledgeResult {
                        source: "PhilArchive".to_string(),
                        title,
                        url: link,
                        extract,
                        section_titles: vec![],
                        total_length: abstract_text.len(),
                        relevance_score: 0.80,
                        fetched_at: Utc::now(),
                    });
                }
            }
        }

        if results.is_empty() {
            // Fallback simplifie : extraire ce qu'on peut du body brut
            let text = Self::strip_html_tags(&body);
            if text.len() > 200 {
                results.push(KnowledgeResult {
                    source: "PhilArchive".to_string(),
                    title: format!("Recherche PhilArchive: {}", query),
                    url,
                    extract: text.chars().take(max_chars).collect(),
                    section_titles: vec![],
                    total_length: text.len(),
                    relevance_score: 0.60,
                    fetched_at: Utc::now(),
                });
                return Ok(results);
            }
            Err(KnowledgeError::NotFound)
        } else {
            Ok(results)
        }
    }
}

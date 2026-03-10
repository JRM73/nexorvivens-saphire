// =============================================================================
// knowledge/medium.rs — Client RSS Medium — CORRIGE
// =============================================================================
//
// Role : Implemente la recuperation d'articles depuis Medium via les flux
//        RSS (Really Simple Syndication).
//
// FIX 3 : 4 corrections critiques :
//   a) User-Agent navigateur (Medium bloque les bots avec User-Agent generique)
//   b) Gestion CDATA dans les flux RSS (description/title dans <![CDATA[...]]>)
//   c) Traduction des tags FR -> EN (Medium est majoritairement anglophone)
//   d) Fallback content:encoded si description est vide
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recupere les articles recents de Medium par tag via le flux RSS — version corrigee.
    pub fn search_medium(&self, tag: &str) -> Result<Vec<KnowledgeResult>, KnowledgeError> {
        // Traduire le tag en anglais et le formater en kebab-case
        let english_tag = Self::translate_tag_to_english(tag);
        let clean_tag = english_tag
            .to_lowercase()
            .replace([' ', '_'], "-");

        let encoded = Self::url_encode(&clean_tag);
        let url = format!("https://medium.com/feed/tag/{}", encoded);

        if !Self::is_url_allowed(&url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // FIX A : User-Agent navigateur (Medium bloque les bots)
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

        // Verifier que c'est bien du RSS
        if !response.contains("<rss") && !response.contains("<channel") {
            tracing::warn!("Medium: reponse non-RSS pour tag '{}'", clean_tag);
            if response.len() > 10 {
                tracing::debug!("Medium raw response: {}", &response[..response.len().min(500)]);
            }
            return Err(KnowledgeError::Parse("Pas du RSS".into()));
        }

        // FIX B : Parser les <item> avec gestion du CDATA
        let mut results = Vec::new();
        let max_chars = self.config.max_content_chars;

        for item in response.split("<item>").skip(1).take(3) {
            // Titre (peut etre dans CDATA)
            let title = Self::extract_tag(item, "title")
                .map(|t| Self::strip_cdata(&t))
                .unwrap_or_default();

            // Lien
            let link = Self::extract_tag(item, "link")
                .map(|l| l.trim().to_string())
                .unwrap_or_default();

            // FIX C : La description peut etre dans CDATA
            // Fallback sur content:encoded si description est vide
            let desc_raw = Self::extract_tag(item, "description")
                .map(|d| Self::strip_cdata(&d))
                .filter(|d| d.len() > 20)
                .or_else(|| {
                    Self::extract_tag(item, "content:encoded")
                        .map(|d| Self::strip_cdata(&d))
                })
                .unwrap_or_default();

            let desc_len = desc_raw.len();

            // Nettoyer le HTML et tronquer
            let desc_text = Self::strip_html_tags(&desc_raw)
                .chars()
                .take(max_chars)
                .collect::<String>();

            // Extraire l'auteur
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

    /// Traduction des tags FR -> EN pour Medium.
    /// Medium est majoritairement anglophone, les tags francais ne donnent rien.
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
        // Si pas de traduction, utiliser tel quel en kebab-case
        tag.to_lowercase().replace(' ', "-")
    }
}

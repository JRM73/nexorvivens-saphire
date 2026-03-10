// =============================================================================
// knowledge/wikipedia.rs — Client API Wikipedia (lecture en profondeur)
// =============================================================================
//
// Role : Implemente la recherche et l'extraction d'articles Wikipedia via
//        l'API MediaWiki. Effectue une recherche en trois etapes :
//        1. Recherche de titres correspondant a la requete
//        2. Recuperation de la liste des sections de l'article
//        3. Extraction du contenu complet (exchars=4000) avec rotation
//           intelligente des sections a chaque relecture du meme article
//
// FIX 1 : exchars=4000 au lieu de exintro=1 (ne lisait que l'intro)
//          + rotation des sections via article_read_count
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Cherche sur Wikipedia et retourne un extrait EN PROFONDEUR de l'article.
    ///
    /// Processus en trois etapes :
    ///   1. Recherche : trouver les pages correspondant a la requete
    ///   2. Sections : recuperer la structure de l'article (titres de sections)
    ///   3. Extraction : recuperer le contenu complet (exchars=4000) et
    ///      selectionner intelligemment une portion differente a chaque lecture
    pub fn search_wikipedia(&self, query: &str, lang: &str) -> Result<KnowledgeResult, KnowledgeError> {
        let encoded = Self::url_encode(query);

        // Etape 1 : Recherche de pages correspondant a la requete
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

        // Etape 2 : Recuperer la liste des sections de l'article
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

        // Etape 3 : Recuperer le contenu COMPLET en texte brut
        // exchars=4000 au lieu de exintro=1 — lit au-dela de l'introduction
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

        // Etape 4 : Selection intelligente du contenu
        // Rotation de section basee sur article_read_count
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

    /// Selectionner une portion interessante de l'article.
    /// Evite de toujours lire l'introduction en utilisant le compteur
    /// de lectures pour decaler la fenetre de lecture.
    fn select_content_section(
        full_text: &str,
        read_count: u32,
        max_chars: usize,
    ) -> String {
        // Decouper le texte en sections (separees par des doubles sauts de ligne)
        // Wikipedia texte brut utilise des paragraphes comme separateurs naturels
        let sections: Vec<&str> = full_text
            .split("\n\n")
            .filter(|s| s.len() > 50)  // Ignorer les sections trop courtes
            .collect();

        if sections.is_empty() {
            return full_text.chars().take(max_chars).collect();
        }

        // Premiere lecture : introduction + debut
        // Lectures suivantes : sections plus profondes (rotation cyclique)
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
                // Prendre juste le debut de cette section
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

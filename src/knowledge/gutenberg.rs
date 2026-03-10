// =============================================================================
// knowledge/gutenberg.rs — Client Project Gutenberg via Gutendex API
// =============================================================================
//
// Role : Recherche de livres classiques (domaine public) sur Project Gutenberg
//        via l'API Gutendex (REST, gratuite, sans cle API).
//
// Processus en 2 etapes :
//   1. Recherche des metadonnees du livre via gutendex.com/books/?search=...
//      Retourne : titre, auteur, ID Gutenberg, URLs des formats disponibles
//   2. Telechargement du texte brut (text/plain) depuis gutenberg.org
//      Extraction d'un passage avec rotation (anti-repetition)
//
// Formats texte tentes dans l'ordre :
//   - text/plain; charset=utf-8
//   - text/plain
//   - text/plain; charset=us-ascii
//
// Rotation : avance de 5 paragraphes a chaque relecture (read_count * 5)
//            pour explorer des parties differentes du livre.
//
// Score de pertinence : 0.85
// =============================================================================

use chrono::Utc;
use super::{WebKnowledge, KnowledgeResult, KnowledgeError};

impl WebKnowledge {
    /// Recherche un livre sur Project Gutenberg via l'API Gutendex.
    ///
    /// Processus :
    ///   1. Envoyer la requete a gutendex.com (recherche FR + EN)
    ///   2. Prendre le premier resultat (le plus pertinent)
    ///   3. Recuperer le texte brut via l'URL du format text/plain
    ///   4. Extraire un passage avec rotation anti-repetition
    ///
    /// Si le texte brut n'est pas disponible, retourne les metadonnees seules.
    pub fn search_gutenberg(&self, query: &str) -> Result<KnowledgeResult, KnowledgeError> {
        // --- Etape 1 : Recherche via Gutendex API ---
        let encoded = Self::url_encode(query);
        let search_url = format!(
            "https://gutendex.com/books/?search={}&languages=fr,en",
            encoded
        );

        // Securite : verifier que le domaine est dans la whitelist
        if !Self::is_url_allowed(&search_url) {
            return Err(KnowledgeError::DomainBlocked);
        }

        // Envoyer la requete de recherche
        let resp_str = self.http_client
            .get(&search_url)
            .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary enrichment)")
            .call()
            .map_err(|e| {
                tracing::warn!("Gutenberg recherche echouee pour '{}': {}", query, e);
                KnowledgeError::Network(e.to_string())
            })?
            .into_string()
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        // Parser la reponse JSON de Gutendex
        let search_resp: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| KnowledgeError::Parse(e.to_string()))?;

        let results = search_resp["results"]
            .as_array()
            .ok_or(KnowledgeError::NotFound)?;

        if results.is_empty() {
            tracing::info!("Gutenberg: aucun resultat pour '{}'", query);
            return Err(KnowledgeError::NotFound);
        }

        // Prendre le premier resultat (le plus pertinent selon Gutendex)
        let book = &results[0];
        let title = book["title"].as_str().unwrap_or("").to_string();
        let author = book["authors"][0]["name"].as_str().unwrap_or("Inconnu").to_string();
        let book_id = book["id"].as_u64().unwrap_or(0);

        // --- Etape 2 : Recuperer un extrait du texte brut ---
        // Chercher l'URL du texte dans les formats disponibles
        // On essaie 3 variantes de content-type
        let text_url = book["formats"]["text/plain; charset=utf-8"]
            .as_str()
            .or_else(|| book["formats"]["text/plain"].as_str())
            .or_else(|| book["formats"]["text/plain; charset=us-ascii"].as_str());

        let extract = if let Some(url) = text_url {
            // Verifier que le domaine du texte est autorise
            if !Self::is_url_allowed(url) {
                tracing::warn!("Gutenberg: domaine du texte non autorise: {}", url);
                format!("Livre : {} par {} (texte non accessible)", title, author)
            } else {
                // Telecharger le texte brut
                match self.http_client
                    .get(url)
                    .set("User-Agent", "Saphire/1.0 (Autonomous Cognitive Entity; literary enrichment)")
                    .call()
                {
                    Ok(resp) => {
                        let text = resp.into_string().unwrap_or_default();
                        // Extraire un passage avec rotation basee sur le compteur de lectures
                        let read_count = self.article_read_count
                            .get(&format!("gutenberg:{}", book_id))
                            .copied()
                            .unwrap_or(0);
                        Self::select_book_passage(&text, read_count, self.config.max_content_chars)
                    }
                    Err(e) => {
                        tracing::warn!("Gutenberg: texte inaccessible pour '{}': {}", title, e);
                        format!("Livre : {} par {}", title, author)
                    }
                }
            }
        } else {
            // Pas de format texte brut disponible pour ce livre
            format!("Livre : {} par {} (pas de texte brut disponible)", title, author)
        };

        // URL de la page Gutenberg pour reference
        let page_url = format!("https://www.gutenberg.org/ebooks/{}", book_id);

        tracing::info!("Gutenberg: '{}' par {} (id: {})", title, author, book_id);

        Ok(KnowledgeResult {
            source: format!("Gutenberg — {}", author),
            title: format!("{} ({})", title, author),
            url: page_url,
            extract,
            section_titles: vec![], // Les livres n'ont pas de sections indexees
            total_length: 0,        // Non calcule (texte trop long)
            relevance_score: 0.85,
            fetched_at: Utc::now(),
        })
    }

    /// Selectionner un passage interessant d'un livre Gutenberg.
    ///
    /// Algorithme :
    ///   1. Ignorer le header Gutenberg ("*** START OF THE PROJECT GUTENBERG...")
    ///   2. Ignorer le footer Gutenberg ("*** END OF THE PROJECT GUTENBERG...")
    ///   3. Decouper le texte en paragraphes (separation par double saut de ligne)
    ///   4. Filtrer les paragraphes de moins de 100 chars (titres, numeros)
    ///   5. Appliquer la rotation : commencer au paragraphe (read_count * 5)
    ///   6. Concatener jusqu'a max_chars caracteres
    ///
    /// Le facteur 5 (vs 3 pour SEP) permet de couvrir plus de texte
    /// car les livres sont beaucoup plus longs que les articles SEP.
    fn select_book_passage(text: &str, read_count: u32, max_chars: usize) -> String {
        // Trouver le debut du contenu reel (apres le header Gutenberg)
        let content_start = text.find("*** START OF")
            .and_then(|pos| text[pos..].find('\n').map(|n| pos + n + 1))
            .unwrap_or(0);

        // Trouver la fin du contenu reel (avant le footer Gutenberg)
        let content_end = text.find("*** END OF")
            .unwrap_or(text.len());

        let clean_text = &text[content_start..content_end];

        // Decouper en paragraphes significatifs (>100 chars)
        let paragraphs: Vec<&str> = clean_text.split("\n\n")
            .map(|p| p.trim())
            .filter(|p| p.len() > 100)
            .collect();

        // Fallback si pas assez de paragraphes
        if paragraphs.is_empty() {
            return clean_text.chars().take(max_chars).collect();
        }

        // Rotation : avancer de 5 paragraphes a chaque lecture
        // Utilise modulo pour boucler quand on atteint la fin
        let start = (read_count as usize * 5) % paragraphs.len().max(1);
        let mut result = String::new();
        for para in paragraphs.iter().skip(start) {
            if result.len() + para.len() > max_chars { break; }
            if !result.is_empty() { result.push_str("\n\n"); }
            result.push_str(para);
        }
        result
    }
}
